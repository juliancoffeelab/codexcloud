//! Alternating-strike state transitions and exact, cached combat forecasts.
use super::damage::{DamageDistribution, for_strikes};
use hex_command_catalog::AttackOutcomes;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct Strikes {
    pub outcomes: AttackOutcomes,
    pub count: u8,
    pub maximum_hp: i32,
    pub reflection: i32,
}
#[derive(Default, Debug)]
pub(crate) struct StrikeEffect {
    pub effective: i32,
    pub healed: i32,
    pub reflected: i32,
    #[cfg(feature = "wasm")]
    pub thorns: i32,
}
impl Strikes {
    /// Damage, then drain, then reflection; a killing hit can still reflect.
    pub fn apply(self, hp: &mut [i32; 2], side: usize, damage: i32) -> StrikeEffect {
        let other = 1 - side;
        let effective = damage.min(hp[other]);
        hp[other] = (hp[other] - damage).max(0);
        let healed = (effective * i32::from(self.outcomes.drain_percent()) / 100)
            .min((self.maximum_hp - hp[side]).max(0));
        hp[side] += healed;
        let reflected = self.reflection.min(hp[side]);
        hp[side] = (hp[side] - self.reflection).max(0);
        StrikeEffect {
            effective,
            healed,
            reflected,
            #[cfg(feature = "wasm")]
            thorns: self.reflection,
        }
    }
    fn distribution(self) -> Rc<DamageDistribution> {
        for_strikes(self.outcomes, 1)
    }
}

