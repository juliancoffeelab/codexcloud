//! Compact reports and statistics derived from complete, owned match results.
pub use super::summaries::{DetailedSummary, borrowed_summary, compact_summary};
use super::{error::SimulationError, schedule::ScheduleOptions, types::MatchResult};
use crate::ai::Random;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Add composition events and state hashes while retaining every recorded phase.
pub fn detailed_summary(result: &MatchResult) -> Result<Value, SimulationError> {
    Ok(serde_json::to_value(borrowed_summary(result, true)?)?)
}
fn tactic_label(config: &Value, side: &str) -> String {
    if let Some(blocks) = config
        .get(format!("{side}Blocks"))
        .filter(|value| !value.is_null())
    {
        format!(
            "{}-{}-{}",
            blocks["recruiter"].as_str().unwrap_or(""),
            blocks["campaign"].as_str().unwrap_or(""),
            blocks["planner"].as_str().unwrap_or("")
        )
    } else {
        config[side].as_str().unwrap_or("").to_owned()
    }
}
fn bot_label(outcome: &Value, side: &str) -> String {
    format!(
        "{}/{}",
        outcome["config"]["armies"][side].as_str().unwrap_or(""),
        tactic_label(&outcome["config"], side)
    )
}
fn is_arena(outcome: &&Value) -> bool {
    outcome["kind"] == "arena"
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Seat-aware pairwise results with uncertainty sampled by whole seed blocks.
pub struct ArenaRow {
    pub policy: String,
    pub opponent: String,
    pub games: usize,
    pub wins: usize,
    pub losses: usize,
    pub unresolved: usize,
    pub win_rate: f64,
    pub decisive_win_rate: Option<f64>,
    pub completion_rate: f64,
    pub net_completed_wins: f64,
    pub seed_blocks: usize,
    pub net_completed_wins95: Option<[f64; 2]>,
    pub mean_rounds: f64,
}
fn labels(outcomes: &[Value]) -> Vec<String> {
    outcomes
        .iter()
        .filter(is_arena)
        .flat_map(|outcome| [bot_label(outcome, "player"), bot_label(outcome, "enemy")])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
struct ArenaObservation<'a> {
    game: &'a Value,
    player: String,
    enemy: String,
}
type Resampling = BTreeMap<usize, Vec<Vec<usize>>>;
fn arena_rows(
    results: &[&ArenaObservation],
    labels: &[String],
    first_seed: u32,
    resampling: &mut Resampling,
) -> Vec<ArenaRow> {
    // Index each seat pair once. Repeatedly formatting both labels for every
    // possible pair made aggregation scan the same games hundreds of times.
    let mut pairs: BTreeMap<(&str, &str), Vec<&ArenaObservation>> = BTreeMap::new();
    for &game in results {
        let player = game.player.as_str();
        let enemy = game.enemy.as_str();
        pairs.entry((player, enemy)).or_default().push(game);
        if player != enemy {
            pairs.entry((enemy, player)).or_default().push(game);
        }
    }
    let mut rows = vec![];
    for policy in labels {
        for opponent in labels {
            let Some(games) = pairs.get(&(policy.as_str(), opponent.as_str())) else {
                continue;
            };
            let win = |observation: &ArenaObservation| {
                observation.game["winner"].as_str().is_some_and(|winner| {
                    if policy == opponent {
                        winner == "player"
                    } else {
                        (if winner == "player" {
                            &observation.player
                        } else {
                            &observation.enemy
                        }) == policy
                    }
                })
            };
            let wins = games.iter().filter(|game| win(game)).count();
            let unresolved = games
                .iter()
                .filter(|game| game.game["winner"].is_null())
                .count();
            let losses = games.len() - wins - unresolved;
            let mut blocks: BTreeMap<u64, (usize, i32)> = BTreeMap::new();
            for game in games {
                let block = blocks
                    .entry(game.game["config"]["seed"].as_u64().unwrap_or(0))
                    .or_default();
                block.0 += 1;
                block.1 += if game.game["winner"].is_null() {
                    0
                } else if win(game) {
                    1
                } else {
                    -1
                };
            }
            let margin95 = if blocks.len() < 2 {
                None
            } else {
                let blocks: Vec<_> = blocks.values().copied().collect();
                let mut samples = Vec::with_capacity(1000);
                if blocks.len() <= 64 {
                    // Every row starts the same RNG stream. Reuse its integer
                    // seed-block multiplicities rather than generate it again.
                    let weights = resampling.entry(blocks.len()).or_insert_with(|| {
                        let mut random = Random(first_seed);
                        (0..1000)
                            .map(|_| {
                                let mut weights = vec![0; blocks.len()];
                                for _ in 0..blocks.len() {
                                    weights
                                        [(random.next() * blocks.len() as f64).floor() as usize] +=
                                        1;
                                }
                                weights
                            })
                            .collect()
                    });
                    for weights in weights {
                        let mut count = 0;
                        let mut margin = 0;
                        for (&repeats, &(total, change)) in weights.iter().zip(&blocks) {
                            count += repeats * total;
                            margin += repeats as i32 * change;
                        }
                        samples.push(margin as f64 / count as f64);
                    }
                } else {
                    // Keep cache memory bounded for very large seed schedules.
                    let mut random = Random(first_seed);
                    for _ in 0..1000 {
                        let mut count = 0;
                        let mut margin = 0;
                        for _ in 0..blocks.len() {
                            let (total, change) =
                                blocks[(random.next() * blocks.len() as f64).floor() as usize];
                            count += total;
                            margin += change;
                        }
                        samples.push(margin as f64 / count as f64);
                    }
                }
                samples.select_nth_unstable_by(974, f64::total_cmp);
                samples[..974].select_nth_unstable_by(24, f64::total_cmp);
                Some([samples[24], samples[974]])
            };
            let total = games.len() as f64;
            rows.push(ArenaRow {
                policy: policy.clone(),
                opponent: opponent.clone(),
                games: games.len(),
                wins,
                losses,
                unresolved,
                win_rate: wins as f64 / total,
                decisive_win_rate: (wins + losses > 0)
                    .then(|| wins as f64 / (wins + losses) as f64),
                completion_rate: (wins + losses) as f64 / total,
                net_completed_wins: (wins as f64 - losses as f64) / total,
                seed_blocks: blocks.len(),
                net_completed_wins95: margin95,
                mean_rounds: games
                    .iter()
                    .map(|game| game.game["rounds"].as_f64().unwrap_or(0.0))
                    .sum::<f64>()
                    / total,
            });
        }
    }
    rows
}
/// Aggregate a complete schedule, rejecting duplicate or missing configurations.
pub fn build_report(
    options: &ScheduleOptions,
    fingerprint: &str,
    source_commit: &str,
    elapsed: f64,
    outcomes: Vec<Value>,
    trace_files: Option<Vec<String>>,
) -> Result<Value, SimulationError> {
    if outcomes.len() != options.expected_games()? {
        return Err("Incomplete schedule coverage".into());
    }
    let mut identities = BTreeSet::new();
    for outcome in &outcomes {
        if !identities.insert(serde_json::to_string(&json!([
            outcome["kind"],
            outcome["config"]
        ]))?) {
            return Err("Duplicate simulation configurations".into());
        }
    }
    let labels = labels(&outcomes);
    let observations: Vec<_> = outcomes
        .iter()
        .filter(is_arena)
        .map(|game| ArenaObservation {
            game,
            player: bot_label(game, "player"),
            enemy: bot_label(game, "enemy"),
        })
        .collect();
    let arena: Vec<_> = observations.iter().collect();
    let mut arena_groups: BTreeMap<(&str, &str, &str), Vec<&ArenaObservation>> = BTreeMap::new();
    for observation in &observations {
        let config = &observation.game["config"];
        arena_groups
            .entry((
                config["armies"]["player"].as_str().unwrap_or(""),
                config["armies"]["enemy"].as_str().unwrap_or(""),
                config["landscape"].as_str().unwrap_or(""),
            ))
            .or_default()
            .push(observation);
    }
    let mut campaign_groups: BTreeMap<(String, String, String, String), Vec<&Value>> =
        BTreeMap::new();
    for game in outcomes.iter().filter(|game| game["kind"] == "campaign") {
        let config = &game["config"];
        campaign_groups
            .entry((
                config["campaign"].as_str().unwrap_or("").to_owned(),
                tactic_label(config, "player"),
                tactic_label(config, "enemy"),
                config["diplomacy"].as_str().unwrap_or("").to_owned(),
            ))
            .or_default()
            .push(game);
    }
    let mut resampling = Resampling::new();
    let rows = arena_rows(&arena, &labels, options.first_seed, &mut resampling);
    let mut strata = vec![];
    for player_army in &options.armies {
        for enemy_army in &options.armies {
            for landscape in &options.landscapes {
                let games = arena_groups
                    .get(&(
                        player_army.as_str(),
                        enemy_army.as_str(),
                        landscape.as_str(),
                    ))
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let rows = arena_rows(games, &labels, options.first_seed, &mut resampling);
                if !rows.is_empty() {
                    strata.push(json!({"playerArmy":player_army,"enemyArmy":enemy_army,"landscape":landscape,"rows":rows}));
                }
            }
        }
    }
    let mut campaign_strata = vec![];
    for ((campaign, player, enemy, alignment), games) in &campaign_groups {
        let chapters: Vec<_> = games
            .iter()
            .flat_map(|game| game["missions"].as_array().into_iter().flatten())
            .collect();
        let mut ids = Vec::new();
        for chapter in &chapters {
            if !ids.contains(&chapter["id"]) {
                ids.push(chapter["id"].clone());
            }
        }
        let missions: Vec<_> = ids.iter().map(|id| {
                let attempts: Vec<_> = chapters.iter().filter(|chapter| chapter["id"] == *id).collect();
                json!({"id":id,"attempted":attempts.len(),"won":attempts.iter().filter(|chapter|chapter["winner"]=="player").count(),"defeated":attempts.iter().filter(|chapter|chapter["winner"]=="enemy").count(),"unresolved":attempts.iter().filter(|chapter|chapter["winner"].is_null()).count()})
            }).collect();
        let last_winner = |game: &&Value| {
            game["missions"]
                .as_array()
                .and_then(|missions| missions.last())
                .map(|mission| mission["winner"].clone())
        };
        campaign_strata.push(json!({"campaign":campaign,"player":player,"enemy":enemy,"diplomacy":alignment,"games":games.len(),"completed":games.iter().filter(|game|game["completed"]==true).count(),"defeated":games.iter().filter(|game|last_winner(game).is_some_and(|winner|winner=="enemy")).count(),"unresolved":games.iter().filter(|game|last_winner(game).is_some_and(|winner|winner.is_null())).count(),"escalated":games.iter().filter(|game|game["missions"].as_array().is_some_and(|missions|missions.iter().any(|mission|mission["escalated"]==true))).count(),"missions":missions}));
    }
    let army_compositions: Vec<_> = labels
        .iter()
        .map(|bot| {
            let mut totals: BTreeMap<String, Value> = BTreeMap::new();
            let mut games = 0;
            for game in &arena {
                for side in ["player", "enemy"] {
                    if (if side == "player" {
                        &game.player
                    } else {
                        &game.enemy
                    }) != bot
                    {
                        continue;
                    }
                    games += 1;
                    for (list, key) in [
                        ("recruits", "recruited"),
                        ("casualties", "lost"),
                        ("promotions", "promoted"),
                    ] {
                        for unit in game.game[list]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter(|unit| unit["side"] == side)
                        {
                            let unit_type = unit["type"].as_str().unwrap_or("").to_owned();
                            let value = totals
                                .entry(unit_type)
                                .or_insert_with(|| json!({"recruited":0,"lost":0,"promoted":0}));
                            value[key] = json!(value[key].as_u64().unwrap_or(0) + 1);
                        }
                    }
                }
            }
            json!({"bot":bot,"games":games,"units":totals})
        })
        .collect();
    let seat_diagnostics: Vec<_> = labels.iter().map(|policy| {
        let games: Vec<_> = arena.iter().copied().filter(|game|game.player==*policy && game.enemy==*policy).collect();
        json!({"policy":policy,"games":games.len(),"playerWins":games.iter().filter(|game|game.game["winner"]=="player").count(),"enemyWins":games.iter().filter(|game|game.game["winner"]=="enemy").count(),"firstWins":games.iter().filter(|game|game.game["winner"]==game.game["config"]["first"]).count(),"secondWins":games.iter().filter(|game|!game.game["winner"].is_null() && game.game["winner"]!=game.game["config"]["first"]).count(),"unresolved":games.iter().filter(|game|game.game["winner"].is_null()).count()})
    }).collect();
    let mut report = json!({"schema":3,"status":"complete","mode":options.mode,"fingerprint":fingerprint,"sourceCommit":source_commit,"engine":"rust","elapsedSeconds":elapsed,"seeds":{"first":options.first_seed,"count":options.seeds,"campaignCount":options.campaign_seeds},"schedule":options,"rows":rows,"strata":strata,"campaignStrata":campaign_strata,"armyCompositions":army_compositions,"seatDiagnostics":seat_diagnostics,"outcomes":outcomes,"detailFormat":"jsonl-byte-ranges-v1"});
    report["recruitmentLimits"] = serde_json::to_value(
        crate::ai::Policy::ALL
            .iter()
            .map(|policy| {
                (
                    policy.as_str(),
                    options
                        .recruit_limit
                        .or_else(|| policy.default_recruitment_limit()),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )?;
    let custom_labels: BTreeSet<_> = report["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|game| {
            ["player", "enemy"].into_iter().filter_map(move |side| {
                game["config"]
                    .get(format!("{side}Blocks"))
                    .filter(|value| !value.is_null())
                    .map(|_| tactic_label(&game["config"], side))
            })
        })
        .collect();
    for label in custom_labels {
        report["recruitmentLimits"][label] = json!(options.recruit_limit);
    }
    if let Some(files) = trace_files {
        report["traceFiles"] = json!(files);
    }
    Ok(report)
}
/// Render the compact report into a readable results table and workload summary.
pub fn markdown(report: &Value) -> String {
    let first = report["seeds"]["first"].as_u64().unwrap_or(0);
    let count = report["seeds"]["count"].as_u64().unwrap_or(1);
    let campaign_count = report["seeds"]["campaignCount"].as_u64().unwrap_or(count);
    let seeds = if report["mode"] == "full" && count != campaign_count {
        format!(
            "arena seeds {first}–{}; campaign seeds {first}–{}",
            first + count - 1,
            first + campaign_count - 1
        )
    } else {
        format!("seeds {first}–{}", first + count - 1)
    };
    let mut text = format!(
        "# Seeded AI {} results\n\nSource: `{}`. Rules/policy fingerprint: `{}`.\n\n{} games, {}, {:.1} seconds.\n\n",
        report["mode"].as_str().unwrap_or(""),
        report["sourceCommit"].as_str().unwrap_or(""),
        report["fingerprint"].as_str().unwrap_or(""),
        report["outcomes"].as_array().map_or(0, Vec::len),
        seeds,
        report["elapsedSeconds"].as_f64().unwrap_or(0.0)
    );
    if let Some(timing) = report.get("turnTiming") {
        text.push_str("## Turn timing\n\nOne turn is one side's own phase. CPU timing excludes journal encoding and report aggregation; turn-transition events are outside the measured phase.\n\n| Policy | Turns | Mean AI decision µs/turn | Mean complete phase µs/turn |\n|---|---:|---:|---:|\n");
        for row in timing["policies"].as_array().into_iter().flatten() {
            text.push_str(&format!(
                "| {} | {} | {:.2} | {:.2} |\n",
                row["policy"].as_str().unwrap_or(""),
                row["turns"],
                row["meanDecisionMicrosPerTurn"].as_f64().unwrap_or(0.0),
                row["meanTurnMicros"].as_f64().unwrap_or(0.0)
            ));
        }
        text.push('\n');
    }
    if report["mode"] != "campaign" {
        text.push_str("| Policy | Opponent | Games | Wins | Losses | Unresolved | Mean rounds |\n|---|---|---:|---:|---:|---:|---:|\n");
        for row in report["rows"].as_array().into_iter().flatten() {
            text.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {:.1} |\n",
                row["policy"].as_str().unwrap_or(""),
                row["opponent"].as_str().unwrap_or(""),
                row["games"],
                row["wins"],
                row["losses"],
                row["unresolved"],
                row["meanRounds"].as_f64().unwrap_or(0.0)
            ));
        }
        text.push('\n');
    }
    if report["mode"] != "arena" {
        text.push_str("| Campaign | Player policy | Enemy policy | Diplomacy | Seed | Completed | Recorded chapters |\n|---|---|---|---|---:|---|---|\n");
        for game in report["outcomes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|game| game["kind"] == "campaign")
        {
            let chapters = game["missions"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|mission| {
                    format!(
                        "{}: {}@{}",
                        mission["id"].as_str().unwrap_or(""),
                        mission["winner"].as_str().unwrap_or("unresolved"),
                        mission["round"]
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            text.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                game["config"]["campaign"].as_str().unwrap_or(""),
                tactic_label(&game["config"], "player"),
                tactic_label(&game["config"], "enemy"),
                game["config"]["diplomacy"].as_str().unwrap_or(""),
                game["config"]["seed"],
                game["completed"],
                chapters
            ));
        }
    }
    text.push_str("\nCaps are unresolved, not draws or defeats. Arena seats and initiative are counterbalanced on identical paired terrain/resources, with race-specific rosters. Confidence intervals resample whole seed blocks. Campaigns retain authored asymmetry, earn their carryover, and receive no retries. Complete phase commands, combats, troop states, ownership, goals and endpoint states are in adjacent result journals, indexed by each outcome's detail reference.\n");
    text
}

#[cfg(test)]
mod tests {
    use super::super::schedule::Mode;
    use super::*;
    use crate::ai::{Diplomacy, Policy};
    #[test]
    fn self_play_statistics_use_player_seat_and_whole_seed_blocks() {
        let options = ScheduleOptions {
            mode: Mode::Arena,
            with: None,
            campaign_custom_player: false,
            campaign_custom_enemy: false,
            first_seed: 1,
            seeds: 2,
            campaign_seeds: 2,
            max_rounds: 2,
            campaign_max_rounds: 2,
            recruit_limit: Some(0),
            armies: vec!["human".into()],
            landscapes: vec!["open".into()],
            campaigns: vec![],
            policies: vec![Policy::Classic],
            campaign_players: None,
            campaign_enemies: None,
            diplomacy: vec![Diplomacy::Pragmatic],
            campaign_diplomacy: Default::default(),
            self_play: false,
            opponent: None,
        };
        let outcomes = [Some("player"),Some("enemy"),None,Some("player")].iter().enumerate().map(|(index,winner)|json!({"kind":"arena","config":{"seed":1+index/2,"armies":{"player":"human","enemy":"human"},"landscape":"open","player":"classic","enemy":"classic","first":if index%2==0{"player"}else{"enemy"}},"winner":winner,"rounds":2,"recruits":[],"casualties":[],"promotions":[]})).collect();
        let report = build_report(&options, "fp", "commit", 0.1, outcomes, None).unwrap();
        let row = &report["rows"][0];
        assert_eq!(row["games"], 4);
        assert_eq!(row["wins"], 2);
        assert_eq!(row["losses"], 1);
        assert_eq!(row["unresolved"], 1);
        assert_eq!(row["seedBlocks"], 2);
        assert_eq!(row["netCompletedWins"], 0.25);
        assert_eq!(row["netCompletedWins95"], json!([0.0, 0.5]));
        assert_eq!(report["armyCompositions"][0]["games"], 8);
        assert_eq!(report["seatDiagnostics"][0]["playerWins"], 2);
    }
}
