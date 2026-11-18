//! Exact independent-strike distributions, shared by UI and policy forecasts.
use hex_command_catalog::AttackOutcomes;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct DamageDistribution {
    pub min: i32,
    pub max: i32,
    pub mean: f64,
    /// Probability of total damage at index + min.
    pub probabilities: Vec<f64>,
}
impl DamageDistribution {
    #[inline]
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
    #[inline]
    pub fn effective(&self, hp: i32) -> f64 {
        self.outcomes()
            .map(|(damage, p)| f64::from(damage.min(hp.max(0))) * p)
            .sum()
    }
    #[inline]
    pub fn outcomes(&self) -> impl Iterator<Item = (i32, f64)> + '_ {
        self.probabilities
            .iter()
            .enumerate()
            .map(|(i, &p)| (self.min + i as i32, p))
    }
}
type DamageCacheKey = (hex_command_catalog::StrikeDistributionKey, u8);
thread_local! {
    static CACHE: RefCell<FxHashMap<DamageCacheKey, Rc<DamageDistribution>>> = RefCell::new(FxHashMap::default());
}
pub fn distribution(outcomes: AttackOutcomes) -> Rc<DamageDistribution> {
    for_strikes(outcomes, outcomes.strike_count())
}
pub(super) fn for_strikes(outcomes: AttackOutcomes, count: u8) -> Rc<DamageDistribution> {
    let key = (outcomes.distribution_key(), count);
    CACHE.with_borrow_mut(|cache| {
        if let Some(result) = cache.get(&key) {
            return result.clone();
        }
        let (low, high) = outcomes.bounds();
        let minimum = low.damage;
        let maximum = high.damage;
        let mut single = vec![0.0; (maximum - minimum + 1) as usize];
        outcomes.fill_probabilities(&mut single);
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
