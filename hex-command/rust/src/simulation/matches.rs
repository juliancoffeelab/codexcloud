//! Complete seeded arena and campaign runs through the shared battle/AI rules.
use super::{error::SimulationError, types::*};
use crate::{
    ai::{
        Diplomacy, PlanningWorkspace, Policy, Random, objectives::objective, tactic::PresetTactic,
    },
    data::*,
    logic::Game,
};
use rustc_hash::FxHashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// Reuse lossless encodings of unchanged boundary values within one chapter.
/// Keys contain every unit field; ownership keys retain the entire ordered map.
#[derive(Default)]
struct RecordingCache {
    units: FxHashMap<Unit, Arc<PublicUnit>>,
    ownership: FxHashMap<Vec<Option<Side>>, Arc<OwnershipSnapshot>>,
    tiles: Option<Arc<Vec<String>>>,
}
impl RecordingCache {
    fn units(&mut self, game: &Game<'_>) -> Result<Vec<Arc<PublicUnit>>, SimulationError> {
        game.units
            .iter()
            .map(|unit| {
                if let Some(encoded) = self.units.get(unit) {
                    return Ok(encoded.clone());
                }
                let encoded = Arc::new(game.public_unit(unit));
                self.units.insert(unit.clone(), encoded.clone());
                Ok(encoded)
            })
            .collect()
    }
    fn ownership(&mut self, game: &Game<'_>) -> Result<Arc<OwnershipSnapshot>, SimulationError> {
        if let Some(encoded) = self.ownership.get(&game.owners) {
            return Ok(encoded.clone());
        }
        let tiles = self
            .tiles
            .get_or_insert_with(|| Arc::new(game.board.ids.clone()))
            .clone();
        let encoded = Arc::new(OwnershipSnapshot {
            tiles,
            owners: game.owners.clone(),
        });
        self.ownership.insert(game.owners.clone(), encoded.clone());
        Ok(encoded)
    }
}