#[derive(Default, Debug)]
pub struct ExchangeForecast {
    /// Initiating weapon damage, and incoming weapon damage plus reflection.
    pub damage: [f64; 2],
    pub minimum: [i32; 2],
    pub maximum: [i32; 2],
    /// Actual weapon HP removed, excluding reflected damage and overkill.
    pub effective: [f64; 2],
    pub healing: [f64; 2],
    pub reflection: f64,
    pub strikes: [f64; 2],
    pub hp_minimum: [i32; 2],
    pub hp_maximum: [i32; 2],
    pub hp_expected: [f64; 2],
    pub death: [f64; 2],
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct ExchangeCacheKey {
    hp: [i32; 2],
    strikes: [ForecastStrikeKey; 2],
}
impl std::hash::Hash for ExchangeCacheKey {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(u64::from(self.hp[0] as u32) | (u64::from(self.hp[1] as u32) << 32));
        for strike in &self.strikes {
            std::hash::Hash::hash(strike, state);
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct ForecastStrikeKey {
    damage: hex_command_catalog::StrikeDistributionKey,
    count: u8,
    drain: u8,
    maximum_hp: i32,
    reflection: i32,
}
impl std::hash::Hash for ForecastStrikeKey {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.damage, state);
        // Fold metadata for hashing only. Equality retains every original
        // bit, including full-width HP/reflection; collisions are ordinary misses.
        state.write_u64(
            (u64::from(self.maximum_hp as u32) | (u64::from(self.reflection as u32) << 32))
                ^ (u64::from(self.count) << 16)
                ^ (u64::from(self.drain) << 24),
        );
    }
}
impl From<Strikes> for ForecastStrikeKey {
    #[inline]
    fn from(strike: Strikes) -> Self {
        Self {
            damage: strike.outcomes.distribution_key(),
            count: strike.count,
            drain: strike.outcomes.drain_percent(),
            maximum_hp: strike.maximum_hp,
            reflection: strike.reflection,
        }
    }
}
#[derive(Clone, Copy)]
struct Node {
    probability: f64,
    minimum: [i32; 2],
    maximum: [i32; 2],
}
thread_local! {
    static CACHE: RefCell<FxHashMap<ExchangeCacheKey, Rc<ExchangeForecast>>> = RefCell::new(FxHashMap::default());
}

pub(crate) fn forecast(hp: [i32; 2], strikes: [Strikes; 2]) -> Rc<ExchangeForecast> {
    let cache = &CACHE;
    let key = ExchangeCacheKey {
        hp,
        strikes: strikes.map(ForecastStrikeKey::from),
    };
    if let Some(cached) = cache.with_borrow(|cache| cache.get(&key).cloned()) {
        return cached;
    }
    let result = calculate_dense(hp, strikes).unwrap_or_else(|| calculate_sparse(hp, strikes));
    let result = Rc::new(result);
    cache.with_borrow_mut(|cache| {
        if cache.len() >= 16384 {
            cache.clear();
        }
        cache.insert(key, result.clone());
    });
    result
}

fn calculate_sparse(hp: [i32; 2], strikes: [Strikes; 2]) -> ExchangeForecast {
    let mut result = ExchangeForecast::default();
    let mut states = FxHashMap::default();
    states.insert(
        hp,
        Node {
            probability: 1.0,
            minimum: [0; 2],
            maximum: [0; 2],
        },
    );
    let distributions = strikes.map(Strikes::distribution);
    for index in 0..strikes[0].count.max(strikes[1].count) {
        for side in 0..2 {
            if index >= strikes[side].count {
                continue;
            }
            let mut next = FxHashMap::<[i32; 2], Node>::default();
            for (hp, state) in states {
                let alive = hp.iter().all(|&h| h > 0);
                let mut advance = |damage, probability| {
                    let p = state.probability * probability;
                    let mut remaining = hp;
                    let mut minimum = state.minimum;
                    let mut maximum = state.maximum;
                    if alive {
                        let effect = strikes[side].apply(&mut remaining, side, damage);
                        result.damage[side] += p * f64::from(damage);
                        result.damage[1 - side] += p * f64::from(strikes[side].reflection);
                        result.effective[side] += p * f64::from(effect.effective);
                        result.healing[side] += p * f64::from(effect.healed);
                        result.reflection += p * f64::from(strikes[side].reflection);
                        result.strikes[side] += p;
                        minimum[side] += damage;
                        maximum[side] += damage;
                        minimum[1 - side] += strikes[side].reflection;
                        maximum[1 - side] += strikes[side].reflection;
                    }
                    next.entry(remaining)
                        .and_modify(|entry| {
                            entry.probability += p;
                            for s in 0..2 {
                                entry.minimum[s] = entry.minimum[s].min(minimum[s]);
                                entry.maximum[s] = entry.maximum[s].max(maximum[s]);
                            }
                        })
                        .or_insert(Node {
                            probability: p,
                            minimum,
                            maximum,
                        });
                };
                if alive {
                    for (damage, p) in distributions[side].outcomes().filter(|&(_, p)| p > 0.0) {
                        advance(damage, p);
                    }
                } else {
                    advance(0, 1.0);
                }
            }
            states = next;
        }
    }
    result.minimum = [i32::MAX; 2];
    result.hp_minimum = [i32::MAX; 2];
    for (hp, node) in &states {
        for (side, &health) in hp.iter().enumerate() {
            result.minimum[side] = result.minimum[side].min(node.minimum[side]);
            result.maximum[side] = result.maximum[side].max(node.maximum[side]);
            result.hp_minimum[side] = result.hp_minimum[side].min(health);
            result.hp_maximum[side] = result.hp_maximum[side].max(health);
            result.hp_expected[side] += node.probability * f64::from(health);
            if health == 0 {
                result.death[side] += node.probability;
            }
        }
    }
    for side in 0..2 {
        result.death[side] = if result.hp_maximum[side] == 0 {
            1.0
        } else {
            result.death[side].clamp(0.0, 1.0)
        };
    }
    result
}

#[derive(Default)]
struct ForecastScratch {
    frontier: [Vec<([i32; 2], Node)>; 2],
    // Upper word identifies the strike generation; lower word the frontier row.
    slots: Vec<u64>,
    generation: u32,
}
impl ForecastScratch {
    fn next_generation(&mut self) -> u32 {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.slots.fill(0);
            self.generation = 1;
        }
        self.generation
    }
}
thread_local! {static FORECAST_SCRATCH:RefCell<ForecastScratch>=RefCell::default();}

