//! Policies choose typed commands from a read-only battle. A workspace retains
//! turn-local commitments and the continuation of a classic unit's action.
#![deny(clippy::wildcard_enum_match_arm)]
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
use crate::{data::*, logic::Command};
pub use campaigner::CampaignerKind as CampaignBlock;
pub use planner::PlannerKind as Planner;
pub use policies::tie_breaking::Random;
pub use presets::Blocks;
pub use recruiter::RecruiterKind as Recruiter;
use serde::{Deserialize, Serialize};

/// Native diagnostics reuse the planner's conservative, independent-enemy
/// forecast without executing a plan or changing the battle.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn audit_leader_exposure(
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    leader: &Unit,
    tiles: impl IntoIterator<Item = usize>,
) -> Vec<(usize, i32)> {
    let mut forecast = policies::threat::Threats::new(phases, leader.side);
    tiles
        .into_iter()
        .map(|tile| (tile, forecast.at(leader, tile, None)))
        .collect()
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::ValueEnum))]
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
        use crate::data::FactionPolicy as F;
        match policy {
            F::Classic => Self::Classic,
            F::Rush => Self::Rush,
            F::Balanced => Self::Balanced,
            F::Rotated => Self::Rotated,
            F::Situational => Self::Situational,
            F::Predictor => Self::Predictor,
        }
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
/// Treatment of retreating and mediated opponents during campaign battles.
pub enum Diplomacy {
    #[default]
    #[serde(alias = "restrained")]
    #[cfg_attr(not(target_arch = "wasm32"), value(alias = "restrained"))]
    Diplomatic,
    #[serde(alias = "pragmatic")]
    #[cfg_attr(not(target_arch = "wasm32"), value(alias = "pragmatic"))]
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
    attacked: bool,
) -> Command {
    Command::SetUnitIntent {
        unit: UnitId(unit.id),
        retreating,
        rebuilding,
        attacked,
        record_retreat: false,
    }
}
fn move_command(game: &context::BattleView<'_, '_>, unit: u32, tile: usize) -> Command {
    Command::Move {
        unit: UnitId(unit),
        tile: TileId(game.board().ids[tile].clone()),
    }
}
fn attack_command(
    game: &context::BattleView<'_, '_>,
    unit: u32,
    target: u32,
    weapon: usize,
) -> Command {
    let actor = game.unit(unit).unwrap();
    Command::Attack {
        unit: UnitId(unit),
        target: UnitId(target),
        weapon: WeaponId(game.weapon(actor, weapon).id.clone()),
    }
}
fn recruit_command(
    game: &context::BattleView<'_, '_>,
    unit: u32,
    kind: usize,
    tile: usize,
) -> Command {
    Command::Recruit {
        unit: UnitId(unit),
        kind: UnitTypeId(game.rules().units[kind].id.clone()),
        tile: TileId(game.board().ids[tile].clone()),
    }
}
fn advance_command(game: &context::BattleView<'_, '_>, unit: u32, kind: usize) -> Command {
    Command::Advance {
        unit: UnitId(unit),
        kind: UnitTypeId(game.rules().units[kind].id.clone()),
    }
}

#[cfg(test)]
mod courier_tests {
    use super::*;
    use crate::data::Hex;
    use crate::{ai::recruiter::fixtures, logic::Game};

