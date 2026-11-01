//! Alternating-strike state transitions and exact, cached combat forecasts.
use super::damage::{DamageDistribution, rolls};
use rustc_hash::FxHashMap as HashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct Strikes {
    pub profile: u32,
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

#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct ExchangeForecast {
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
struct Key {
    hp: [i32; 2],
    profiles: [u32; 2],
    counts: [u8; 2],
    modifiers: [i32; 2],
    reflections: [i32; 2],
}
impl std::hash::Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let pair = |v: [i32; 2]| u64::from(v[0] as u32) | (u64::from(v[1] as u32) << 32);
        state.write_u64(pair(self.hp));
        state.write_u64(u64::from(self.profiles[0]) << 8 | u64::from(self.counts[0]));
        state.write_u64(u64::from(self.profiles[1]) << 8 | u64::from(self.counts[1]));
        state.write_u64(pair(self.modifiers));
        state.write_u64(pair(self.reflections));
    }
}
type Profile = (i32, i32, u8, u8, i32);
#[derive(Default)]
struct Profiles {
    ids: HashMap<Profile, u32>,
    next: u32,
}
#[derive(Debug)]
pub(super) struct AttackClasses {
    offsets: Vec<usize>,
    ids: Vec<u32>,
}
impl AttackClasses {
    pub(super) fn get(&self, kind: usize, weapon: usize) -> u32 {
        self.ids[self.offsets[kind] + weapon]
    }
}
thread_local! {static PROFILES:RefCell<Profiles>=RefCell::new(Profiles::default());}
// IDs are shared by identical mechanical profiles, including across catalogs.
// Never recycle an ID while an older battle or cached result can retain it.
pub(super) fn attack_classes(rules: &crate::data::Rules) -> Rc<AttackClasses> {
    PROFILES.with_borrow_mut(|profiles| {
        if profiles.ids.len() > 4096 {
            profiles.ids.clear();
        }
        let mut offsets = Vec::with_capacity(rules.units.len());
        let mut ids = Vec::with_capacity(rules.units.iter().map(|u| u.attacks.len()).sum());
        for unit in rules.units.iter() {
            offsets.push(ids.len());
            for attack in unit.attacks.iter() {
                let key = (
                    attack.damage,
                    attack.spread,
                    attack.count,
                    attack.drain,
                    unit.hp,
                );
                let id = if let Some(&id) = profiles.ids.get(&key) {
                    id
                } else {
                    profiles.next = profiles
                        .next
                        .checked_add(1)
                        .expect("combat profile IDs exhausted");
                    let id = profiles.next;
                    profiles.ids.insert(key, id);
                    id
                };
                ids.push(id);
            }
        }
        Rc::new(AttackClasses { offsets, ids })
    })
}
#[derive(Clone, Copy, Default)]
struct Node {
    probability: f64,
    minimum: [i32; 2],
    maximum: [i32; 2],
}
// Values use direct HP indexing. Small key-only tables retain the original
// bucket order, including across growth, so floating-point sums remain stable.
#[derive(Default)]
struct Grid {
    nodes: Vec<Node>,
    occupied: Vec<bool>,
    stride: usize,
    order: rustc_hash::FxHashSet<[i32; 2]>,
    tables: Vec<rustc_hash::FxHashSet<[i32; 2]>>,
    level: usize,
}
impl Grid {
    fn prepare(&mut self, states: usize) {
        if self.nodes.len() < states {
            self.nodes.resize(states, Node::default());
            self.occupied.resize(states, false);
        }
        let mut maximum = self
            .tables
            .iter()
            .map(|table| table.capacity())
            .max()
            .unwrap_or(0)
            .max(self.order.capacity());
        while maximum < states {
            let table =
                rustc_hash::FxHashSet::with_capacity_and_hasher(maximum + 1, Default::default());
            maximum = table.capacity();
            self.tables.push(table);
        }
        if self.order.capacity() == 0 {
            self.order = std::mem::take(&mut self.tables[0]);
        }
    }
    fn index(&self, hp: [i32; 2]) -> usize {
        hp[0] as usize * self.stride + hp[1] as usize
    }
    fn clear(&mut self) {
        for &hp in &self.order {
            let index = self.index(hp);
            self.occupied[index] = false;
        }
        self.order.clear();
        if self.level != 0 {
            std::mem::swap(&mut self.order, &mut self.tables[self.level]);
            std::mem::swap(&mut self.order, &mut self.tables[0]);
            self.level = 0;
        }
    }
    fn reset(&mut self, stride: usize) {
        self.clear();
        self.stride = stride;
    }
    fn insert(&mut self, hp: [i32; 2], node: Node) {
        let index = self.index(hp);
        if self.occupied[index] {
            let entry = &mut self.nodes[index];
            entry.probability += node.probability;
            for side in 0..2 {
                entry.minimum[side] = entry.minimum[side].min(node.minimum[side]);
                entry.maximum[side] = entry.maximum[side].max(node.maximum[side]);
            }
        } else {
            if self.order.len() == self.order.capacity() {
                std::mem::swap(&mut self.order, &mut self.tables[self.level]);
                self.level += 1;
                std::mem::swap(&mut self.order, &mut self.tables[self.level]);
                self.order
                    .extend(self.tables[self.level - 1].iter().copied());
                self.tables[self.level - 1].clear();
            }
            self.order.insert(hp);
            self.occupied[index] = true;
            self.nodes[index] = node;
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[inline]
fn forecast_hash(key: &Key) -> u64 {
    use std::hash::BuildHasher;
    rustc_hash::FxBuildHasher.hash_one(key)
}
#[cfg(not(target_arch = "wasm32"))]
struct ForecastMetadata {
    hash: u64,
    key: Key,
    recent: std::cell::Cell<bool>,
}
#[derive(Default)]
struct ForecastCache {
    #[cfg(target_arch = "wasm32")]
    indices: HashMap<Key, u32>,
    #[cfg(not(target_arch = "wasm32"))]
    indices: hashbrown::HashTable<u32>,
    values: Vec<ExchangeForecast>,
    #[cfg(not(target_arch = "wasm32"))]
    metadata: Vec<ForecastMetadata>,
    #[cfg(not(target_arch = "wasm32"))]
    cursor: usize,
}
impl ForecastCache {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    fn get(&self, key: &Key) -> Option<&ExchangeForecast> {
        #[cfg(target_arch = "wasm32")]
        let &index = self.indices.get(key)?;
        #[cfg(not(target_arch = "wasm32"))]
        let hash = forecast_hash(key);
        #[cfg(not(target_arch = "wasm32"))]
        let &index = self.indices.find(hash, |&i| {
            let meta = &self.metadata[i as usize];
            meta.hash == hash && meta.key == *key
        })?;
        #[cfg(not(target_arch = "wasm32"))]
        self.metadata[index as usize].recent.set(true);
        Some(&self.values[index as usize])
    }
    fn reserve(&mut self, capacity: usize) {
        #[cfg(target_arch = "wasm32")]
        self.indices
            .reserve(capacity.saturating_sub(self.indices.len()));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let metadata = &self.metadata;
            self.indices
                .reserve(capacity.saturating_sub(self.indices.len()), |&i| {
                    metadata[i as usize].hash
                });
            self.metadata
                .reserve(capacity.saturating_sub(self.metadata.len()));
        }
        self.values
            .reserve(capacity.saturating_sub(self.values.len()));
    }
    #[cfg(target_arch = "wasm32")]
    fn insert(&mut self, key: Key, value: ExchangeForecast) {
        if self.values.len() >= 2048 {
            self.indices.clear();
            self.values.clear();
        }
        let index = self.values.len() as u32;
        self.values.push(value);
        self.indices.insert(key, index);
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn insert(&mut self, key: Key, value: ExchangeForecast) {
        let hash = forecast_hash(&key);
        let metadata = ForecastMetadata {
            hash,
            key,
            recent: std::cell::Cell::new(true),
        };
        let index = if self.values.len() < 32768 {
            let index = self.values.len();
            self.values.push(value);
            self.metadata.push(metadata);
            index
        } else {
            // Keep hot matchups without clearing the entire cache. Only small
            // metadata rows participate in lookup, eviction and table rehashing.
            while self.metadata[self.cursor].recent.replace(false) {
                self.cursor = (self.cursor + 1) % self.values.len();
            }
            let index = self.cursor;
            self.indices
                .find_entry(self.metadata[index].hash, |&i| i as usize == index)
                .expect("cached forecast index")
                .remove();
            self.values[index] = value;
            self.metadata[index] = metadata;
            self.cursor = (self.cursor + 1) % self.values.len();
            index
        };
        let metadata = &self.metadata;
        self.indices
            .insert_unique(hash, index as u32, |&i| metadata[i as usize].hash);
    }
}
thread_local! {
    static SCRATCH: RefCell<[Grid;2]> = RefCell::new(Default::default());
    static CACHE: RefCell<ForecastCache> = RefCell::new(ForecastCache::default());
}

/// Reserve forecast storage during battle construction, before any policy call.
pub(super) fn prepare(maximum_hp: i32) {
    let states = (maximum_hp.max(0) as usize + 1).saturating_pow(2);
    SCRATCH.with_borrow_mut(|scratch| {
        for grid in scratch {
            grid.prepare(states);
        }
    });
    CACHE.with_borrow_mut(|cache| {
        cache.reserve(if cfg!(target_arch = "wasm32") {
            2048
        } else {
            32768
        })
    });
}

pub(super) fn forecast(hp: [i32; 2], strikes: [Strikes; 2]) -> ExchangeForecast {
    let key = Key {
        hp,
        profiles: strikes.map(|s| if s.count == 0 { 0 } else { s.profile }),
        counts: strikes.map(|s| s.count),
        modifiers: strikes.map(|s| if s.count == 0 { 0 } else { s.modifier }),
        reflections: strikes.map(|s| if s.count == 0 { 0 } else { s.reflection }),
    };
    if let Some(cached) = CACHE.with_borrow(|cache| cache.get(&key).copied()) {
        return cached;
    }
    let mut result = ExchangeForecast::default();
    let [mut states, mut next] = SCRATCH.with_borrow_mut(std::mem::take);
    let stride = hp[1].max(strikes[1].maximum_hp).max(0) as usize + 1;
    states.reset(stride);
    next.reset(stride);
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
            next.clear();
            for &hp in &states.order {
                let state = states.nodes[states.index(hp)];
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
                    next.insert(
                        remaining,
                        Node {
                            probability: p,
                            minimum,
                            maximum,
                        },
                    );
                };
                if alive {
                    for (damage, p) in distributions[side].outcomes().filter(|&(_, p)| p > 0.0) {
                        advance(damage, p);
                    }
                } else {
                    advance(0, 1.0);
                }
            }
            states.clear();
            std::mem::swap(&mut states, &mut next);
        }
    }
    result.minimum = [i32::MAX; 2];
    result.hp_minimum = [i32::MAX; 2];
    for &hp in &states.order {
        let node = states.nodes[states.index(hp)];
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
    states.clear();
    next.clear();
    SCRATCH.with_borrow_mut(|scratch| *scratch = [states, next]);
    CACHE.with_borrow_mut(|cache| cache.insert(key, result));
    result
}
