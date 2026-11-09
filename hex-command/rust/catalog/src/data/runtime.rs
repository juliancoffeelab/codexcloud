//! Flat prepared mechanics and numeric live-state references. Wire definitions
//! remain owned by the catalog; queries borrow rows and slices without ownership wrappers.
use super::*;
use std::{
    ops::{Deref, DerefMut},
    sync::OnceLock,
};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnitKind(pub usize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileIndex(pub usize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WeaponKey(pub u32);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}
impl Span {
    pub fn range(self) -> std::ops::Range<usize> {
        self.start..self.start + self.len
    }
}

/// The source definitions own text and authored arrays. One prepared table is
/// rebuilt only after a content edit, before a live game borrows it.
#[derive(Debug)]
pub struct UnitDefinitions {
    source: Vec<UnitRule>,
    prepared: OnceLock<UnitMechanics>,
}
impl Clone for UnitDefinitions {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            prepared: OnceLock::new(),
        }
    }
}
impl From<Vec<UnitRule>> for UnitDefinitions {
    fn from(source: Vec<UnitRule>) -> Self {
        let value = Self {
            source,
            prepared: OnceLock::new(),
        };
        value.mechanics();
        value
    }
}
impl Deref for UnitDefinitions {
    type Target = Vec<UnitRule>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.source
    }
}
impl DerefMut for UnitDefinitions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.prepared.take();
        &mut self.source
    }
}
impl UnitDefinitions {
    pub fn mechanics(&self) -> &UnitMechanics {
        self.prepared
            .get_or_init(|| UnitMechanics::prepare(&self.source))
    }
}
#[cfg(any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")))]
impl crate::frozen::Freeze for UnitDefinitions {
    type Frozen = <Vec<UnitRule> as crate::frozen::Freeze>::Frozen;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        crate::frozen::Freeze::freeze(&self.source)
    }
    fn thaw(value: &Self::Frozen) -> Self {
        Vec::<UnitRule>::thaw(value).into()
    }
}

#[derive(Debug, Default)]
struct WeaponRegistry {
    keys: HashMap<String, WeaponKey>,
    names: Vec<String>,
}
impl WeaponRegistry {
    fn intern(&mut self, name: &str) -> WeaponKey {
        if let Some(key) = self.keys.get(name) {
            return *key;
        }
        let key = WeaponKey(u32::try_from(self.names.len()).expect("weapon identity limit"));
        self.names.push(name.to_owned());
        self.keys.insert(name.to_owned(), key);
        key
    }
}
#[derive(Clone, Debug)]
pub struct UnitMechanics {
    pub units: Vec<UnitStats>,
    pub weapons: Vec<WeaponRules>,
    pub costs: Vec<Option<u16>>,
    pub environment: Vec<[[i32; 3]; 2]>,
    pub advances: Vec<usize>,
    weapon_registry: std::sync::Arc<std::sync::Mutex<WeaponRegistry>>,
    pub weapon_sources: Vec<(usize, usize)>,
}
#[derive(Clone, Debug)]
pub struct UnitStats {
    pub hp: i32,
    pub r#move: usize,
    pub damage: i32,
    pub range: u8,
    pub cost: i32,
    pub attacks: Span,
    pub damage_modifiers: DamageModifiers,
    pub race: Race,
    pub role: UnitRole,
    pub healing: i32,
    pub thorns: i32,
    pub thorn_aura: i32,
    pub level: u32,
    pub family: Family,
    pub experience_needed: Option<u32>,
    pub advances: Span,
    pub costs: usize,
    pub environment: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WeaponRules {
    pub damage: i32,
    pub spread: i32,
    pub count: u8,
    pub damage_type: DamageType,
    pub category: Category,
    pub range: u8,
    pub drain: u8,
    pub identity: WeaponKey,
    pub source: usize,
}
impl WeaponRules {
    fn prepare(value: &Weapon, identity: WeaponKey, source: usize) -> Self {
        Self {
            damage: value.damage,
            spread: value.spread,
            count: value.count,
            damage_type: value.damage_type,
            category: value.category,
            range: value.range,
            drain: value.drain,
            identity,
            source,
        }
    }
    #[inline]
    pub fn key(&self) -> WeaponKey {
        self.identity
    }
    #[inline]
    pub fn matches_key(&self, key: &WeaponKey) -> bool {
        self.identity == *key
    }
}
impl UnitMechanics {
    fn intern_weapon(&self, name: &str) -> WeaponKey {
        self.weapon_registry
            .lock()
            .expect("weapon registry")
            .intern(name)
    }
    pub fn weapon_id(&self, key: WeaponKey) -> String {
        self.weapon_registry
            .lock()
            .expect("weapon registry")
            .names
            .get(key.0 as usize)
            .cloned()
            .unwrap_or_else(|| format!("unknown:{}", key.0))
    }

