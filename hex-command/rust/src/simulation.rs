use crate::{engine::Game, model::*, objectives::objective, policies::Random};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaConfig {
    pub seed: u32,
    pub armies: Armies,
    pub landscape: String,
    pub player: String,
    pub enemy: String,
    pub first: Side,
    pub max_rounds: u32,
    pub recruit_limit: usize,
}
#[derive(Clone, Deserialize)]
pub struct Armies {
    pub player: String,
    pub enemy: String,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignConfig {
    pub seed: u32,
    pub diplomacy: String,
    pub campaign: String,
    pub player: String,
    pub enemy: String,
    pub max_rounds: u32,
    pub recruit_limit: usize,
}
fn army(id: &str) -> (&str, &str, Vec<&str>) {
    match id {
        "human" => (
            "commander",
            "swordsman",
            vec!["swordsman", "spearman", "archer", "scout", "adept"],
        ),
        "woodland" => ("wildkeeper", "boar", vec!["boar", "wolf", "thorncaller"]),
        "undead" => (
            "necromancer",
            "skeleton",
            vec!["skeleton", "bonearcher", "adept"],
        ),
        "coastal" => (
            "tidekeeper",
            "reefguard",
            vec!["reefguard", "harpooner", "wavemender"],
        ),
        _ => panic!("unknown army"),
    }
}
fn axial_id(q: i32, r: i32) -> String {
    format!("{},{}", q + r.div_euclid(2) + 4, r + 4)
}
pub fn create_arena(rules: Arc<Rules>, config: &ArenaConfig) -> Game {
    let mut random = Random(config.seed);
    let mut state = rules
        .maps
        .iter()
        .filter(|((id, _), _)| {
            id == if config.landscape == "crossings" || config.landscape == "islands" {
                "river"
            } else {
                "mirror"
            }
        })
        .max_by_key(|((_, revision), _)| revision)
        .unwrap()
        .1
        .clone();
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
                id: axial_id(q, r),
                col,
                row: r + 4,
                terrain: "plains".into(),
                owner: None,
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
        let terrain = match config.landscape.as_str() {
            "wooded" => {
                if draw < 0.25 {
                    "forest"
                } else if draw < 0.32 {
                    "grove"
                } else if draw < 0.4 {
                    "hill"
                } else {
                    "plains"
                }
            }
            "crossings" => {
                if q == 0 {
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
    if config.landscape == "crossings" {
        terrain(0, -1, "bridge", None);
        terrain(0, 1, "bridge", None);
    }
    if config.landscape == "islands" {
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
    state.units.clear();
    state.next_id = 1;
    for side in [Side::Player, Side::Enemy] {
        let (leader, screen, _) = army(if side == Side::Player {
            &config.armies.player
        } else {
            &config.armies.enemy
        });
        let sign = if side == Side::Player { -1 } else { 1 };
        for (kind, q, r) in [
            (leader, sign * 3, 0),
            (screen, sign * 2, 0),
            (screen, sign * 3, -sign),
        ] {
            let rule = &rules.units[rules.types[kind]];
            state.units.push(PublicUnit {
                id: format!("u{}", state.next_id),
                r#type: kind.into(),
                side,
                tile: axial_id(q, r),
                hp: rule.hp,
                moved: false,
                attacked: false,
                experience: Some(0),
                name: None,
                role: rule.leader.then_some("leader".into()),
                retreating: None,
                rebuilding: None,
            });
            state.next_id += 1;
        }
    }
    let gold = 18 + (random.next() * 3.0).floor() as i32 * 3;
    state.gold = Gold {
        player: gold,
        enemy: gold,
    };
    state.round = 1;
    state.turn = config.first;
    state.winner = None;
    state.log.clear();
    state.campaign = None;
    state.last_combat = None;
    let mut game = Game::from_state(rules, state);
    game.pools = Some([
        army(&config.armies.player)
            .2
            .into_iter()
            .map(|k| game.kind(k))
            .collect(),
        army(&config.armies.enemy)
            .2
            .into_iter()
            .map(|k| game.kind(k))
            .collect(),
    ]);
    game.arena_first = Some(config.first);
    game
}
fn phase(
    game: &mut Game,
    policy: &str,
    seed: u32,
    limit: usize,
    goals: Option<&[Objective]>,
    restrained: bool,
) -> Value {
    let before = game.public_units();
    let side = game.turn;
    let round = game.round;
    let records = Arc::new(Mutex::new(Records::default()));
    game.recordings = Some(records.clone());
    game.play_policy(policy, seed, limit, goals, restrained);
    if let Some(state) = &mut game.campaign {
        state.dialogue_index = state.dialogue.len();
    }
    let records = records.lock().unwrap();
    let count = |side| game.units.iter().filter(|u| u.side == side).count();
    let health = |side| {
        game.units
            .iter()
            .filter(|u| u.side == side)
            .map(|u| u.hp)
            .sum::<i32>()
    };
    let mut result = json!({"side":side,"round":round,"actions":records.actions,"losses":records.losses,"combats":records.combats,"living":{"player":count(Side::Player),"enemy":count(Side::Enemy)},"health":{"player":health(Side::Player),"enemy":health(Side::Enemy)},"gold":{"player":game.gold[0],"enemy":game.gold[1]},"before":before,"units":game.public_units(),"ownership":game.board.ids.iter().enumerate().map(|(t,id)|json!({"tile":id,"owner":game.owners[t]})).collect::<Vec<_>>(),"goals":game.mission.as_ref().map_or(vec![],|m|m.goals.iter().map(|g|game.goal_progress(g)).collect()),"objectives":goals.unwrap_or(&[])});
    if let Some(state) = &game.campaign {
        result["campaign"] = serde_json::to_value(state).unwrap();
    }
    result
}
pub fn run_arena(rules: Arc<Rules>, config_value: Value) -> Value {
    let config: ArenaConfig = serde_json::from_value(config_value.clone()).expect("arena config");
    let mut game = create_arena(rules, &config);
    let initial = game.state();
    game.begin_turn(config.first);
    let territory = ["4,4", "3,2", "5,6"];
    let mut held = [0u32; 2];
    let mut visits: [HashSet<String>; 2] = Default::default();
    let mut phases = vec![];
    let mut reason = "cap";
    let mut rounds = 0;
    for index in 0..config.max_rounds * 2 {
        if game.winner.is_some() {
            break;
        }
        let side = game.turn;
        let mut goals = vec![objective(
            "required-territory",
            "capture",
            territory.iter().map(|s| s.to_string()).collect(),
            24.0,
        )];
        for tile in 0..game.terrain.len() {
            if game.terrain_is(tile, "village") && game.owners[tile] != Some(side) {
                goals.push(objective(
                    &format!("village:{}", game.board.ids[tile]),
                    "capture",
                    vec![game.board.ids[tile].clone()],
                    8.0,
                ));
            }
        }
        goals.push(objective(
            "opposing-leader",
            "defeat",
            game.units
                .iter()
                .filter(|u| u.side != side && game.rules.units[u.kind].leader)
                .map(|u| game.board.ids[u.tile].clone())
                .collect(),
            12.0,
        ));
        for tile in 0..game.terrain.len() {
            if game.terrain_is(tile, "ruins")
                && !visits[side.index()].contains(&game.board.ids[tile])
            {
                goals.push(objective(
                    &format!("optional:{}", game.board.ids[tile]),
                    "visit",
                    vec![game.board.ids[tile].clone()],
                    4.0,
                ));
            }
        }
        let record = phase(
            &mut game,
            if side == Side::Player {
                &config.player
            } else {
                &config.enemy
            },
            config.seed,
            config.recruit_limit,
            Some(&goals),
            false,
        );
        for command in record["actions"].as_array().unwrap() {
            if command["kind"] == "move" {
                let id = command["tile"].as_str().unwrap();
                if game.terrain_is(game.site(id), "ruins") {
                    visits[side.index()].insert(id.into());
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
                .all(|t| game.owners[game.site(t)] == Some(owner))
            {
                held[owner.index()] = 0;
            } else if owner != side {
                held[owner.index()] += 1;
            }
            if held[owner.index()] >= if config.landscape == "islands" { 6 } else { 4 } {
                game.winner = Some(owner);
                reason = "territory";
                break;
            }
        }
        if game.winner.is_none() && index + 1 < config.max_rounds * 2 {
            game.end_turn();
        }
    }
    let score = |side: Side| json!({"controlled":territory.iter().filter(|t|game.owners[game.site(t)]==Some(side)).count(),"held":held[side.index()],"optionalVisits":visits[side.index()].len()});
    json!({"config":config_value,"winner":game.winner,"reason":reason,"rounds":rounds,"phases":phases,"initial":initial,"final":game.state(),"objectives":{"player":score(Side::Player),"enemy":score(Side::Enemy)}})
}
pub fn run_campaign(rules: Arc<Rules>, config_value: Value) -> Value {
    let config: CampaignConfig =
        serde_json::from_value(config_value.clone()).expect("campaign config");
    let chapters = rules.campaigns[&config.campaign].missions.len();
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
    let mut missions = vec![];
    for _ in 0..chapters {
        let state = game.campaign.as_mut().unwrap();
        state.dialogue_index = state.dialogue.len();
        let initial = game.state();
        let mut phases = vec![];
        for index in 0..config.max_rounds * 2 {
            if game.winner.is_some() {
                break;
            }
            let policy = if game.turn == Side::Player {
                &config.player
            } else {
                &config.enemy
            };
            phases.push(phase(
                &mut game,
                policy,
                config.seed,
                config.recruit_limit,
                None,
                config.diplomacy == "restrained",
            ));
            if game.winner.is_none() && index + 1 < config.max_rounds * 2 {
                game.end_turn();
            }
        }
        missions.push(json!({"id":game.map_id,"winner":game.winner,"round":game.round,"goals":game.mission.as_ref().unwrap().goals.iter().map(|g|game.goal_progress(g)).collect::<Vec<_>>(),"phases":phases,"initial":initial,"final":game.state()}));
        if game.winner != Some(Side::Player) || game.campaign.as_ref().unwrap().completed {
            break;
        }
        game = game.next_campaign();
    }
    json!({"config":config_value,"completed":game.campaign.as_ref().unwrap().completed,"missions":missions})
}
pub fn run(rules: Arc<Rules>, request: Value) -> Value {
    if request.get("campaign").is_some() {
        run_campaign(rules, request)
    } else {
        run_arena(rules, request)
    }
}
