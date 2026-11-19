//! Complete seeded arena and campaign runs through the shared battle/AI rules.
use super::{error::SimulationError, types::*};
use crate::{
    ai::{
        Diplomacy, PlanningWorkspace, Policy, policies::objectives::objective,
        runtime::tactic::PresetTactic,
    },
    data::*,
    logic::BattleSession,
};
use hex_command_battle::CampaignProgram;
use rustc_hash::FxHashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// Reuse lossless encodings of unchanged boundary values within one chapter.
/// Keys contain every unit field; ownership keys retain the entire ordered map.
#[derive(Default)]
struct RecordingCache {
    units: FxHashMap<(Unit, UnitPerceptionFacts), Arc<PublicUnit>>,
    ownership: FxHashMap<Vec<Option<String>>, Arc<OwnershipSnapshot>>,
    tiles: Option<Arc<Vec<Hex>>>,
}
impl RecordingCache {
    fn units(&mut self, game: &BattleSession<'_>) -> Result<Vec<Arc<PublicUnit>>, SimulationError> {
        game.authoritative_units()
            .iter()
            .map(|unit| {
                let key = (*unit, game.authoritative_perception(unit).clone());
                if let Some(encoded) = self.units.get(&key) {
                    return Ok(encoded.clone());
                }
                let encoded = Arc::new(game.public_unit(unit));
                self.units.insert(key, encoded.clone());
                Ok(encoded)
            })
            .collect()
    }
    fn ownership(
        &mut self,
        game: &BattleSession<'_>,
    ) -> Result<Arc<OwnershipSnapshot>, SimulationError> {
        if let Some(encoded) = self.ownership.get(game.owners()) {
            return Ok(encoded.clone());
        }
        let tiles = self
            .tiles
            .get_or_insert_with(|| Arc::new(game.board().ids.clone()))
            .clone();
        let encoded = Arc::new(OwnershipSnapshot {
            tiles,
            owners: game.owners().clone(),
        });
        self.ownership
            .insert(game.owners().clone(), encoded.clone());
        Ok(encoded)
    }
}

