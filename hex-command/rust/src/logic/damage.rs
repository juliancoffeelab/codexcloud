//! Exact independent-strike distributions, shared by UI and policy forecasts.
use crate::data::Weapon;
use rustc_hash::FxHashMap as HashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub(crate) struct DamageDistribution {
    pub min: i32,
    pub max: i32,
    pub mean: f64,
    /// Probability of total damage at index + min.
    pub probabilities: Vec<f64>,
}
impl DamageDistribution {
    pub fn chance(&self, hp: i32) -> f64 {
        if hp <= self.min {
            return 1.0;
        }
        if hp > self.max {
            return 0.0;
        }
        self.probabilities[(hp - self.min) as usize..]
            .iter()
            .sum::<f64>()
            .clamp(0.0, 1.0)
    }
    pub fn effective(&self, hp: i32) -> f64 {
        if hp >= self.max {
            return self.mean;
        }
        self.outcomes()
            .map(|(damage, p)| f64::from(damage.min(hp.max(0))) * p)
            .sum()
    }
    pub fn outcomes(&self) -> impl Iterator<Item = (i32, f64)> + '_ {
        self.probabilities
            .iter()
            .enumerate()
            .map(|(i, &p)| (self.min + i as i32, p))
    }
}
type Key = (i32, i32, u8, i32);
thread_local! {
    static CACHE: RefCell<HashMap<Key, Rc<DamageDistribution>>> = RefCell::new(HashMap::default());
}
pub(crate) fn rounded(attack: &Weapon, modifier: i32) -> i32 {
    if attack.damage - attack.spread + modifier >= 0 {
        // An unclamped uniform roll has an integer mean. Computing that integer
        // directly preserves rounding while avoiding a probability-table query.
        (f64::from(attack.damage + modifier) * f64::from(attack.count)) as i32
    } else if attack.damage + attack.spread + modifier <= 0 {
        0
    } else {
        distribution(attack, modifier).mean.round() as i32
    }
}
pub(crate) fn distribution(attack: &Weapon, modifier: i32) -> Rc<DamageDistribution> {
    rolls(attack.damage, attack.spread, attack.count, modifier)
}
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn rolls(damage: i32, spread: i32, count: u8, modifier: i32) -> Rc<DamageDistribution> {
    let key = (damage, spread, count, modifier);
    CACHE.with_borrow_mut(|cache| {
        if let Some(result) = cache.get(&key) {
            return result.clone();
        }
        let low = (damage - spread).max(0);
        let high = damage + spread;
        let minimum = (low + modifier).max(0);
        let maximum = (high + modifier).max(0);
        let mut single = vec![0.0; (maximum - minimum + 1) as usize];
        for raw in low..=high {
            single[((raw + modifier).max(0) - minimum) as usize] += 1.0 / f64::from(high - low + 1);
        }
        let mut probabilities = vec![1.0];
        for _ in 0..count {
            let mut next = vec![0.0; probabilities.len() + single.len() - 1];
            for (i, &p) in probabilities.iter().enumerate() {
                for (j, &q) in single.iter().enumerate() {
                    next[i + j] += p * q;
                }
            }
            probabilities = next;
        }
        let min = minimum * i32::from(count);
        let max = maximum * i32::from(count);
        let mean = probabilities
            .iter()
            .enumerate()
            .map(|(i, &p)| f64::from(min + i as i32) * p)
            .sum();
        let result = Rc::new(DamageDistribution {
            min,
            max,
            mean,
            probabilities,
        });
        // Content reloads and diagnostics must not grow thread-local memory forever.
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(key, result.clone());
        result
    })
}

#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/damage_profiles.rs"));

