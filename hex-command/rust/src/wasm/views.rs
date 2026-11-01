//! Borrowed wire projections: publication does not construct an owned snapshot.
use super::*;
use serde::ser::{SerializeSeq, SerializeStruct};

pub(super) struct Handle<'a> {
    pub id: u32,
    pub view: View<'a>,
}
impl Serialize for Handle<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serializer.serialize_struct("BattleHandle", 2)?;
        value.serialize_field("id", &self.id)?;
        value.serialize_field("view", &self.view)?;
        value.end()
    }
}
pub(super) struct View<'a> {
    pub game: &'a Game<'static>,
    pub revision: u32,
}
pub(super) struct Saved<'a> {
    pub game: &'a Game<'static>,
    pub seed: u32,
}
pub(super) struct Dispatch<'a> {
    pub view: View<'a>,
}
struct StateView<'a>(&'a Game<'static>);
struct Tiles<'a>(&'a Game<'static>);
struct Units<'a>(&'a Game<'static>);
struct UnitKey(u32);
impl Serialize for UnitKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut bytes = [0u8; 11];
        let mut offset = bytes.len();
        let mut n = self.0;
        loop {
            offset -= 1;
            bytes[offset] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        offset -= 1;
        bytes[offset] = b'u';
        serializer
            .serialize_str(std::str::from_utf8(&bytes[offset..]).expect("ASCII unit identifier"))
    }
}
impl Serialize for View<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("EngineView", 2)?;
        s.serialize_field("revision", &self.revision)?;
        s.serialize_field("state", &StateView(self.game))?;
        s.end()
    }
}
impl Serialize for Saved<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("SavedGame", 2)?;
        s.serialize_field("seed", &self.seed)?;
        s.serialize_field("state", &StateView(self.game))?;
        s.end()
    }
}
impl Serialize for Dispatch<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("DispatchResult", 2)?;
        s.serialize_field("view", &self.view)?;
        s.serialize_field("combat", &Playback(self.view.game))?;
        s.end()
    }
}
impl Serialize for StateView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let g = self.0;
        let mut s = serializer.serialize_struct("State", 17)?;
        if let Some(armies) = &g.armies {
            s.serialize_field("armies", armies)?;
        }
        s.serialize_field("mapId", &g.map_id)?;
        s.serialize_field("mapRevision", &g.map_revision)?;
        s.serialize_field("cols", &g.cols)?;
        s.serialize_field("rows", &g.rows)?;
        s.serialize_field("round", &g.round)?;
        s.serialize_field("turn", &g.turn)?;
        s.serialize_field(
            "gold",
            &Gold {
                player: g.gold[0],
                enemy: g.gold[1],
            },
        )?;
        s.serialize_field("winner", &g.winner)?;
        s.serialize_field("nextId", &g.next_id)?;
        s.serialize_field("combatRng", &g.combat_rng)?;
        s.serialize_field("log", &g.log)?;
        s.serialize_field("tiles", &Tiles(g))?;
        s.serialize_field("units", &Units(g))?;
        if let Some(campaign) = &g.campaign {
            s.serialize_field("campaign", campaign)?;
        }
        if g.combat_present {
            s.serialize_field("lastCombat", &g.last_combat)?;
        }
        s.end()
    }
}
impl Serialize for Tiles<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TileView<'a> {
            id: &'a str,
            col: i32,
            row: i32,
            terrain: &'a str,
            owner: Option<Side>,
            #[serde(skip_serializing_if = "Option::is_none")]
            connections: &'a Option<Vec<HexDirection>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            dead_ends: &'a Option<Vec<HexDirection>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            floor: &'a Option<String>,
        }
        let g = self.0;
        let mut seq = serializer.serialize_seq(Some(g.board.ids.len()))?;
        for (index, id) in g.board.ids.iter().enumerate() {
            seq.serialize_element(&TileView {
                id,
                col: g.board.positions[index].0,
                row: g.board.positions[index].1,
                terrain: &g.rules.terrain_names[g.terrain[index]],
                owner: g.owners[index],
                connections: &g.connections[index],
                dead_ends: &g.dead_ends[index],
                floor: &g.floors[index],
            })?;
        }
        seq.end()
    }
}
impl Serialize for Units<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let g = self.0;
        let mut seq = serializer.serialize_seq(Some(g.units.len()))?;
        for u in &g.units {
            seq.serialize_element(&UnitProjection { game: g, unit: u })?;
        }
        seq.end()
    }
}

