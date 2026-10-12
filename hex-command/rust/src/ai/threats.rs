use crate::{data::*, logic::Game};
use rustc_hash::FxHashMap as HashMap;
// Dense reachability sets intersect attack origins a machine word at a time.
use crate::logic::geometry::TileSet;
pub(super) struct Threats<'game, 'catalog> {
    source: &'game Game<'catalog>,
    side: Side,
    normal: Option<(Game<'catalog>, Vec<(Unit, Vec<TileSet>)>)>,
    cache: HashMap<(Option<u32>, usize, usize, Option<u32>), i32>,
    forecasts: HashMap<(u32, Option<u32>), (Game<'catalog>, Vec<(Unit, Vec<TileSet>)>)>,
    arena: HashMap<Option<u32>, Vec<(Unit, Vec<TileSet>)>>,
}
impl<'game, 'catalog> Threats<'game, 'catalog> {
    pub fn new(g: &'game Game<'catalog>, side: Side) -> Self {
        Self {
            source: g,
            side,
            normal: None,
            cache: HashMap::default(),
            forecasts: HashMap::default(),
            arena: HashMap::default(),
        }
    }
    fn arena_enemies(source: &Game, side: Side, actor: Option<u32>) -> Vec<(Unit, Vec<TileSet>)> {
        let mut occupancy = vec![None; source.terrain.len()];
        if let Some(actor) = actor {
            for ally in source
                .units
                .iter()
                .filter(|u| u.side == side && u.id != actor)
            {
                occupancy[ally.tile] = Some(side);
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
                let positions = if actor.is_some() {
                    occupancy[enemy.tile] = Some(enemy.side);
                    let positions = source.threat_reach(&enemy, Some(&occupancy));
                    occupancy[enemy.tile] = None;
                    positions
                } else {
                    source.threat_reach(&enemy, None)
                };
                let covered = Self::coverage(source, &enemy, &positions);
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
        let mut occupancy = vec![None; board.terrain.len()];
        for ally in board
            .units
            .iter()
            .filter(|u| u.side == side && Some(u.id) != excluded)
        {
            occupancy[ally.tile] = Some(ally.side);
        }
        board
            .units
            .iter()
            .filter(|u| u.side != side)
            .map(|enemy| {
                let previous = occupancy[enemy.tile];
                occupancy[enemy.tile] = Some(enemy.side);
                let positions = board.threat_reach(enemy, Some(&occupancy));
                occupancy[enemy.tile] = previous;
                (enemy.clone(), Self::coverage(board, enemy, &positions))
            })
            .collect()
    }
    fn coverage(board: &Game, unit: &Unit, positions: &TileSet) -> Vec<TileSet> {
        board.rules.units[unit.kind]
            .attacks
            .iter()
            .map(|attack| {
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
            for weapon in 0..board.rules.units[enemy.kind].attacks.len() {
                if positions[weapon].contains(target.tile) {
                    best = best.max(board.damage(enemy, target, weapon));
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
        let important = self.source.essential(u);
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
            Self::pressure(self.source, &target, enemies, eliminated, important)
        } else if !important {
            let (shadow, enemies) = self.normal.get_or_insert_with(|| {
                let mut shadow = self.source.fork();
                shadow.arena_first = None;
                shadow.end_turn();
                let enemies = shadow
                    .units
                    .iter()
                    .filter(|u| u.side != self.side)
                    .map(|u| {
                        let positions = shadow.threat_reach(u, None);
                        (u.clone(), Self::coverage(&shadow, u, &positions))
                    })
                    .collect();
                (shadow, enemies)
            });
            Self::pressure(shadow, &target, enemies, eliminated, false)
        } else {
            let sensitive = self.source.mission.as_ref().is_some_and(|m| {
                m.goals.iter().any(|g| {
                    let eligible = (matches!(g, Goal::Escort { .. })
                        && u.role == Some(Role::Courier))
                        || (matches!(g, Goal::Escape { .. }) && u.role == Some(Role::Leader))
                        || (matches!(g, Goal::Vigil { .. }) && u.name.as_ref() == g.companion())
                        || (matches!(g, Goal::Seal { .. })
                            && g.unit_type().is_none_or(|k| {
                                self.source.rules.units[self.source.kind(k)].family
                                    == self.source.rules.units[u.kind].family
                            }));
                    eligible
                        && (g.tile().into_iter().chain(g.tiles()).any(|site| {
                            site == &self.source.board.ids[u.tile]
                                || site == &self.source.board.ids[tile]
                        }))
                }) || self.source.turn == Side::Player
                    && m.waves.iter().any(|w| {
                        self.source.round >= w.round
                            && !self
                                .source
                                .campaign
                                .as_ref()
                                .unwrap()
                                .marked(&format!("wave:{}", w.id))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
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
                let mut threats = Threats::new(&game, side);
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
                                (u.clone(), Threats::coverage(&shadow, u, &positions))
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
        let mut threats = Threats::new(&game, actor.side);
        let visits = Cell::new(0);
        let tiles = [actor.tile, actor.tile]
            .into_iter()
            .inspect(|_| visits.set(visits.get() + 1));
        assert_eq!(threats.minimum(&actor, tiles), 0);
        assert_eq!(visits.get(), 1);
        assert_eq!(threats.minimum(&actor, []), i32::MAX);
    }
}
