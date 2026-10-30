//! Static map templates; owned state is created only at battle and transport boundaries.
use super::*;
use std::borrow::Cow;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature="bake", databake(path=hex_command_catalog::data))]
pub struct TileTemplate {
    pub id: Cow<'static, str>,
    pub col: i32,
    pub row: i32,
    pub terrain: Cow<'static, str>,
    pub owner: Option<Side>,
    pub connections: Option<Cow<'static, [HexDirection]>>,
    pub dead_ends: Option<Cow<'static, [HexDirection]>>,
    pub floor: Option<Cow<'static, str>>,
}
impl From<&Tile> for TileTemplate {
    fn from(value: &Tile) -> Self {
        Self {
            id: value.id.clone().into(),
            col: value.col,
            row: value.row,
            terrain: value.terrain.clone().into(),
            owner: value.owner,
            connections: value.connections.as_ref().map(|items| items.clone().into()),
            dead_ends: value.dead_ends.as_ref().map(|items| items.clone().into()),
            floor: value.floor.as_ref().map(|s| s.clone().into()),
        }
    }
}
impl From<&TileTemplate> for Tile {
    fn from(value: &TileTemplate) -> Self {
        Self {
            id: value.id.to_string(),
            col: value.col,
            row: value.row,
            terrain: value.terrain.to_string(),
            owner: value.owner,
            connections: value.connections.as_ref().map(|items| items.to_vec()),
            dead_ends: value.dead_ends.as_ref().map(|items| items.to_vec()),
            floor: value.floor.as_ref().map(ToString::to_string),
        }
    }
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature="bake", databake(path=hex_command_catalog::data))]
pub struct UnitTemplate {
    pub id: Cow<'static, str>,
    pub r#type: Cow<'static, str>,
    pub side: Side,
    pub tile: Cow<'static, str>,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    pub experience: Option<u32>,
    pub name: Option<Cow<'static, str>>,
    pub role: Option<Role>,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
}
impl From<&PublicUnit> for UnitTemplate {
    fn from(value: &PublicUnit) -> Self {
        Self {
            id: value.id.clone().into(),
            r#type: value.r#type.clone().into(),
            side: value.side,
            tile: value.tile.clone().into(),
            hp: value.hp,
            moved: value.moved,
            attacked: value.attacked,
            experience: value.experience,
            name: value.name.as_ref().map(|s| s.clone().into()),
            role: value.role,
            retreating: value.retreating,
            rebuilding: value.rebuilding,
        }
    }
}
impl From<&UnitTemplate> for PublicUnit {
    fn from(value: &UnitTemplate) -> Self {
        Self {
            id: value.id.to_string(),
            r#type: value.r#type.to_string(),
            side: value.side,
            tile: value.tile.to_string(),
            hp: value.hp,
            moved: value.moved,
            attacked: value.attacked,
            experience: value.experience,
            name: value.name.as_ref().map(ToString::to_string),
            role: value.role,
            retreating: value.retreating,
            rebuilding: value.rebuilding,
        }
    }
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature="bake", databake(path=hex_command_catalog::data))]
pub struct StateTemplate {
    pub armies: Option<[Cow<'static, str>; 2]>,
    pub map_id: Cow<'static, str>,
    pub map_revision: u32,
    pub cols: i32,
    pub rows: i32,
    pub round: u32,
    pub turn: Side,
    pub gold: Gold,
    pub winner: Option<Side>,
    pub next_id: u32,
    pub combat_rng: Option<u32>,
    pub log: Cow<'static, [Cow<'static, str>]>,
    pub tiles: Cow<'static, [TileTemplate]>,
    pub units: Cow<'static, [UnitTemplate]>,
    pub campaign: Option<CampaignState>,
    pub last_combat: Option<Option<LastCombat>>,
}
impl From<&State> for StateTemplate {
    fn from(value: &State) -> Self {
        Self {
            armies: value
                .armies
                .as_ref()
                .map(|a| [a[0].clone().into(), a[1].clone().into()]),
            map_id: value.map_id.clone().into(),
            map_revision: value.map_revision,
            cols: value.cols,
            rows: value.rows,
            round: value.round,
            turn: value.turn,
            gold: value.gold.clone(),
            winner: value.winner,
            next_id: value.next_id,
            combat_rng: value.combat_rng,
            log: value.log.iter().cloned().map(Into::into).collect(),
            tiles: value.tiles.iter().map(TileTemplate::from).collect(),
            units: value.units.iter().map(UnitTemplate::from).collect(),
            campaign: value.campaign.clone(),
            last_combat: value.last_combat.clone(),
        }
    }
}
impl From<&StateTemplate> for State {
    fn from(value: &StateTemplate) -> Self {
        Self {
            armies: value
                .armies
                .as_ref()
                .map(|a| [a[0].to_string(), a[1].to_string()]),
            map_id: value.map_id.to_string(),
            map_revision: value.map_revision,
            cols: value.cols,
            rows: value.rows,
            round: value.round,
            turn: value.turn,
            gold: value.gold.clone(),
            winner: value.winner,
            next_id: value.next_id,
            combat_rng: value.combat_rng,
            log: value.log.iter().map(ToString::to_string).collect(),
            tiles: value.tiles.iter().map(Into::into).collect(),
            units: value.units.iter().map(Into::into).collect(),
            campaign: value.campaign.clone(),
            last_combat: value.last_combat.clone(),
        }
    }
}
impl crate::storage::Snapshot for State {
    type Template = StateTemplate;
    fn freeze(&self) -> Self::Template {
        self.into()
    }
    fn thaw(value: &Self::Template) -> Self {
        value.into()
    }
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature="bake", databake(path=hex_command_catalog::data))]
pub struct MapTemplate {
    pub id: Cow<'static, str>,
    pub revision: u32,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub hidden: bool,
    pub cols: i32,
    pub rows: i32,
    pub terrain: Cow<'static, [TileTemplate]>,
    pub units: Cow<'static, [StartingUnit]>,
}
impl From<&MapDefinition> for MapTemplate {
    fn from(value: &MapDefinition) -> Self {
        Self {
            id: value.id.clone().into(),
            revision: value.revision,
            name: value.name.clone().into(),
            description: value.description.clone().into(),
            hidden: value.hidden,
            cols: value.cols,
            rows: value.rows,
            terrain: value.terrain.iter().map(TileTemplate::from).collect(),
            units: value.units.clone().into(),
        }
    }
}
impl From<&MapTemplate> for MapDefinition {
    fn from(value: &MapTemplate) -> Self {
        Self {
            id: value.id.to_string(),
            revision: value.revision,
            name: value.name.to_string(),
            description: value.description.to_string(),
            hidden: value.hidden,
            cols: value.cols,
            rows: value.rows,
            terrain: value.terrain.iter().map(Into::into).collect(),
            units: value.units.to_vec(),
        }
    }
}
impl crate::storage::Snapshot for MapDefinition {
    type Template = MapTemplate;
    fn freeze(&self) -> Self::Template {
        self.into()
    }
    fn thaw(value: &Self::Template) -> Self {
        value.into()
    }
}
