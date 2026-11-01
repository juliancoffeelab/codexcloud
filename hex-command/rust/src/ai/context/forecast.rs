use crate::{data::*, logic::Game};
use rustc_hash::FxHashMap as HashMap;
// Dense reachability sets intersect attack origins a machine word at a time.
use crate::logic::geometry::TileSet;
// A kind is not a combat identity: two units of that kind can carry different
// armor/weapons. Use the actual actor, and invalidate retained results whenever
// any roster member's equipment changes (including the attacking enemies).
type RiskKey = (u32, bool, usize, Option<u32>);
type ArenaEnemies = HashMap<Option<u32>, Vec<(Unit, Vec<TileSet>)>>;
type ArenaUnit = (
    u32,
    usize,
    usize,
    Side,
    Option<String>,
    Option<Role>,
    Vec<Item>,
);
/// A nonlethal attack changes HP/action flags, but arena movement forecasts and
/// incoming damage also depend on equipment, allegiance and actor roles.
/// Retain those calculations for the next decision in the same planning phase.
#[derive(Default)]
pub(in crate::ai) struct ArenaThreatWorkspace {
    terrain: Vec<usize>,
    roster: Vec<ArenaUnit>,
    side: Option<Side>,
    arena: ArenaEnemies,
    risks: HashMap<RiskKey, i32>,
}
pub(in crate::ai) struct Threats<'game, 'catalog> {
    orders: Option<&'game crate::ai::orders::TacticalOrders>,
    source: &'game Game<'catalog>,
    side: Side,
    normal: Option<(Game<'catalog>, Vec<(Unit, Vec<TileSet>)>)>,
    cache: HashMap<RiskKey, i32>,
    forecasts: HashMap<(u32, Option<u32>), (Game<'catalog>, Vec<(Unit, Vec<TileSet>)>)>,
    arena: ArenaEnemies,
    workspace: Option<&'game mut ArenaThreatWorkspace>,
}
impl<'game, 'catalog> Threats<'game, 'catalog> {
    /// Potential exposure without an allied screen, so a successful screen does
    /// not cause Campaigner to release its escort on the following turn.
    pub(in crate::ai) fn unshielded(game: &super::BattleView<'_, 'catalog>, unit: &Unit) -> i32 {
        let game = game.engine();
        let mut board = game.fork();
        board.arena_first = None;
        board.end_turn();
        if board.winner.is_some() {
            return 0;
        }
        let Some(target) = board.unit(unit.id) else {
            return 0;
        };
        let enemies: Vec<_> = board
            .units
            .iter()
            .filter(|u| game.hostile(unit, u))
            .map(|enemy| {
                let mut blocked = TileSet::new(board.terrain.len());
                blocked.insert(enemy.tile);
                let reach = board.threat_reach_blocked(enemy, blocked);
                (enemy.clone(), Self::coverage_engine(&board, enemy, &reach))
            })
            .collect();
        Self::pressure(&board, target, &enemies, None, true)
    }
    pub fn new(g: &'game super::BattleView<'_, 'catalog>, side: Side) -> Self {
        let g = g.engine();
        Self {
            orders: None,
            source: g,
            side,
            normal: None,
            cache: HashMap::default(),
            forecasts: HashMap::default(),
            arena: HashMap::default(),
            workspace: None,
        }
    }
    pub(in crate::ai) fn protected(
        g: &'game super::BattleView<'_, 'catalog>,
        side: Side,
        orders: &'game crate::ai::orders::TacticalOrders,
    ) -> Self {
        let mut threats = Self::new(g, side);
        threats.orders = Some(orders);
        threats
    }
    pub(in crate::ai) fn with_orders(
        mut self,
        orders: &'game crate::ai::orders::TacticalOrders,
    ) -> Self {
        self.orders = Some(orders);
        self
    }
    pub fn with_workspace(
        g: &'game super::BattleView<'_, 'catalog>,
        side: Side,
        workspace: &'game mut ArenaThreatWorkspace,
    ) -> Self {
        let mut threats = Self::new(g, side);
        let g = g.engine();
        if g.campaign.is_none() {
            let same = workspace.side == Some(side)
                && workspace.terrain == g.terrain
                && workspace.roster.len() == g.units.len()
                && workspace.roster.iter().zip(&g.units).all(
                    |((id, kind, tile, side, faction, role, items), u)| {
                        (*id, *kind, *tile, *side) == (u.id, u.kind, u.tile, u.side)
                            && *faction == u.faction
                            && *role == u.role
                            && *items == u.items
                    },
                );
            if !same {
                workspace.side = Some(side);
                workspace.terrain.clone_from(&g.terrain);
                workspace.roster.clear();
                workspace.roster.extend(g.units.iter().map(|u| {
                    (
                        u.id,
                        u.kind,
                        u.tile,
                        u.side,
                        u.faction.clone(),
                        u.role,
                        u.items.clone(),
                    )
                }));
                workspace.arena.clear();
                workspace.risks.clear();
            }
            threats.arena = std::mem::take(&mut workspace.arena);
            threats.cache = std::mem::take(&mut workspace.risks);
            threats.workspace = Some(workspace);
        }
        threats
    }
    fn arena_enemies(source: &Game, side: Side, actor: Option<u32>) -> Vec<(Unit, Vec<TileSet>)> {
        let mut occupied = if actor.is_some() {
            TileSet::new(source.terrain.len())
        } else {
            source.occupied_tiles.clone()
        };
        if let Some(actor) = actor {
            for ally in source
                .units
                .iter()
                .filter(|u| u.side == side && u.id != actor)
            {
                occupied.insert(ally.tile);
            }
        }
        source
            .units
            .iter()
            .filter(|u| source.hostile_to_phase(u, side))
            .map(|enemy| {
                let mut enemy = enemy.clone();
                enemy.moved = false;
                enemy.attacked = false;
                let mut blockers = occupied.clone();
                if actor.is_some() {
                    blockers.insert(enemy.tile);
                }
                let positions = source.threat_reach_blocked(&enemy, blockers);
                let covered = Self::coverage_engine(source, &enemy, &positions);
                (enemy, covered)
            })
            .collect()
    }
    // Essential actors forecast each opponent independently. Friendly blockers
    // remain, while competing opponents and the hypothetical moving actor can
    // be excluded without cloning or replacing the forecast roster.
    fn independent_enemies(
        board: &Game<'catalog>,
        side: Side,
        excluded: Option<u32>,
    ) -> Vec<(Unit, Vec<TileSet>)> {
        let mut occupied = TileSet::new(board.terrain.len());
        for ally in board
            .units
            .iter()
            .filter(|u| u.side == side && Some(u.id) != excluded)
        {
            occupied.insert(ally.tile);
        }
        board
            .units
            .iter()
            .filter(|u| board.hostile_to_phase(u, side))
            .map(|enemy| {
                let mut blockers = occupied.clone();
                blockers.insert(enemy.tile);
                let positions = board.threat_reach_blocked(enemy, blockers);
                (
                    enemy.clone(),
                    Self::coverage_engine(board, enemy, &positions),
                )
            })
            .collect()
    }
    pub(in crate::ai) fn coverage(
        board: &super::BattleView<'_, '_>,
        unit: &Unit,
        positions: &TileSet,
    ) -> Vec<TileSet> {
        Self::coverage_engine(board.engine(), unit, positions)
    }
    pub(in crate::ai) fn coverage_engine(
        board: &Game,
        unit: &Unit,
        positions: &TileSet,
    ) -> Vec<TileSet> {
        board
            .weapons(unit)
            .map(|attack| {
                if let TileSet::Small(positions) = positions {
                    let masks = board.attack_masks(attack.category, attack.range);
                    let mut origins = *positions;
                    let mut covered = 0;
                    while origins != 0 {
                        covered |= masks[origins.trailing_zeros() as usize];
                        origins &= origins - 1;
                    }
                    return TileSet::Small(covered);
                }
                let geometry = board.attack_origins(attack.category, attack.range);
                let mut covered = TileSet::new(board.terrain.len());
                // Range and intervening-terrain legality are reciprocal. Union
                // targets from reachable origins instead of testing every tile.
                positions.for_each(|origin| covered.union_with(&geometry[origin]));
                covered
            })
            .collect()
    }
    fn pressure(
        board: &Game,
        target: &Unit,
        enemies: &[(Unit, Vec<TileSet>)],
        eliminated: Option<u32>,
        sum: bool,
    ) -> i32 {
        let mut first = 0;
        let mut second = 0;
        let mut total = 0;
        for (enemy, positions) in enemies {
            if Some(enemy.id) == eliminated {
                continue;
            }
            let mut best = 0;
            for weapon in 0..board.weapons(enemy).count() {
                if positions[weapon].contains(target.tile) {
                    best = best.max(if sum {
                        board
                            .damage_distribution(enemy, target, weapon, target.tile)
                            .max
                    } else {
                        board.damage(enemy, target, weapon)
                    });
                }
            }
            total += best;
            if best > first {
                second = first;
                first = best;
            } else if best > second {
                second = best;
            }
        }
        if sum { total } else { first + second }
    }
    /// Risk is nonnegative, so the first zero is the exact global minimum.
    pub fn minimum(&mut self, unit: &Unit, tiles: impl IntoIterator<Item = usize>) -> i32 {
        let mut minimum = i32::MAX;
        for tile in tiles {
            minimum = minimum.min(self.at(unit, tile, None));
            if minimum == 0 {
                break;
            }
        }
        minimum
    }
    pub fn at(&mut self, u: &Unit, tile: usize, eliminated: Option<u32>) -> i32 {
        if self.source.winner.is_some() {
            return 0;
        }
        let important = self.orders.map_or_else(
            || u.is_leader(),
            |orders| orders.important(&super::BattleView::new(self.source), u),
        );
        let key = (u.id, important, tile, eliminated);
        if let Some(&risk) = self.cache.get(&key) {
            return risk;
        }
        let mut target = u.clone();
        target.tile = tile;
        let risk = if self.source.campaign.is_none() {
            // Arena turn-start only resets enemy actions and heals HP. Neither
            // healing nor income changes damage or movement coverage. Essential
            // forecasts ignore competing enemies, so a killed enemy can be
            // subtracted without rebuilding every survivor's forecast.
            let enemies = self
                .arena
                .entry(important.then_some(u.id))
                .or_insert_with(|| {
                    Self::arena_enemies(self.source, self.side, important.then_some(u.id))
                });
            Self::pressure(self.source, &target, enemies, eliminated, important)
        } else if !important {
            let (shadow, enemies) = self.normal.get_or_insert_with(|| {
                let mut shadow = self.source.fork();
                shadow.arena_first = None;
                shadow.end_turn();
                let enemies = shadow
                    .units
                    .iter()
                    .filter(|u| self.source.hostile_to_phase(u, self.side))
                    .map(|u| {
                        let positions = shadow.threat_reach(u, None);
                        (u.clone(), Self::coverage_engine(&shadow, u, &positions))
                    })
                    .collect();
                (shadow, enemies)
            });
            Self::pressure(shadow, &target, enemies, eliminated, false)
        } else {
            let sensitive = self.orders.is_some_and(|orders| {
                orders.pending_arrivals
                    || orders.transition_sensitive.iter().any(|(actor, sites)| {
                        actor.0 == u.id
                            && sites.iter().any(|site| {
                                site.0 == self.source.board.ids[u.tile]
                                    || site.0 == self.source.board.ids[tile]
                            })
                    })
            });
            if sensitive {
                let mut board = self.source.fork();
                board.arena_first = None;
                if let Some(index) = board.index(u.id) {
                    board.units[index].tile = tile;
                }
                board.units.retain(|o| Some(o.id) != eliminated);
                board.roster_changed();
                board.end_turn();
                if board.winner.is_some() {
                    0
                } else {
                    let target = board.unit(u.id).unwrap().clone();
                    let enemies = Self::independent_enemies(&board, u.side, None);
                    Self::pressure(&board, &target, &enemies, None, true)
                }
            } else {
                let fkey = (u.id, eliminated);
                let (board, enemies) = self.forecasts.entry(fkey).or_insert_with(|| {
                    let mut board = self.source.fork();
                    board.arena_first = None;
                    board.units.retain(|o| Some(o.id) != eliminated);
                    board.roster_changed();
                    board.end_turn();
                    let enemies = if board.winner.is_some() {
                        vec![]
                    } else {
                        Self::independent_enemies(&board, u.side, Some(u.id))
                    };
                    (board, enemies)
                });
                Self::pressure(board, &target, enemies, None, true)
            }
        };
        self.cache.insert(key, risk);
        risk
    }
}
impl Drop for Threats<'_, '_> {
    fn drop(&mut self) {
        if let Some(workspace) = &mut self.workspace {
            workspace.arena = std::mem::take(&mut self.arena);
            workspace.risks = std::mem::take(&mut self.cache);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    #[test]
    fn arena_workspace_reuses_hp_only_changes_and_invalidates_roster_changes() {
        let rules = Catalog::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let side = game.turn;
        let mut workspace = ArenaThreatWorkspace::default();
        let enemy = game.units.iter().find(|u| u.side != side).unwrap().id;
        for change in 0..3 {
            if change == 1 {
                let target = game.units.iter_mut().find(|u| u.id == enemy).unwrap();
                target.hp -= 1;
                target.attacked = true;
            } else if change == 2 {
                game.units.retain(|u| u.id != enemy);
                game.roster_changed();
            }
            let view = crate::ai::context::BattleView::new(&game);
            let mut cached = Threats::with_workspace(&view, side, &mut workspace);
            let view = crate::ai::context::BattleView::new(&game);
            let mut fresh = Threats::new(&view, side);
            for unit in game.units.iter().filter(|u| u.side == side) {
                for eliminated in [None, Some(enemy)] {
                    for tile in 0..game.terrain.len() {
                        assert_eq!(
                            cached.at(unit, tile, eliminated),
                            fresh.at(unit, tile, eliminated)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn arena_shortcut_matches_full_turn_forecasts_with_eliminations() {
        let rules = Catalog::load().unwrap();
        for side in [Side::Player, Side::Enemy] {
            let mut game = Game::new(&rules, "mirror");
            game.begin_turn(side);
            let enemy = game.units.iter().find(|u| u.side != side).unwrap().id;
            let ordinary = game
                .units
                .iter()
                .find(|u| u.side == side && !game.essential(u))
                .unwrap();
            let essential = game
                .units
                .iter()
                .find(|u| u.side == side && game.essential(u))
                .unwrap();
            for unit in [ordinary, essential] {
                let important = game.essential(unit);
                let view = crate::ai::context::BattleView::new(&game);
                let mut threats = Threats::new(&view, side);
                for eliminated in [None, Some(enemy)] {
                    let mut shadow = game.fork();
                    shadow.arena_first = None;
                    if important {
                        shadow.units.retain(|u| Some(u.id) != eliminated);
                        shadow.roster_changed();
                    }
                    shadow.end_turn();
                    let enemies: Vec<_> = if important {
                        Threats::independent_enemies(&shadow, side, Some(unit.id))
                    } else {
                        shadow
                            .units
                            .iter()
                            .filter(|u| u.side != side)
                            .map(|u| {
                                let positions = shadow.threat_reach(u, None);
                                (
                                    u.clone(),
                                    Threats::coverage(
                                        &crate::ai::context::BattleView::new(&shadow),
                                        u,
                                        &positions,
                                    ),
                                )
                            })
                            .collect()
                    };
                    for tile in 0..game.terrain.len() {
                        let mut target = unit.clone();
                        target.tile = tile;
                        let expected = Threats::pressure(
                            &shadow,
                            &target,
                            &enemies,
                            if important { None } else { eliminated },
                            important,
                        );
                        assert_eq!(threats.at(unit, tile, eliminated), expected);
                    }
                }
            }
        }
    }
    #[test]
    fn zero_danger_stops_the_minimum_search_without_visiting_later_tiles() {
        let rules = Catalog::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        game.units.retain(|u| u.side == Side::Player);
        game.roster_changed();
        let actor = game.units[0].clone();
        let view = crate::ai::context::BattleView::new(&game);
        let mut threats = Threats::new(&view, actor.side);
        let visits = Cell::new(0);
        let tiles = [actor.tile, actor.tile]
            .into_iter()
            .inspect(|_| visits.set(visits.get() + 1));
        assert_eq!(threats.minimum(&actor, tiles), 0);
        assert_eq!(visits.get(), 1);
        assert_eq!(threats.minimum(&actor, []), i32::MAX);
    }
}