pub(super) struct Decision<'a> {
    pub game: &'a Game<'static>,
    pub revision: u32,
    pub command: &'a Action,
}
impl Serialize for Decision<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AiDecision", 2)?;
        s.serialize_field("revision", &self.revision)?;
        s.serialize_field(
            "command",
            &ActionWire {
                game: self.game,
                command: self.command,
            },
        )?;
        s.end()
    }
}
struct ActionWire<'a> {
    game: &'a Game<'static>,
    command: &'a Action,
}
impl Serialize for ActionWire<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "kind", rename_all = "camelCase")]
        enum Wire<'a> {
            Move {
                unit: u32,
                tile: &'a str,
            },
            Attack {
                unit: u32,
                target: u32,
                weapon: &'a str,
            },
            Recruit {
                unit: u32,
                #[serde(rename = "type")]
                kind: &'a str,
                tile: &'a str,
            },
            Advance {
                unit: u32,
                #[serde(rename = "type")]
                kind: &'a str,
            },
            EndTurn,
            SetUnitIntent {
                unit: u32,
                retreating: Option<bool>,
                rebuilding: Option<bool>,
                attacked: bool,
                record_retreat: bool,
            },
        }
        let g = self.game;
        let wire = match *self.command {
            Action::Move { unit, tile } => Wire::Move {
                unit: unit.0,
                tile: &g.board.ids[tile],
            },
            Action::Attack {
                unit,
                target,
                weapon,
            } => Wire::Attack {
                unit: unit.0,
                target: target.0,
                weapon: &g.rules.units[g.unit(unit.0).expect("planned actor").kind].attacks[weapon]
                    .id,
            },
            Action::Recruit { unit, kind, tile } => Wire::Recruit {
                unit: unit.0,
                kind: &g.rules.units[kind].id,
                tile: &g.board.ids[tile],
            },
            Action::Advance { unit, kind } => Wire::Advance {
                unit: unit.0,
                kind: &g.rules.units[kind].id,
            },
            Action::EndTurn => Wire::EndTurn,
            Action::SetUnitIntent {
                unit,
                retreating,
                rebuilding,
                attacked,
                record_retreat,
            } => Wire::SetUnitIntent {
                unit: unit.0,
                retreating,
                rebuilding,
                attacked,
                record_retreat,
            },
        };
        wire.serialize(serializer)
    }
}

struct UnitProjection<'a> {
    game: &'a Game<'static>,
    unit: &'a Unit,
}
impl Serialize for UnitProjection<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct UnitView<'a> {
            id: UnitKey,
            #[serde(rename = "type")]
            kind: &'a str,
            side: Side,
            tile: &'a str,
            hp: i32,
            moved: bool,
            attacked: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            experience: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: &'a Option<std::rc::Rc<str>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            role: &'a Option<Role>,
            #[serde(skip_serializing_if = "Option::is_none")]
            retreating: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            rebuilding: Option<bool>,
        }
        let g = self.game;
        let u = self.unit;
        UnitView {
            id: UnitKey(u.id),
            kind: &g.rules.units[u.kind].id,
            side: u.side,
            tile: &g.board.ids[u.tile],
            hp: u.hp,
            moved: u.moved,
            attacked: u.attacked,
            experience: u.experience,
            name: &u.name,
            role: &u.role,
            retreating: u.retreating,
            rebuilding: u.rebuilding,
        }
        .serialize(serializer)
    }
}
struct Playback<'a>(&'a Game<'static>);
impl Serialize for Playback<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let g = self.0;
        if !g.combat_animation.present {
            return serializer.serialize_none();
        }
        struct Strikes<'a>(&'a Game<'static>);
        impl Serialize for Strikes<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                #[derive(Serialize)]
                struct Strike<'a> {
                    weapon: &'a str,
                    #[serde(flatten)]
                    strike: &'a crate::logic::playback::Strike,
                }
                let mut seq =
                    serializer.serialize_seq(Some(self.0.combat_animation.strikes.len()))?;
                for strike in &self.0.combat_animation.strikes {
                    seq.serialize_element(&Strike {
                        weapon: &self.0.rules.units[strike.kind].attacks[strike.weapon].name,
                        strike,
                    })?;
                }
                seq.end()
            }
        }
        let units = g.combat_animation.units.as_ref().expect("playback units");
        let mut s = serializer.serialize_struct("CombatAnimation", 2)?;
        s.serialize_field(
            "units",
            &[
                UnitProjection {
                    game: g,
                    unit: &units[0],
                },
                UnitProjection {
                    game: g,
                    unit: &units[1],
                },
            ],
        )?;
        s.serialize_field("strikes", &Strikes(g))?;
        s.end()
    }
}

/// Publish immutable catalog fields directly, without materializing map snapshots.
pub(super) struct Catalog<'a>(pub &'a Rules);
struct CatalogMaps<'a>(&'a Rules);
struct CatalogCampaigns<'a>(&'a Rules);
impl Serialize for Catalog<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut catalog = serializer.serialize_struct("CatalogView", 5)?;
        catalog.serialize_field("units", &self.0.units)?;
        catalog.serialize_field("terrain", &self.0.terrain_definitions)?;
        catalog.serialize_field("races", &self.0.race_definitions)?;
        catalog.serialize_field("maps", &CatalogMaps(self.0))?;
        catalog.serialize_field("campaigns", &CatalogCampaigns(self.0))?;
        catalog.end()
    }
}
impl Serialize for CatalogMaps<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut maps = serializer.serialize_seq(Some(self.0.map_definitions.len()))?;
        for map in self.0.map_definitions.iter() {
            match map {
                Seed::Baked(template) => maps.serialize_element(template)?,
                #[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
                Seed::Owned(map) => maps.serialize_element(map)?,
                #[cfg(all(target_arch = "wasm32", not(feature = "runtime-catalog")))]
                Seed::Owned(_) => {
                    return Err(serde::ser::Error::custom(
                        "default catalog map must be baked",
                    ));
                }
            }
        }
        maps.end()
    }
}
impl Serialize for CatalogCampaigns<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut campaigns = serializer.serialize_seq(Some(self.0.campaigns.len()))?;
        for campaign in self.0.campaigns.values() {
            campaigns.serialize_element(campaign)?;
        }
        campaigns.end()
    }
}
