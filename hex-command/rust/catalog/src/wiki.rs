//! Authored facts for explaining content to people. Never use these records for battle calculations.
//! `own_*` fields describe individual contributions, not a resolved unit in a battlefield.
use crate::*;
use serde::Serialize;

pub struct Wiki<'a> {
    catalogue: &'a Catalogue,
}
impl Catalogue {
    pub fn wiki(&self) -> Wiki<'_> {
        Wiki { catalogue: self }
    }
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiAttack {
    pub id: Text,
    pub name: Text,
    pub sound: Text,
    pub damage: i32,
    pub spread: i32,
    pub count: u8,
    pub drain: u8,
    pub category: Category,
    pub damage_type: DamageType,
    pub range: u8,
}
impl WikiAttack {
    pub(super) fn from_handle(attack: AttackHandle<'_>) -> Self {
        let source = attack.source;
        Self {
            id: source.id.clone(),
            name: source.name.clone(),
            sound: source.sound.clone(),
            damage: source.damage,
            spread: source.spread,
            count: source.count,
            drain: source.drain,
            category: source.category,
            damage_type: source.damage_type,
            range: source.range,
        }
    }
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiAbility {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub effects: Vec<ScopedEffect>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiUnit {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub ability_label: Text,
    pub race: Race,
    pub family: Family,
    pub role: UnitRole,
    pub level: u32,
    pub max_hp: i32,
    pub max_xp: u32,
    pub own_movement: usize,
    pub price: i32,
    pub promotion_ids: Vec<Text>,
    pub own_modifiers: StatModifiers,
    pub own_abilities: Vec<WikiAbility>,
    pub own_attacks: Vec<WikiAttack>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiRace {
    pub id: Race,
    pub name: Text,
    pub own_modifiers: StatModifiers,
    pub own_abilities: Vec<WikiAbility>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiTerrain {
    pub id: Terrain,
    pub role: TerrainRole,
    pub name: Text,
    pub description: Text,
    pub cost: u16,
    pub passable: bool,
    pub income: i32,
    pub capturable: bool,
    pub blocks_shots: bool,
    pub own_modifiers: EnvironmentalModifiers,
    pub own_effects: Vec<ScopedEffect>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiItem {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub own_modifiers: StatModifiers,
    pub own_effects: Vec<ScopedEffect>,
    pub own_attacks: Vec<WikiAttack>,
}
fn ability(source: &crate::schema::AbilityDefinition) -> WikiAbility {
    WikiAbility {
        id: source.id.clone(),
        name: source.name.clone(),
        description: source.description.clone(),
        effects: source.effects.clone(),
    }
}
impl Wiki<'_> {
    pub fn units(&self) -> impl Iterator<Item = WikiUnit> + '_ {
        self.catalogue.units().map(|unit| self.unit(unit.id()))
    }
    pub fn unit(&self, id: &str) -> WikiUnit {
        let unit = self.catalogue.unit(id);
        WikiUnit {
            id: id.into(),
            name: unit.name().into(),
            description: unit.description().into(),
            ability_label: unit.metadata.ability.clone(),
            race: unit.race(),
            family: unit.family(),
            role: unit.role(),
            level: unit.level(),
            max_hp: unit.max_hp,
            max_xp: unit.max_xp,
            own_movement: unit.movement,
            price: unit.price,
            promotion_ids: unit.promotion_ids().to_vec(),
            own_modifiers: unit.modifiers.clone(),
            own_abilities: unit.abilities.iter().map(ability).collect(),
            own_attacks: unit
                .attacks
                .iter()
                .map(|source| WikiAttack::from_handle(AttackHandle { source }))
                .collect(),
        }
    }
    pub fn races(&self) -> impl Iterator<Item = WikiRace> + '_ {
        self.catalogue.races().map(|race| self.race(race.id()))
    }
    pub fn race(&self, id: Race) -> WikiRace {
        let race = self.catalogue.race(id);
        WikiRace {
            id,
            name: race.name().into(),
            own_modifiers: race.modifiers(),
            own_abilities: race.abilities.iter().map(ability).collect(),
        }
    }
    pub fn terrains(&self) -> impl Iterator<Item = WikiTerrain> + '_ {
        self.catalogue
            .terrains()
            .map(|terrain| self.terrain(terrain.id()))
    }
    pub fn terrain(&self, id: Terrain) -> WikiTerrain {
        let terrain = self.catalogue.terrain(id);
        let source = &terrain.source;
        WikiTerrain {
            id,
            role: source.role,
            name: terrain.name().into(),
            description: terrain.description().into(),
            cost: source.cost,
            passable: source.passable,
            income: terrain.income(),
            capturable: terrain.capturable(),
            blocks_shots: terrain.blocks_shots(),
            own_modifiers: source.damage_modifiers.clone(),
            own_effects: source.effects.clone(),
        }
    }
    pub fn items(&self) -> impl Iterator<Item = WikiItem> + '_ {
        self.catalogue.items().map(|item| self.item(item.id()))
    }
    pub fn item(&self, id: &str) -> WikiItem {
        let item = self.catalogue.item(id);
        WikiItem {
            id: id.into(),
            name: item.name().into(),
            description: item.description().into(),
            own_modifiers: item.modifiers(),
            own_effects: item.source.effects.clone(),
            own_attacks: item.attacks().map(WikiAttack::from_handle).collect(),
        }
    }
    pub fn abilities(&self) -> impl Iterator<Item = WikiAbility> + '_ {
        self.catalogue.abilities().map(|a| ability(&a.source))
    }
    pub fn ability(&self, id: &str) -> WikiAbility {
        ability(&self.catalogue.ability(id).source)
    }
    pub fn environment_effects(&self) -> impl Iterator<Item = &EnvironmentDefinition> {
        self.catalogue.environment_effects()
    }
    pub fn environment_effect(&self, id: &str) -> &EnvironmentDefinition {
        self.catalogue.environment_effect(id)
    }
    pub fn map(&self, id: &str, revision: u32) -> &MapDefinition {
        self.catalogue.map(id, revision)
    }
    pub fn campaign(&self, id: &str) -> &CampaignDefinition {
        self.catalogue.campaign(id)
    }
    pub fn maps(&self) -> impl Iterator<Item = &MapDefinition> {
        self.catalogue.maps()
    }
    pub fn campaigns(&self) -> impl Iterator<Item = &CampaignDefinition> {
        self.catalogue.campaigns()
    }
}
