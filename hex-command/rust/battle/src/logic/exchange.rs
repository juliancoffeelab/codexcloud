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
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Key {
    hp: [i32; 2],
    strikes: [Strikes; 2],
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
    let key = Key { hp, strikes };
    if let Some(cached) = CACHE.with_borrow(|cache| cache.get(&key).cloned()) {
        return cached;
    }
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
    let result = Rc::new(result);
    CACHE.with_borrow_mut(|cache| {
        if cache.len() >= 16384 {
            cache.clear();
        }
        cache.insert(key, result.clone());
    });
    result
}
