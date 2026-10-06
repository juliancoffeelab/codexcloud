use crate::{engine::Game, model::*};
use std::collections::HashMap;
pub struct Threats {
    source: Game,
    shadow: Game,
    enemies: Vec<(Unit, Vec<bool>)>,
    cache: HashMap<(u32, usize, usize, Option<u32>), i32>,
    origins: HashMap<(usize, usize, u8), Vec<usize>>,
    forecasts: HashMap<(u32, Option<u32>), (Game, Vec<(Unit, Vec<bool>)>)>,
}
impl Threats {
    pub fn new(g: &Game, side: Side) -> Self {
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
            source: g.fork(),
            shadow,
            enemies,
            cache: HashMap::new(),
            origins: HashMap::new(),
            forecasts: HashMap::new(),
        }
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
                    let eligible = (g.kind == "escort" && u.role.as_deref() == Some("courier"))
                        || (g.kind == "escape" && u.role.as_deref() == Some("leader"))
                        || (g.kind == "vigil" && u.name == g.companion)
                        || (g.kind == "seal"
                            && g.unit_type.as_ref().is_none_or(|k| {
                                self.source.rules.units[self.source.kind(k)].family
                                    == self.source.rules.units[u.kind].family
                            }));
                    eligible
                        && (g.sites().contains(&self.source.board.ids[u.tile])
                            || g.sites().contains(&self.source.board.ids[tile]))
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
                board.end_turn();
                if board.winner.is_some() {
                    0
                } else {
                    let roster = board.units.clone();
                    let target = board.unit(u.id).unwrap().clone();
                    let enemies: Vec<_> = roster
                        .iter()
                        .filter(|e| e.side != u.side)
                        .map(|e| {
                            board.units = roster
                                .iter()
                                .filter(|o| o.side == u.side || o.id == e.id)
                                .cloned()
                                .collect();
                            let mut positions = vec![false; board.terrain.len()];
                            positions[e.tile] = true;
                            for (t, _) in board.reachable(e) {
                                positions[t] = true;
                            }
                            (e.clone(), positions)
                        })
                        .collect();
                    Self::pressure(&board, &target, &enemies, None, true, &mut self.origins)
                }
            } else {
                let fkey = (u.id, eliminated);
                let (board, enemies) = self.forecasts.entry(fkey).or_insert_with(|| {
                    let mut board = self.source.fork();
                    board.arena_first = None;
                    board.units.retain(|o| Some(o.id) != eliminated);
                    board.end_turn();
                    let roster = board.units.clone();
                    let enemies = if board.winner.is_some() {
                        vec![]
                    } else {
                        roster
                            .iter()
                            .filter(|e| e.side != u.side)
                            .map(|e| {
                                board.units = roster
                                    .iter()
                                    .filter(|o| o.id == e.id || (o.side == u.side && o.id != u.id))
                                    .cloned()
                                    .collect();
                                let mut positions = vec![false; board.terrain.len()];
                                positions[e.tile] = true;
                                for (t, _) in board.reachable(e) {
                                    positions[t] = true;
                                }
                                (e.clone(), positions)
                            })
                            .collect()
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