pub(crate) fn create_arena<'catalog>(
    rules: &'catalog Catalogue,
    config: &ArenaConfig,
) -> Result<BattleSession<'catalog>, SimulationError> {
    validate_arena(rules, config)?;
    let landscape = rules
        .meta()
        .simulation
        .arena
        .scenarios
        .iter()
        .find(|l| l.id == config.landscape)
        .unwrap();
    let variant =
        &landscape.variants[config.seed.saturating_sub(1) as usize % landscape.variants.len()];
    let map_id = &variant.map;
    let map = rules.map(map_id, variant.map_revision);
    let mut state = BattleSession::from_map(rules, map).state();
    if matches!(variant.forces, ArenaForces::Scenario) {
        state.turn = config.first;
        state.round = 1;
        state.winner = None;
        state.campaign = None;
        state.log.clear();
        state.last_combat = None;
        let mut game = BattleSession::from_state(rules, state);
        game.set_combat_rng(config.seed);
        // None preserves each authored faction's own recruitment pool and gold.
        game.configure_arena(None, Some(config.first));
        return Ok(game);
    }
    state.units.clear();
    state.next_id = 1;
    let mut starting_cost = [0; 2];
    for (seat, (faction, army)) in [
        ("player", &config.armies.player),
        ("enemy", &config.armies.enemy),
    ]
    .into_iter()
    .enumerate()
    {
        let roster = roster(rules, army)?;
        let leader = roster.leader.as_str();
        crate::logic::place_selected_leader(rules, &mut state, faction, leader)?;
        starting_cost[seat] += rules.unit(leader).recruitment_price(None);
    }
    let ArenaForces::Selected { budget } = variant.forces else {
        unreachable!()
    };
    if starting_cost.iter().any(|&cost| cost > budget) {
        return Err("Arena starting troops exceed the shared budget".into());
    }
    state.gold = Gold::two(
        budget - starting_cost[TurnPhase::Player.index()],
        budget - starting_cost[TurnPhase::Ai.index()],
    );
    state.round = 1;
    state.turn = config.first;
    state.winner = None;
    state.log.clear();
    state.campaign = None;
    state.last_combat = None;
    let mut game = BattleSession::from_state(rules, state);
    game.set_combat_rng(config.seed);
    let pools = Some([
        roster(rules, &config.armies.player)?
            .pool
            .iter()
            .map(|k| game.kind(k))
            .collect(),
        roster(rules, &config.armies.enemy)?
            .pool
            .iter()
            .map(|k| game.kind(k))
            .collect(),
    ]);
    game.configure_arena(pools, Some(config.first));
    Ok(game)
}
fn roster<'a>(
    rules: &'a Catalogue,
    id: &str,
) -> Result<&'a crate::data::ArmyRoster, SimulationError> {
    rules
        .meta()
        .simulation
        .rosters
        .iter()
        .find(|roster| roster.id == id)
        .ok_or_else(|| SimulationError::InvalidConfig(format!("Unknown army: {id}")))
}
fn validate_limits(rounds: u32, recruits: Option<usize>) -> Result<(), SimulationError> {
    if !(1..=1000).contains(&rounds) {
        return Err("maxRounds must be 1..1000".into());
    }
    if recruits.is_some_and(|limit| limit > 20) {
        return Err("recruitLimit must be 0..20".into());
    }
    Ok(())
}
/// Validate roster, landscape and execution limits before constructing an arena.
pub fn validate_arena(rules: &Catalogue, config: &ArenaConfig) -> Result<(), SimulationError> {
    validate_limits(config.max_rounds, config.recruit_limit)?;
    roster(rules, &config.armies.player)?;
    roster(rules, &config.armies.enemy)?;
    if !rules
        .meta()
        .simulation
        .arena
        .scenarios
        .iter()
        .any(|item| item.id == config.landscape)
    {
        return Err(SimulationError::InvalidConfig(format!(
            "Unknown landscape: {}",
            config.landscape
        )));
    }
    Ok(())
}
fn phase(
    game: &mut BattleSession<'_>,
    recording: &mut RecordingCache,
    policy: Policy,
    seed: u32,
    limit: Option<usize>,
    goals: Option<&[Objective]>,
    diplomacy: Diplomacy,
    tactic: &mut PresetTactic,
) -> Result<Phase, SimulationError> {
    let timing = crate::ai::profiling::Turn::start(policy, game.turn());
    let before = recording.units(game)?;
    let side = game.turn();
    let round = game.round();
    let leader_audit = super::leader_audit::begin(game);
    game.ensure_recording();
    let scoped = !game.factions().is_empty();
    let controllers = if scoped {
        game.automatic_factions()
            .into_iter()
            .map(|(id, policy)| (Some(id), policy.map(Into::into)))
            .collect()
    } else {
        vec![(None, None)]
    };
    let mut diagnostics = None;
    for (index, (faction, authored_policy)) in controllers.into_iter().enumerate() {
        if game.winner().is_some() {
            break;
        }
        let selected = authored_policy.unwrap_or(policy);
        let mut authored_tactic = PresetTactic::from_blocks(selected.blocks());
        let selected_tactic = if authored_policy.is_some() {
            &mut authored_tactic
        } else {
            &mut *tactic
        };
        let mut workspace = PlanningWorkspace::new(
            seed.wrapping_add(index as u32),
            limit.or_else(|| selected.default_recruitment_limit()),
            goals.map(<[Objective]>::to_vec),
            diplomacy,
        );
        let composition = selected_tactic.composition();
        if !composition.campaign.supports_campaign() && game.campaign_state().is_some() {
            return Err(CommandError::BattleOnlyPolicy.into());
        }
        game.set_active_faction(faction);
        let mut recruitment = None;
        let planning = loop {
            let plan = {
                let _decision = crate::ai::profiling::Decision::start(selected, game.turn());
                selected_tactic.run_turn(game, &mut workspace)?
            };
            recruitment.get_or_insert(plan.recruitment);
            crate::ai::runtime::execute_proposals(game, &mut workspace, plan.commands)?;
            if workspace.finished() || game.winner().is_some() {
                break plan.planning;
            }
        };
        game.set_active_faction(None);
        if diagnostics.is_none() {
            diagnostics = workspace.diagnostics(planning, recruitment.unwrap_or_default());
        }
    }
    game.dismiss_all_dialogue();
    let records = game.take_recordings().unwrap_or_default();
    let count = |side| {
        game.authoritative_units()
            .iter()
            .filter(|u| u.phase == side)
            .count()
    };
    let health = |side| {
        game.authoritative_units()
            .iter()
            .filter(|u| u.phase == side)
            .map(|u| u.hp)
            .sum()
    };
    let phase = Phase {
        leader_audit: super::leader_audit::finish(game, leader_audit),
        planning: diagnostics,
        phase: side,
        round,
        records,
        living: Sides {
            player: count(TurnPhase::Player),
            enemy: count(TurnPhase::Ai),
        },
        health: Sides {
            player: health(TurnPhase::Player),
            enemy: health(TurnPhase::Ai),
        },
        gold: game.gold().clone(),
        before,
        units: recording.units(game)?,
        ownership: recording.ownership(game)?,
        goals: game.event_goal_progress(),
        objectives: goals.unwrap_or(&[]).to_vec(),
        campaign: game.campaign_snapshot(),
    };
    if let Some(timing) = timing {
        timing.finish();
    }
    Ok(phase)
}
/// Execute one arena until victory or its cap, retaining every phase and endpoint.
pub fn run_arena(rules: &Catalogue, config: ArenaConfig) -> Result<ArenaResult, SimulationError> {
    run_arena_capturing(rules, config, None)
}
pub(super) fn run_arena_capturing(
    rules: &Catalogue,
    config: ArenaConfig,
    mut capture: Option<&mut super::phase_fixtures::Capture>,
) -> Result<ArenaResult, SimulationError> {
    let mut tactics = [
        PresetTactic::from_blocks(
            config
                .player_blocks
                .unwrap_or_else(|| config.player.blocks()),
        ),
        PresetTactic::from_blocks(config.enemy_blocks.unwrap_or_else(|| config.enemy.blocks())),
    ];
    let mut game = create_arena(rules, &config)?;
    let initial = game.state();
    game.begin_turn(config.first);
    let landscape = rules
        .meta()
        .simulation
        .arena
        .scenarios
        .iter()
        .find(|item| item.id == config.landscape)
        .ok_or("Unknown arena landscape")?;
    let territory = &landscape.territory;
    let hold_turns = landscape.hold_turns;
    let deathmatch = config.objective == ArenaObjective::Deathmatch;
    let mut held = [0u32; 2];
    let mut visits: [HashSet<Hex>; 2] = Default::default();
    let mut phases = vec![];
    let mut recording = RecordingCache::default();
    let mut reason = "cap";
    let mut rounds = 0;
    let seats = [("player", TurnPhase::Player), ("enemy", TurnPhase::Ai)];
    for index in 0..config.max_rounds * 2 {
        if game.winner().is_some() {
            break;
        }
        let (faction, side) = seats[(index as usize + config.first.index()) % seats.len()];
        debug_assert_eq!(side, game.turn());
        let mut goals = if deathmatch {
            vec![]
        } else {
            vec![objective(
                "required-territory",
                ObjectiveKind::Capture,
                territory.clone(),
                24.0,
            )]
        };
        for tile in 0..game.terrain().len() {
            if game.terrain_definition(tile).income() > 0 && game.captures(tile, faction) {
                goals.push(objective(
                    &format!("village:{}", game.board().ids[tile]),
                    ObjectiveKind::Capture,
                    vec![game.board().ids[tile].clone()],
                    8.0,
                ));
            }
        }
        goals.push(objective(
            "opposing-leader",
            ObjectiveKind::Defeat,
            game.units()
                .iter()
                .filter(|u| game.hostile_to_faction(u, faction) && u.is_leader())
                .map(|u| game.board().ids[u.tile].clone())
                .collect(),
            12.0,
        ));
        for tile in 0..game.terrain().len() {
            if !deathmatch
                && game.terrain_definition(tile).id() == Terrain::Ruins
                && !visits[side.index()].contains(&game.board().ids[tile])
            {
                goals.push(objective(
                    &format!("optional:{}", game.board().ids[tile]),
                    ObjectiveKind::Visit,
                    vec![game.board().ids[tile].clone()],
                    4.0,
                ));
            }
        }
        if let Some(capture) = capture.as_deref_mut() {
            capture.observe(&game, Some(&goals));
        }
        let record = phase(
            &mut game,
            &mut recording,
            if faction == "player" {
                config.player
            } else {
                config.enemy
            },
            config.seed,
            config.recruit_limit,
            Some(&goals),
            Diplomacy::Unrestricted,
            &mut tactics[side.index()],
        )?;
        for command in &record.records.actions {
            if let ActionRecord::Move { tile, .. } = command {
                if (game.terrain_definition(game.site(tile)).id() == Terrain::Ruins) {
                    visits[side.index()].insert(tile.clone());
                }
            }
        }
        phases.push(record);
        rounds = phases.len().div_ceil(2);
        if game.winner().is_some() {
            reason = "leader";
            break;
        }
        for (seat, &(owner, phase)) in seats.iter().enumerate().filter(|_| !deathmatch) {
            if !territory
                .iter()
                .all(|tile| game.owns(game.site(tile), owner))
            {
                held[seat] = 0;
            } else if phase != side {
                held[seat] += 1;
            }
            if held[seat] >= hold_turns {
                game.declare_outcome(if owner == "player" {
                    BattleResult::PlayerVictory
                } else {
                    BattleResult::PlayerDefeat
                });
                reason = "territory";
                break;
            }
        }
        if game.winner().is_none() && index + 1 < config.max_rounds * 2 {
            game.start_recording();
            game.end_turn();
        }
    }
    let score = |faction: &str, seat: usize| ArenaScore {
        controlled: territory
            .iter()
            .filter(|tile| game.owns(game.site(tile), faction))
            .count(),
        held: held[seat],
        optional_visits: visits[seat].len(),
    };
    Ok(ArenaResult {
        winner: game.winner(),
        reason: reason.into(),
        rounds,
        phases,
        initial,
        r#final: game.state(),
        objectives: Sides {
            player: score("player", 0),
            enemy: score("enemy", 1),
        },
        config,
    })
}
/// Execute reached campaign chapters with veteran carryover and no retries.
pub fn run_campaign(
    rules: &Catalogue,
    config: CampaignConfig,
    definition: &CampaignProgram,
) -> Result<CampaignResult, SimulationError> {
    run_campaign_capturing(rules, config, definition, None)
}
pub(super) fn run_campaign_capturing(
    rules: &Catalogue,
    config: CampaignConfig,
    definition: &CampaignProgram,
    mut capture: Option<&mut super::phase_fixtures::Capture>,
) -> Result<CampaignResult, SimulationError> {
    validate_limits(config.max_rounds, config.recruit_limit)?;
    if definition.id() != config.campaign {
        return Err(SimulationError::InvalidConfig(
            "campaign definition does not match configuration".into(),
        ));
    }
    let chapters = definition.chapter_count();
    let mut game = BattleSession::start_campaign_from_definition(
        rules,
        definition,
        0,
        vec![],
        0,
        FlagHistory::new(),
        vec![],
        vec![],
    );
    let mut tactics = [
        PresetTactic::from_blocks(
            config
                .player_blocks
                .unwrap_or_else(|| config.player.blocks()),
        ),
        PresetTactic::from_blocks(config.enemy_blocks.unwrap_or_else(|| config.enemy.blocks())),
    ];
    game.set_combat_rng(config.seed);
    let mut missions = vec![];
    for _ in 0..chapters {
        if !game.is_campaign() {
            return Err("Campaign state missing".into());
        }
        game.dismiss_all_dialogue();
        let initial = game.state();
        let mut phases = vec![];
        let mut recording = RecordingCache::default();
        for index in 0..config.max_rounds * 2 {
            if game.winner().is_some() {
                break;
            }
            let policy = if game.turn() == TurnPhase::Player {
                config.player
            } else {
                config.enemy
            };
            let side = game.turn();
            let objective_goals = super::objectives::player_goals(&game, &config.objective);
            if let Some(capture) = capture.as_deref_mut() {
                capture.observe(&game, objective_goals.as_deref());
            }
            phases.push(phase(
                &mut game,
                &mut recording,
                policy,
                config.seed,
                config.recruit_limit,
                objective_goals.as_deref(),
                config.diplomacy,
                &mut tactics[side.index()],
            )?);
            if game.winner().is_none() && index + 1 < config.max_rounds * 2 {
                game.start_recording();
                game.end_turn();
            }
        }
        missions.push(MissionResult {
            escalated: rules
                .meta()
                .simulation
                .campaign
                .escalation_flags
                .get(game.map_id())
                .is_some_and(|flag| {
                    game.campaign_state()
                        .as_ref()
                        .is_some_and(|c| c.chosen(flag))
                }),
            id: game.map_id().to_owned(),
            winner: game.winner(),
            round: game.round(),
            goals: game.event_goal_progress(),
            phases,
            initial,
            r#final: game.state(),
        });
        if game.winner() != Some(BattleResult::PlayerVictory)
            || game
                .campaign_state()
                .as_ref()
                .ok_or("Campaign state missing")?
                .completed
        {
            break;
        }
        game = game.next_campaign();
    }
    Ok(CampaignResult {
        config,
        completed: game
            .campaign_state()
            .as_ref()
            .ok_or("Campaign state missing")?
            .completed,
        missions,
    })
}
/// Dispatch a validated configuration to the corresponding complete match runner.
pub fn run(
    rules: &Catalogue,
    config: MatchConfig,
    definitions: &super::CampaignDefinitions<'_>,
) -> Result<MatchResult, SimulationError> {
    match config {
        MatchConfig::Arena(config) => run_arena(rules, config).map(MatchResult::Arena),
        MatchConfig::Campaign(config) => {
            let definition = definitions.get(&config.campaign, &config.objective)?;
            run_campaign(rules, config, definition).map(MatchResult::Campaign)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recruitment_defaults_match_classic_and_leave_player_tactics_uncapped() {
        let rules = Catalogue::load().unwrap();
        let base = BattleSession::new(&rules, "mirror");
        for (policy, limit) in [
            (Policy::Classic, None),
            (Policy::Balanced, None),
            (Policy::Classic, Some(1)),
            (Policy::Balanced, Some(1)),
            (Policy::Balanced, Some(0)),
        ] {
            let mut game = base.fork();
            let faction = "player";
            crate::tests::edit(&mut game, |state| {
                state.units.retain(|unit| unit.role == Some(Role::Leader));
                state.gold[faction] = 1000;
            });
            let record = phase(
                &mut game,
                &mut RecordingCache::default(),
                policy,
                7,
                limit,
                None,
                Diplomacy::Unrestricted,
                &mut PresetTactic::preset(policy),
            )
            .unwrap();
            let recruited = record
                .records
                .actions
                .iter()
                .filter(|action| matches!(action, ActionRecord::Recruit { .. }))
                .count();
            match (policy, limit) {
                (_, Some(limit)) => assert_eq!(recruited, limit),
                (Policy::Classic, None) => assert_eq!(recruited, 2),
                (Policy::Balanced, None) => assert!(recruited > 2),
                _ => unreachable!(),
            }
        }
    }
    #[test]
    fn cached_boundaries_share_unchanged_values_and_detach_changed_state() {
        let rules = Catalogue::load().unwrap();
        let mut game = BattleSession::new(&rules, "mirror");
        let mut recording = RecordingCache::default();
        let before = recording.units(&game).unwrap();
        let same = recording.units(&game).unwrap();
        assert!(Arc::ptr_eq(&before[0], &same[0]));
        let expected = serde_json::to_value(&before[0]).unwrap();
        crate::tests::edit(&mut game, |state| {
            state.units[0].hp -= 1;
            state.units[0].moved = true;
        });
        let changed = recording.units(&game).unwrap();
        assert!(!Arc::ptr_eq(&before[0], &changed[0]));
        assert_eq!(serde_json::to_value(&before[0]).unwrap(), expected);
        assert_eq!(
            serde_json::to_value(&changed[0]).unwrap(),
            serde_json::to_value(game.public_unit(&game.units()[0])).unwrap()
        );
        let owners = recording.ownership(&game).unwrap();
        assert!(Arc::ptr_eq(&owners, &recording.ownership(&game).unwrap()));
        crate::tests::edit(&mut game, |state| {
            state.tiles[0].owner = Some(
                if state.tiles[0].owner.as_deref() == Some("player") {
                    "enemy"
                } else {
                    "player"
                }
                .into(),
            );
        });
        let changed = recording.ownership(&game).unwrap();
        assert!(!Arc::ptr_eq(&owners, &changed));
        assert!(Arc::ptr_eq(&owners.tiles, &changed.tiles));
        assert_ne!(owners.owners, changed.owners);
    }

    #[test]
    fn deathmatch_omits_holding_and_optional_objectives() {
        let rules = Catalogue::load().unwrap();
        let mut config: ArenaConfig = serde_json::from_value(serde_json::json!({
            "seed": 17, "armies": {"player": "human", "enemy": "skeletons"},
            "landscape": "open", "player": "balanced", "enemy": "balanced",
            "first": "player", "maxRounds": 30, "recruitLimit": null
        }))
        .unwrap();
        assert_eq!(config.objective, ArenaObjective::Territory);
        assert!(
            serde_json::to_value(&config)
                .unwrap()
                .get("objective")
                .is_none()
        );
        config.objective = ArenaObjective::Deathmatch;
        let result = run_arena(&rules, config).unwrap();
        assert!(["leader", "cap"].contains(&result.reason.as_str()));
        assert_eq!(result.winner.is_some(), result.reason == "leader");
        assert!(result.phases.iter().all(|p| {
            p.objectives
                .iter()
                .all(|o| o.id == "opposing-leader" || o.id.starts_with("village:"))
        }));
        assert!(
            result
                .phases
                .iter()
                .any(|p| p.objectives.iter().any(|o| o.id == "opposing-leader"))
        );
        assert_eq!(result.objectives.player.held, 0);
        assert_eq!(result.objectives.enemy.held, 0);
    }

    #[test]
    fn wooded_arena_has_cover_and_crossings_have_fords_and_an_island_fort() {
        let rules = Catalogue::load().unwrap();
        let mut config = ArenaConfig {
            objective: Default::default(),
            seed: 61,
            armies: Armies {
                player: rules.meta().simulation.rosters[0].id.clone(),
                enemy: rules.meta().simulation.rosters[0].id.clone(),
            },
            landscape: "wooded".into(),
            player: Policy::Balanced,
            enemy: Policy::Balanced,
            player_blocks: None,
            enemy_blocks: None,
            first: TurnPhase::Player,
            max_rounds: 30,
            recruit_limit: Some(1),
        };
        let woods = create_arena(&rules, &config).unwrap();
        let trees = (0..woods.terrain().len())
            .filter(|&tile| {
                woods.terrain_definition(tile).id() == Terrain::Forest
                    || woods.terrain_definition(tile).id() == Terrain::Grove
            })
            .count();
        assert!(trees * 3 >= woods.terrain().len() * 2);

        config.landscape = "crossings".into();
        let river = create_arena(&rules, &config).unwrap();
        let keeps: Vec<_> = (0..river.terrain().len())
            .filter(|&tile| river.is_keep(tile))
            .collect();
        let reachable = |start, bridges: bool, fords: bool| {
            let mut visited = HashSet::from([start]);
            let mut pending = vec![start];
            while let Some(tile) = pending.pop() {
                for &next in &river.board().adjacent[tile] {
                    if river.terrain_definition(next).id() != Terrain::Water
                        && (bridges || river.terrain_definition(next).id() != Terrain::Bridge)
                        && (fords || river.terrain_definition(next).id() != Terrain::Ford)
                        && visited.insert(next)
                    {
                        pending.push(next);
                    }
                }
            }
            visited
        };
        assert!(reachable(keeps[0], true, false).contains(&keeps[1]));
        assert!(reachable(keeps[0], false, true).contains(&keeps[1]));
        assert!(!reachable(keeps[0], false, false).contains(&keeps[1]));
        for keep in &keeps {
            let bank = reachable(*keep, false, false);
            assert_eq!(
                bank.iter()
                    .filter(|&&tile| {
                        river.terrain_definition(tile).income() > 0
                            && rules
                                .meta()
                                .simulation
                                .arena
                                .scenarios
                                .iter()
                                .find(|a| a.id == "crossings")
                                .unwrap()
                                .territory
                                .contains(&river.board().ids[tile])
                    })
                    .count(),
                1
            );
        }
        let fort = (0..river.terrain().len())
            .find(|&tile| river.terrain_definition(tile).id() == Terrain::Fort)
            .unwrap();
        assert!(river.board().adjacent[fort].iter().all(|&tile| {
            river.terrain_definition(tile).id() == Terrain::Water
                || river.terrain_definition(tile).id() == Terrain::Bridge
        }));
    }

    #[test]
    fn arena_starts_with_only_leaders_and_the_remaining_paired_budget() {
        let rules = Catalogue::load().unwrap();
        for seed in [0u32, 1, 2, 10] {
            for landscape in ["open", "wooded", "crossings", "islands"] {
                for player in &rules.meta().simulation.rosters {
                    for enemy in &rules.meta().simulation.rosters {
                        let config = ArenaConfig {
                            objective: Default::default(),
                            seed,
                            armies: Armies {
                                player: player.id.clone(),
                                enemy: enemy.id.clone(),
                            },
                            landscape: landscape.into(),
                            player: Policy::Classic,
                            enemy: Policy::Balanced,
                            player_blocks: None,
                            enemy_blocks: None,
                            first: TurnPhase::Ai,
                            max_rounds: 30,
                            recruit_limit: Some(1),
                        };
                        let game = create_arena(&rules, &config).unwrap();
                        let total = |side| {
                            let troops: Vec<_> =
                                game.units().iter().filter(|u| u.phase == side).collect();
                            assert_eq!(troops.len(), 1);
                            assert!(troops[0].is_leader());
                            game.gold_for_unit(troops[0])
                                + troops
                                    .iter()
                                    .map(|u| {
                                        rules.units().nth(u.kind).unwrap().recruitment_price(None)
                                    })
                                    .sum::<i32>()
                        };
                        let budget = total(TurnPhase::Player);
                        assert_eq!(budget, total(TurnPhase::Ai));
                        let scenario = rules
                            .meta()
                            .simulation
                            .arena
                            .scenarios
                            .iter()
                            .find(|s| s.id == landscape)
                            .unwrap();
                        let variant = &scenario.variants
                            [seed.saturating_sub(1) as usize % scenario.variants.len()];
                        let ArenaForces::Selected { budget: expected } = variant.forces else {
                            unreachable!()
                        };
                        assert_eq!(budget, expected);
                        assert!(game.gold().values().all(|&gold| gold >= 0));
                        assert_eq!(game.turn(), TurnPhase::Ai);
                        for unit in game.units() {
                            let sign = if unit.phase == TurnPhase::Player {
                                -1
                            } else {
                                1
                            };
                            assert_eq!(game.board().positions[unit.tile], (sign * 3 + 4, 4));
                        }
                    }
                }
            }
        }
    }
}
