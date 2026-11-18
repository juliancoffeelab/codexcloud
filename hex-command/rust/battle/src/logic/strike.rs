//! Executed strike identity and its output-only playback record.
use super::battle::BattlePosition;
#[cfg(feature = "wasm")]
use super::exchange::StrikeEffect;
use crate::data::*;
use serde::Serialize;

// Only this module can bind a roll to its actor/weapon or construct playback.
pub(super) struct ResolvedStrike<'catalog> {
    raw: i32,
    damage: i32,
    #[cfg(feature = "wasm")]
    outcomes: AttackOutcomes,
    #[cfg(feature = "wasm")]
    actor: u32,
    #[cfg(feature = "wasm")]
    weapon: AttackHandle<'catalog>,
    marker: std::marker::PhantomData<&'catalog ()>,
    #[cfg(not(target_arch = "wasm32"))]
    mitigation: DamageMitigationRecord,
}
impl ResolvedStrike<'_> {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn raw(&self) -> i32 {
        self.raw
    }
    pub(super) fn damage(&self) -> i32 {
        self.damage
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn mitigation(&self) -> DamageMitigationRecord {
        self.mitigation.clone()
    }

    #[cfg(feature = "wasm")]
    pub(super) fn into_playback(self, effect: StrikeEffect, health: [i32; 2]) -> CombatStrike {
        let (low, high) = self.outcomes.bounds();
        let protection = self.outcomes.protection();
        CombatStrike {
            weapon: self.weapon.name().into(),
            weapon_id: self.weapon.id().into(),
            unit_id: format!("u{}", self.actor),
            roll_min: low.raw,
            roll_max: high.raw,
            raw: self.raw,
            modifier: protection.armour,
            terrain: protection.terrain,
            damage: self.damage,
            effective: effect.effective,
            drain: self.outcomes.drain_percent(),
            healed: effect.healed,
            thorns: effect.thorns,
            reflected: effect.reflected,
            health,
        }
    }
}
impl<'catalog> BattlePosition<'catalog> {
    pub(super) fn roll_strike(
        &mut self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> ResolvedStrike<'catalog> {
        let outcomes = self.attack_outcomes_at(attacker, target, weapon, tile);
        let rolled = outcomes.roll(|| {
            self.runtime.combat_rng = self
                .combat_rng
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            let mut sample = self.runtime.combat_rng;
            sample = (sample ^ (sample >> 16)).wrapping_mul(0x21f0aaad);
            sample = (sample ^ (sample >> 15)).wrapping_mul(0x735a2d97);
            sample ^ (sample >> 15)
        });
        let raw = rolled.raw;
        let damage = rolled.damage;
        ResolvedStrike {
            raw,
            damage,
            #[cfg(feature = "wasm")]
            outcomes,
            #[cfg(feature = "wasm")]
            actor: attacker.id,
            #[cfg(feature = "wasm")]
            weapon: self.with_attacks(attacker, |attacks| attacks[weapon]),
            marker: std::marker::PhantomData,
            #[cfg(not(target_arch = "wasm32"))]
            mitigation: {
                let after_armour = rolled.after_armour;
                let hp = target.hp.max(0);
                DamageMitigationRecord {
                    available_hp: hp,
                    effective_damage: Some(damage.min(hp)),
                    raw_damage: raw,
                    after_armour_damage: after_armour,
                    after_terrain_damage: damage,
                    effective_armour_prevented: raw.min(hp) - after_armour.min(hp),
                    effective_terrain_prevented: after_armour.min(hp) - damage.min(hp),
                }
            },
        }
    }
}

/// The actual roll, modifiers and HP effects of one executed strike.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase")]
pub struct CombatStrike {
    weapon: Text,
    /// Stable presentation identity; weapon remains the display label.
    weapon_id: Text,
    unit_id: Text,
    roll_min: i32,
    roll_max: i32,
    raw: i32,
    modifier: i32,
    terrain: i32,
    damage: i32,
    effective: i32,
    drain: u8,
    healed: i32,
    thorns: i32,
    reflected: i32,
    health: [i32; 2],
}
