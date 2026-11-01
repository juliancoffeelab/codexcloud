use crate::reuse::Map as HashMap;
use crate::reuse::{Vec, vec};
use crate::{data::*, logic::Game};
use std::rc::Rc;
// Dense reachability sets intersect attack origins a machine word at a time.
use crate::logic::geometry::TileSet;
type RiskKey = (Option<u32>, usize, usize, Option<u32>);
type Coverage = smallvec::SmallVec<[TileSet; 3]>;
#[derive(Clone, Copy)]
struct Enemy {
    id: u32,
    kind: usize,
}
impl From<&Unit> for Enemy {
    fn from(unit: &Unit) -> Self {
        Self {
            id: unit.id,
            kind: unit.kind,
        }
    }
}
struct Forecast {
    terrain: Rc<[usize]>,
    enemies: Vec<(Enemy, Coverage)>,
}
type ArenaEnemies = HashMap<Option<u32>, Vec<(Enemy, Coverage)>>;
/// A nonlethal attack changes HP/action flags, but arena movement forecasts and
/// incoming damage depend only on the roster's identities, kinds and positions.
/// Retain those calculations for the next decision in the same planning phase.
#[derive(Default)]
pub(in crate::ai) struct ArenaThreatWorkspace {
    terrain: Option<Rc<[usize]>>,
    roster: Vec<(u32, usize, usize, Side)>,
    side: Option<Side>,
    arena: ArenaEnemies,
    risks: HashMap<RiskKey, i32>,
    #[cfg(not(target_arch = "wasm32"))]
    campaign: Option<[usize; 10]>,
    #[cfg(not(target_arch = "wasm32"))]
    round: u32,
    #[cfg(not(target_arch = "wasm32"))]
    holds: std::vec::Vec<(String, u32, u32)>,
    #[cfg(not(target_arch = "wasm32"))]
    owners: std::vec::Vec<Option<Side>>,
    #[cfg(not(target_arch = "wasm32"))]
    normal: Option<Forecast>,
    #[cfg(not(target_arch = "wasm32"))]
    forecasts: crate::reuse::ScopedMap<(u32, Option<u32>), Forecast, 32>,
}
pub(in crate::ai) struct Threats<'game, 'catalog> {
    orders: Option<&'game crate::ai::orders::TacticalOrders>,
    source: &'game Game<'catalog>,
    side: Side,
    normal: Option<Forecast>,
    cache: HashMap<RiskKey, i32>,
    forecasts: crate::reuse::ScopedMap<(u32, Option<u32>), Forecast, 32>,
    arena: ArenaEnemies,
    workspace: Option<&'game mut ArenaThreatWorkspace>,
}
impl<'game, 'catalog> Threats<'game, 'catalog> {
    /// Potential exposure without an allied screen, so a successful screen does
    /// not cause Campaigner to release its escort on the following turn.
    pub(in crate::ai) fn unshielded(game: &super::BattleView<'_, 'catalog>, unit: &Unit) -> i32 {
        let game = game.engine();
        let mut board = game.forecast_fork();
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
            .filter(|u| u.side != unit.side)
            .map(|enemy| {
                let mut blocked = TileSet::new(board.terrain.len());
                blocked.insert(enemy.tile);
                let reach = board.threat_reach_blocked(enemy, blocked);
                (
                    Enemy::from(enemy),
                    Self::coverage_engine(&board, enemy, &reach),
                )
            })
            .collect();
        Self::pressure(&board, &board.terrain, target, &enemies, None, true)
    }
    pub fn new(g: &'game super::BattleView<'_, 'catalog>, side: Side) -> Self {
        let g = g.engine();
        Self {
            orders: None,
            source: g,
            side,
            normal: None,
            cache: HashMap::default(),
            forecasts: crate::reuse::ScopedMap::default(),
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
        #[cfg(not(target_arch = "wasm32"))]
        let campaign = g.campaign.as_ref().map(|c| {
            [
                g.mission.as_ref().map_or(0, |m| Rc::as_ptr(m) as usize),
                c.mission,
                c.marks.len(),
                c.choices.as_ref().map_or(0, |v| v.len()),
                c.fallen_companions.as_ref().map_or(0, |v| v.len()),
                c.refuge
                    .as_ref()
                    .map_or(usize::MAX, |v| v.held_turns as usize),
                c.refuge
                    .as_ref()
                    .map_or(usize::MAX, |v| v.last_counted_round as usize),
                c.refuge.as_ref().map_or(0, |v| v.escalated as usize),
                c.vigil
                    .as_ref()
                    .map_or(usize::MAX, |v| v.held_turns as usize),
                c.vigil
                    .as_ref()
                    .map_or(usize::MAX, |v| v.last_counted_round as usize),
            ]
        });
        #[cfg(not(target_arch = "wasm32"))]
        let reusable = true;
        #[cfg(target_arch = "wasm32")]
        let reusable = g.campaign.is_none();
        if reusable {
            #[cfg(not(target_arch = "wasm32"))]
            let campaign_same = workspace.campaign == campaign
                && (campaign.is_none()
                    || (workspace.round == g.round
                        && workspace.owners == g.owners
                        && workspace
                            .holds
                            .iter()
                            .map(|(key, held, last)| (key.as_str(), *held, *last))
                            .eq(g
                                .campaign
                                .iter()
                                .filter_map(|c| c.holds.as_ref())
                                .flat_map(|holds| holds.iter())
                                .map(|(key, v)| {
                                    (key.as_str(), v.held_turns, v.last_counted_round)
                                }))));
            #[cfg(target_arch = "wasm32")]
            let campaign_same = true;
            let same_scene = campaign_same
                && workspace.side == Some(side)
                && workspace
                    .terrain
                    .as_ref()
                    .is_some_and(|terrain| Rc::ptr_eq(terrain, &g.terrain));
            let same = same_scene
                && workspace.roster.len() == g.units.len()
                && workspace
                    .roster
                    .iter()
                    .zip(&g.units)
                    .all(|(&(id, kind, tile, side), u)| {
                        (id, kind, tile, side) == (u.id, u.kind, u.tile, u.side)
                    });
            #[cfg(not(target_arch = "wasm32"))]
            let moved_actor = if !same && same_scene && workspace.roster.len() == g.units.len() {
                let mut moved = None;
                let mut valid = true;
                for (&(id, kind, tile, owner), unit) in workspace.roster.iter().zip(&g.units) {
                    if (id, kind, owner) != (unit.id, unit.kind, unit.side) {
                        valid = false;
                        break;
                    }
                    if tile != unit.tile {
                        if moved.is_some() || owner != side {
                            valid = false;
                            break;
                        }
                        moved = Some(id);
                    }
                }
                valid.then_some(moved).flatten()
            } else {
                None
            };
            if !same {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    workspace.campaign = campaign;
                    workspace.round = g.round;
                    workspace.owners.clone_from(&g.owners);
                    workspace.holds.clear();
                    workspace.holds.extend(
                        g.campaign
                            .iter()
                            .filter_map(|c| c.holds.as_ref())
                            .flat_map(|holds| holds.iter())
                            .map(|(key, v)| (key.clone(), v.held_turns, v.last_counted_round)),
                    );
                    workspace.normal = None;
                    // Independent forecasts exclude their actor from blockers.
                    // Moving that actor preserves its forecasts; every other
                    // positional forecast must be rebuilt.
                    if let Some(actor) = moved_actor {
                        workspace.forecasts.retain(|key| key.0 == actor);
                    } else {
                        workspace.forecasts = Default::default();
                    }
                }
                workspace.side = Some(side);
                workspace.terrain = Some(g.terrain.clone());
                workspace.roster.clear();
                workspace
                    .roster
                    .extend(g.units.iter().map(|u| (u.id, u.kind, u.tile, u.side)));
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(actor) = moved_actor {
                    workspace.arena.retain(|key, _| *key == Some(actor));
                    workspace.risks.retain(|key, _| key.0 == Some(actor));
                } else {
                    workspace.arena.clear();
                    workspace.risks.clear();
                }
                #[cfg(target_arch = "wasm32")]
                {
                    workspace.arena.clear();
                    workspace.risks.clear();
                }
            }
            threats.arena = std::mem::take(&mut workspace.arena);
            threats.cache = std::mem::take(&mut workspace.risks);
            #[cfg(not(target_arch = "wasm32"))]
            {
                threats.normal = workspace.normal.take();
                threats.forecasts = std::mem::take(&mut workspace.forecasts);
                if campaign.is_some() {
                    threats.cache.clear();
                }
            }
            threats.workspace = Some(workspace);
        }
        threats
    }
    fn arena_enemies(source: &Game, side: Side, actor: Option<u32>) -> Vec<(Enemy, Coverage)> {
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
            .filter(|u| u.side != side)
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
                (Enemy::from(&enemy), covered)
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
    ) -> Vec<(Enemy, Coverage)> {
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
            .filter(|u| u.side != side)
            .map(|enemy| {
                let mut blockers = occupied.clone();
                blockers.insert(enemy.tile);
                let positions = board.threat_reach_blocked(enemy, blockers);
                (
                    Enemy::from(enemy),
                    Self::coverage_engine(board, enemy, &positions),
                )
            })
            .collect()
    }
    pub(in crate::ai) fn coverage(
        board: &super::BattleView<'_, '_>,
        unit: &Unit,
        positions: &TileSet,
    ) -> Coverage {
        Self::coverage_engine(board.engine(), unit, positions)
    }
    pub(in crate::ai) fn coverage_engine(
        board: &Game,
        unit: &Unit,
        positions: &TileSet,
    ) -> Coverage {
        board.rules.units[unit.kind]
            .attacks
            .iter()
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
        terrain: &[usize],
        target: &Unit,
        enemies: &[(Enemy, Coverage)],
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
            for weapon in 0..board.rules.units[enemy.kind].attacks.len() {
                if positions[weapon].contains(target.tile) {
                    best = best.max(if sum {
                        board.maximum_damage(enemy.kind, target, weapon, terrain[target.tile])
                    } else {
                        board.expected_damage_kind(enemy.kind, target, weapon, terrain[target.tile])
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
            || self.source.rules.units[u.kind].leader,
            |orders| orders.important(&super::BattleView::new(self.source), u),
        );
        let key = (important.then_some(u.id), u.kind, tile, eliminated);
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
            Self::pressure(
                self.source,
                &self.source.terrain,
                &target,
                enemies,
                eliminated,
                important,
            )
        } else if !important {
            let forecast = self.normal.get_or_insert_with(|| {
                let mut shadow = self.source.forecast_fork();
                shadow.arena_first = None;
                shadow.end_turn();
                let enemies = shadow
                    .units
                    .iter()
                    .filter(|u| u.side != self.side)
                    .map(|u| {
                        let positions = shadow.threat_reach(u, None);
                        (
                            Enemy::from(u),
                            Self::coverage_engine(&shadow, u, &positions),
                        )
                    })
                    .collect();
                Forecast {
                    terrain: shadow.terrain.clone(),
                    enemies,
                }
            });
            Self::pressure(
                self.source,
                &forecast.terrain,
                &target,
                &forecast.enemies,
                eliminated,
                false,
            )
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
                let mut board = self.source.forecast_fork();
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
                    Self::pressure(&board, &board.terrain, &target, &enemies, None, true)
                }
            } else {
                let fkey = (u.id, eliminated);
                let forecast = self.forecasts.entry(fkey).or_insert_with(|| {
                    let mut board = self.source.forecast_fork();
                    board.arena_first = None;
                    board.units.retain(|o| Some(o.id) != eliminated);
                    board.roster_changed();
                    board.end_turn();
                    let enemies = if board.winner.is_some() {
                        vec![]
                    } else {
                        Self::independent_enemies(&board, u.side, Some(u.id))
                    };
                    Forecast {
                        terrain: board.terrain.clone(),
                        enemies,
                    }
                });
                Self::pressure(
                    self.source,
                    &forecast.terrain,
                    &target,
                    &forecast.enemies,
                    None,
                    true,
                )
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
            #[cfg(not(target_arch = "wasm32"))]
            {
                workspace.normal = self.normal.take();
                workspace.forecasts = std::mem::take(&mut self.forecasts);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reuse::vec;
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
                                    Enemy::from(u),
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
                            &shadow.terrain,
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

#[cfg(target_arch = "wasm32")]
pub(in crate::ai) fn prepare_storage(tiles: usize, weapons: usize) {
    crate::reuse::prepare_maps::<RiskKey, i32>(tiles * 4, 2);
    let _ = weapons;
    crate::reuse::prepare::<(Enemy, Coverage)>(12, tiles + 1);
}
