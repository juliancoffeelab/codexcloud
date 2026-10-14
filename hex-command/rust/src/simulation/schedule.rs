//! Deterministic race/policy/map schedules with counterbalanced seats and initiative.
use super::{error::SimulationError, types::*};
use crate::{
    ai::{Diplomacy, Policy},
    data::{Rules, Side},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Validated dimensions and per-match limits for a native workload.
pub struct ScheduleOptions {
    pub mode: Mode,
    pub first_seed: u32,
    pub seeds: u32,
    pub max_rounds: u32,
    pub recruit_limit: usize,
    pub armies: Vec<String>,
    pub landscapes: Vec<String>,
    pub campaigns: Vec<String>,
    pub policies: Vec<Policy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign_players: Option<Vec<Policy>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign_enemies: Option<Vec<Policy>>,
    pub diplomacy: Vec<Diplomacy>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub self_play: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub vs_classic: bool,
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
            first_seed: 1,
            seeds: 2,
            max_rounds: rules.arena.default_max_rounds,
            recruit_limit: rules.arena.default_recruit_limit,
            armies: rules.rosters.iter().map(|item| item.id.clone()).collect(),
            landscapes: rules
                .arena
                .landscapes
                .iter()
                .map(|item| item.id.clone())
                .collect(),
            campaigns: rules.campaign_defaults.campaigns.clone(),
            policies: parse(&rules.arena.policies)?,
            campaign_players: None,
            campaign_enemies: None,
            self_play: false,
            vs_classic: false,
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
    /// Count both seats/initiatives and campaign diplomacy settings with checked arithmetic.
    pub fn expected_games(&self) -> Result<usize, SimulationError> {
        let entrants = self
            .armies
            .len()
            .checked_mul(self.policies.len())
            .ok_or("Schedule too large")?;
        let arena = if self.mode == Mode::Campaign {
            0
        } else {
            let opponents = if self.vs_classic {
                let nonclassic = self
                    .policies
                    .iter()
                    .filter(|p| **p != Policy::Classic)
                    .count();
                nonclassic
                    .checked_mul(2)
                    .and_then(|n| {
                        n.checked_add(usize::from(self.policies.contains(&Policy::Classic)))
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
            .iter()
            .filter(|p| p.supports_campaign())
            .count();
        let campaign_enemies = self
            .campaign_enemies
            .as_ref()
            .unwrap_or(&self.policies)
            .iter()
            .filter(|p| p.supports_campaign())
            .count();
        let campaign = if self.mode == Mode::Arena {
            0
        } else {
            self.campaigns
                .len()
                .checked_mul(campaign_players)
                .and_then(|n| n.checked_mul(campaign_enemies))
                .and_then(|n| n.checked_mul(self.diplomacy.len()))
                .ok_or("Schedule too large")?
        };
        arena
            .checked_add(campaign)
            .and_then(|n| n.checked_mul(self.seeds as usize))
            .ok_or_else(|| "Schedule too large".into())
    }
}
/// Deterministic schedule order matches the TypeScript comparison harness.
pub fn schedule(options: &ScheduleOptions) -> Result<Vec<MatchConfig>, SimulationError> {
    if options.vs_classic && options.self_play {
        return Err("--vs-classic cannot be combined with --self-play".into());
    }
    if options.self_play && options.mode != Mode::Arena {
        return Err("--self-play requires --mode arena".into());
    }
    let end = options
        .first_seed
        .checked_add(options.seeds)
        .ok_or("Seed range overflows u32")?;
    let count = options.expected_games()?;
    let mut configs = Vec::new();
    configs.try_reserve(count).map_err(|error| {
        SimulationError::InvalidConfig(format!("Schedule allocation failed: {error}"))
    })?;
    let mut policies = options.policies.clone();
    if options.vs_classic && !policies.contains(&Policy::Classic) {
        policies.push(Policy::Classic);
    }
    let entrants: Vec<_> = options
        .armies
        .iter()
        .flat_map(|army| policies.iter().map(move |policy| (army, *policy)))
        .collect();
    for seed in options.first_seed..end {
        if options.mode != Mode::Campaign {
            for landscape in &options.landscapes {
                for a in 0..entrants.len() {
                    for b in a..entrants.len() {
                        if options.vs_classic
                            && ((!matches!(entrants[a].1, Policy::Classic)
                                && !matches!(entrants[b].1, Policy::Classic))
                                || (entrants[a].1 == Policy::Classic
                                    && entrants[b].1 == Policy::Classic
                                    && !options.policies.contains(&Policy::Classic)))
                        {
                            continue;
                        }
                        if options.self_play && entrants[a].1 != entrants[b].1 {
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
        if options.mode != Mode::Arena {
            for campaign in &options.campaigns {
                for player in options
                    .campaign_players
                    .as_ref()
                    .unwrap_or(&options.policies)
                    .iter()
                    .filter(|p| p.supports_campaign())
                {
                    for enemy in options
                        .campaign_enemies
                        .as_ref()
                        .unwrap_or(&options.policies)
                        .iter()
                        .filter(|p| p.supports_campaign())
                    {
                        for diplomacy in &options.diplomacy {
                            configs.push(MatchConfig::Campaign(CampaignConfig {
                                campaign: campaign.clone(),
                                player: *player,
                                enemy: *enemy,
                                diplomacy: *diplomacy,
                                seed,
                                max_rounds: options.max_rounds,
                                recruit_limit: options.recruit_limit,
                            }));
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
    fn fixed_classic_panel_preserves_seats_and_initiative_without_duplicate_matches() {
        let rules = Rules::load().unwrap();
        let mut options = ScheduleOptions::defaults(&rules).unwrap();
        options.mode = Mode::Arena;
        options.seeds = 1;
        options.armies = vec!["a".into(), "b".into()];
        options.landscapes = vec!["open".into()];
        options.policies = vec![Policy::Mixed, Policy::Priority];
        options.vs_classic = true;
        for include_classic in [false, true] {
            if include_classic {
                options.policies.push(Policy::Classic);
            }
            let fixed = schedule(&options).unwrap();
            assert_eq!(fixed.len(), options.expected_games().unwrap());
            let mut unique = std::collections::HashSet::new();
            for config in &fixed {
                assert!(unique.insert(serde_json::to_string(config).unwrap()));
            }
            let mut all = options.clone();
            all.vs_classic = false;
            if !include_classic {
                all.policies.push(Policy::Classic);
            }
            let expected: Vec<_> = schedule(&all)
                .unwrap()
                .into_iter()
                .filter(|config| {
                    matches!(config, MatchConfig::Arena(c)
                    if (c.player == Policy::Classic || c.enemy == Policy::Classic)
                        && (include_classic || c.player != c.enemy))
                })
                .collect();
            assert_eq!(
                serde_json::to_value(fixed).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
        }
        options.self_play = true;
        assert!(schedule(&options).is_err());
    }

    #[test]
    fn seats_and_initiatives_are_counterbalanced() {
        let options = ScheduleOptions {
            mode: Mode::Full,
            first_seed: 5,
            seeds: 1,
            max_rounds: 1,
            recruit_limit: 0,
            armies: vec!["a".into(), "b".into()],
            landscapes: vec!["open".into()],
            campaigns: vec!["chapter".into()],
            policies: vec![Policy::Classic],
            campaign_players: None,
            campaign_enemies: None,
            diplomacy: vec![Diplomacy::Pragmatic],
            self_play: false,
            vs_classic: false,
        };
        let configs = schedule(&options).unwrap();
        assert_eq!(configs.len(), 9);
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
        self_play_options.policies = vec![Policy::Classic, Policy::Priority, Policy::Battler];
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
        let mut mixed_capabilities = options.clone();
        mixed_capabilities.policies = vec![Policy::Priority, Policy::Battler];
        let mixed_schedule = schedule(&mixed_capabilities).unwrap();
        assert_eq!(
            mixed_schedule.len(),
            mixed_capabilities.expected_games().unwrap()
        );
        assert_eq!(mixed_schedule.len(), 33);
        assert!(
            mixed_schedule.iter().any(
                |config| matches!(config, MatchConfig::Arena(c) if c.player == Policy::Battler)
            )
        );
        assert_eq!(mixed_schedule.iter().filter(|config| matches!(config, MatchConfig::Campaign(c) if c.player == Policy::Priority && c.enemy == Policy::Priority)).count(), 1);
        let mut independent_roles = options.clone();
        independent_roles.policies = vec![
            Policy::Classic,
            Policy::Rush,
            Policy::Mixed,
            Policy::Priority,
            Policy::Battler,
        ];
        independent_roles.campaign_players = Some(vec![Policy::Mixed, Policy::Priority]);
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
                && [Policy::Mixed, Policy::Priority].contains(&c.player))
        );
        let mut unique = std::collections::HashSet::new();
        for config in configs {
            assert!(unique.insert(serde_json::to_string(&config).unwrap()));
        }
    }
}
