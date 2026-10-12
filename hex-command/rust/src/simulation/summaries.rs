//! Borrow full traces during serialization; allocate only small composition summaries.
use super::{
    error::SimulationError,
    records::{DetailRecord, sha256},
    types::*,
};
use crate::data::{ActionRecord, Progress, PublicUnit, Side, State};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct UnitEvent<'a> {
    side: Side,
    #[serde(rename = "type")]
    kind: &'a str,
}
#[derive(Serialize)]
struct Events<'a> {
    casualties: Vec<&'a PublicUnit>,
    recruits: Vec<UnitEvent<'a>>,
    promotions: Vec<UnitEvent<'a>>,
    #[serde(rename = "stateHash", skip_serializing_if = "Option::is_none")]
    state_hash: Option<String>,
}
fn events<'a>(
    phases: &'a [Phase],
    final_state: &State,
    hash_state: bool,
) -> Result<Events<'a>, SimulationError> {
    let mut result = Events {
        casualties: vec![],
        recruits: vec![],
        promotions: vec![],
        state_hash: None,
    };
    for phase in phases {
        result.casualties.extend(&phase.records.losses);
        for action in &phase.records.actions {
            match action {
                ActionRecord::Recruit { unit_type, .. } => result.recruits.push(UnitEvent {
                    side: phase.side,
                    kind: unit_type,
                }),
                ActionRecord::Advance { unit_type, .. } => result.promotions.push(UnitEvent {
                    side: phase.side,
                    kind: unit_type,
                }),
                _ => {}
            }
        }
    }
    if hash_state {
        // Keep canonical, sorted-object hashing used by existing native result journals.
        result.state_hash = Some(sha256(&super::canonical::state_bytes(final_state)?));
    }
    Ok(result)
}
#[derive(Serialize)]
pub struct ArenaSummary<'a> {
    kind: &'static str,
    #[serde(flatten)]
    result: &'a ArenaResult,
    #[serde(flatten)]
    events: Events<'a>,
}
#[derive(Serialize)]
pub struct MissionSummary<'a> {
    #[serde(flatten)]
    result: &'a MissionResult,
    #[serde(flatten)]
    events: Events<'a>,
}
#[derive(Serialize)]
pub struct CampaignSummary<'a> {
    kind: &'static str,
    config: &'a CampaignConfig,
    completed: bool,
    missions: Vec<MissionSummary<'a>>,
}
/// A summary that references every phase and endpoint instead of cloning them into JSON values.
#[derive(Serialize)]
#[serde(untagged)]
pub enum DetailedSummary<'a> {
    Arena(ArenaSummary<'a>),
    Campaign(CampaignSummary<'a>),
}
/// Build borrowed details, optionally omitting all derived endpoint hashing.
pub fn borrowed_summary(
    result: &MatchResult,
    hash_states: bool,
) -> Result<DetailedSummary<'_>, SimulationError> {
    match result {
        MatchResult::Arena(result) => Ok(DetailedSummary::Arena(ArenaSummary {
            kind: "arena",
            result,
            events: events(&result.phases, &result.r#final, hash_states)?,
        })),
        MatchResult::Campaign(result) => {
            let missions = result
                .missions
                .iter()
                .map(|mission| {
                    Ok(MissionSummary {
                        result: mission,
                        events: events(&mission.phases, &mission.r#final, hash_states)?,
                    })
                })
                .collect::<Result<_, SimulationError>>()?;
            Ok(DetailedSummary::Campaign(CampaignSummary {
                kind: "campaign",
                config: &result.config,
                completed: result.completed,
                missions,
            }))
        }
    }
}
/// Lossless Arrow header; complete phase boundaries live in indexed columns.
pub(crate) fn arrow_header(summary: &DetailedSummary<'_>) -> Result<String, SimulationError> {
    #[derive(Serialize)]
    struct ArenaHeader<'a> {
        kind: &'static str,
        config: &'a ArenaConfig,
        winner: Option<Side>,
        reason: &'a str,
        rounds: usize,
        phases: &'a [()],
        initial: &'a State,
        r#final: &'a State,
        objectives: &'a Sides<ArenaScore>,
        #[serde(flatten)]
        events: &'a Events<'a>,
    }
    #[derive(Serialize)]
    struct MissionHeader<'a> {
        id: &'a str,
        winner: Option<Side>,
        round: u32,
        goals: &'a [Progress],
        phases: &'a [()],
        initial: &'a State,
        r#final: &'a State,
        #[serde(flatten)]
        events: &'a Events<'a>,
    }
    #[derive(Serialize)]
    struct CampaignHeader<'a> {
        kind: &'static str,
        config: &'a CampaignConfig,
        completed: bool,
        missions: Vec<MissionHeader<'a>>,
    }
    Ok(match summary {
        DetailedSummary::Arena(s) => serde_json::to_string(&ArenaHeader {
            kind: "arena",
            config: &s.result.config,
            winner: s.result.winner,
            reason: &s.result.reason,
            rounds: s.result.rounds,
            phases: &[],
            initial: &s.result.initial,
            r#final: &s.result.r#final,
            objectives: &s.result.objectives,
            events: &s.events,
        })?,
        DetailedSummary::Campaign(s) => serde_json::to_string(&CampaignHeader {
            kind: "campaign",
            config: s.config,
            completed: s.completed,
            missions: s
                .missions
                .iter()
                .map(|m| MissionHeader {
                    id: &m.result.id,
                    winner: m.result.winner,
                    round: m.result.round,
                    goals: &m.result.goals,
                    phases: &[],
                    initial: &m.result.initial,
                    r#final: &m.result.r#final,
                    events: &m.events,
                })
                .collect(),
        })?,
    })
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArenaCompact<'a> {
    kind: &'static str,
    config: &'a ArenaConfig,
    winner: Option<Side>,
    reason: &'a str,
    rounds: usize,
    objectives: &'a Sides<ArenaScore>,
    #[serde(flatten)]
    events: &'a Events<'a>,
    detail: DetailRecord,
    elapsed_ms: f64,
}
#[derive(Serialize)]
struct MissionCompact<'a> {
    id: &'a str,
    winner: Option<Side>,
    round: u32,
    goals: &'a [Progress],
    #[serde(flatten)]
    events: &'a Events<'a>,
    escalated: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignCompact<'a> {
    kind: &'static str,
    config: &'a CampaignConfig,
    completed: bool,
    missions: Vec<MissionCompact<'a>>,
    detail: DetailRecord,
    elapsed_ms: f64,
}
/// Serialize only compact statistics and journal references; never serialize or clone full traces.
pub fn compact_summary(
    summary: &DetailedSummary<'_>,
    detail: DetailRecord,
    elapsed_ms: f64,
) -> Result<Value, SimulationError> {
    match summary {
        DetailedSummary::Arena(summary) => {
            let result = summary.result;
            Ok(serde_json::to_value(ArenaCompact {
                kind: "arena",
                config: &result.config,
                winner: result.winner,
                reason: &result.reason,
                rounds: result.rounds,
                objectives: &result.objectives,
                events: &summary.events,
                detail,
                elapsed_ms,
            })?)
        }
        DetailedSummary::Campaign(summary) => {
            let missions = summary
                .missions
                .iter()
                .map(|mission| MissionCompact {
                    id: &mission.result.id,
                    winner: mission.result.winner,
                    round: mission.result.round,
                    goals: &mission.result.goals,
                    events: &mission.events,
                    escalated: mission
                        .result
                        .r#final
                        .campaign
                        .as_ref()
                        .and_then(|campaign| campaign.refuge.as_ref())
                        .is_some_and(|refuge| refuge.escalated),
                })
                .collect();
            Ok(serde_json::to_value(CampaignCompact {
                kind: "campaign",
                config: summary.config,
                completed: summary.completed,
                missions,
                detail,
                elapsed_ms,
            })?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::Policy;
    use crate::data::{Gold, Records};
    use serde_json::json;
    fn fixture() -> MatchResult {
        let state = State {
            map_id: "fixture".into(),
            map_revision: 1,
            cols: 1,
            rows: 1,
            round: 1,
            turn: Side::Player,
            gold: Gold {
                player: 1,
                enemy: 2,
            },
            winner: None,
            next_id: 1,
            log: vec![],
            tiles: vec![],
            units: vec![],
            campaign: None,
            last_combat: Some(None),
        };
        MatchResult::Arena(ArenaResult {
            config: ArenaConfig {
                seed: 1,
                armies: Armies {
                    player: "human".into(),
                    enemy: "human".into(),
                },
                landscape: "open".into(),
                player: Policy::Classic,
                enemy: Policy::Classic,
                first: Side::Player,
                max_rounds: 1,
                recruit_limit: 0,
            },
            winner: None,
            reason: "cap".into(),
            rounds: 1,
            phases: vec![Phase {
                side: Side::Player,
                round: 1,
                records: Records {
                    actions: vec![
                        ActionRecord::Recruit {
                            unit: "u1".into(),
                            unit_type: "archer".into(),
                            tile: "0,0".into(),
                        },
                        ActionRecord::Advance {
                            unit: "u1".into(),
                            unit_type: "ranger".into(),
                        },
                    ],
                    losses: vec![],
                    combats: vec![],
                },
                living: Sides {
                    player: 0,
                    enemy: 0,
                },
                health: Sides {
                    player: 0,
                    enemy: 0,
                },
                gold: state.gold.clone(),
                before: vec![],
                units: vec![],
                ownership: std::sync::Arc::new(OwnershipSnapshot {
                    tiles: std::sync::Arc::new(vec![]),
                    owners: vec![],
                }),
                goals: vec![],
                objectives: vec![],
                campaign: None,
            }],
            initial: state.clone(),
            r#final: state,
            objectives: Sides {
                player: ArenaScore {
                    controlled: 0,
                    held: 0,
                    optional_visits: 0,
                },
                enemy: ArenaScore {
                    controlled: 0,
                    held: 0,
                    optional_visits: 0,
                },
            },
        })
    }
    #[test]
    fn borrowed_details_preserve_wire_data_and_comparison_omits_hashes() {
        let result = fixture();
        let summary = borrowed_summary(&result, true).unwrap();
        let value = serde_json::to_value(&summary).unwrap();
        let mut expected = serde_json::to_value(&result).unwrap();
        expected["kind"] = json!("arena");
        expected["casualties"] = json!([]);
        expected["recruits"] = json!([{"side":"player","type":"archer"}]);
        expected["promotions"] = json!([{"side":"player","type":"ranger"}]);
        expected["stateHash"] = json!(sha256(&serde_json::to_vec(&expected["final"]).unwrap()));
        assert_eq!(value, expected);
        assert!(value["final"]["lastCombat"].is_null());
        assert!(value["final"].get("campaign").is_none());
        let comparison = serde_json::to_value(borrowed_summary(&result, false).unwrap()).unwrap();
        assert!(comparison.get("stateHash").is_none());
        expected.as_object_mut().unwrap().remove("stateHash");
        assert_eq!(comparison, expected);
        let compact = compact_summary(
            &summary,
            DetailRecord {
                file: "details.jsonl".into(),
                offset: 0,
                bytes: 1,
                sha256: "hash".into(),
            },
            1.0,
        )
        .unwrap();
        assert!(compact.get("phases").is_none());
        assert!(compact.get("initial").is_none());
        assert!(compact.get("final").is_none());
        assert_eq!(compact["recruits"], value["recruits"]);
        assert_eq!(compact["stateHash"], value["stateHash"]);
    }
    #[test]
    fn campaign_compaction_retains_chapters_and_escalation() {
        let MatchResult::Arena(arena) = fixture() else {
            unreachable!()
        };
        let mut final_state = arena.r#final;
        final_state.campaign = Some(serde_json::from_value(json!({"id":"fixture","mission":0,"recall":[],"carryGold":0,"marks":[],"dialogue":[],"dialogueIndex":0,"outcome":null,"completed":false,"refuge":{"heldTurns":0,"lastCountedRound":0,"escalated":true}})).unwrap());
        let result = MatchResult::Campaign(CampaignResult {
            config: CampaignConfig {
                seed: 1,
                diplomacy: crate::ai::Diplomacy::Pragmatic,
                campaign: "fixture".into(),
                player: Policy::Classic,
                enemy: Policy::Classic,
                max_rounds: 1,
                recruit_limit: 0,
            },
            completed: false,
            missions: vec![MissionResult {
                id: "chapter".into(),
                winner: None,
                round: 1,
                goals: vec![],
                phases: arena.phases,
                initial: arena.initial,
                r#final: final_state,
            }],
        });
        let summary = borrowed_summary(&result, true).unwrap();
        let detail = serde_json::to_value(&summary).unwrap();
        let compact = compact_summary(
            &summary,
            DetailRecord {
                file: "details.jsonl".into(),
                offset: 0,
                bytes: 1,
                sha256: "hash".into(),
            },
            1.0,
        )
        .unwrap();
        assert_eq!(compact["missions"][0]["escalated"], true);
        assert_eq!(
            compact["missions"][0]["recruits"],
            detail["missions"][0]["recruits"]
        );
        assert_eq!(
            compact["missions"][0]["stateHash"],
            detail["missions"][0]["stateHash"]
        );
        assert!(compact["missions"][0].get("phases").is_none());
        assert!(detail["missions"][0]["final"]["lastCombat"].is_null());
        let comparison = serde_json::to_value(borrowed_summary(&result, false).unwrap()).unwrap();
        assert!(comparison["missions"][0].get("stateHash").is_none());
        assert_eq!(
            comparison["missions"][0]["phases"],
            detail["missions"][0]["phases"]
        );
    }
}
