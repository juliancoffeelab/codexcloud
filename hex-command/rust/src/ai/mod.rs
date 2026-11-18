//! Policies choose typed commands from a read-only battle. A workspace retains
//! turn-local commitments and the continuation of a classic unit's action.
#![deny(clippy::wildcard_enum_match_arm)]
use hex_command_battle::RuntimeCommand;
pub mod campaigner;
pub mod context;
pub mod orders;
pub mod planner;
pub mod policies;
mod presets;
#[cfg(not(target_arch = "wasm32"))]
pub mod profiling;
pub mod recruiter;
mod roles;
use crate::data::*;
pub use campaigner::CampaignerKind;
pub use planner::PlannerKind;
pub use policies::tie_breaking::Random;
pub use presets::Blocks;
pub use recruiter::RecruiterKind;
use serde::{Deserialize, Serialize};

/// Native diagnostics reuse the planner's conservative, independent-enemy
/// forecast without executing a plan or changing the battle.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn audit_leader_exposure(
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    leader: &Unit,
    tiles: impl IntoIterator<Item = usize>,
) -> Vec<(usize, i32)> {
    let mut forecast = policies::threat::Threats::new(phases, phases.view().unit_faction(&leader));
    tiles
        .into_iter()
        .map(|tile| (tile, forecast.at(leader, tile, None)))
        .collect()
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "simulation", not(target_arch = "wasm32")),
    derive(clap::ValueEnum)
)]
#[serde(rename_all = "lowercase")]
/// Tactical and recruitment policy, preserving each policy's stable choices.
pub enum Policy {
    #[default]
    Classic,
    Rush,
    Balanced,
    Rotated,
    Situational,
    Predictor,
}
impl Policy {
    /// Supported presets in their stable authored order.
    pub const ALL: [Self; 6] = [
        Self::Classic,
        Self::Rush,
        Self::Balanced,
        Self::Rotated,
        Self::Situational,
        Self::Predictor,
    ];
    /// Stable JSON/CLI policy identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Rush => "rush",
            Self::Balanced => "balanced",
            Self::Rotated => "rotated",
            Self::Situational => "situational",
            Self::Predictor => "predictor",
        }
    }
}
impl From<crate::data::FactionPolicy> for Policy {
    fn from(policy: crate::data::FactionPolicy) -> Self {
        use crate::data::FactionPolicy;
        match policy {
            FactionPolicy::Classic => Self::Classic,
            FactionPolicy::Rush => Self::Rush,
            FactionPolicy::Balanced => Self::Balanced,
            FactionPolicy::Rotated => Self::Rotated,
            FactionPolicy::Situational => Self::Situational,
            FactionPolicy::Predictor => Self::Predictor,
        }
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "simulation", not(target_arch = "wasm32")),
    derive(clap::ValueEnum)
)]
#[serde(rename_all = "lowercase")]
/// Treatment of retreating and mediated opponents during campaign battles.
pub enum Diplomacy {
    #[default]
    #[serde(alias = "restrained")]
    #[cfg_attr(
        all(feature = "simulation", not(target_arch = "wasm32")),
        value(alias = "restrained")
    )]
    Diplomatic,
    #[serde(alias = "pragmatic")]
    #[cfg_attr(
        all(feature = "simulation", not(target_arch = "wasm32")),
        value(alias = "pragmatic")
    )]
    Unrestricted,
}
impl Diplomacy {
    /// Stable JSON/CLI treatment identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Diplomatic => "diplomatic",
            Self::Unrestricted => "unrestricted",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Tactical placement grouping of the unit's explicitly authored role.
pub enum TacticalRole {
    Frontline,
    Ranged,
    Healer,
}
pub mod runtime;
pub use runtime::PlanningWorkspace;
#[cfg(any(feature = "wasm", test))]
pub(crate) use runtime::choose_game_command;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use runtime::play_blocks;

fn intent_command(
    unit: &Unit,
    retreating: Option<bool>,
    rebuilding: Option<bool>,
    attack_spent: bool,
) -> RuntimeCommand {
    RuntimeCommand::SetUnitIntent {
        unit: UnitId(unit.id),
        retreating,
        rebuilding,
        attack_spent,
        record_retreat: false,
    }
}
fn move_command(unit: u32, tile: usize) -> RuntimeCommand {
    RuntimeCommand::Move {
        unit: UnitId(unit),
        tile: TileIndex(tile),
    }
}
fn attack_command(
    game: &context::BattleView<'_, '_>,
    unit: u32,
    target: u32,
    weapon: usize,
) -> RuntimeCommand {
    let actor = game.unit(unit).unwrap();
    RuntimeCommand::Attack {
        unit: UnitId(unit),
        target: UnitId(target),
        weapon: game.weapon_key(actor, weapon),
    }
}
fn recruit_command(unit: u32, kind: usize, tile: usize) -> RuntimeCommand {
    RuntimeCommand::Recruit {
        unit: UnitId(unit),
        kind: UnitKind(kind),
        tile: TileIndex(tile),
    }
}
fn advance_command(unit: u32, kind: usize) -> RuntimeCommand {
    RuntimeCommand::Advance {
        unit: UnitId(unit),
        kind: UnitKind(kind),
    }
}
