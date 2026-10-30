//! Tunable recruitment valuation; defaults preserve the existing adaptive policies.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct RecruitmentWeights {
    pub offense: f64,
    pub endurance_reference_hp: f64,
    pub retaliation: f64,
    pub exchange_retaliation: f64,
    pub drain: f64,
    pub kill: f64,
    pub healing: f64,
    pub aura_thorns: f64,
    pub intrinsic_thorns: f64,
    pub future_wounds: f64,
    pub support_recipients: f64,
    pub healing_overlap: f64,
    pub aura_overlap: f64,
    pub future_enemies: f64,
    pub village_capture: f64,
    pub speed: f64,
    pub leader_pressure: f64,
    /// Caps the combined village and movement bonus; None preserves the old formula.
    pub movement_cap: Option<f64>,
    pub duplicate_penalty: f64,
    pub wait_penalty: f64,
}
impl Default for RecruitmentWeights {
    fn default() -> Self {
        Self {
            offense: 1.0,
            endurance_reference_hp: 20.0,
            retaliation: 0.5,
            exchange_retaliation: 0.5,
            drain: 0.5,
            kill: 3.0,
            healing: 2.0,
            aura_thorns: 2.0,
            intrinsic_thorns: 2.0,
            future_wounds: 0.2,
            support_recipients: 1.0,
            healing_overlap: 1.0,
            aura_overlap: 1.0,
            future_enemies: 4.0,
            village_capture: 0.1,
            speed: 0.0,
            leader_pressure: 0.0,
            movement_cap: None,
            duplicate_penalty: 0.2,
            wait_penalty: 0.25,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct RecruitmentProfiles {
    pub standard: RecruitmentWeights,
    pub aggressive_predictor: RecruitmentWeights,
}
impl RecruitmentProfiles {
    pub(super) fn validate(&self) -> Result<(), String> {
        for (name, profile) in [
            ("standard", self.standard),
            ("aggressivePredictor", self.aggressive_predictor),
        ] {
            for (field, value) in [
                ("offense", profile.offense),
                ("enduranceReferenceHp", profile.endurance_reference_hp),
                ("retaliation", profile.retaliation),
                ("exchangeRetaliation", profile.exchange_retaliation),
                ("drain", profile.drain),
                ("kill", profile.kill),
                ("healing", profile.healing),
                ("auraThorns", profile.aura_thorns),
                ("intrinsicThorns", profile.intrinsic_thorns),
                ("futureWounds", profile.future_wounds),
                ("supportRecipients", profile.support_recipients),
                ("healingOverlap", profile.healing_overlap),
                ("auraOverlap", profile.aura_overlap),
                ("futureEnemies", profile.future_enemies),
                ("villageCapture", profile.village_capture),
                ("speed", profile.speed),
                ("leaderPressure", profile.leader_pressure),
                ("duplicatePenalty", profile.duplicate_penalty),
                ("waitPenalty", profile.wait_penalty),
                ("movementCap", profile.movement_cap.unwrap_or(0.0)),
            ] {
                if !value.is_finite() || value < 0.0 {
                    return Err(format!("{name}.{field} must be finite and nonnegative"));
                }
            }
            if profile.endurance_reference_hp <= 0.0 {
                return Err(format!("{name}.enduranceReferenceHp must be positive"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiles_reject_bad_coefficients_and_unknown_names() {
        let mut profiles = RecruitmentProfiles::default();
        profiles.standard.endurance_reference_hp = 0.0;
        assert!(profiles.validate().is_err());
        profiles.standard.endurance_reference_hp = 20.0;
        profiles.aggressive_predictor.speed = -1.0;
        assert!(profiles.validate().is_err());
        profiles.aggressive_predictor.speed = f64::NAN;
        assert!(profiles.validate().is_err());
        assert!(serde_json::from_str::<RecruitmentWeights>(r#"{"speeed": 1}"#).is_err());
    }
}
