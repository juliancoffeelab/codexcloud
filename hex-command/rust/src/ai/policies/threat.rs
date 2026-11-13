//! Exposure policy, actor isolation and signature-owned threat caches.
use crate::{ai::context::BattleView, data::*};
use hex_command_battle::{PhaseForecasts, PhaseSnapshot};
use rustc_hash::FxHashMap as HashMap;
// Dense reachability sets intersect attack origins a machine word at a time.
use crate::logic::geometry::TileSet;
// A kind is not a combat identity: two units of that kind can carry different
// armor/weapons. Use the actual actor, and invalidate retained results whenever
// any roster member's equipment changes (including the attacking enemies).
type RiskKey = (u32, bool, usize, Option<u32>);
struct PotentialThreat {
    unit: Unit,
    origins: TileSet,
    coverage: Vec<TileSet>,
}
type ArenaEnemies = HashMap<Option<u32>, Vec<PotentialThreat>>;
type ArenaUnit = (
    u32,
    usize,
    usize,
    TurnPhase,
    Option<Role>,
    NameKey,
    InventoryId,
);
/// A nonlethal attack changes HP/action flags, but arena movement forecasts and
/// incoming damage also depend on equipment, allegiance and actor roles.
/// Retain those calculations for the next decision in the same planning phase.
#[derive(Default)]
pub(in crate::ai) struct ArenaThreatWorkspace {
    terrain: Vec<usize>,
    roster: Vec<ArenaUnit>,
    side: Option<TurnPhase>,
    arena: ArenaEnemies,
    risks: HashMap<RiskKey, i32>,
}
pub(in crate::ai) struct Threats<'game, 'catalog> {
    orders: Option<&'game crate::ai::orders::TacticalOrders>,
    source: &'game BattleView<'game, 'catalog>,
    phases: PhaseForecasts<'game, 'catalog>,
    side: TurnPhase,
    normal: Option<(PhaseSnapshot<'catalog>, Vec<PotentialThreat>)>,
    cache: HashMap<RiskKey, i32>,
    forecasts: HashMap<(u32, Option<u32>), (PhaseSnapshot<'catalog>, Vec<PotentialThreat>)>,
    arena: ArenaEnemies,
    workspace: Option<&'game mut ArenaThreatWorkspace>,
}
impl<'game, 'catalog> Threats<'game, 'catalog> {
    /// Potential exposure without an allied screen, so a successful screen does
    /// not cause Campaigner to release its escort on the following turn.
    pub(in crate::ai) fn unshielded(phases: PhaseForecasts<'_, 'catalog>, unit: &Unit) -> i32 {
        let board = phases.next_phase();
        if board.winner().is_some() {
            return 0;
        }
        let Some(target) = board.unit(unit.id) else {
            return 0;
        };
        let enemies: Vec<_> = board
            .units()
            .iter()
            .filter(|u| board.hostile(target, u))
            .map(|enemy| {
                let mut blocked = TileSet::new(board.terrain().len());
                blocked.insert(enemy.tile);
                let reach = board.threat_reach_blocked(enemy, blocked);
                PotentialThreat {
                    unit: *enemy,
                    coverage: Self::coverage_engine(&board, enemy, &reach),
                    origins: reach,
                }
            })
            .collect();
        Self::pressure(&board, target, &enemies, None, true)
    }
    pub fn new(phases: PhaseForecasts<'game, 'catalog>, side: TurnPhase) -> Self {
        Self {
            orders: None,
            source: phases.view(),
            phases,
            side,
            normal: None,
            cache: HashMap::default(),
            forecasts: HashMap::default(),
            arena: HashMap::default(),
            workspace: None,
        }
    }
    pub(in crate::ai) fn protected(
        phases: PhaseForecasts<'game, 'catalog>,
        side: TurnPhase,
        orders: &'game crate::ai::orders::TacticalOrders,
    ) -> Self {
        let mut threats = Self::new(phases, side);
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
        phases: PhaseForecasts<'game, 'catalog>,
        side: TurnPhase,
        workspace: &'game mut ArenaThreatWorkspace,
    ) -> Self {
        let g = phases.view();
        let mut threats = Self::new(phases, side);
        if !g.is_campaign() && !g.perception_sensitive() {
            let same = workspace.side == Some(side)
                && workspace.terrain == *g.terrain()
                && workspace.roster.len() == g.units().len()
                && workspace.roster.iter().zip(g.units()).all(
                    |((id, kind, tile, side, role, faction, inventory), u)| {
                        (*id, *kind, *tile, *side) == (u.id, u.kind, u.tile, u.side)
                            && *role == u.role
                            && *faction == u.faction
                            && *inventory == u.inventory
                    },
                );
            if !same {
                workspace.side = Some(side);
                workspace.terrain.clone_from(g.terrain());
                workspace.roster.clear();
                workspace.roster.extend(
                    g.units()
                        .iter()
                        .map(|u| (u.id, u.kind, u.tile, u.side, u.role, u.faction, u.inventory)),
                );
                workspace.arena.clear();
                workspace.risks.clear();
            }
            threats.arena = std::mem::take(&mut workspace.arena);
            threats.cache = std::mem::take(&mut workspace.risks);
            threats.workspace = Some(workspace);
        }
        threats
    }
    fn arena_enemies(
        source: &BattleView<'_, '_>,
        side: TurnPhase,
        actor: Option<u32>,
    ) -> Vec<PotentialThreat> {
        let mut occupied = if actor.is_some() {
            TileSet::new(source.terrain().len())
        } else {
            source.occupied_tiles().clone()
        };
        if let Some(actor) = actor {
            for ally in source
                .units()
                .iter()
                .filter(|u| u.side == side && u.id != actor)
            {
                occupied.insert(ally.tile);
            }
        }
        source
            .units()
            .iter()
            .filter(|u| source.hostile_to_phase(u, side))
            .map(|enemy| {
                let mut enemy = enemy.clone();
                enemy.moved = false;
                enemy.attack_spent = false;
                let mut blockers = occupied.clone();
                if actor.is_some() {
                    blockers.insert(enemy.tile);
                }
                let positions = source.threat_reach_blocked(&enemy, blockers);
                let covered = Self::coverage_engine(source, &enemy, &positions);
                PotentialThreat {
                    unit: enemy,
                    coverage: covered,
                    origins: positions,
                }
            })
            .collect()
    }
    // Essential actors forecast each opponent independently. Friendly blockers
    // remain, while competing opponents and the hypothetical moving actor can
    // be excluded without cloning or replacing the forecast roster.
    fn independent_enemies(
        board: &BattleView<'_, 'catalog>,
        side: TurnPhase,
        excluded: Option<u32>,
    ) -> Vec<PotentialThreat> {
        let mut occupied = TileSet::new(board.terrain().len());
        for ally in board
            .units()
            .iter()
            .filter(|u| u.side == side && Some(u.id) != excluded)
        {
            occupied.insert(ally.tile);
        }
        board
            .units()
            .iter()
            .filter(|u| board.hostile_to_phase(u, side))
            .map(|enemy| {
                let mut blockers = occupied.clone();
                blockers.insert(enemy.tile);
                let positions = board.threat_reach_blocked(enemy, blockers);
                PotentialThreat {
                    unit: *enemy,
                    coverage: Self::coverage_engine(board, enemy, &positions),
                    origins: positions,
                }
            })
            .collect()
    }
    pub(in crate::ai) fn coverage(
        board: &BattleView<'_, '_>,
        unit: &Unit,
        positions: &TileSet,
    ) -> Vec<TileSet> {
        Self::coverage_engine(board, unit, positions)
    }
    pub(in crate::ai) fn coverage_engine(
        board: &BattleView<'_, '_>,
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
                let mut covered = TileSet::new(board.terrain().len());
                // Range and intervening-terrain legality are reciprocal. Union
                // targets from reachable origins instead of testing every tile.
                positions.for_each(|origin| covered.union_with(&geometry[origin]));
                covered
            })
            .collect()
    }
    fn pressure(
        board: &BattleView<'_, '_>,
        target: &Unit,
        enemies: &[PotentialThreat],
        eliminated: Option<u32>,
        sum: bool,
    ) -> i32 {
        let mut first = 0;
        let mut second = 0;
        let mut total = 0;
        for threat in enemies {
            let enemy = &threat.unit;
            let positions = &threat.coverage;
            if Some(enemy.id) == eliminated {
                continue;
            }
            let mut best = 0;
            for weapon in 0..board.weapons(enemy).count() {
                if positions[weapon].contains(target.tile)
                    && board.can_threaten_from(enemy, target, weapon, &threat.origins)
                {
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
    pub fn at(&mut self, u: &Unit, tile: usize, eliminated: Option<u32>) -> i32 {
        if self.source.winner().is_some() {
            return 0;
        }
        let important = self
            .orders
            .map_or_else(|| u.is_leader(), |orders| orders.important(self.source, u));
        let key = (u.id, important, tile, eliminated);
        if let Some(&risk) = self.cache.get(&key) {
            return risk;
        }
        let mut target = u.clone();
        target.tile = tile;
        let risk = if !self.source.is_campaign() {
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
                let shadow = self.phases.next_phase();
                let enemies = shadow
                    .units()
                    .iter()
                    .filter(|u| shadow.hostile_to_phase(u, self.side))
                    .map(|u| {
                        let positions =
                            shadow.threat_reach_blocked(u, shadow.occupied_tiles().clone());
                        PotentialThreat {
                            unit: *u,
                            coverage: Self::coverage_engine(&shadow, u, &positions),
                            origins: positions,
                        }
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
                                site.0 == self.source.board().ids[u.tile]
                                    || site.0 == self.source.board().ids[tile]
                            })
                    })
            });
            if sensitive {
                let board = self.phases.next_phase_with_actor(u.id, tile, eliminated);
                if board.winner().is_some() {
                    0
                } else {
                    board.unit(u.id).map_or(0, |target| {
                        let enemies = Self::independent_enemies(&board, u.side, None);
                        Self::pressure(&board, target, &enemies, None, true)
                    })
                }
            } else {
                let fkey = (u.id, eliminated);
                let (board, enemies) = self.forecasts.entry(fkey).or_insert_with(|| {
                    let board = self.phases.next_phase_without(eliminated);
                    let enemies = if board.winner().is_some() {
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
    use crate::logic::Game;
    #[test]
    fn arena_workspace_reuses_hp_only_changes_and_invalidates_roster_changes() {
        let rules = Catalog::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let side = game.turn();
        let mut workspace = ArenaThreatWorkspace::default();
        let enemy = game.units().iter().find(|u| u.side != side).unwrap().id;
        for change in 0..3 {
            if change != 0 {
                let mut state = game.state();
                if change == 1 {
                    let target = state
                        .units
                        .iter_mut()
                        .find(|u| u.id == format!("u{enemy}"))
                        .unwrap();
                    target.hp -= 1;
                    target.attack_spent = true;
                } else {
                    state.units.retain(|u| u.id != format!("u{enemy}"));
                }
                game = Game::try_from_state(&rules, state).unwrap();
            }
            let mut cached = Threats::with_workspace(game.phase_forecasts(), side, &mut workspace);
            let mut fresh = Threats::new(game.phase_forecasts(), side);
            for unit in game.units().iter().filter(|u| u.side == side) {
                for eliminated in [None, Some(enemy)] {
                    for tile in 0..game.terrain().len() {
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
        for side in [TurnPhase::Player, TurnPhase::Ai] {
            let mut game = Game::new(&rules, "mirror");
            game.begin_turn(side);
            let enemy = game.units().iter().find(|u| u.side != side).unwrap().id;
            let ordinary = game
                .units()
                .iter()
                .find(|u| u.side == side && !u.is_leader())
                .unwrap();
            let essential = game
                .units()
                .iter()
                .find(|u| u.side == side && u.is_leader())
                .unwrap();
            for unit in [ordinary, essential] {
                let important = unit.is_leader();
                let mut threats = Threats::new(game.phase_forecasts(), side);
                for eliminated in [None, Some(enemy)] {
                    let shadow = if important {
                        game.phase_forecasts().next_phase_without(eliminated)
                    } else {
                        game.phase_forecasts().next_phase()
                    };
                    let enemies: Vec<_> = if important {
                        Threats::independent_enemies(&shadow, side, Some(unit.id))
                    } else {
                        shadow
                            .units()
                            .iter()
                            .filter(|u| u.side != side)
                            .map(|u| {
                                let positions =
                                    shadow.threat_reach_blocked(u, shadow.occupied_tiles().clone());
                                PotentialThreat {
                                    unit: *u,
                                    coverage: Threats::coverage(&shadow, u, &positions),
                                    origins: positions,
                                }
                            })
                            .collect()
                    };
                    for tile in 0..game.terrain().len() {
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
}

/// Exposure beyond the actor's remaining life cannot represent an additional loss.
pub(in crate::ai) fn cap_exposure(unit: &Unit, incoming: i32, important: bool) -> i32 {
    if important {
        incoming.min(unit.hp.max(1))
    } else {
        incoming
    }
}
