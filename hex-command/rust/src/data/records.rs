//! Owned action, combat and loss records produced by simulation sessions.

use super::*;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
/// Commands recorded by simulations in their established wire shape.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ActionRecord {
    Move {
        unit: String,
        tile: String,
    },
    Attack {
        unit: String,
        target: String,
        weapon: String,
    },
    Recruit {
        unit: String,
        #[serde(rename = "type")]
        unit_type: String,
        tile: String,
    },
    Advance {
        unit: String,
        #[serde(rename = "type")]
        unit_type: String,
    },
}
/// Actual adjacent healing after terrain healing and HP caps. Equal strongest
/// providers share attribution; weaker overlapping healers receive no credit.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealingRecord {
    pub target: String,
    pub providers: Vec<String>,
    pub amount: i32,
}
/// Reflection separated from weapon retaliation. Effective damage is the extra
/// HP removed beyond ordinary retaliation, excluding overkill. Aura ties share
/// attribution; intrinsic reflection belongs to the combat's target.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThornRecord {
    pub damage: i32,
    pub aura_damage: i32,
    pub effective_damage: i32,
    pub aura_sources: Vec<String>,
}
/// The positional and damage details of a resolved attack for native reports.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatRecord {
    pub unit: String,
    pub target: String,
    pub weapon: String,
    pub from: String,
    pub target_tile: String,
    pub damage: i32,
    pub retaliation: i32,
    #[serde(default)]
    pub count: u8,
    #[serde(default)]
    pub damage_per_hit: i32,
    #[serde(default)]
    pub retaliation_count: u8,
    #[serde(default)]
    pub retaliation_per_hit: i32,
    pub mediation_at_risk: bool,
    #[serde(default)]
    pub thorns: Option<ThornRecord>,
}
impl Serialize for CombatRecord {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Frozen records predate strike metadata. Their default count of zero
        // preserves missing fields; current attacks always have a positive count.
        let strikes = self.count != 0;
        let mut record = serializer.serialize_struct(
            "CombatRecord",
            (if strikes { 12 } else { 8 }) + usize::from(self.thorns.is_some()),
        )?;
        record.serialize_field("unit", &self.unit)?;
        record.serialize_field("target", &self.target)?;
        record.serialize_field("weapon", &self.weapon)?;
        record.serialize_field("from", &self.from)?;
        record.serialize_field("targetTile", &self.target_tile)?;
        record.serialize_field("damage", &self.damage)?;
        record.serialize_field("retaliation", &self.retaliation)?;
        if strikes {
            record.serialize_field("count", &self.count)?;
            record.serialize_field("damagePerHit", &self.damage_per_hit)?;
            record.serialize_field("retaliationCount", &self.retaliation_count)?;
            record.serialize_field("retaliationPerHit", &self.retaliation_per_hit)?;
        }
        if let Some(thorns) = &self.thorns {
            record.serialize_field("thorns", thorns)?;
        }
        record.serialize_field("mediationAtRisk", &self.mediation_at_risk)?;
        record.end()
    }
}
/// Exclusively owned action, casualty and combat histories for one simulation session.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Records {
    pub actions: Vec<ActionRecord>,
    pub losses: Vec<PublicUnit>,
    pub combats: Vec<CombatRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub healing: Vec<HealingRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combat_records_preserve_frozen_omissions_and_current_zero_strike_values() {
        let frozen = serde_json::json!({
            "unit": "u1", "target": "u2", "weapon": "hit", "from": "0,0",
            "targetTile": "0,1", "damage": 0, "retaliation": 0,
            "mediationAtRisk": false
        });
        let mut current = frozen.clone();
        current["count"] = serde_json::json!(3);
        current["damagePerHit"] = serde_json::json!(0);
        current["retaliationCount"] = serde_json::json!(0);
        current["retaliationPerHit"] = serde_json::json!(0);
        for expected in [frozen, current] {
            let record: CombatRecord = serde_json::from_value(expected.clone()).unwrap();
            assert_eq!(serde_json::to_value(record).unwrap(), expected);
        }
    }
}
