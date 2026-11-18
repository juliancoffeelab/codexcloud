//! Simulation-only inspection. These outputs are not a gameplay or neural query API.
//! Numeric diagnostics reuse the same private composition as battle execution.
use super::*;
use serde::Serialize;

/// Authored unit-and-race profile, before map, equipment or selected battlefield effects.
/// This explicitly scoped view supports cross-roster reports without inventing a battle.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitProfile<'a> {
    pub name: &'a str,
    pub family: Family,
    pub race: Race,
    pub hp: i32,
    pub level: u32,
    pub cost: i32,
    #[serde(rename = "move")]
    pub movement: usize,
    pub role: UnitRole,
    pub damage_modifiers: DamageModifiers,
}
pub fn unit_profile(unit: &UnitDefinition) -> UnitProfile<'_> {
    let armour = |kind| {
        UnitCapabilities::armour_from(
            &unit.racial.damage_modifiers,
            std::iter::once(&unit.modifiers),
            &[],
            kind,
        )
    };
    UnitProfile {
        name: unit.name(),
        family: unit.family(),
        race: unit.race(),
        hp: unit.max_hp,
        level: unit.level(),
        cost: unit.recruitment_price(None),
        movement: unit.movement,
        role: unit.role(),
        damage_modifiers: DamageModifiers {
            blade: armour(DamageType::Blade),
            pierce: armour(DamageType::Pierce),
            impact: armour(DamageType::Impact),
        },
    }
}
/// Authored weapons for a roster-wide diagnostic; no hypothetical actor is fabricated.
pub fn attacks(unit: &UnitDefinition) -> impl Iterator<Item = AttackHandle<'_>> {
    unit.attacks.iter().map(|source| AttackHandle { source })
}
/// Synthetic protection inputs for diagnostic tables. Normal forecasts require a defender.
pub fn attack_outcomes(attack: AttackHandle<'_>, armour: i32, terrain: i32) -> AttackOutcomes {
    AttackOutcomes::from_attack(attack, armour, terrain, 0)
}

#[derive(Debug, PartialEq, Eq)]
pub struct EffectTotal {
    pub amount: i32,
    /// Indices into the supplied UnitInputs::selected_effects, preserving distinct emitters.
    /// Add groups report nonzero inputs; Max groups report every tied nonzero winner.
    pub contributors: Vec<usize>,
}
fn effect_total(
    caps: &UnitCapabilities<'_>,
    select: impl Fn(&EffectValue) -> Option<i32>,
) -> EffectTotal {
    let effects = caps.inputs.selected_effects;
    let mut contributors = Vec::new();
    let amount = UnitCapabilities::effect_total_with_groups(
        effects.iter().copied(),
        &select,
        |group, combine, total| {
            for (index, occurrence) in effects.iter().enumerate() {
                if occurrence.stacking().group != group {
                    continue;
                }
                if let Some(value) = select(occurrence.value()) {
                    let contributes = match combine {
                        EffectCombine::Add => value != 0,
                        EffectCombine::Max => value != 0 && value == total,
                        EffectCombine::Union => unreachable!("numeric group already checked"),
                    };
                    if contributes {
                        contributors.push(index);
                    }
                }
            }
        },
    );
    contributors.sort_unstable();
    EffectTotal {
        amount,
        contributors,
    }
}
pub fn healing(caps: &UnitCapabilities<'_>) -> EffectTotal {
    effect_total(caps, |effect| match effect {
        EffectValue::Healing { power } => Some(*power),
        _ => None,
    })
}
pub fn thorns(caps: &UnitCapabilities<'_>, category: Category) -> EffectTotal {
    effect_total(caps, |effect| match effect {
        EffectValue::Thorns {
            damage_per_strike,
            incoming_category,
        } if *incoming_category == category => Some(*damage_per_strike),
        _ => None,
    })
}
