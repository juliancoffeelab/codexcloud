//! Alternating-strike state transitions and exact, cached combat forecasts.
use super::damage::{DamageDistribution, rolls};
use rustc_hash::FxHashMap as HashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct Strikes {
    pub damage: i32,
    pub spread: i32,
    pub count: u8,
    pub modifier: i32,
    pub drain: u8,
    pub maximum_hp: i32,
    pub reflection: i32,
}
#[derive(Default, Debug)]
pub(super) struct StrikeEffect {
    pub effective: i32,
    pub healed: i32,
    pub reflected: i32,
}
impl Strikes {
    /// Damage, then drain, then reflection; a killing hit can still reflect.
    pub fn apply(self, hp: &mut [i32; 2], side: usize, damage: i32) -> StrikeEffect {
        let other = 1 - side;
        let effective = damage.min(hp[other]);
        hp[other] = (hp[other] - damage).max(0);
        let healed =
            (effective * i32::from(self.drain) / 100).min((self.maximum_hp - hp[side]).max(0));
        hp[side] += healed;
        let reflected = self.reflection.min(hp[side]);
        hp[side] = (hp[side] - self.reflection).max(0);
        StrikeEffect {
            effective,
            healed,
            reflected,
        }
    }
    fn distribution(self) -> Rc<DamageDistribution> {
        rolls(self.damage, self.spread, 1, self.modifier)
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
struct Key([u64; 7]);
impl std::hash::Hash for Key {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Keep the integer-hasher path: Hash for a word array otherwise hashes
        // a byte slice, doing unnecessary byte-mixing for these numeric facts.
        for word in self.0 {
            state.write_u64(word);
        }
    }
}
#[inline]
fn pair(first: i32, second: i32) -> u64 {
    u64::from(first as u32) | (u64::from(second as u32) << 32)
}
impl Strikes {
    #[inline]
    fn cache_words(self) -> [u64; 3] {
        // Lossless fields, including every bit of negative signed modifiers.
        // Hash whole words rather than mixing each byte-sized field separately.
        [
            pair(self.damage, self.spread),
            pair(self.modifier, self.maximum_hp),
            u64::from(self.reflection as u32)
                | (u64::from(self.count) << 32)
                | (u64::from(self.drain) << 40),
        ]
    }
}
#[derive(Clone, Copy)]
struct Node {
    probability: f64,
    minimum: [i32; 2],
    maximum: [i32; 2],
}
thread_local! {
    static CACHE: RefCell<HashMap<Key, Rc<ExchangeForecast>>> = RefCell::new(HashMap::default());
}

pub(super) fn forecast(hp: [i32; 2], strikes: [Strikes; 2]) -> Rc<ExchangeForecast> {
    let cache = &CACHE;
    let words = strikes.map(Strikes::cache_words);
    let key = Key([
        pair(hp[0], hp[1]),
        words[0][0],
        words[0][1],
        words[0][2],
        words[1][0],
        words[1][1],
        words[1][2],
    ]);
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
    let mut states = HashMap::default();
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
            let mut next = HashMap::<[i32; 2], Node>::default();
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
