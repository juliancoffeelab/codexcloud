//! Stream established sorted-key endpoint bytes without allocating a JSON value tree.
use crate::data::{Gold, LastCombat, PublicUnit, State, Tile};
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};

struct Canonical<'a, T>(&'a T);
struct Slice<'a, T>(&'a [T]);
impl<T> Serialize for Slice<'_, T>
where
    for<'a> Canonical<'a, T>: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            sequence.serialize_element(&Canonical(value))?;
        }
        sequence.end()
    }
}
impl Serialize for Canonical<'_, State> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = self.0;
        let mut map = serializer.serialize_map(None)?;
        if let Some(campaign) = &s.campaign {
            let value = serde_json::to_value(campaign).map_err(serde::ser::Error::custom)?;
            map.serialize_entry("campaign", &value)?;
        }
        map.serialize_entry("cols", &s.cols)?;
        map.serialize_entry("gold", &Canonical(&s.gold))?;
        if let Some(combat) = &s.last_combat {
            map.serialize_entry("lastCombat", &combat.as_ref().map(Canonical))?;
        }
        map.serialize_entry("log", &s.log)?;
        map.serialize_entry("mapId", &s.map_id)?;
        map.serialize_entry("mapRevision", &s.map_revision)?;
        map.serialize_entry("nextId", &s.next_id)?;
        map.serialize_entry("round", &s.round)?;
        map.serialize_entry("rows", &s.rows)?;
        map.serialize_entry("tiles", &Slice(&s.tiles))?;
        map.serialize_entry("turn", &s.turn)?;
        map.serialize_entry("units", &Slice(&s.units))?;
        map.serialize_entry("winner", &s.winner)?;
        map.end()
    }
}
impl Serialize for Canonical<'_, Tile> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let t = self.0;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("col", &t.col)?;
        if let Some(value) = &t.connections {
            map.serialize_entry("connections", value)?;
        }
        if let Some(value) = &t.dead_ends {
            map.serialize_entry("deadEnds", value)?;
        }
        if let Some(value) = &t.floor {
            map.serialize_entry("floor", value)?;
        }
        map.serialize_entry("id", &t.id)?;
        map.serialize_entry("owner", &t.owner)?;
        map.serialize_entry("row", &t.row)?;
        map.serialize_entry("terrain", &t.terrain)?;
        map.end()
    }
}
impl Serialize for Canonical<'_, PublicUnit> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let u = self.0;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("attacked", &u.attacked)?;
        if let Some(value) = &u.experience {
            map.serialize_entry("experience", value)?;
        }
        map.serialize_entry("hp", &u.hp)?;
        map.serialize_entry("id", &u.id)?;
        map.serialize_entry("moved", &u.moved)?;
        if let Some(value) = &u.name {
            map.serialize_entry("name", value)?;
        }
        if let Some(value) = &u.rebuilding {
            map.serialize_entry("rebuilding", value)?;
        }
        if let Some(value) = &u.retreating {
            map.serialize_entry("retreating", value)?;
        }
        if let Some(value) = &u.role {
            map.serialize_entry("role", value)?;
        }
        map.serialize_entry("side", &u.side)?;
        map.serialize_entry("tile", &u.tile)?;
        map.serialize_entry("type", &u.r#type)?;
        map.end()
    }
}
impl Serialize for Canonical<'_, Gold> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("enemy", &self.0.enemy)?;
        map.serialize_entry("player", &self.0.player)?;
        map.end()
    }
}
impl Serialize for Canonical<'_, LastCombat> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("attackId", &self.0.attack_id)?;
        map.serialize_entry("attackerId", &self.0.attacker_id)?;
        map.serialize_entry("sound", &self.0.sound)?;
        map.end()
    }
}
pub(super) fn state_bytes(state: &State) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&Canonical(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{Catalog, HexDirection},
        logic::Game,
    };
    #[test]
    fn streamed_bytes_match_sorted_objects_with_optional_and_campaign_fields() {
        let rules = Catalog::load().unwrap();
        let mut states = vec![
            Game::new(&rules, "mirror").state(),
            Game::start_campaign(&rules, "green-accord", 0, vec![], 0, vec![], vec![], vec![])
                .state(),
        ];
        for s in &mut states {
            s.tiles[0].connections = Some(vec![HexDirection::E]);
            s.tiles[0].dead_ends = Some(vec![HexDirection::E]);
            s.tiles[0].floor = Some("河😀".into());
            s.units[0].name = Some("name\\\"\n".into());
            s.units[0].retreating = Some(false);
            s.units[0].rebuilding = Some(true);
            s.units[0].experience = None;
            for combat in [
                None,
                Some(None),
                Some(Some(LastCombat {
                    attack_id: "attack".into(),
                    attacker_id: "u1".into(),
                    sound: "sound".into(),
                })),
            ] {
                s.last_combat = combat;
                assert_eq!(
                    state_bytes(s).unwrap(),
                    serde_json::to_vec(&serde_json::to_value(&*s).unwrap()).unwrap()
                );
            }
        }
    }
}
