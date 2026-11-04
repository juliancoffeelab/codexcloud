//! Deterministic race/policy/map schedules with counterbalanced seats and initiative.
use super::{error::SimulationError, types::*};
use crate::{
    ai::{Blocks, CampaignBlock, Diplomacy, Planner, Policy, Recruiter},
    data::{Rules, Side},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) const DEFAULT_ARENA_SEEDS: u32 = 50;
pub(super) const DEFAULT_CAMPAIGN_SEEDS: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
/// Which independent workload families are scheduled.
pub enum Mode {
    Arena,
    Campaign,
    Full,
}
impl Mode {
    /// Stable CLI/report label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Arena => "arena",
            Self::Campaign => "campaign",
            Self::Full => "full",
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
/// Independently chosen blocks; omitted slots resolve from the workload defaults.
pub struct WithSelection {
    pub recruiter: Option<Recruiter>,
    pub campaign: Option<CampaignBlock>,
    pub planner: Option<Planner>,
}
impl WithSelection {
    pub fn parse(value: &str) -> Result<Self, SimulationError> {
        let parts: Vec<_> = value.split(',').collect();
        if parts.len() != 3 {
            return Err("--with requires recruiter,campaigner,planner; use - for defaults".into());
        }
        fn component<T: serde::de::DeserializeOwned>(
            value: &str,
        ) -> Result<Option<T>, SimulationError> {
            if value == "-" {
                return Ok(None);
            }
            Ok(Some(serde_json::from_value(serde_json::Value::String(
                value.to_owned(),
            ))?))
        }
        Ok(Self {
            recruiter: component(if parts[0] == "rotated" {
                "rotation"
            } else {
                parts[0]
            })?,
            campaign: component(parts[1])?,
            planner: component(parts[2])?,
        })
    }
    pub fn resolve(self, arena: bool) -> Blocks {
        let defaults = Policy::Balanced.blocks();
        Blocks {
            recruiter: self.recruiter.unwrap_or(defaults.recruiter),
            campaign: self.campaign.unwrap_or(if arena {
                CampaignBlock::Arena
            } else {
                CampaignBlock::Mission
            }),
            planner: self.planner.unwrap_or(defaults.planner),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Validated dimensions and per-match limits for a native workload.
pub struct ScheduleOptions {
    pub mode: Mode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with: Option<WithSelection>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub campaign_custom_player: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub campaign_custom_enemy: bool,
    pub first_seed: u32,
    pub seeds: u32,
    pub campaign_seeds: u32,
    pub max_rounds: u32,
    pub campaign_max_rounds: u32,
    /// Explicit override for both sides; None uses each policy's default.
    pub recruit_limit: Option<usize>,
    pub armies: Vec<String>,
    pub landscapes: Vec<String>,
    pub campaigns: Vec<String>,
    pub policies: Vec<Policy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign_players: Option<Vec<Policy>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign_enemies: Option<Vec<Policy>>,
    pub diplomacy: Vec<Diplomacy>,
    pub objectives: Option<Vec<String>>,
    pub campaign_objectives: BTreeMap<String, Vec<String>>,
    pub campaign_diplomacy: BTreeMap<String, Vec<Diplomacy>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub self_play: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponent: Option<Policy>,
}
impl ScheduleOptions {
    /// Resolve authored simulation defaults without starting any matches.
    pub fn defaults(rules: &Rules) -> Result<Self, SimulationError> {
        let parse = |ids: &[String]| -> Result<Vec<Policy>, SimulationError> {
            ids.iter()
                .map(|id| {
                    serde_json::from_value(serde_json::Value::String(id.clone()))
                        .map_err(SimulationError::from)
                })
                .collect()
        };
        Ok(Self {
            mode: Mode::Arena,
            with: None,
            campaign_custom_player: false,
            campaign_custom_enemy: false,
            first_seed: 1,
            seeds: DEFAULT_ARENA_SEEDS,
            campaign_seeds: DEFAULT_CAMPAIGN_SEEDS,
            max_rounds: rules.arena.default_max_rounds,
            campaign_max_rounds: rules.campaign_defaults.default_max_rounds,
            recruit_limit: None,
            armies: rules.rosters.iter().map(|item| item.id.clone()).collect(),
            landscapes: rules
                .arena
                .landscapes
                .iter()
                .map(|item| item.id.clone())
                .collect(),
            campaigns: rules.campaign_defaults.campaigns.clone(),
            policies: parse(&rules.arena.policies)?,
            objectives: None,
            campaign_objectives: rules
                .campaign_defaults
                .objectives
                .iter()
                .map(|(id, routes)| (id.clone(), routes.keys().cloned().collect()))
                .collect(),
            campaign_players: None,
            campaign_enemies: None,
            self_play: false,
            opponent: None,
            campaign_diplomacy: rules
                .campaigns
                .iter()
                .map(|(id, campaign)| {
                    let modes = campaign
                        .player_diplomacy
                        .iter()
                        .map(|mode| {
                            serde_json::from_value(serde_json::Value::String(mode.clone()))
                                .map_err(SimulationError::from)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok((id.clone(), modes))
                })
                .collect::<Result<_, SimulationError>>()?,
            diplomacy: rules
                .campaign_defaults
                .diplomacy
                .iter()
                .map(|id| {
                    serde_json::from_value(serde_json::Value::String(id.clone()))
                        .map_err(SimulationError::from)
                })
                .collect::<Result<_, _>>()?,
        })
    }
    /// Select only player diplomacy modes authored for this campaign.
    pub fn diplomacy_for<'a>(&'a self, campaign: &'a str) -> impl Iterator<Item = &'a Diplomacy> {
        self.diplomacy.iter().filter(move |mode| {
            self.campaign_diplomacy
                .get(campaign)
                .is_none_or(|allowed| allowed.contains(mode))
        })
    }
    pub fn objectives_for(&self, campaign: &str) -> Vec<String> {
        let authored = vec!["authored".to_owned()];
        let allowed = self.campaign_objectives.get(campaign).unwrap_or(&authored);
        match &self.objectives {
            Some(selected) => selected
                .iter()
                .filter(|mode| mode.as_str() == "authored" || allowed.contains(mode))
                .cloned()
                .collect(),
            None => allowed.clone(),
        }
    }
    /// Count both seats/initiatives and campaign treatment settings with checked arithmetic.
    pub fn expected_games(&self) -> Result<usize, SimulationError> {
        let entrants = self
            .armies
            .len()
            .checked_mul(self.policies.len() + usize::from(self.with.is_some()))
            .ok_or("Schedule too large")?;
        let arena = if self.mode == Mode::Campaign {
            0
        } else {
            let opponents = if let Some(opponent) = self.opponent {
                let nonclassic = if self.with.is_some() {
                    1
                } else {
                    self.policies.iter().filter(|p| **p != opponent).count()
                };
                nonclassic
                    .checked_mul(2)
                    .and_then(|n| {
                        n.checked_add(usize::from(
                            self.with.is_none() && self.policies.contains(&opponent),
                        ))
                    })
                    .and_then(|n| n.checked_mul(self.armies.len()))
                    .and_then(|n| n.checked_mul(self.armies.len()))
                    .ok_or("Schedule too large")?
            } else {
                entrants
                    .checked_mul(if self.self_play {
                        self.armies.len()
                    } else {
                        entrants
                    })
                    .ok_or("Schedule too large")?
            };
            self.landscapes
                .len()
                .checked_mul(2)
                .and_then(|n| n.checked_mul(opponents))
                .ok_or("Schedule too large")?
        };
        let campaign_players = self
            .campaign_players
            .as_ref()
            .unwrap_or(&self.policies)
            .len()
            + usize::from(self.campaign_custom_player);
        let campaign_enemies = self
            .campaign_enemies
            .as_ref()
            .unwrap_or(&self.policies)
            .len()
            + usize::from(self.campaign_custom_enemy);
        let campaign = if self.mode == Mode::Arena {
            0
        } else {
            self.campaigns
                .iter()
                .try_fold(0usize, |total, campaign| {
                    total
                        .checked_add(
                            self.diplomacy_for(campaign)
                                .count()
                                .checked_mul(self.objectives_for(campaign).len())
                                .ok_or("Schedule too large")?,
                        )
                        .ok_or("Schedule too large")
                })?
                .checked_mul(campaign_players)
                .and_then(|n| n.checked_mul(campaign_enemies))
                .ok_or("Schedule too large")?
        };
        arena
            .checked_mul(self.seeds as usize)
            .and_then(|n| {
                campaign
                    .checked_mul(self.campaign_seeds as usize)
                    .and_then(|c| n.checked_add(c))
            })
            .ok_or_else(|| "Schedule too large".into())
    }
}
/// Deterministic schedule order matches the TypeScript comparison harness.
pub fn schedule(options: &ScheduleOptions) -> Result<Vec<MatchConfig>, SimulationError> {
    if options.opponent.is_some() && options.self_play {
        return Err("--vs cannot be combined with --self-play".into());
    }
    if options.self_play && options.mode != Mode::Arena {
        return Err("--self-play requires --mode arena".into());
    }
    let end = options
        .first_seed
        .checked_add(match options.mode {
            Mode::Arena => options.seeds,
            Mode::Campaign => options.campaign_seeds,
            Mode::Full => options.seeds.max(options.campaign_seeds),
        })
        .ok_or("Seed range overflows u32")?;
    let count = options.expected_games()?;
    let mut configs = Vec::new();
    configs.try_reserve(count).map_err(|error| {
        SimulationError::InvalidConfig(format!("Schedule allocation failed: {error}"))
    })?;
    let mut participants: Vec<_> = if options.with.is_some() && options.opponent.is_some() {
        vec![(options.opponent.unwrap(), None)]
    } else {
        options
            .policies
            .iter()
            .map(|policy| (*policy, None))
            .collect()
    };
    if let Some(opponent) = options.opponent {
        if !participants
            .iter()
            .any(|(policy, blocks)| *policy == opponent && blocks.is_none())
        {
            participants.push((opponent, None));
        }
    }
    if let Some(selection) = options.with {
        participants.push((Policy::Balanced, Some(selection.resolve(true))));
    }
    let entrants: Vec<_> = options
        .armies
        .iter()
        .flat_map(|army| {
            participants
                .iter()
                .map(move |(policy, blocks)| (army, *policy, *blocks))
        })
        .collect();
    let mut campaign_players: Vec<_> = options
        .campaign_players
        .as_ref()
        .unwrap_or(&options.policies)
        .iter()
        .map(|policy| (*policy, None))
        .collect();
    let mut campaign_enemies: Vec<_> = options
        .campaign_enemies
        .as_ref()
        .unwrap_or(&options.policies)
        .iter()
        .map(|policy| (*policy, None))
        .collect();
    for (custom, participants) in [
        (options.campaign_custom_player, &mut campaign_players),
        (options.campaign_custom_enemy, &mut campaign_enemies),
    ] {
        if custom {
            let selection = options
                .with
                .ok_or("custom campaign participant requires --with")?;
            let blocks = selection.resolve(false);
            if options.mode != Mode::Arena && !blocks.campaign.supports_campaign() {
                return Err(
                    "An arena campaigner requires --mode arena when selected for campaigns".into(),
                );
            }
            participants.push((Policy::Balanced, Some(blocks)));
        }
    }
    for seed in options.first_seed..end {
        if options.mode != Mode::Campaign && seed - options.first_seed < options.seeds {
            for landscape in &options.landscapes {
                for a in 0..entrants.len() {
                    for b in a..entrants.len() {
                        if let Some(opponent) = options.opponent {
                            let fixed = |index: usize| {
                                entrants[index].1 == opponent && entrants[index].2.is_none()
                            };
                            if (!fixed(a) && !fixed(b))
                                || (fixed(a)
                                    && fixed(b)
                                    && !(options.with.is_none()
                                        && options.policies.contains(&opponent)))
                            {
                                continue;
                            }
                        }
                        if options.self_play
                            && (entrants[a].1, entrants[a].2) != (entrants[b].1, entrants[b].2)
                        {
                            continue;
                        }
                        let seats = if a == b {
                            vec![(a, b)]
                        } else {
                            vec![(a, b), (b, a)]
                        };
                        for (left, right) in seats {
                            for first in [Side::Player, Side::Enemy] {
                                configs.push(MatchConfig::Arena(ArenaConfig {
                                    seed,
                                    max_rounds: options.max_rounds,
                                    recruit_limit: options.recruit_limit,
                                    landscape: landscape.clone(),
                                    first,
                                    player: entrants[left].1,
                                    enemy: entrants[right].1,
                                    player_blocks: entrants[left].2,
                                    enemy_blocks: entrants[right].2,
                                    armies: Armies {
                                        player: entrants[left].0.clone(),
                                        enemy: entrants[right].0.clone(),
                                    },
                                }));
                            }
                        }
                    }
                }
            }
        }
        if options.mode != Mode::Arena && seed - options.first_seed < options.campaign_seeds {
            for campaign in &options.campaigns {
                for (player, player_blocks) in &campaign_players {
                    for (enemy, enemy_blocks) in &campaign_enemies {
                        for diplomacy in options.diplomacy_for(campaign) {
                            for objective in options.objectives_for(campaign) {
                                configs.push(MatchConfig::Campaign(CampaignConfig {
                                    campaign: campaign.clone(),
                                    player: *player,
                                    enemy: *enemy,
                                    player_blocks: *player_blocks,
                                    enemy_blocks: *enemy_blocks,
                                    diplomacy: *diplomacy,
                                    objective,
                                    seed,
                                    max_rounds: options.campaign_max_rounds,
                                    recruit_limit: options.recruit_limit,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
    debug_assert_eq!(configs.len(), count);
    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_tactic_is_added_to_matrix_and_campaign_roles() {
        let rules = Rules::load().unwrap();
        let mut options = ScheduleOptions::defaults(&rules).unwrap();
        options.mode = Mode::Arena;
        options.seeds = 1;
        options.opponent = Some(Policy::Balanced);
        options.with = Some(WithSelection {
            recruiter: Some(crate::ai::Recruiter::Rotation),
            campaign: None,
            planner: Some(crate::ai::Planner::Queued),
        });
        options.policies = vec![
            crate::ai::Policy::Classic,
            crate::ai::Policy::Rush,
            crate::ai::Policy::Balanced,
        ];
        options.armies = vec!["a".into(), "b".into()];
        options.landscapes = vec!["open".into()];
        let fixed = schedule(&options).unwrap();
        assert_eq!(fixed.len(), 16);
        assert_eq!(fixed.len(), options.expected_games().unwrap());
        assert!(fixed.iter().all(|config| matches!(config, MatchConfig::Arena(config) if
            (config.player_blocks.is_some() && config.enemy == crate::ai::Policy::Balanced && config.enemy_blocks.is_none())
            || (config.enemy_blocks.is_some() && config.player == crate::ai::Policy::Balanced && config.player_blocks.is_none()))));
        options.opponent = None;
        let all = schedule(&options).unwrap();
        assert_eq!(all.len(), 128);
        assert_eq!(all.len(), options.expected_games().unwrap());
        assert!(all.iter().any(|config| matches!(config, MatchConfig::Arena(config) if config.player_blocks.is_some() && config.enemy == crate::ai::Policy::Balanced && config.enemy_blocks.is_none())));
        let unique: std::collections::HashSet<_> = all
            .iter()
            .map(|config| serde_json::to_string(config).unwrap())
            .collect();
        assert_eq!(unique.len(), all.len());
        options.self_play = true;
        let self_play = schedule(&options).unwrap();
        assert_eq!(self_play.len(), 32);
        assert!(self_play.iter().all(|config| matches!(config, MatchConfig::Arena(config) if config.player == config.enemy && config.player_blocks == config.enemy_blocks)));
        let mut campaign = ScheduleOptions::defaults(&rules).unwrap();
        campaign.mode = Mode::Campaign;
        campaign.seeds = 1;
        campaign.campaign_seeds = 1;
        campaign.with = options.with;
        campaign.campaign_players = Some(vec![crate::ai::Policy::Balanced]);
        campaign.campaign_enemies = Some(vec![crate::ai::Policy::Classic]);
        campaign.campaign_custom_enemy = true;
        campaign.campaigns = vec!["example".into()];
        campaign.campaign_diplomacy = std::collections::BTreeMap::from([(
            "example".into(),
            vec![crate::ai::Diplomacy::Unrestricted],
        )]);
        let configs = schedule(&campaign).unwrap();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs.len(), campaign.expected_games().unwrap());
        assert!(configs.iter().all(|config| matches!(config, MatchConfig::Campaign(config) if config.player_blocks.is_none() && config.enemy_blocks.is_none_or(|blocks| blocks.campaign == crate::ai::CampaignBlock::Mission && blocks.recruiter == crate::ai::Recruiter::Rotation))));
    }

    #[test]
    fn authored_diplomacy_limits_campaign_runs_and_counts() {
        let rules = Rules::load().unwrap();
        let mut options = ScheduleOptions::defaults(&rules).unwrap();
        options.mode = Mode::Campaign;
        options.seeds = 2;
        options.campaign_seeds = 2;
        options.campaigns = vec!["single".into(), "choice".into()];
        options.policies = vec![Policy::Balanced];
        options.campaign_diplomacy = BTreeMap::from([
            ("single".into(), vec![Diplomacy::Unrestricted]),
            (
                "choice".into(),
                vec![Diplomacy::Unrestricted, Diplomacy::Diplomatic],
            ),
        ]);
        let configs = schedule(&options).unwrap();
        assert_eq!(options.expected_games().unwrap(), 6);
        assert_eq!(configs.len(), 6);
        assert!(configs.iter().all(|config| match config {
            MatchConfig::Campaign(config) =>
                config.campaign != "single" || config.diplomacy == Diplomacy::Unrestricted,
            _ => false,
        }));
        options.diplomacy = vec![Diplomacy::Diplomatic];
        let configs = schedule(&options).unwrap();
        assert_eq!(options.expected_games().unwrap(), 2);
        assert_eq!(configs.len(), 2);
        assert!(configs.iter().all(|config| matches!(config,
            MatchConfig::Campaign(config) if config.campaign == "choice" && config.diplomacy == Diplomacy::Diplomatic)));
    }
    #[test]
    fn fixed_opponent_panel_preserves_seats_and_initiative_without_duplicate_matches() {
        let rules = Rules::load().unwrap();
        let mut options = ScheduleOptions::defaults(&rules).unwrap();
        options.mode = Mode::Arena;
        options.seeds = 1;
        options.armies = vec!["a".into(), "b".into()];
        options.landscapes = vec!["open".into()];
        for opponent in [Policy::Classic, Policy::Balanced] {
            options.policies = vec![Policy::Rush, Policy::Situational];
            options.opponent = Some(opponent);
            for include_fixed in [false, true] {
                if include_fixed {
                    options.policies.push(opponent);
                }
                let fixed = schedule(&options).unwrap();
                assert_eq!(fixed.len(), options.expected_games().unwrap());
                let mut unique = std::collections::HashSet::new();
                for config in &fixed {
                    assert!(unique.insert(serde_json::to_string(config).unwrap()));
                }
                let mut all = options.clone();
                all.opponent = None;
                if !include_fixed {
                    all.policies.push(opponent);
                }
                let expected: Vec<_> = schedule(&all)
                    .unwrap()
                    .into_iter()
                    .filter(|config| {
                        matches!(config, MatchConfig::Arena(c)
                    if (c.player == opponent || c.enemy == opponent)
                        && (include_fixed || c.player != c.enemy))
                    })
                    .collect();
                assert_eq!(
                    serde_json::to_value(fixed).unwrap(),
                    serde_json::to_value(expected).unwrap()
                );
            }
        }
        options.self_play = true;
        assert!(schedule(&options).is_err());
    }

    #[test]
    fn seats_and_initiatives_are_counterbalanced() {
        let options = ScheduleOptions {
            mode: Mode::Full,
            with: None,
            campaign_custom_player: false,
            campaign_custom_enemy: false,
            first_seed: 5,
            seeds: 1,
            campaign_seeds: 1,
            max_rounds: 1,
            campaign_max_rounds: 1,
            recruit_limit: Some(0),
            armies: vec!["a".into(), "b".into()],
            landscapes: vec!["open".into()],
            campaigns: vec!["chapter".into()],
            policies: vec![Policy::Classic],
            campaign_players: None,
            campaign_enemies: None,
            diplomacy: vec![Diplomacy::Unrestricted],
            campaign_diplomacy: BTreeMap::new(),
            objectives: None,
            campaign_objectives: BTreeMap::new(),
            self_play: false,
            opponent: None,
        };
        let configs = schedule(&options).unwrap();
        assert_eq!(configs.len(), 9);
        let mut mixed = options.clone();
        mixed.seeds = 2;
        mixed.campaign_seeds = 3;
        let mixed_configs = schedule(&mixed).unwrap();
        assert_eq!(mixed.expected_games().unwrap(), 19);
        assert_eq!(mixed_configs.len(), 19);
        assert_eq!(
            mixed_configs
                .iter()
                .filter(|c| matches!(c, MatchConfig::Arena(_)))
                .count(),
            16
        );
        assert_eq!(
            mixed_configs
                .iter()
                .filter(|c| matches!(c, MatchConfig::Campaign(_)))
                .count(),
            3
        );
        assert!(mixed_configs.iter().all(|c| match c {
            MatchConfig::Arena(c) => (5..7).contains(&c.seed),
            MatchConfig::Campaign(c) => (5..8).contains(&c.seed),
        }));

        let arena: Vec<_> = configs
            .iter()
            .filter_map(|config| {
                if let MatchConfig::Arena(config) = config {
                    Some(config)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            arena
                .iter()
                .filter(|config| config.armies.player == "a" && config.armies.enemy == "a")
                .count(),
            2
        );
        assert_eq!(
            arena
                .iter()
                .filter(|config| config.armies.player != config.armies.enemy)
                .count(),
            4
        );
        let mut self_play_options = options.clone();
        self_play_options.mode = Mode::Arena;
        self_play_options.policies = vec![Policy::Classic, Policy::Rush, Policy::Balanced];
        self_play_options.self_play = true;
        let self_play = schedule(&self_play_options).unwrap();
        assert_eq!(self_play.len(), 24);
        assert_eq!(self_play.len(), self_play_options.expected_games().unwrap());
        assert!(
            self_play
                .iter()
                .all(|config| matches!(config, MatchConfig::Arena(c) if c.player == c.enemy))
        );
        for policy in &self_play_options.policies {
            for first in [Side::Player, Side::Enemy] {
                assert_eq!(self_play.iter().filter(|c|matches!(c, MatchConfig::Arena(c) if c.player == *policy && c.first == first)).count(), 4);
            }
        }
        self_play_options.mode = Mode::Campaign;
        assert!(schedule(&self_play_options).is_err());
        assert_eq!(
            arena
                .iter()
                .filter(|config| config.first == Side::Player)
                .count(),
            4
        );
        let mut independent_roles = options.clone();
        independent_roles.policies = Policy::ALL.to_vec();
        independent_roles.campaign_players = Some(vec![Policy::Rush, Policy::Balanced]);
        independent_roles.campaign_enemies = Some(vec![Policy::Classic]);
        let role_schedule = schedule(&independent_roles).unwrap();
        assert_eq!(
            role_schedule.len(),
            independent_roles.expected_games().unwrap()
        );
        let campaign_matches: Vec<_> = role_schedule
            .iter()
            .filter_map(|c| {
                if let MatchConfig::Campaign(c) = c {
                    Some(c)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(campaign_matches.len(), 2);
        assert!(
            campaign_matches.iter().all(|c| c.enemy == Policy::Classic
                && [Policy::Rush, Policy::Balanced].contains(&c.player))
        );
        let mut unique = std::collections::HashSet::new();
        for config in configs {
            assert!(unique.insert(serde_json::to_string(&config).unwrap()));
        }
    }
}