    #[test]
    fn required_courier_can_detour_around_stationary_allies() {
        fn is_command_setunitintent(value: &Command) -> bool {
            match value {
                Command::SetUnitIntent { .. } => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::EndTurn
                | Command::DismissDialogue
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::DropItem { .. }
                | Command::SkipDialogue => false,
            }
        }
        fn is_command_move(value: &Command) -> bool {
            match value {
                Command::Move {
                    unit: UnitId(5), ..
                } => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::EndTurn
                | Command::DismissDialogue
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::DropItem { .. }
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        let mut game = Game::start_campaign(
            &rules,
            "borrowed-tide",
            0,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
        );
        let sites: std::collections::HashMap<_, _> = game
            .board()
            .ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();
        let target = game.site(&Hex::new(9, 3));
        let blocker_tile = game.site(&Hex::new(9, 4));
        fixtures::units(&mut game, |units| {
            units.retain(|u| [1, 2, 4, 5].contains(&u.id));
            for (id, tile) in [
                (1, Hex::new(8, 5)),
                (2, Hex::new(7, 3)),
                (4, Hex::new(8, 3)),
                (5, Hex::new(8, 4)),
            ] {
                let tile = *sites.get(&tile).unwrap();
                units.iter_mut().find(|u| u.id == id).unwrap().tile = tile;
            }
            // Keep allies stationary for this turn: the courier must take a detour.
            for ally in units.iter_mut().filter(|u| u.id != 5) {
                ally.moved = true;
                ally.attacked = true;
            }

            // Block the other direct approach as well.
            units.push(Unit {
                faction: None,
                items: vec![],
                ai_group: None,
                id: 100,
                kind: rules.types["reefguard"],
                tile: blocker_tile,
                side: Side::Player,
                hp: rules.units[rules.types["reefguard"]].hp,
                moved: true,
                attacked: true,
                experience: Some(0),
                character: None,
                role: None,
                retreating: None,
                rebuilding: None,
            });
        });
        let courier = game.unit(5).unwrap();
        let static_route = game.routes(courier.kind, &[target]);
        assert!(
            game.reachable(courier)
                .iter()
                .all(|&(tile, _)| static_route[tile] >= static_route[courier.tile])
        );
        let mut workspace = PlanningWorkspace::new(1, 0, None, Diplomacy::Diplomatic);
        let command = loop {
            let command = choose_game_command(&game, Policy::Balanced, &mut workspace).unwrap();
            if is_command_setunitintent(&command) {
                game.execute(command).unwrap();
            } else {
                break command;
            }
        };
        assert!(is_command_move(&command), "{command:?}");
        game.execute(command).unwrap();
        assert_ne!(game.board().ids[game.unit(5).unwrap().tile], Hex::new(8, 4));
    }

    #[test]
    fn allies_clear_approaches_and_complete_the_escort() {
        fn is_command_endturn(value: &Command) -> bool {
            match value {
                Command::EndTurn => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::DismissDialogue
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::DropItem { .. }
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        for policy in [Policy::Classic, Policy::Balanced] {
            let mut game = Game::start_campaign(
                &rules,
                "borrowed-tide",
                0,
                vec![],
                0,
                vec![],
                vec![],
                vec![],
            );
            fixtures::edit(&mut game, |state| {
                state.units.retain(|u| u.side == Side::Player);
                for (id, tile) in [
                    (1, Hex::new(8, 5)),
                    (2, Hex::new(7, 3)),
                    (3, Hex::new(9, 4)),
                    (4, Hex::new(8, 3)),
                    (5, Hex::new(8, 4)),
                ] {
                    let tile = tile.to_owned();
                    state
                        .units
                        .iter_mut()
                        .find(|u| u.id == format!("u{id}"))
                        .unwrap()
                        .tile = tile;
                }
                for tile in [Hex::new(6, 2), Hex::new(8, 5)] {
                    let tile = tile.to_owned();
                    state
                        .tiles
                        .iter_mut()
                        .find(|t| t.hex() == tile)
                        .unwrap()
                        .owner = Some("player".into());
                }
            });
            let mut workspace = PlanningWorkspace::new(1, 0, None, Diplomacy::Diplomatic);
            for _ in 0..40 {
                let command = choose_game_command(&game, policy, &mut workspace).unwrap();
                if is_command_endturn(&command) {
                    break;
                }
                game.execute(command).unwrap();
                if game.winner().is_some() {
                    break;
                }
            }
            assert_eq!(
                game.winner(),
                Some(Side::Player),
                "{policy:?}: {:?}",
                game.public_units()
            );
            assert_eq!(game.board().ids[game.unit(5).unwrap().tile], Hex::new(9, 3));
        }
    }
}