// Frontier insertion order determines floating-point accumulation order.
// Tiny score differences can change planner tie decisions.
fn calculate_dense(hp: [i32; 2], strikes: [Strikes; 2]) -> Option<ExchangeForecast> {
    let height = usize::try_from(hp[0].max(strikes[0].maximum_hp))
        .ok()?
        .checked_add(1)?;
    let width = usize::try_from(hp[1].max(strikes[1].maximum_hp))
        .ok()?
        .checked_add(1)?;
    let cells = height.checked_mul(width)?;
    // Unusually large authored HP keeps the sparse algorithm and its full domain.
    if cells > 65_536 || hp.iter().any(|&value| value < 0) {
        return None;
    }
    Some(FORECAST_SCRATCH.with_borrow_mut(|scratch| {
        let mut result = ExchangeForecast::default();
        scratch.frontier[0].clear();
        scratch.frontier[1].clear();
        if scratch.slots.len() < cells {
            scratch.slots.resize(cells, 0);
        }
        let mut current = 0;
        scratch.frontier[0].push((
            hp,
            Node {
                probability: 1.0,
                minimum: [0; 2],
                maximum: [0; 2],
            },
        ));
        let distributions = strikes.map(Strikes::distribution);
        for index in 0..strikes[0].count.max(strikes[1].count) {
            for side in 0..2 {
                if index >= strikes[side].count {
                    continue;
                }
                let next = 1 - current;
                scratch.frontier[next].clear();
                let generation = scratch.next_generation();
                for row in 0..scratch.frontier[current].len() {
                    let (hp, state) = scratch.frontier[current][row];
                    let alive = hp.iter().all(|&h| h > 0);
                    let mut advance = |damage, probability| {
                        let p = state.probability * probability;
                        let mut remaining = hp;
                        let mut minimum = state.minimum;
                        let mut maximum = state.maximum;
                        if alive {
                            let effect = strikes[side].apply(&mut remaining, side, damage);
                            result.damage[side] += p * f64::from(damage);
                            result.damage[1 - side] += p * f64::from(strikes[side].reflection);
                            result.effective[side] += p * f64::from(effect.effective);
                            result.healing[side] += p * f64::from(effect.healed);
                            result.reflection += p * f64::from(strikes[side].reflection);
                            result.strikes[side] += p;
                            minimum[side] += damage;
                            maximum[side] += damage;
                            minimum[1 - side] += strikes[side].reflection;
                            maximum[1 - side] += strikes[side].reflection;
                        }
                        let cell = remaining[0] as usize * width + remaining[1] as usize;
                        let slot = scratch.slots[cell];
                        if (slot >> 32) as u32 == generation {
                            let entry = &mut scratch.frontier[next][slot as u32 as usize].1;
                            entry.probability += p;
                            for s in 0..2 {
                                entry.minimum[s] = entry.minimum[s].min(minimum[s]);
                                entry.maximum[s] = entry.maximum[s].max(maximum[s]);
                            }
                        } else {
                            let row = scratch.frontier[next].len();
                            scratch.slots[cell] = (u64::from(generation) << 32) | row as u64;
                            scratch.frontier[next].push((
                                remaining,
                                Node {
                                    probability: p,
                                    minimum,
                                    maximum,
                                },
                            ));
                        }
                    };
                    if alive {
                        for (damage, p) in distributions[side].outcomes().filter(|&(_, p)| p > 0.0)
                        {
                            advance(damage, p);
                        }
                    } else {
                        advance(0, 1.0);
                    }
                }
                current = next;
            }
        }
        result.minimum = [i32::MAX; 2];
        result.hp_minimum = [i32::MAX; 2];
        for (hp, node) in &scratch.frontier[current] {
            for (side, &health) in hp.iter().enumerate() {
                result.minimum[side] = result.minimum[side].min(node.minimum[side]);
                result.maximum[side] = result.maximum[side].max(node.maximum[side]);
                result.hp_minimum[side] = result.hp_minimum[side].min(health);
                result.hp_maximum[side] = result.hp_maximum[side].max(health);
                result.hp_expected[side] += node.probability * f64::from(health);
                if health == 0 {
                    result.death[side] += node.probability;
                }
            }
        }
        for side in 0..2 {
            result.death[side] = if result.hp_maximum[side] == 0 {
                1.0
            } else {
                result.death[side].clamp(0.0, 1.0)
            };
        }
        result
    }))
}
