use crate::{data::*, logic::Game};
use std::collections::HashMap;
// Dense reachability sets intersect attack origins a machine word at a time.
use crate::logic::geometry::TileSet;
pub(super) struct Threats<'game, 'catalog> {
    source: &'game Game<'catalog>,
    shadow: Game<'catalog>,
    enemies: Vec<(Unit, Vec<TileSet>)>,
    cache: HashMap<(Option<u32>, usize, usize, Option<u32>), i32>,
    forecasts: HashMap<(u32, Option<u32>), (Game<'catalog>, Vec<(Unit, Vec<TileSet>)>)>,
}
impl<'game, 'catalog> Threats<'game, 'catalog> {
    pub fn new(g: &'game Game<'catalog>, side: Side) -> Self {
        let mut shadow = g.fork();
        shadow.arena_first = None;
        shadow.end_turn();
        let enemies = shadow
            .units
            .iter()
            .filter(|u| u.side != side)
            .map(|u| {
                let mut positions = TileSet::new(shadow.terrain.len());
                positions.insert(u.tile);
                for (t, _) in shadow.reachable(u) {
                    positions.insert(t);
                }
                (u.clone(), Self::coverage(&shadow, u, &positions))
            })
            .collect();
        Self {
            source: g,
            shadow,
            enemies,
            cache: HashMap::new(),
            forecasts: HashMap::new(),
        }
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
                let mut positions = TileSet::new(board.terrain.len());
                positions.insert(enemy.tile);
                for (tile, _) in board.reachable_with_occupancy(enemy, &occupancy) {
                    positions.insert(tile);
                }
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
                for (tile, origins) in geometry.iter().enumerate() {
                    if origins.intersects(positions) {
                        covered.insert(tile);
                    }
                }
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
        let risk = if !important {
            Self::pressure(&self.shadow, &target, &self.enemies, eliminated, false)
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
    fn zero_danger_stops_the_minimum_search_without_visiting_later_tiles() {
        let rules = Catalog::embedded().unwrap();
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
