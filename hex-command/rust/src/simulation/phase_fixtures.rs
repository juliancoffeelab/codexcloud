//! Optional planner-entry snapshots. Normal journals never allocate these states.
use super::{SimulationError, types::*};
use crate::{
    ai::{Blocks, Diplomacy, Policy},
    data::{Objective, Side, State},
    logic::Game,
};
use serde::{Deserialize, Serialize};

fn zero_limit() -> Option<usize> {
    Some(0)
}
/// Existing phase protocol; absent budgets retain its historical zero-recruit default.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PhaseRequest {
    #[serde(default)]
    pub inspect_recruitment: bool,
    #[serde(default = "authored_objective")]
    pub objective: String,
    #[serde(default)]
    pub blocks: Option<Blocks>,
    pub state: State,
    pub policy: Policy,
    pub seed: u32,
    /// Null selects each controller's policy default, as in the match runner.
    #[serde(default = "zero_limit")]
    pub recruitment_limit: Option<usize>,
    #[serde(default)]
    pub goals: Option<Vec<Objective>>,
    #[serde(default)]
    pub diplomacy: Diplomacy,
    #[serde(default)]
    pub advance_turn: bool,
    #[serde(default)]
    pub begin_turn: bool,
    #[serde(default)]
    pub arena: Option<ArenaConfig>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CaptureRequest {
    pub config: MatchConfig,
    pub rounds: Vec<u32>,
    pub side: Side,
}
#[derive(Serialize)]
pub(crate) struct Position {
    pub label: String,
    pub campaign: bool,
    pub request: PhaseRequest,
}
pub(crate) struct Capture {
    config: MatchConfig,
    rounds: Vec<u32>,
    side: Side,
    pub positions: Vec<Position>,
}
impl Capture {
    pub fn new(request: CaptureRequest) -> Result<Self, SimulationError> {
        let cap = match &request.config {
            MatchConfig::Arena(c) => c.max_rounds,
            MatchConfig::Campaign(c) => c.max_rounds,
        };
        let mut rounds = request.rounds.clone();
        rounds.sort_unstable();
        rounds.dedup();
        if rounds.is_empty()
            || rounds.len() > 32
            || rounds.len() != request.rounds.len()
            || rounds.iter().any(|r| *r == 0 || *r > cap)
        {
            return Err(SimulationError::InvalidConfig(
                "Select 1–32 distinct capture rounds within the match cap".into(),
            ));
        }
        Ok(Self {
            config: request.config,
            rounds,
            side: request.side,
            positions: vec![],
        })
    }
    /// Called after all turn transitions and before the phase's first planner.
    pub fn observe(&mut self, game: &Game<'_>, goals: Option<&[Objective]>) {
        if game.turn() != self.side || !self.rounds.contains(&game.round()) {
            return;
        }
        let (policy, blocks, seed, limit, diplomacy, objective, arena, campaign) =
            match &self.config {
                MatchConfig::Arena(c) => (
                    if game.turn() == Side::Player {
                        c.player
                    } else {
                        c.enemy
                    },
                    if game.turn() == Side::Player {
                        c.player_blocks
                    } else {
                        c.enemy_blocks
                    },
                    c.seed,
                    c.recruit_limit,
                    Diplomacy::Unrestricted,
                    authored_objective(),
                    Some(c.clone()),
                    false,
                ),
                MatchConfig::Campaign(c) => (
                    if game.turn() == Side::Player {
                        c.player
                    } else {
                        c.enemy
                    },
                    if game.turn() == Side::Player {
                        c.player_blocks
                    } else {
                        c.enemy_blocks
                    },
                    c.seed,
                    c.recruit_limit,
                    c.diplomacy,
                    c.objective.clone(),
                    None,
                    true,
                ),
            };
        self.positions.push(Position {
            label: format!("{}/{:?}/round-{}", game.map_id(), game.turn(), game.round()),
            campaign,
            request: PhaseRequest {
                state: game.state(),
                policy,
                blocks,
                seed,
                recruitment_limit: limit,
                goals: goals.map(<[Objective]>::to_vec),
                diplomacy,
                objective,
                arena,
                advance_turn: false,
                begin_turn: false,
                inspect_recruitment: false,
            },
        });
    }
}
