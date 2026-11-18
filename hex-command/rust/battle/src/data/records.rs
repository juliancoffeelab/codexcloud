//! Owned action, combat and loss records produced by simulation sessions.

use super::*;
use crate::data::Text;
use serde::{Deserialize, Serialize};
/// Commands recorded by simulations in their established wire shape.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ActionRecord {
    Drop {
        unit: Text,
        item: Text,
    },
    Move {
        requested: Option<Hex>,
        unit: Text,
        tile: Hex,

        path: Vec<Hex>,
    },
    Attack {
        unit: Text,
        target: Text,
        weapon: Text,
    },
    Recruit {
        unit: Text,
        #[serde(rename = "type")]
        unit_type: Text,
        tile: Hex,

        cost: Option<i32>,
    },
    Advance {
        unit: Text,
        #[serde(rename = "type")]
        unit_type: Text,
    },
}
/// Actual adjacent healing after terrain healing and HP caps. Equal strongest
/// providers share attribution; weaker overlapping healers receive no credit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealingRecord {
    pub target: Text,
    pub providers: Vec<Text>,
    pub amount: i32,
}
/// Reflection separated from weapon retaliation. Effective damage is the actual
/// HP removed by reflection as each initiating strike lands, excluding overkill.
/// Aura ties share attribution; intrinsic reflection belongs to the combat's target.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThornRecord {
    pub damage: i32,
    pub aura_damage: i32,
    pub effective_damage: i32,
    pub aura_sources: Vec<Text>,
}
/// Damage stages for an executed weapon, before HP caps. Armour includes racial
/// and unit modifiers; terrain includes the target's environmental modifiers.
/// Stage differences are signed: vulnerabilities produce negative prevention.
/// Effective prevention caps each strike at its available HP, attributing armour
/// first and terrain second, so overkill cannot earn avoided-HP credit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DamageMitigationRecord {
    pub available_hp: i32,
    /// Summed per-strike HP damage, accounting for drain between strikes.
    pub effective_damage: Option<i32>,
    pub raw_damage: i32,
    pub after_armour_damage: i32,
    pub after_terrain_damage: i32,
    pub effective_armour_prevented: i32,
    pub effective_terrain_prevented: i32,
}
/// Initiating weapon and actual weapon retaliation, excluding reflected thorns.
/// A defender killed before its first reply has no retaliation entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatMitigationRecord {
    pub attack: DamageMitigationRecord,
    pub retaliation: Option<DamageMitigationRecord>,
}
/// The positional and damage details of a resolved attack for native reports.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatRecord {
    pub unit: Text,
    pub target: Text,
    pub weapon: Text,
    pub from: Hex,
    pub target_tile: Hex,
    pub damage: i32,
    pub retaliation: i32,

    pub count: u8,

    pub damage_per_hit: i32,

    pub raw_rolls: Vec<i32>,

    pub rolls: Vec<i32>,

    pub response_raw_rolls: Vec<i32>,

    pub response_rolls: Vec<i32>,

    pub retaliation_count: u8,

    pub retaliation_per_hit: i32,
    pub mediation_at_risk: bool,

    pub thorns: Option<ThornRecord>,

    pub mitigation: Option<CombatMitigationRecord>,
}
/// Exclusively owned action, casualty and combat histories for one simulation session.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Records {
    pub actions: Vec<ActionRecord>,
    pub losses: Vec<PublicUnit>,
    pub combats: Vec<CombatRecord>,

    pub healing: Vec<HealingRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Hex;
    #[test]
    fn combat_records_require_current_strike_metadata_and_preserve_zero_values() {
        let frozen = serde_json::json!({
            "unit": "u1", "target": "u2", "weapon": "hit", "from": Hex::new(0, 0),
            "targetTile": Hex::new(0, 1), "damage": 0, "retaliation": 0,
            "mediationAtRisk": false
        });
        assert!(serde_json::from_value::<CombatRecord>(frozen.clone()).is_err());
        let mut current = frozen.clone();
        for key in ["rawRolls", "rolls", "responseRawRolls", "responseRolls"] {
            current[key] = serde_json::json!([]);
        }
        current["thorns"] = serde_json::Value::Null;
        current["mitigation"] = serde_json::Value::Null;
        current["count"] = serde_json::json!(3);
        current["damagePerHit"] = serde_json::json!(0);
        current["retaliationCount"] = serde_json::json!(0);
        current["retaliationPerHit"] = serde_json::json!(0);
        let mut measured = current.clone();
        measured["mitigation"] = serde_json::json!({
            "attack": {"availableHp": 10, "effectiveDamage":null, "rawDamage": 6, "afterArmourDamage": 3,
                "afterTerrainDamage": 0, "effectiveArmourPrevented": 3,
                "effectiveTerrainPrevented": 3}, "retaliation": null
        });
        for expected in [current, measured] {
            let record: CombatRecord = serde_json::from_value(expected.clone()).unwrap();
            assert_eq!(serde_json::to_value(record).unwrap(), expected);
        }
    }
}