fn axial_id(q: i32, r: i32) -> String {
    format!("{},{}", q + r.div_euclid(2) + 4, r + 4)
}
pub(crate) fn create_arena<'catalog>(
    rules: &'catalog Rules,
    config: &ArenaConfig,
) -> Result<Game<'catalog>, SimulationError> {
    validate_arena(rules, config)?;
    let mut random = Random(config.seed);
    let landscape = rules
        .arena
        .landscapes
        .iter()
        .find(|l| l.id == config.landscape)
        .unwrap();
    let map_id = landscape.map.as_deref().unwrap_or("mirror");
    let mut state = rules
        .maps
        .iter()
        .chain(rules.arena_maps.iter())
        .filter(|((id, _), _)| id == map_id)
        .max_by_key(|((_, revision), _)| revision)
        .ok_or("Arena layout is unavailable")?
        .1
        .clone();
    if landscape.map.is_none() {
        generate_arena_terrain(&mut state, &config.landscape, &mut random);
    }
    state.units.clear();
    state.next_id = 1;
    let mut starting_cost = [0; 2];
    for side in [Side::Player, Side::Enemy] {
        let roster = roster(
            rules,
            if side == Side::Player {
                &config.armies.player
            } else {
                &config.armies.enemy
            },
        )?;
        let leader = roster.leader.as_str();
        let start = state
            .tiles
            .iter()
            .find(|tile| tile.terrain == "keep" && tile.owner == Some(side))
            .ok_or("Arena requires an owned keep for each side")?
            .id
            .clone();
        let rule = &rules.units[rules.types[leader]];
        starting_cost[side.index()] += rule.cost;
        state.units.push(PublicUnit {
            id: format!("u{}", state.next_id),
            r#type: leader.into(),
            side,
            tile: start.clone(),
            hp: rule.hp,
            moved: false,
            attacked: false,
            experience: Some(0),
            name: None,
            role: rule.leader.then_some(Role::Leader),
            retreating: None,
            rebuilding: None,
        });
        state.next_id += 1;
    }
    let budget = rules.arena.starting_budget + (random.next() * 3.0).floor() as i32 * 3;
    if starting_cost.iter().any(|&cost| cost > budget) {
        return Err("Arena starting troops exceed the shared budget".into());
    }
    state.gold = Gold {
        player: budget - starting_cost[Side::Player.index()],
        enemy: budget - starting_cost[Side::Enemy.index()],
    };
    state.round = 1;
    state.turn = config.first;
    state.winner = None;
    state.log.clear();
    state.campaign = None;
    state.last_combat = None;
    let mut game = Game::from_state(rules, state);
    game.combat_rng = config.seed;
    game.pools = Some([
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
    game.arena_first = Some(config.first);
    Ok(game)
}
fn generate_arena_terrain(state: &mut State, landscape: &str, random: &mut Random) {
    // Preserve generated water-arena identities without the retired skirmish template.
    if matches!(landscape, "crossings" | "islands") {
        state.map_id = "river".into();
        state.map_revision = 2;
    }
    state.cols = 9;
    state.rows = 9;
    state.tiles.clear();
    for r in -4i32..=4 {
        for q in -4i32..=4 {
            if (q + r).abs() > 4 {
                continue;
            }
            let col = q + r.div_euclid(2) + 4;
            state.tiles.push(Tile {
                fog: None,
                id: axial_id(q, r),
                col,
                row: r + 4,
                terrain: "plains".into(),
                owner: None,
                connections: None,
                dead_ends: None,
                floor: None,
            });
        }
    }
    let mut used = HashSet::new();
    for index in 0..state.tiles.len() {
        let tile = &state.tiles[index];
        if used.contains(&tile.id) {
            continue;
        }
        let partner = state
            .tiles
            .iter()
            .position(|other| other.row == 8 - tile.row && other.col == 8 - tile.col - tile.row % 2)
            .unwrap();
        used.insert(tile.id.clone());
        used.insert(state.tiles[partner].id.clone());
        let draw = random.next();
        let r = tile.row - 4;
        let q = tile.col - r.div_euclid(2) - 4;
        let terrain = match landscape {
            "wooded" => {
                // Most unreserved ground is woodland; starters and settlements stay clear.
                if draw < 0.75 {
                    "forest"
                } else if draw < 0.9 {
                    "grove"
                } else if draw < 0.95 {
                    "hill"
                } else {
                    "plains"
                }
            }
            "crossings" => {
                // The curved river widens only around the island fort.
                let river = match r {
                    -4..=-2 => q == 2,
                    -1 => (0..=1).contains(&q),
                    0 => q.abs() <= 1,
                    1 => (-1..=0).contains(&q),
                    2..=4 => q == -2,
                    _ => false,
                };
                if river {
                    "water"
                } else if draw < 0.2 {
                    "marsh"
                } else if draw < 0.3 {
                    "forest"
                } else {
                    "plains"
                }
            }
            "islands" => {
                let home = q.abs() >= 2
                    && r.abs() <= 2
                    && !(q == -2 && r > 0)
                    && !(q == 2 && r < 0)
                    && !(q == -3 && r == -1)
                    && !(q == 3 && r == 1);
                let fort = q == 0 && r == 0 || q == -1 && r == 1 || q == 1 && r == -1;
                let village = q == 0 && (r.abs() == 2 || r.abs() == 3);
                if home || fort || village {
                    if draw < 0.15 {
                        "forest"
                    } else if draw < 0.25 {
                        "hill"
                    } else if draw < 0.45 {
                        "marsh"
                    } else {
                        "plains"
                    }
                } else {
                    "water"
                }
            }
            _ => "plains",
        };
        state.tiles[index].terrain = terrain.into();
        state.tiles[partner].terrain = terrain.into();
    }
    let mut terrain = |q, r, kind: &str, owner| {
        let tile = state
            .tiles
            .iter_mut()
            .find(|t| t.id == axial_id(q, r))
            .unwrap();
        tile.terrain = kind.into();
        tile.owner = owner;
    };
    for (q, r, kind, owner) in [
        (-3, 0, "keep", Some(Side::Player)),
        (3, 0, "keep", Some(Side::Enemy)),
        (0, 0, "fort", None),
        (0, -2, "village", None),
        (0, 2, "village", None),
        (-2, 0, "plains", None),
        (2, 0, "plains", None),
        (-3, 1, "plains", None),
        (3, -1, "plains", None),
        (-2, -1, "village", None),
        (2, 1, "village", None),
        (-1, 3, "ruins", None),
        (1, -3, "ruins", None),
    ] {
        terrain(q, r, kind, owner);
    }
    if landscape == "crossings" {
        for (q, r) in [(-1, 0), (1, 0)] {
            terrain(q, r, "bridge", None);
        }
        for (q, r) in [(2, -3), (-2, 3)] {
            terrain(q, r, "ford", None);
        }
    }
    if landscape == "islands" {
        for (q, r) in [(-1, 0), (1, 0)] {
            terrain(q, r, "bridge", None);
        }
        for (q, r) in [
            (-2, 1),
            (2, -1),
            (0, -1),
            (0, 1),
            (1, -2),
            (-1, 2),
            (-1, -2),
            (1, 2),
            (2, -2),
            (-2, 2),
            (-1, -1),
            (1, 1),
        ] {
            terrain(q, r, "ford", None);
        }
    }
}
fn roster<'a>(rules: &'a Rules, id: &str) -> Result<&'a crate::data::ArmyRoster, SimulationError> {
    rules
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
pub fn validate_arena(rules: &Rules, config: &ArenaConfig) -> Result<(), SimulationError> {
    validate_limits(config.max_rounds, config.recruit_limit)?;
    roster(rules, &config.armies.player)?;
    roster(rules, &config.armies.enemy)?;
    if !rules
        .arena
        .landscapes
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
    game: &mut Game<'_>,
    recording: &mut RecordingCache,
    policy: Policy,
    seed: u32,
    limit: Option<usize>,
    goals: Option<&[Objective]>,
    diplomacy: Diplomacy,
    tactic: &mut PresetTactic,
) -> Result<Phase, SimulationError> {
    let timing = crate::ai::profiling::Turn::start(policy, game.turn);
    let before = recording.units(game)?;
    let side = game.turn;
    let round = game.round;
    let leader_audit = super::leader_audit::begin(game);
    game.recordings.get_or_insert_with(Records::default);
    let mut workspace = PlanningWorkspace::new(
        seed,
        limit.or_else(|| policy.default_recruitment_limit()),
        goals.map(<[Objective]>::to_vec),
        diplomacy,
    );
    let composition = tactic.composition();
    if !composition.campaign.supports_campaign() && game.campaign.is_some() {
        return Err(CommandError::BattleOnlyPolicy.into());
    }
    let mut recruitment = None;
    let planning = loop {
        let plan = {
            let _decision = crate::ai::profiling::Decision::start(policy, game.turn);
            tactic.run_turn(game, &mut workspace)?
        };
        recruitment.get_or_insert(plan.recruitment);
        for command in plan.commands {
            game.execute(command)?;
        }
        if workspace.finished() || game.winner.is_some() {
            break plan.planning;
        }
    };
    if let Some(state) = &mut game.campaign {
        state.dialogue_index = state.dialogue.len();
    }
    let records = game.recordings.take().unwrap_or_default();
    let count = |side| game.units.iter().filter(|u| u.side == side).count();
    let health = |side| {
        game.units
            .iter()
            .filter(|u| u.side == side)
            .map(|u| u.hp)
            .sum()
    };
    let phase = Phase {
        leader_audit: super::leader_audit::finish(game, leader_audit),
        planning: workspace.diagnostics(planning, recruitment.unwrap_or_default()),
        side,
        round,
        records,
        living: Sides {
            player: count(Side::Player),
            enemy: count(Side::Enemy),
        },
        health: Sides {
            player: health(Side::Player),
            enemy: health(Side::Enemy),
        },
        gold: Gold {
            player: game.gold[0],
            enemy: game.gold[1],
        },
        before,
        units: recording.units(game)?,
        ownership: recording.ownership(game)?,
        goals: game.mission.as_ref().map_or_else(Vec::new, |mission| {
            mission
                .goals
                .iter()
                .map(|goal| game.goal_progress(goal))
                .collect()
        }),
        objectives: goals.unwrap_or(&[]).to_vec(),
        campaign: game.campaign.clone(),
    };
    if let Some(timing) = timing {
        timing.finish();
    }
    Ok(phase)
}
/// Execute one arena until victory or its cap, retaining every phase and endpoint.
pub fn run_arena(rules: &Rules, config: ArenaConfig) -> Result<ArenaResult, SimulationError> {
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
        .arena
        .landscapes
        .iter()
        .find(|item| item.id == config.landscape)
        .ok_or("Unknown arena landscape")?;
    let territory = landscape
        .territory
        .as_ref()
        .unwrap_or(&rules.arena.territory);
    let hold_turns = landscape.hold_turns;
    let mut held = [0u32; 2];
    let mut visits: [HashSet<String>; 2] = Default::default();
    let mut phases = vec![];
    let mut recording = RecordingCache::default();
    let mut reason = "cap";
    let mut rounds = 0;
    for index in 0..config.max_rounds * 2 {
        if game.winner.is_some() {
            break;
        }
        let side = game.turn;
        let mut goals = vec![objective(
            "required-territory",
            ObjectiveKind::Capture,
            territory.clone(),
            24.0,
        )];
        for tile in 0..game.terrain.len() {
            if game.terrain_is(tile, Terrain::Village) && game.owners[tile] != Some(side) {
                goals.push(objective(
                    &format!("village:{}", game.board.ids[tile]),
                    ObjectiveKind::Capture,
                    vec![game.board.ids[tile].clone()],
                    8.0,
                ));
            }
        }
        goals.push(objective(
            "opposing-leader",
            ObjectiveKind::Defeat,
            game.units
                .iter()
                .filter(|u| u.side != side && game.rules.units[u.kind].leader)
                .map(|u| game.board.ids[u.tile].clone())
                .collect(),
            12.0,
        ));
        for tile in 0..game.terrain.len() {
            if game.terrain_is(tile, Terrain::Ruins)
                && !visits[side.index()].contains(&game.board.ids[tile])
            {
                goals.push(objective(
                    &format!("optional:{}", game.board.ids[tile]),
                    ObjectiveKind::Visit,
                    vec![game.board.ids[tile].clone()],
                    4.0,
                ));
            }
        }
        let record = phase(
            &mut game,
            &mut recording,
            if side == Side::Player {
                config.player
            } else {
                config.enemy
            },
            config.seed,
            config.recruit_limit,
            Some(&goals),
            Diplomacy::Pragmatic,
            &mut tactics[side.index()],
        )?;
        for command in &record.records.actions {
            if let ActionRecord::Move { tile, .. } = command {
                if game.terrain_is(game.site(tile), Terrain::Ruins) {
                    visits[side.index()].insert(tile.clone());
                }
            }
        }
        phases.push(record);
        rounds = phases.len().div_ceil(2);
        if game.winner.is_some() {
            reason = "leader";
            break;
        }
        for owner in [Side::Player, Side::Enemy] {
            if !territory
                .iter()
                .all(|tile| game.owners[game.site(tile)] == Some(owner))
            {
                held[owner.index()] = 0;
            } else if owner != side {
                held[owner.index()] += 1;
            }
            if held[owner.index()] >= hold_turns {
                game.winner = Some(owner);
                reason = "territory";
                break;
            }
        }
        if game.winner.is_none() && index + 1 < config.max_rounds * 2 {
            game.recordings = Some(Records::default());
            game.end_turn();
        }
    }
    let score = |side: Side| ArenaScore {
        controlled: territory
            .iter()
            .filter(|tile| game.owners[game.site(tile)] == Some(side))
            .count(),
        held: held[side.index()],
        optional_visits: visits[side.index()].len(),
    };
    Ok(ArenaResult {
        winner: game.winner,
        reason: reason.into(),
        rounds,
        phases,
        initial,
        r#final: game.state(),
        objectives: Sides {
            player: score(Side::Player),
            enemy: score(Side::Enemy),
        },
        config,
    })
}
/// Execute reached campaign chapters with veteran carryover and no retries.
pub fn run_campaign(
    rules: &Rules,
    config: CampaignConfig,
) -> Result<CampaignResult, SimulationError> {
    validate_limits(config.max_rounds, config.recruit_limit)?;
    let chapters = rules
        .campaigns
        .get(&config.campaign)
        .ok_or_else(|| {
            SimulationError::InvalidConfig(format!("Unknown campaign: {}", config.campaign))
        })?
        .missions
        .len();
    let mut game = Game::start_campaign(
        rules,
        &config.campaign,
        0,
        vec![],
        0,
        vec![],
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
    game.combat_rng = config.seed;
    let mut missions = vec![];
    for _ in 0..chapters {
        let state = game.campaign.as_mut().ok_or("Campaign state missing")?;
        state.dialogue_index = state.dialogue.len();
        let initial = game.state();
        let mut phases = vec![];
        let mut recording = RecordingCache::default();
        for index in 0..config.max_rounds * 2 {
            if game.winner.is_some() {
                break;
            }
            let policy = if game.turn == Side::Player {
                config.player
            } else {
                config.enemy
            };
            let side = game.turn;
            phases.push(phase(
                &mut game,
                &mut recording,
                policy,
                config.seed,
                config.recruit_limit,
                None,
                config.diplomacy,
                &mut tactics[side.index()],
            )?);
            if game.winner.is_none() && index + 1 < config.max_rounds * 2 {
                game.recordings = Some(Records::default());
                game.end_turn();
            }
        }
        missions.push(MissionResult {
            id: game.map_id.clone(),
            winner: game.winner,
            round: game.round,
            goals: game
                .mission
                .as_ref()
                .ok_or("Campaign mission missing")?
                .goals
                .iter()
                .map(|goal| game.goal_progress(goal))
                .collect(),
            phases,
            initial,
            r#final: game.state(),
        });
        if game.winner != Some(Side::Player)
            || game
                .campaign
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
            .campaign
            .as_ref()
            .ok_or("Campaign state missing")?
            .completed,
        missions,
    })
}
/// Dispatch a validated configuration to the corresponding complete match runner.
pub fn run(rules: &Rules, config: MatchConfig) -> Result<MatchResult, SimulationError> {
    match config {
        MatchConfig::Arena(config) => run_arena(rules, config).map(MatchResult::Arena),
        MatchConfig::Campaign(config) => run_campaign(rules, config).map(MatchResult::Campaign),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recruitment_defaults_match_classic_and_leave_player_tactics_uncapped() {
        let rules = Rules::load().unwrap();
        let base = Game::new(&rules, "mirror");
        for (policy, limit) in [
            (Policy::Classic, None),
            (Policy::Balanced, None),
            (Policy::Classic, Some(1)),
            (Policy::Balanced, Some(1)),
            (Policy::Balanced, Some(0)),
        ] {
            let mut game = base.fork();
            game.units.retain(|unit| rules.units[unit.kind].leader);
            game.roster_changed();
            game.gold[game.turn.index()] = 1000;
            let record = phase(
                &mut game,
                &mut RecordingCache::default(),
                policy,
                7,
                limit,
                None,
                Diplomacy::Pragmatic,
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
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let mut recording = RecordingCache::default();
        let before = recording.units(&game).unwrap();
        let same = recording.units(&game).unwrap();
        assert!(Arc::ptr_eq(&before[0], &same[0]));
        let expected = serde_json::to_value(&before[0]).unwrap();
        game.units[0].hp -= 1;
        game.units[0].moved = true;
        let changed = recording.units(&game).unwrap();
        assert!(!Arc::ptr_eq(&before[0], &changed[0]));
        assert_eq!(serde_json::to_value(&before[0]).unwrap(), expected);
        assert_eq!(
            serde_json::to_value(&changed[0]).unwrap(),
            serde_json::to_value(game.public_unit(&game.units[0])).unwrap()
        );
        let owners = recording.ownership(&game).unwrap();
        assert!(Arc::ptr_eq(&owners, &recording.ownership(&game).unwrap()));
        game.owners[0] = Some(game.owners[0].unwrap_or(Side::Enemy).other());
        let changed = recording.ownership(&game).unwrap();
        assert!(!Arc::ptr_eq(&owners, &changed));
        assert!(Arc::ptr_eq(&owners.tiles, &changed.tiles));
        assert_ne!(owners.owners, changed.owners);
    }

    #[test]
    fn wooded_arena_has_cover_and_crossings_have_fords_and_an_island_fort() {
        let rules = Rules::load().unwrap();
        let mut config = ArenaConfig {
            seed: 61,
            armies: Armies {
                player: rules.rosters[0].id.clone(),
                enemy: rules.rosters[0].id.clone(),
            },
            landscape: "wooded".into(),
            player: Policy::Balanced,
            enemy: Policy::Balanced,
            player_blocks: None,
            enemy_blocks: None,
            first: Side::Player,
            max_rounds: 30,
            recruit_limit: Some(1),
        };
        let woods = create_arena(&rules, &config).unwrap();
        let trees = (0..woods.terrain.len())
            .filter(|&tile| {
                woods.terrain_is(tile, Terrain::Forest) || woods.terrain_is(tile, Terrain::Grove)
            })
            .count();
        assert!(trees * 3 >= woods.terrain.len() * 2);

        config.landscape = "crossings".into();
        let river = create_arena(&rules, &config).unwrap();
        let keeps: Vec<_> = (0..river.terrain.len())
            .filter(|&tile| river.terrain_is(tile, Terrain::Keep))
            .collect();
        let reachable = |start, bridges: bool, fords: bool| {
            let mut visited = HashSet::from([start]);
            let mut pending = vec![start];
            while let Some(tile) = pending.pop() {
                for &next in &river.board.adjacent[tile] {
                    if !river.terrain_is(next, Terrain::Water)
                        && (bridges || !river.terrain_is(next, Terrain::Bridge))
                        && (fords || !river.terrain_is(next, Terrain::Ford))
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
                        river.terrain_is(tile, Terrain::Village)
                            && rules.arena.territory.contains(&river.board.ids[tile])
                    })
                    .count(),
                1
            );
        }
        let fort = (0..river.terrain.len())
            .find(|&tile| river.terrain_is(tile, Terrain::Fort))
            .unwrap();
        assert!(river.board.adjacent[fort].iter().all(|&tile| {
            river.terrain_is(tile, Terrain::Water) || river.terrain_is(tile, Terrain::Bridge)
        }));
    }

    #[test]
    fn arena_starts_with_only_leaders_and_the_remaining_paired_budget() {
        let rules = Rules::load().unwrap();
        for seed in [1, 2, 10] {
            for landscape in ["open", "wooded", "crossings", "islands"] {
                for player in &rules.rosters {
                    for enemy in &rules.rosters {
                        let config = ArenaConfig {
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
                            first: Side::Enemy,
                            max_rounds: 30,
                            recruit_limit: Some(1),
                        };
                        let game = create_arena(&rules, &config).unwrap();
                        let total = |side| {
                            let troops: Vec<_> =
                                game.units.iter().filter(|u| u.side == side).collect();
                            assert_eq!(troops.len(), 1);
                            assert!(rules.units[troops[0].kind].leader);
                            game.gold[side.index()]
                                + troops.iter().map(|u| rules.units[u.kind].cost).sum::<i32>()
                        };
                        let budget = total(Side::Player);
                        assert_eq!(budget, total(Side::Enemy));
                        assert!([0, 3, 6].contains(&(budget - rules.arena.starting_budget)));
                        assert!(game.gold.iter().all(|&gold| gold >= 0));
                        assert_eq!(game.turn, Side::Enemy);
                        for unit in &game.units {
                            let sign = if unit.side == Side::Player { -1 } else { 1 };
                            assert_eq!(game.board.ids[unit.tile], axial_id(sign * 3, 0));
                        }
                    }
                }
            }
        }
    }
}
