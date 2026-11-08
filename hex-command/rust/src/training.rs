//! Native training contract. Numeric observations never expose authored scripts.
use crate::{
    ai::{Planner, context::BattleView, orders::*},
    data::*,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
pub const SCHEMA: u32 = 1;
pub const FEATURES: usize = 32;
pub const FEATURE_NAMES: [&str; FEATURES] = [
    "end_turn",
    "move",
    "attack",
    "advance",
    "round",
    "allies",
    "enemies",
    "gold",
    "hp_fraction",
    "hp",
    "leader",
    "moved",
    "attacked",
    "cost",
    "movement",
    "protected",
    "nearest_enemy",
    "enemy_distance_gain",
    "nearest_ally",
    "allies_adjacent",
    "enemies_adjacent",
    "captures",
    "goal_distance",
    "goal_distance_gain",
    "goal_weight",
    "hold_goal",
    "damage",
    "retaliation",
    "kill_probability",
    "target_hp",
    "target_leader",
    "objective_count",
];
#[derive(Clone)]
pub struct Choices {
    pub commands: Vec<Command>,
    pub features: Vec<[f32; FEATURES]>,
}
fn actor_matches(actor: &Option<String>, unit: &Unit) -> bool {
    actor
        .as_ref()
        .is_none_or(|a| a == &format!("u{}", unit.id) || unit.character.as_ref() == Some(a))
}
pub fn candidates(game: &BattleView<'_, '_>, orders: &TacticalOrders) -> Choices {
    let mut commands = vec![Command::EndTurn];
    for unit in game
        .units()
        .iter()
        .filter(|u| game.belongs_to_phase(u, game.turn()) && game.can_act(u.id))
    {
        if !unit.moved {
            for &(tile, _) in game.reachable(unit).iter() {
                if tile != unit.tile {
                    let c = Command::Move {
                        unit: UnitId(unit.id),
                        tile: TileId(game.board().ids[tile].clone()),
                    };
                    if game.permits(&c) {
                        commands.push(c);
                    }
                }
            }
        }
        if !unit.attacked {
            for target in game.units().iter().filter(|t| game.hostile(unit, t)) {
                for weapon in game.attack_options(unit, target, unit.tile) {
                    let c = Command::Attack {
                        unit: UnitId(unit.id),
                        target: UnitId(target.id),
                        weapon: WeaponId(game.weapon(unit, weapon).id.clone()),
                    };
                    if game.permits(&c) {
                        commands.push(c);
                    }
                }
            }
        }
        if let Some(kind) = orders.promotion(game, unit) {
            commands.push(Command::Advance {
                unit: UnitId(unit.id),
                kind: UnitTypeId(game.rules().units[kind].id.clone()),
            });
        }
    }
    let features = commands.iter().map(|c| encode(game, orders, c)).collect();
    Choices { commands, features }
}
fn encode(g: &BattleView<'_, '_>, orders: &TacticalOrders, c: &Command) -> [f32; FEATURES] {
    let mut x = [0.; FEATURES];
    let allies: Vec<_> = g
        .units()
        .iter()
        .filter(|u| g.belongs_to_phase(u, g.turn()))
        .collect();
    let enemies: Vec<_> = g
        .units()
        .iter()
        .filter(|u| g.hostile_to_phase(u, g.turn()))
        .collect();
    x[4] = g.round() as f32 / 30.;
    x[5] = allies.len() as f32 / 10.;
    x[6] = enemies.len() as f32 / 10.;
    x[7] = g.gold_for_phase(g.turn()) as f32 / 100.;
    x[31] = orders.targets.len() as f32 / 10.;
    let (id, tile, target, weapon) = match c {
        Command::EndTurn => {
            x[0] = 1.;
            return x;
        }
        Command::Move { unit, tile } => {
            x[1] = 1.;
            (unit.0, Some(g.site(&tile.0)), None, None)
        }
        Command::Attack {
            unit,
            target,
            weapon,
        } => {
            x[2] = 1.;
            (unit.0, None, Some(target.0), Some(&weapon.0))
        }
        Command::Advance { unit, .. } => {
            x[3] = 1.;
            (unit.0, None, None, None)
        }
        _ => unreachable!(),
    };
    let u = g.unit(id).unwrap();
    let r = &g.rules().units[u.kind];
    let tile = tile.unwrap_or(u.tile);
    x[8] = u.hp as f32 / r.hp.max(1) as f32;
    x[9] = u.hp as f32 / 30.;
    x[10] = u.is_leader() as u8 as f32;
    x[11] = u.moved as u8 as f32;
    x[12] = u.attacked as u8 as f32;
    x[13] = r.cost as f32 / 30.;
    x[14] = r.r#move as f32 / 10.;
    x[15] = orders.protects(id) as u8 as f32;
    let dist = |a, b| g.board().distance(a, b) as f32;
    let before = enemies
        .iter()
        .map(|e| dist(u.tile, e.tile))
        .fold(30., f32::min);
    let after = enemies
        .iter()
        .map(|e| dist(tile, e.tile))
        .fold(30., f32::min);
    x[16] = after / 20.;
    x[17] = (before - after) / 10.;
    x[18] = allies
        .iter()
        .filter(|a| a.id != id)
        .map(|a| dist(tile, a.tile))
        .fold(30., f32::min)
        / 20.;
    x[19] = allies
        .iter()
        .filter(|a| a.id != id && dist(tile, a.tile) == 1.)
        .count() as f32
        / 6.;
    x[20] = enemies.iter().filter(|a| dist(tile, a.tile) == 1.).count() as f32 / 6.;
    x[21] = g.captures(tile, g.turn()) as u8 as f32;
    if let Some((goal, d)) = orders
        .targets
        .iter()
        .filter(|o| actor_matches(&o.actor, u) && o.family.as_ref().is_none_or(|f| *f == r.family))
        .filter_map(|o| {
            o.tiles
                .iter()
                .map(|t| dist(tile, g.site(t)))
                .reduce(f32::min)
                .map(|d| (o, d))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
    {
        let old = goal
            .tiles
            .iter()
            .map(|t| dist(u.tile, g.site(t)))
            .fold(30., f32::min);
        x[22] = d / 20.;
        x[23] = (old - d) / 10.;
        x[24] = goal.weight as f32 / 10.;
        x[25] = goal.hold.unwrap_or(false) as u8 as f32;
    }
    if let Some(t) = target.and_then(|id| g.unit(id)) {
        let w = g.weapons(u).position(|w| Some(&w.id) == weapon).unwrap();
        let forecast = g.exchange(u, t, w, u.tile);
        x[26] = forecast.effective[0] as f32 / 30.;
        x[27] = forecast.damage[1] as f32 / 30.;
        x[28] = forecast.death[1] as f32;
        x[29] = t.hp as f32 / 30.;
        x[30] = t.is_leader() as u8 as f32;
    }
    x
}
#[derive(Default)]
pub struct Dataset {
    pub features: Vec<[f32; FEATURES]>,
    pub offsets: Vec<i64>,
    pub labels: Vec<i64>,
    pub missing: usize,
    pub limit: usize,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Layer {
    pub weights: Vec<Vec<f32>>,
    pub bias: Vec<f32>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Model {
    pub schema: u32,
    pub feature_names: Vec<String>,
    pub layers: Vec<Layer>,
}
impl Model {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA || self.feature_names != FEATURE_NAMES {
            return Err("model feature schema mismatch".into());
        }
        if self.layers.is_empty() || self.layers.len() > 8 {
            return Err("invalid layer count".into());
        }
        let mut width = FEATURES;
        for layer in &self.layers {
            if layer.bias.is_empty()
                || layer.bias.len() > 1024
                || layer.weights.len() != layer.bias.len()
                || layer
                    .weights
                    .iter()
                    .any(|r| r.len() != width || r.iter().any(|v| !v.is_finite()))
                || layer.bias.iter().any(|v| !v.is_finite())
            {
                return Err("invalid model dimensions or non-finite weights".into());
            }
            width = layer.bias.len();
        }
        if width != 1 {
            return Err("model must produce one logit".into());
        }
        Ok(())
    }
    pub fn score(&self, input: &[f32]) -> f32 {
        let mut x = input.to_vec();
        for (i, l) in self.layers.iter().enumerate() {
            x = l
                .weights
                .iter()
                .zip(&l.bias)
                .map(|(w, b)| {
                    let y = w.iter().zip(&x).map(|(a, b)| a * b).sum::<f32>() + b;
                    if i + 1 == self.layers.len() {
                        y
                    } else {
                        y.max(0.)
                    }
                })
                .collect();
        }
        x[0]
    }
}
pub type Controller = Box<dyn FnMut(&Choices) -> usize>;
#[derive(Default)]
struct Session {
    data: Option<Dataset>,
    model: Option<Model>,
    controller: Option<Controller>,
}
thread_local! { static SESSION: RefCell<Session> = RefCell::new(Session::default()); }
struct Restore(Option<Session>);
impl Drop for Restore {
    fn drop(&mut self) {
        SESSION.with(|s| *s.borrow_mut() = self.0.take().unwrap());
    }
}
pub fn scoped<T>(
    data: Option<Dataset>,
    model: Option<Model>,
    controller: Option<Controller>,
    f: impl FnOnce() -> T,
) -> (T, Option<Dataset>) {
    let old = SESSION.with(|s| {
        s.replace(Session {
            data,
            model,
            controller,
        })
    });
    let _restore = Restore(Some(old));
    let result = f();
    let data = SESSION.with(|s| s.borrow_mut().data.take());
    (result, data)
}
pub(crate) fn record(g: &BattleView<'_, '_>, orders: &TacticalOrders, command: &Command) {
    let enabled = SESSION.with(|s| {
        s.borrow()
            .data
            .as_ref()
            .is_some_and(|d| d.labels.len() < d.limit)
    });
    if !enabled
        || !matches!(
            command,
            Command::Move { .. }
                | Command::Attack { .. }
                | Command::Advance { .. }
                | Command::EndTurn
        )
    {
        return;
    }
    let choices = candidates(g, orders);
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let d = s.data.as_mut().unwrap();
        if let Some(index) = choices.commands.iter().position(|c| c == command) {
            if d.offsets.is_empty() {
                d.offsets.push(0);
            }
            d.features.extend(choices.features);
            d.offsets.push(d.features.len() as i64);
            d.labels.push(index as i64);
        } else {
            d.missing += 1;
        }
    });
}
pub(crate) fn configured() -> bool {
    SESSION.with(|s| {
        let s = s.borrow();
        s.model.is_some() || s.controller.is_some()
    })
}
pub(crate) fn select(choices: &Choices) -> usize {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(c) = s.controller.as_mut() {
            return c(choices);
        }
        let m = s
            .model
            .as_ref()
            .expect("Learned planner requires a model or controller");
        // Match torch.argmax: retain the first candidate on equal logits.
        let mut best = (0, f32::NEG_INFINITY);
        for (index, features) in choices.features.iter().enumerate() {
            let score = m.score(features);
            if score > best.1 {
                best = (index, score);
            }
        }
        best.0
    })
}
pub fn learned_blocks(mut blocks: crate::ai::Blocks) -> crate::ai::Blocks {
    blocks.planner = Planner::Learned;
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_logits_follow_pytorch_argmax() {
        let model = Model {
            schema: SCHEMA,
            feature_names: FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
            layers: vec![Layer {
                weights: vec![vec![0.; FEATURES]],
                bias: vec![0.],
            }],
        };
        let choices = Choices {
            commands: vec![Command::EndTurn; 3],
            features: vec![[0.; FEATURES]; 3],
        };
        let (index, _) = scoped(None, Some(model), None, || select(&choices));
        assert_eq!(index, 0);
    }
    #[test]
    fn candidates_execute_on_mechanical_forecasts() {
        let rules = Rules::load().unwrap();
        let mut game = crate::logic::Game::new(&rules, "mirror");
        let orders = TacticalOrders::default();
        for step in 0..48 {
            if game.winner().is_some() {
                break;
            }
            let choices = candidates(&game.view(), &orders);
            assert_eq!(choices.commands.len(), choices.features.len());
            for (command, features) in choices.commands.iter().zip(&choices.features) {
                assert!(features.iter().all(|v| v.is_finite()));
                let mut draft = game.action_forecasts().fork();
                draft
                    .execute(command.clone())
                    .unwrap_or_else(|e| panic!("candidate {command:?}: {e}"));
            }
            let index = (step * 31) % choices.commands.len();
            game.execute(choices.commands[index].clone()).unwrap();
        }
    }
    #[test]
    fn recording_preserves_complete_teacher_trace() {
        use crate::simulation::{ArenaConfig, Armies, run_arena};
        let rules = Rules::load().unwrap();
        let config = ArenaConfig {
            objective: Default::default(),
            seed: 33,
            armies: Armies {
                player: "human".into(),
                enemy: "skeletons".into(),
            },
            landscape: "open".into(),
            player: crate::ai::Policy::Balanced,
            enemy: crate::ai::Policy::Balanced,
            player_blocks: None,
            enemy_blocks: None,
            first: Side::Player,
            max_rounds: 30,
            recruit_limit: None,
        };
        let baseline = run_arena(&rules, config.clone()).unwrap();
        let (recorded, data) = scoped(
            Some(Dataset {
                limit: 10000,
                ..Default::default()
            }),
            None,
            None,
            || run_arena(&rules, config).unwrap(),
        );
        assert_eq!(
            serde_json::to_value(baseline).unwrap(),
            serde_json::to_value(recorded).unwrap()
        );
        let data = data.unwrap();
        assert!(!data.labels.is_empty());
        assert_eq!(data.missing, 0);
    }
}