#[cfg(target_arch = "wasm32")]
thread_local! {
    static POOL:RefCell<DistributionPool>=RefCell::new(DistributionPool::default());
    static SCRATCH:RefCell<Scratch>=RefCell::new(Scratch::default());
}
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct Scratch {
    single: Vec<f64>,
    previous: Vec<f64>,
    next: Vec<f64>,
}
#[cfg(target_arch = "wasm32")]
impl Scratch {
    fn fill(&mut self, key: Key, result: &mut DamageDistribution) {
        let (damage, spread, count, modifier) = key;
        let low = (damage - spread).max(0);
        let high = damage + spread;
        let minimum = (low + modifier).max(0);
        let maximum = (high + modifier).max(0);
        self.single.clear();
        self.single.resize((maximum - minimum + 1) as usize, 0.0);
        for raw in low..=high {
            self.single[((raw + modifier).max(0) - minimum) as usize] +=
                1.0 / f64::from(high - low + 1);
        }
        self.previous.clear();
        self.previous.push(1.0);
        for _ in 0..count {
            self.next.clear();
            self.next
                .resize(self.previous.len() + self.single.len() - 1, 0.0);
            for (i, &p) in self.previous.iter().enumerate() {
                for (j, &q) in self.single.iter().enumerate() {
                    self.next[i + j] += p * q;
                }
            }
            std::mem::swap(&mut self.previous, &mut self.next);
        }
        result.probabilities.clear();
        result.probabilities.extend_from_slice(&self.previous);
        result.mean = result
            .probabilities
            .iter()
            .enumerate()
            .map(|(i, &p)| f64::from(result.min + i as i32) * p)
            .sum();
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct DistributionPool {
    slots: Vec<Rc<DamageDistribution>>,
    cursor: usize,
}
#[cfg(target_arch = "wasm32")]
impl DistributionPool {
    fn fill(&mut self, key: Key) -> Rc<DamageDistribution> {
        let count = self.slots.len();
        let index = (0..count)
            .map(|n| (self.cursor + n) % count)
            .find(|&i| Rc::strong_count(&self.slots[i]) == 1)
            .unwrap_or_else(|| {
                self.slots.push(Rc::new(DamageDistribution {
                    min: 0,
                    max: 0,
                    mean: 0.0,
                    probabilities: Vec::new(),
                }));
                count
            });
        self.cursor = (index + 1) % self.slots.len();
        let result = Rc::get_mut(&mut self.slots[index]).expect("unleased distribution");
        let (damage, spread, count, modifier) = key;
        result.min = ((damage - spread).max(0) + modifier).max(0) * i32::from(count);
        result.max = (damage + spread + modifier).max(0) * i32::from(count);
        SCRATCH.with_borrow_mut(|scratch| scratch.fill(key, result));
        self.slots[index].clone()
    }
}
#[cfg(target_arch = "wasm32")]
pub(super) fn rolls(damage: i32, spread: i32, count: u8, modifier: i32) -> Rc<DamageDistribution> {
    let key = (damage, spread, count, modifier);
    CACHE.with_borrow_mut(|cache| {
        if let Some(result) = cache.get(&key) {
            return result.clone();
        }
        if cache.len() >= 128 {
            cache.clear();
        }
        let result = POOL.with_borrow_mut(|pool| pool.fill(key));
        cache.insert(key, result.clone());
        result
    })
}
#[cfg(target_arch = "wasm32")]
pub(super) fn prepare() {
    thread_local! {static READY:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
    if READY.with(|ready| ready.replace(true)) {
        return;
    }

    CACHE.with_borrow_mut(|cache| cache.reserve(128));
    let mut maximum = 1;
    for &(damage, spread, count, modifier) in DAMAGE_PROFILES {
        let low = (damage - spread).max(0);
        let high = damage + spread;
        let minimum = (low + modifier).max(0);
        let upper = (high + modifier).max(0);
        maximum = maximum
            .max(((upper - minimum) * i32::from(count) + 1) as usize)
            .max((upper - minimum + 1) as usize);
    }
    POOL.with_borrow_mut(|pool| {
        pool.slots.reserve_exact(144);
        for _ in 0..144 {
            pool.slots.push(Rc::new(DamageDistribution {
                min: 0,
                max: 0,
                mean: 0.0,
                probabilities: Vec::with_capacity(maximum),
            }));
        }
    });
    SCRATCH.with_borrow_mut(|scratch| {
        scratch.single.reserve_exact(maximum);
        scratch.previous.reserve_exact(maximum);
        scratch.next.reserve_exact(maximum);
    });
}
