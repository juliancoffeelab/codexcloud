//! Exact independent-strike distributions, shared by UI and policy forecasts.
use crate::data::Weapon;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct DamageDistribution {
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
    static CACHE: RefCell<HashMap<Key, Rc<DamageDistribution>>> = RefCell::new(HashMap::new());
}
pub fn distribution(attack: &Weapon, modifier: i32) -> Rc<DamageDistribution> {
    rolls(attack.damage, attack.spread, attack.count, modifier)
}
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