    fn prepare(source: &[UnitRule]) -> Self {
        let mut result = Self {
            units: Vec::with_capacity(source.len()),
            weapons: Vec::with_capacity(source.iter().map(|u| u.attacks.len()).sum()),
            costs: Vec::with_capacity(source.iter().map(|u| u.costs.len()).sum()),
            environment: Vec::with_capacity(source.iter().map(|u| u.environment.len()).sum()),
            advances: Vec::new(),
            weapon_registry: Default::default(),
            weapon_sources: Vec::new(),
        };
        let kinds: HashMap<_, _> = source
            .iter()
            .enumerate()
            .map(|(i, u)| (u.id.as_str(), i))
            .collect();
        for (kind, unit) in source.iter().enumerate() {
            let attacks = Span {
                start: result.weapons.len(),
                len: unit.attacks.len(),
            };
            for (slot, attack) in unit.attacks.iter().enumerate() {
                let identity = result.intern_weapon(&attack.id);
                let index = result.weapons.len();
                result
                    .weapons
                    .push(WeaponRules::prepare(attack, identity, index));
                result.weapon_sources.push((kind, slot));
            }
            let costs = result.costs.len();
            result.costs.extend_from_slice(&unit.costs);
            let environment = result.environment.len();
            result.environment.extend_from_slice(&unit.environment);
            let advances = Span {
                start: result.advances.len(),
                len: unit.advances_to.len(),
            };
            result
                .advances
                .extend(unit.advances_to.iter().map(|id| kinds[id.as_str()]));
            result.units.push(UnitStats {
                hp: unit.hp,
                r#move: unit.r#move,
                damage: unit.damage,
                range: unit.range,
                cost: unit.cost,
                attacks,
                damage_modifiers: unit.damage_modifiers.clone(),
                race: unit.race,
                role: unit.role,
                healing: unit.healing,
                thorns: unit.thorns,
                thorn_aura: unit.thorn_aura,
                level: unit.level,
                family: unit.family,
                experience_needed: unit.experience_needed,
                advances,
                costs,
                environment,
            });
        }
        result
    }
    #[inline]
    pub fn attacks(&self, kind: usize) -> &[WeaponRules] {
        &self.weapons[self.units[kind].attacks.range()]
    }
    #[inline]
    pub fn environment(&self, kind: usize, terrain: usize) -> &[[i32; 3]; 2] {
        &self.environment[self.units[kind].environment + terrain]
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NameKey(std::num::NonZeroU32);
#[derive(Clone, Debug, Default)]
pub struct Names {
    values: Vec<String>,
    index: HashMap<String, NameKey>,
}
impl Names {
    pub fn find(&self, value: &str) -> Option<NameKey> {
        self.index.get(value).copied()
    }
    pub fn intern(&mut self, value: String) -> NameKey {
        if let Some(key) = self.find(&value) {
            return key;
        }
        let key = NameKey(
            std::num::NonZeroU32::new(u32::try_from(self.values.len() + 1).expect("name limit"))
                .unwrap(),
        );
        self.index.insert(value.clone(), key);
        self.values.push(value);
        key
    }
    #[inline]
    pub fn get(&self, key: NameKey) -> &str {
        &self.values[key.0.get() as usize - 1]
    }
    pub fn resolve(&self, key: Option<NameKey>) -> Option<&str> {
        key.map(|key| self.get(key))
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InventoryId(pub usize);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ItemId(pub usize);
#[derive(Clone, Debug)]
struct Loadout {
    members: Span,
    attacks: Span,
    armor: ItemArmor,
}
#[derive(Clone, Debug, Default)]
struct ItemFacts {
    items: Vec<Item>,
    weapons: Vec<WeaponRules>,
    sources: Vec<(usize, usize)>,
    spans: Vec<Span>,
}
#[derive(Clone, Debug)]
pub struct Inventories {
    facts: std::sync::Arc<ItemFacts>,
    rows: Vec<Loadout>,
    members: Vec<ItemId>,
    attacks: Vec<WeaponRules>,
}
impl Default for Inventories {
    fn default() -> Self {
        Self {
            facts: std::sync::Arc::default(),
            rows: vec![Loadout {
                members: Span::default(),
                attacks: Span::default(),
                armor: ItemArmor {
                    blade: 0,
                    pierce: 0,
                    impact: 0,
                },
            }],
            members: Vec::new(),
            attacks: Vec::new(),
        }
    }
}
impl Inventories {
    pub fn add_item(&mut self, item: Item, base: &UnitMechanics) -> ItemId {
        let facts = std::sync::Arc::make_mut(&mut self.facts);
        let id = ItemId(facts.items.len());
        facts.spans.push(Span {
            start: facts.weapons.len(),
            len: item.attacks.len(),
        });
        for (slot, attack) in item.attacks.iter().enumerate() {
            let identity = base.intern_weapon(&attack.id);
            let index = base.weapons.len() + facts.weapons.len();
            facts
                .weapons
                .push(WeaponRules::prepare(attack, identity, index));
            facts.sources.push((id.0, slot));
        }
        facts.items.push(item);
        id
    }
    pub fn load(&mut self, items: Vec<Item>, base: &UnitMechanics) -> InventoryId {
        if items.is_empty() {
            return InventoryId(0);
        }
        let start = self.members.len();
        for item in items {
            let id = self.add_item(item, base);
            self.members.push(id);
        }
        self.finish(start)
    }
    fn finish(&mut self, start: usize) -> InventoryId {
        let attacks = self.attacks.len();
        let mut armor = ItemArmor {
            blade: 0,
            pierce: 0,
            impact: 0,
        };
        for id in &self.members[start..] {
            let item = &self.facts.items[id.0];
            if let Some(a) = &item.armor {
                armor.blade += a.blade;
                armor.pierce += a.pierce;
                armor.impact += a.impact;
            }
            self.attacks
                .extend_from_slice(&self.facts.weapons[self.facts.spans[id.0].range()]);
        }
        let id = InventoryId(self.rows.len());
        self.rows.push(Loadout {
            members: Span {
                start,
                len: self.members.len() - start,
            },
            attacks: Span {
                start: attacks,
                len: self.attacks.len() - attacks,
            },
            armor,
        });
        id
    }
    pub fn edit(
        &mut self,
        old: InventoryId,
        remove: Option<usize>,
        add: Option<ItemId>,
    ) -> InventoryId {
        let span = self.rows[old.0].members;
        let start = self.members.len();
        for (slot, index) in span.range().enumerate() {
            if Some(slot) != remove {
                self.members.push(self.members[index]);
            }
        }
        if let Some(id) = add {
            self.members.push(id);
        }
        if start == self.members.len() {
            InventoryId(0)
        } else {
            self.finish(start)
        }
    }
    #[inline]
    pub fn item(&self, id: ItemId) -> &Item {
        &self.facts.items[id.0]
    }
    pub fn item_ids(&self, id: InventoryId) -> &[ItemId] {
        &self.members[self.rows[id.0].members.range()]
    }
    pub fn items(&self, id: InventoryId) -> impl ExactSizeIterator<Item = &Item> + Clone {
        self.item_ids(id).iter().map(|id| self.item(*id))
    }
    #[inline]
    pub fn attacks(&self, id: InventoryId) -> &[WeaponRules] {
        &self.attacks[self.rows[id.0].attacks.range()]
    }
    #[inline]
    pub fn modifier(&self, id: InventoryId, kind: DamageType) -> i32 {
        self.rows[id.0].armor.get(kind)
    }
    pub fn source(&self, index: usize, base: &UnitMechanics) -> &Weapon {
        let (item, slot) = self.facts.sources[index - base.weapons.len()];
        &self.facts.items[item].attacks[slot]
    }
}

/// An effective campaign definition. Preparation and simulation edits use the same type.
#[derive(Clone, Debug)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct CampaignRules {
    definition: std::sync::Arc<Campaign>,
}
impl From<Campaign> for CampaignRules {
    fn from(definition: Campaign) -> Self {
        Self {
            definition: std::sync::Arc::new(definition),
        }
    }
}
impl std::ops::Deref for CampaignRules {
    type Target = Campaign;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}
impl std::ops::DerefMut for CampaignRules {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        std::sync::Arc::make_mut(&mut self.definition)
    }
}
impl CampaignRules {
    pub fn snapshot(&self) -> Campaign {
        (*self.definition).clone()
    }
    #[inline]
    pub fn same_definition(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.definition, &other.definition)
    }
    #[inline]
    pub fn identity(&self) -> usize {
        std::sync::Arc::as_ptr(&self.definition) as usize
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GroundItemState {
    pub tile: usize,
    pub item: ItemId,
}

impl<'a> IntoIterator for &'a UnitDefinitions {
    type Item = &'a UnitRule;
    type IntoIter = std::slice::Iter<'a, UnitRule>;
    fn into_iter(self) -> Self::IntoIter {
        self.source.iter()
    }
}
impl<'a> IntoIterator for &'a mut UnitDefinitions {
    type Item = &'a mut UnitRule;
    type IntoIter = std::slice::IterMut<'a, UnitRule>;
    fn into_iter(self) -> Self::IntoIter {
        self.prepared.take();
        self.source.iter_mut()
    }
}
