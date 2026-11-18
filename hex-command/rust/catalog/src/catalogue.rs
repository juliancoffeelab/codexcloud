//! Definition objects own authored data. Their storage is never a public transport contract.
use crate::schema;
#[cfg(feature = "wasm")]
include!(concat!(env!("OUT_DIR"), "/documents.rs"));
use crate::{
    Category, ContentError, DamageModifiers, DamageType, EffectScope, EffectValue, JsonBundle,
    Race, StackingRule, StatModifiers, Terrain, Text, UnitCapabilities, UnitInputs,
};

pub struct Catalogue {
    pub(crate) instance_identity: usize,
    pub(crate) combat_preparations: std::sync::Mutex<
        rustc_hash::FxHashMap<
            crate::prepared_combat::CombatPreparationKey,
            std::sync::Arc<crate::PreparedCombat>,
        >,
    >,
    pub(crate) meta: crate::CatalogueMeta,
    /// SHA-256 of ordered document paths and exact source bytes.
    pub content_fingerprint: [u8; 32],
    pub(crate) units: Vec<UnitDefinition>,
    pub(crate) races: Vec<RaceDefinition>,
    pub(crate) terrain: Vec<TerrainDefinition>,
    pub(crate) items: Vec<ItemDefinition>,
    pub(crate) abilities: Vec<AbilityDefinition>,
    pub(crate) environment: Vec<crate::EnvironmentDefinition>,
    pub(crate) maps: Vec<crate::MapDefinition>,
    pub(crate) campaigns: Vec<crate::CampaignDefinition>,
    pub(crate) concealment: Vec<crate::ConcealmentDefinition>,
}
impl Catalogue {
    #[inline]
    pub fn meta(&self) -> &crate::CatalogueMeta {
        &self.meta
    }
    pub fn load() -> Result<Self, ContentError> {
        crate::loader::load_from(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data"))
    }
    pub fn load_from(root: impl AsRef<std::path::Path>) -> Result<Self, ContentError> {
        crate::loader::load_from(root)
    }
    pub fn from_json_bundle(bundle: &JsonBundle) -> Result<Self, ContentError> {
        crate::loader::parse(bundle)
    }
    /// Embedded storage must be constructed inside this crate, never through public raw records.
    #[cfg(feature = "wasm")]
    pub fn embedded() -> Self {
        let bundle = EMBEDDED_DOCUMENTS
            .iter()
            .map(|(path, source)| ((*path).to_owned(), (*source).to_owned()))
            .collect();
        Self::from_json_bundle(&bundle).expect("embedded current Catalogue parses")
    }
    #[inline]
    pub fn unit(&self, id: &str) -> &UnitDefinition {
        self.units
            .iter()
            .find(|v| v.id() == id)
            .expect("known unit definition")
    }
    #[inline]
    pub fn race(&self, id: Race) -> &RaceDefinition {
        self.races
            .iter()
            .find(|v| v.id() == id)
            .expect("known race definition")
    }
    #[inline]
    pub fn terrain(&self, id: Terrain) -> &TerrainDefinition {
        self.terrain
            .iter()
            .find(|v| v.id() == id)
            .expect("known terrain definition")
    }
    #[inline]
    pub fn item(&self, id: &str) -> &ItemDefinition {
        self.items
            .iter()
            .find(|v| v.id() == id)
            .expect("known item definition")
    }
    #[inline]
    pub fn ability(&self, id: &str) -> &AbilityDefinition {
        self.abilities
            .iter()
            .find(|v| v.id() == id)
            .expect("known ability definition")
    }
    #[inline]
    pub fn environment_effect(&self, id: &str) -> &crate::EnvironmentDefinition {
        self.environment
            .iter()
            .find(|v| v.id() == id)
            .expect("known environment definition")
    }
    #[inline]
    pub fn map(&self, id: &str, revision: u32) -> &crate::MapDefinition {
        self.maps
            .iter()
            .find(|v| v.id() == id && v.revision() == revision)
            .expect("known map definition")
    }
    #[inline]
    pub fn campaign(&self, id: &str) -> &crate::CampaignDefinition {
        self.campaigns
            .iter()
            .find(|v| v.id() == id)
            .expect("known campaign definition")
    }
    #[inline]
    pub fn concealment_kind(&self, id: crate::ConcealmentKind) -> &crate::ConcealmentDefinition {
        self.concealment
            .iter()
            .find(|v| v.id() == id)
            .expect("known concealment kind")
    }
    #[inline]
    pub fn concealment_kinds(&self) -> impl Iterator<Item = &crate::ConcealmentDefinition> {
        self.concealment.iter()
    }
    #[inline]
    pub fn units(&self) -> impl Iterator<Item = &UnitDefinition> {
        self.units.iter()
    }
    #[inline]
    pub fn races(&self) -> impl Iterator<Item = &RaceDefinition> {
        self.races.iter()
    }
    #[inline]
    pub fn terrains(&self) -> impl Iterator<Item = &TerrainDefinition> {
        self.terrain.iter()
    }
    #[inline]
    pub fn items(&self) -> impl Iterator<Item = &ItemDefinition> {
        self.items.iter()
    }
    #[inline]
    pub fn abilities(&self) -> impl Iterator<Item = &AbilityDefinition> {
        self.abilities.iter()
    }
    #[inline]
    pub fn environment_effects(&self) -> impl Iterator<Item = &crate::EnvironmentDefinition> {
        self.environment.iter()
    }
    #[inline]
    pub fn maps(&self) -> impl Iterator<Item = &crate::MapDefinition> {
        self.maps.iter()
    }
    #[inline]
    pub fn campaigns(&self) -> impl Iterator<Item = &crate::CampaignDefinition> {
        self.campaigns.iter()
    }
}

pub struct UnitDefinition {
    pub(crate) instance_identity: usize,
    pub max_hp: i32,
    pub max_xp: u32,
    pub(crate) metadata: schema::UnitMetadata,
    pub(crate) movement: usize,
    pub(crate) price: i32,
    pub(crate) modifiers: StatModifiers,
    pub(crate) abilities: Vec<schema::AbilityDefinition>,
    pub(crate) attacks: Vec<schema::Weapon>,
    pub(crate) racial: schema::RaceDefinition,
    pub(crate) racial_abilities: Vec<schema::AbilityDefinition>,
}
impl UnitDefinition {
    #[inline]
    pub fn id(&self) -> &str {
        &self.metadata.id
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.metadata.name
    }
    #[inline]
    pub fn description(&self) -> &str {
        &self.metadata.description
    }
    #[inline]
    pub fn race(&self) -> Race {
        self.metadata.race
    }
    #[inline]
    pub fn family(&self) -> crate::Family {
        self.metadata.family
    }
    #[inline]
    pub fn role(&self) -> crate::UnitRole {
        self.metadata.role
    }
    #[inline]
    pub fn level(&self) -> u32 {
        self.metadata.level
    }
    #[inline]
    pub fn promotion_ids(&self) -> &[Text] {
        &self.metadata.advances_to
    }
    #[inline]
    pub fn recruitment_price(&self, explicit_price: Option<i32>) -> i32 {
        match explicit_price {
            Some(v) => v,
            None => self.price,
        }
    }
    /// Opaque owned attack identities/geometry in unit-then-equipment order.
    /// Numeric combat queries remain on composed capabilities/outcomes.
    #[inline]
    pub fn attacks<'a: 's, 's>(
        &'a self,
        equipment: &'s [&'a ItemDefinition],
    ) -> impl Iterator<Item = AttackHandle<'a>> + Clone + 's {
        self.attacks
            .iter()
            .chain(equipment.iter().flat_map(|item| &item.source.attacks))
            .map(|source| AttackHandle { source })
    }
    /// Enumerates only owned emissions. Incoming effects are never re-emitted.
    #[inline]
    pub fn effects<'a: 's, 's>(
        &'a self,
        equipment: &'s [&'a ItemDefinition],
        gained: &'s [&'a AbilityDefinition],
    ) -> impl Iterator<Item = EffectHandle<'a>> + Clone + 's {
        self.racial_abilities
            .iter()
            .chain(&self.abilities)
            .flat_map(|a| &a.effects)
            .chain(equipment.iter().flat_map(|i| &i.source.effects))
            .chain(gained.iter().flat_map(|a| &a.source.effects))
            .map(EffectHandle::from_value)
    }
    #[inline]
    pub fn compose<'a>(&'a self, inputs: UnitInputs<'a>) -> UnitCapabilities<'a> {
        UnitCapabilities::new(self, inputs)
    }
}
pub struct RaceDefinition {
    pub(crate) source: schema::RaceDefinition,
    pub(crate) abilities: Vec<schema::AbilityDefinition>,
}
impl RaceDefinition {
    #[inline]
    pub fn id(&self) -> Race {
        self.source.id
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.source.name
    }
    #[cfg(feature = "wasm")]
    #[inline]
    pub(crate) fn modifiers(&self) -> StatModifiers {
        StatModifiers {
            sight: self.source.sight.clone(),
            damage_modifiers: self.source.damage_modifiers.clone(),
            terrain_modifiers: self
                .source
                .terrain_modifiers
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
        }
    }
    #[inline]
    pub fn effects(&self) -> impl Iterator<Item = EffectHandle<'_>> {
        self.abilities
            .iter()
            .flat_map(|a| &a.effects)
            .map(EffectHandle::from_value)
    }
}
pub struct TerrainDefinition {
    pub(crate) instance_identity: usize,
    pub(crate) source: schema::TerrainDefinition,
}
impl TerrainDefinition {
    #[inline]
    pub fn id(&self) -> Terrain {
        self.source.id
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.source.name
    }
    #[inline]
    pub fn description(&self) -> &str {
        &self.source.description
    }
    #[inline]
    pub fn effects(&self) -> impl Iterator<Item = EffectHandle<'_>> {
        self.source.effects.iter().map(EffectHandle::from_value)
    }
    /// Tests the authored terrain restriction only. Battle evaluates illumination and detection.
    #[inline]
    pub fn matches_concealment_requirements(
        &self,
        requirements: &crate::ConcealmentRequirements,
    ) -> bool {
        match &requirements.terrains {
            None => true,
            Some(roles) => roles.contains(&self.source.role),
        }
    }
    /// Legacy terrain classification for Classic's category-specific heuristics.
    #[deprecated(
        note = "Classic-only heuristic; use contextual mechanics for new policy decisions"
    )]
    #[inline]
    pub fn matches(&self, role: crate::TerrainRole) -> bool {
        self.source.role == role
    }
    /// Recruitment-site classification; battle checks ownership, leader, funds and placement.
    #[inline]
    pub fn allows_recruitment(&self) -> bool {
        self.source.role == crate::TerrainRole::Keep
    }
    #[inline]
    pub fn capturable(&self) -> bool {
        self.source.capturable
    }
    #[inline]
    pub fn income(&self) -> i32 {
        self.source.income
    }
    #[inline]
    pub fn blocks_shots(&self) -> bool {
        self.source.blocks_shots
    }
}
pub struct ItemDefinition {
    pub(crate) instance_identity: usize,
    pub(crate) source: schema::Item,
}
impl ItemDefinition {
    #[inline]
    pub fn id(&self) -> &str {
        &self.source.id
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.source.name
    }
    #[inline]
    pub fn description(&self) -> &str {
        &self.source.description
    }
    #[inline]
    pub fn attacks(&self) -> impl Iterator<Item = AttackHandle<'_>> {
        self.source
            .attacks
            .iter()
            .map(|source| AttackHandle { source })
    }
    #[inline]
    pub fn effects(&self) -> impl Iterator<Item = EffectHandle<'_>> {
        self.source.effects.iter().map(EffectHandle::from_value)
    }
    #[cfg(feature = "wasm")]
    #[inline]
    pub(crate) fn modifiers(&self) -> StatModifiers {
        StatModifiers {
            sight: self.source.sight.clone(),
            damage_modifiers: match &self.source.armor {
                Some(a) => DamageModifiers {
                    blade: a.blade,
                    pierce: a.pierce,
                    impact: a.impact,
                },
                None => DamageModifiers {
                    blade: 0,
                    pierce: 0,
                    impact: 0,
                },
            },
            terrain_modifiers: std::collections::BTreeMap::new(),
        }
    }
}
pub struct AbilityDefinition {
    pub(crate) source: schema::AbilityDefinition,
}
impl AbilityDefinition {
    #[inline]
    pub fn id(&self) -> &str {
        &self.source.id
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.source.name
    }
    #[inline]
    pub fn description(&self) -> &str {
        &self.source.description
    }
    #[inline]
    pub fn effects(&self) -> impl Iterator<Item = EffectHandle<'_>> {
        self.source.effects.iter().map(EffectHandle::from_value)
    }
}
#[derive(Clone, Copy)]
pub struct AttackHandle<'a> {
    pub(crate) source: &'a schema::Weapon,
}
impl<'a> AttackHandle<'a> {
    #[inline]
    pub fn id(&self) -> &'a str {
        &self.source.id
    }
    #[inline]
    pub fn name(&self) -> &'a str {
        &self.source.name
    }
    #[inline]
    pub fn sound(&self) -> &'a str {
        &self.source.sound
    }
    #[inline]
    pub fn category(&self) -> Category {
        self.source.category
    }
    #[inline]
    pub fn range(&self) -> u8 {
        self.source.range
    }
    #[inline]
    pub fn damage_type(&self) -> DamageType {
        self.source.damage_type
    }
}
#[derive(Clone, Copy)]
pub struct EffectHandle<'a> {
    source: &'a crate::ScopedEffect,
}
impl<'a> EffectHandle<'a> {
    /// Accepts authored event values as well as effects owned by definitions.
    #[inline]
    pub fn from_value(source: &'a crate::ScopedEffect) -> Self {
        Self { source }
    }
    #[inline]
    pub fn scope(&self) -> &EffectScope {
        &self.source.scope
    }
    #[inline]
    pub fn stacking(&self) -> &StackingRule {
        &self.source.stacking
    }
    #[inline]
    pub fn value(&self) -> &EffectValue {
        &self.source.effect
    }
}
