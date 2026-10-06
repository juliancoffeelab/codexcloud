use crate::{data::*, logic::Game};
use std::collections::HashMap;
pub(super) struct Threats<'game, 'catalog> {
    source: &'game Game<'catalog>,
    shadow: Game<'catalog>,
    enemies: Vec<(Unit, Vec<bool>)>,
    cache: HashMap<(u32, usize, usize, Option<u32>), i32>,
    origins: HashMap<(usize, usize, u8), Vec<usize>>,
    forecasts: HashMap<(u32, Option<u32>), (Game<'catalog>, Vec<(Unit, Vec<bool>)>)>,
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
                let mut positions = vec![false; shadow.terrain.len()];
                positions[u.tile] = true;
                for (t, _) in shadow.reachable(u) {
                    positions[t] = true;
                }
                (u.clone(), positions)
            })
            .collect();
        Self {
            source: g,
            shadow,
            enemies,
            cache: HashMap::new(),
            origins: HashMap::new(),
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
    ) -> Vec<(Unit, Vec<bool>)> {
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
                let mut positions = vec![false; board.terrain.len()];
                positions[enemy.tile] = true;
                for (tile, _) in board.reachable_with_occupancy(enemy, &occupancy) {
                    positions[tile] = true;
                }
                occupancy[enemy.tile] = previous;
                (enemy.clone(), positions)
            })
            .collect()
    }
    fn pressure(
        board: &Game,
        target: &Unit,
        enemies: &[(Unit, Vec<bool>)],
        eliminated: Option<u32>,
        sum: bool,
        origins: &mut HashMap<(usize, usize, u8), Vec<usize>>,
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
                let attack = &board.rules.units[enemy.kind].attacks[weapon];
                let from = origins
                    .entry((target.tile, attack.category.index(), attack.range))
                    .or_insert_with(|| {
                        (0..board.terrain.len())
                            .filter(|&t| board.can_attack(enemy, t, target, weapon))
                            .collect()
                    });
                if from.iter().any(|&t| positions[t]) {
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
    pub fn at(&mut self, u: &Unit, tile: usize, eliminated: Option<u32>) -> i32 {
        let key = (u.id, u.kind, tile, eliminated);
        if let Some(&risk) = self.cache.get(&key) {
            return risk;
        }
        let mut target = u.clone();
        target.tile = tile;
        let risk = if !self.source.essential(u) {
            Self::pressure(
                &self.shadow,
                &target,
                &self.enemies,
                eliminated,
                false,
                &mut self.origins,
            )
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
                    Self::pressure(&board, &target, &enemies, None, true, &mut self.origins)
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
                Self::pressure(board, &target, enemies, None, true, &mut self.origins)
            }
        };
        self.cache.insert(key, risk);
        risk
    }
}
