//! Derived source indexes; positions and eligibility are always checked live.
use super::BattlePosition;
use crate::data::*;
#[derive(Clone, Copy)]
#[repr(usize)]
pub(crate) enum EffectKind {
    Healing,
    Thorns,
    Sight,
    Concealment,
    Light,
    Stats,
}
#[derive(Debug)]
pub(crate) struct EffectSourceIndex {
    pub units: [Vec<u32>; 6],
    pub all_units: rustc_hash::FxHashSet<u32>,
    pub unit_mask: u8,
    pub unit_domains: rustc_hash::FxHashMap<u32, u8>,
    pub terrain_domains: Vec<u8>,
    pub all_terrain: Vec<usize>,
    pub terrain: [Vec<usize>; 6],
    pub movement_cache: Option<std::rc::Rc<MovementCache>>,
    pub sight_profiles: std::cell::RefCell<
        rustc_hash::FxHashMap<SightKey, (Option<u16>, Vec<crate::data::ConcealmentKind>)>,
    >,
    pub movement: bool,
    pub damage: bool,
}
/// Evaluated Catalogue results; usable only while no spatial movement effects exist.
/// Inventory contents, rather than branch-local inventory IDs, identify equipment.
#[derive(Debug)]
pub(crate) struct MovementCache {
    pub limits:
        std::cell::RefCell<rustc_hash::FxHashMap<(usize, crate::data::NameKey, usize), usize>>,
    terrain: Vec<usize>,
    modifiers: std::rc::Rc<crate::data::StatModifiers>,
    factions: std::rc::Rc<Vec<crate::data::Faction>>,
    pub costs: std::cell::RefCell<
        rustc_hash::FxHashMap<(usize, crate::data::NameKey, usize), std::rc::Rc<Vec<u16>>>,
    >,
    pub classes: std::cell::RefCell<
        rustc_hash::FxHashMap<(usize, crate::data::NameKey, usize, usize), usize>,
    >,
}
/// Pointer identity backed by a strong owner: COW edits cannot reuse the key.
#[derive(Clone, Debug)]
pub(crate) struct Snapshot<T>(pub std::rc::Rc<T>);
impl<T> PartialEq for Snapshot<T> {
    fn eq(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for Snapshot<T> {}
impl<T> std::hash::Hash for Snapshot<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&(std::rc::Rc::as_ptr(&self.0) as usize), state)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SightKey {
    pub kind: usize,
    pub equipment: usize,
    pub own_relation: (bool, bool),
    pub group: crate::data::SightContribution,
    pub modifiers: Snapshot<crate::data::StatModifiers>,
    pub perception: Vec<crate::data::PerceptionEffect>,
    pub illumination: crate::data::Illumination,
}
impl EffectSourceIndex {
    pub(crate) fn new() -> Self {
        Self {
            units: std::array::from_fn(|_| Vec::new()),
            all_units: rustc_hash::FxHashSet::default(),
            unit_mask: 0,
            unit_domains: rustc_hash::FxHashMap::default(),
            terrain_domains: Vec::new(),
            all_terrain: Vec::new(),
            terrain: std::array::from_fn(|_| Vec::new()),
            movement_cache: None,
            sight_profiles: std::cell::RefCell::new(rustc_hash::FxHashMap::default()),
            movement: false,
            damage: false,
        }
    }
}
fn kind(effect: &EffectValue) -> EffectKind {
    match effect {
        EffectValue::Healing { .. } => EffectKind::Healing,
        EffectValue::Thorns { .. } => EffectKind::Thorns,
        EffectValue::Sight { .. } => EffectKind::Sight,
        EffectValue::Concealment { .. } => EffectKind::Concealment,
        EffectValue::Light { .. } => EffectKind::Light,
        EffectValue::StatModifiers { .. } => EffectKind::Stats,
    }
}
fn flags(effect: &EffectValue, index: &mut EffectSourceIndex) {
    if let EffectValue::StatModifiers { modifiers } = effect {
        index.movement |= !modifiers.terrain_modifiers.is_empty();
        let d = &modifiers.damage_modifiers;
        index.damage |= [d.blade, d.pierce, d.impact].iter().any(|v| *v != 0)
            || modifiers.terrain_modifiers.values().any(|m| {
                let d = &m.damage_modifiers;
                [d.melee, d.ranged, d.blade, d.pierce, d.impact]
                    .iter()
                    .any(|v| *v != 0)
            });
    }
}
impl BattlePosition<'_> {
    pub(crate) fn refresh_effect_sources(&mut self) {
        let mut index = EffectSourceIndex::new();
        index.terrain_domains.resize(self.terrain.len(), 0);
        for unit in &self.runtime.units {
            let mut present = [false; 6];
            let owned = self.definitions.owned(unit, &self.runtime.inventories);
            for &effect in &owned.effects {
                present[kind(effect.value()) as usize] = true;
                flags(effect.value(), &mut index);
                if let EffectValue::StatModifiers { modifiers } = effect.value() {
                    let sight = &modifiers.sight;
                    present[EffectKind::Sight as usize] |= sight.range != 0
                        || sight.unlimited
                        || sight.sight_modifiers.lit != 0
                        || sight.sight_modifiers.unlit != 0
                        || sight.sight_modifiers.darkness != 0
                        || !sight.penetrates.is_empty();
                }
            }
            present[EffectKind::Light as usize] |= self.unit_perception[unit.perception.0]
                .effects
                .iter()
                .any(|effect| effect.light.is_some());
            let domains = present
                .iter()
                .enumerate()
                .fold(0u8, |mask, (i, p)| mask | (u8::from(*p) << i));
            index.unit_mask |= domains;
            if domains != 0 {
                index.all_units.insert(unit.id);
                index.unit_domains.insert(unit.id, domains);
            }
            for (kind, present) in present.into_iter().enumerate() {
                if present {
                    index.units[kind].push(unit.id);
                }
            }
        }
        for origin in 0..self.terrain.len() {
            let mut present = [false; 6];
            for effect in self.terrain_definition(origin).effects().chain(
                self.tile_effects[origin]
                    .iter()
                    .map(EffectHandle::from_value),
            ) {
                present[kind(effect.value()) as usize] = true;
                flags(effect.value(), &mut index);
                if let EffectValue::StatModifiers { modifiers } = effect.value() {
                    let sight = &modifiers.sight;
                    present[EffectKind::Sight as usize] |= sight.range != 0
                        || sight.unlimited
                        || sight.sight_modifiers.lit != 0
                        || sight.sight_modifiers.unlit != 0
                        || sight.sight_modifiers.darkness != 0
                        || !sight.penetrates.is_empty();
                }
            }
            let domains = present
                .iter()
                .enumerate()
                .fold(0u8, |mask, (i, p)| mask | (u8::from(*p) << i));
            index.terrain_domains[origin] = domains;
            if domains != 0 {
                index.all_terrain.push(origin);
            }
            for (kind, present) in present.into_iter().enumerate() {
                if present {
                    index.terrain[kind].push(origin);
                }
            }
        }
        if !index.movement {
            index.movement_cache = Some(match &self.effect_sources.movement_cache {
                Some(cache)
                    if cache.terrain == self.terrain
                        && cache.modifiers == self.map_modifiers
                        && std::rc::Rc::ptr_eq(&cache.factions, &self.factions) =>
                {
                    std::rc::Rc::clone(cache)
                }
                _ => std::rc::Rc::new(MovementCache {
                    limits: std::cell::RefCell::new(rustc_hash::FxHashMap::default()),
                    terrain: self.terrain.clone(),
                    modifiers: std::rc::Rc::clone(&self.map_modifiers),
                    factions: std::rc::Rc::clone(&self.factions),
                    costs: std::cell::RefCell::new(rustc_hash::FxHashMap::default()),
                    classes: std::cell::RefCell::new(rustc_hash::FxHashMap::default()),
                }),
            });
        }
        self.runtime.effect_sources = std::rc::Rc::new(index);
    }
}
