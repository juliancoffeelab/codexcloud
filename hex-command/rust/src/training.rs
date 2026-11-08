//! Native training contract. Numeric observations never expose authored scripts.
use crate::{
    ai::{Planner, context::BattleView, orders::*},
    data::*,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
pub const SCHEMA: u32 = 2;
mod features;
pub use features::{Choices, FEATURE_NAMES, FEATURES, candidates};
#[derive(Default)]
pub struct Dataset {
    pub features: Vec<[f32; FEATURES]>,
    pub offsets: Vec<i64>,
    pub labels: Vec<i64>,
    pub missing: usize,
    pub limit: usize,
    pub side: Option<Side>,
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
            .is_some_and(|d| d.labels.len() < d.limit && d.side.is_none_or(|side| side == g.turn()))
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
