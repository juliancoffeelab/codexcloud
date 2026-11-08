//! Native training contract. Numeric observations never expose authored scripts.
use crate::{
    ai::{Planner, context::BattleView, orders::*},
    data::*,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
pub const SCHEMA: u32 = 4;
mod duties;
mod features;
pub mod spatial;
pub use features::{Choices, FEATURE_NAMES, FEATURES, LEGACY_FEATURES, candidates};
#[derive(Default)]
pub struct Dataset {
    pub features: Vec<[f32; FEATURES]>,
    pub offsets: Vec<i64>,
    pub labels: Vec<i64>,
    pub missing: usize,
    pub limit: usize,
    pub side: Option<Side>,
    pub spatial: bool,
    pub boards: Vec<spatial::Board>,
    pub indices: Vec<[i64; 3]>,
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
    #[serde(default)]
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub trees: Vec<Tree>,
    #[serde(default)]
    pub spatial: Option<spatial::Encoder>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Tree {
    pub nodes: Vec<TreeNode>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TreeNode {
    Leaf {
        value: f64,
    },
    Split {
        feature: usize,
        threshold: f64,
        left: usize,
        right: usize,
        default_left: bool,
    },
}
impl Tree {
    fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() || self.nodes.len() > 8191 {
            return Err("invalid tree size".into());
        }
        let mut depths = vec![0usize; self.nodes.len()];
        depths[0] = 1;
        for (i, node) in self.nodes.iter().enumerate() {
            if depths[i] == 0 || depths[i] > 13 {
                return Err("unreachable or overly deep tree node".into());
            }
            match node {
                TreeNode::Leaf { value } => {
                    if !value.is_finite() {
                        return Err("non-finite leaf".into());
                    }
                }
                TreeNode::Split {
                    feature,
                    threshold,
                    left,
                    right,
                    ..
                } => {
                    if *feature >= FEATURES || !threshold.is_finite() || left == right {
                        return Err("invalid tree split".into());
                    }
                    for &child in [left, right] {
                        if child <= i || child >= self.nodes.len() || depths[child] != 0 {
                            return Err("invalid tree topology".into());
                        }
                        depths[child] = depths[i] + 1;
                    }
                }
            }
        }
        Ok(())
    }
    fn score(&self, input: &[f32]) -> f64 {
        let mut index = 0;
        loop {
            match &self.nodes[index] {
                TreeNode::Leaf { value } => return *value,
                TreeNode::Split {
                    feature,
                    threshold,
                    left,
                    right,
                    default_left,
                } => {
                    let value = input[*feature];
                    let goes_left = if value.is_nan() {
                        *default_left
                    } else {
                        f64::from(value) <= *threshold
                    };
                    index = if goes_left { *left } else { *right };
                }
            }
        }
    }
}
impl Model {
    /// Preserve legacy input meanings, inserting zero weights before CNN gathers.
    pub fn upgrade(&mut self) -> Result<(), String> {
        let old_schema = if self.spatial.is_some() { 3 } else { 2 };
        if self.schema != old_schema {
            return self.validate();
        }
        if self.feature_names != FEATURE_NAMES[..LEGACY_FEATURES] {
            return Err("legacy model feature names differ".into());
        }
        let embedding = if self.spatial.is_some() {
            4 * spatial::EMBEDDING
        } else {
            0
        };
        if let Some(encoder) = &mut self.spatial {
            let names = spatial::channel_names();
            if encoder.channel_names != names[..spatial::LEGACY_CHANNELS]
                || encoder.convolutions.len() != 2
            {
                return Err("legacy spatial schema differs".into());
            }
            let first = &mut encoder.convolutions[0];
            if first.input_channels != spatial::LEGACY_CHANNELS
                || first.output_channels != spatial::EMBEDDING
                || first.weights.len() != spatial::EMBEDDING * spatial::LEGACY_CHANNELS * 7
            {
                return Err("legacy convolution dimensions differ".into());
            }
            let mut weights = vec![0.; spatial::EMBEDDING * spatial::CHANNELS * 7];
            for output in 0..spatial::EMBEDDING {
                let old = output * spatial::LEGACY_CHANNELS * 7;
                let new = output * spatial::CHANNELS * 7;
                weights[new..new + spatial::LEGACY_CHANNELS * 7]
                    .copy_from_slice(&first.weights[old..old + spatial::LEGACY_CHANNELS * 7]);
            }
            first.weights = weights;
            first.input_channels = spatial::CHANNELS;
            encoder.channel_names = names;
        }
        if let Some(first) = self.layers.first_mut() {
            for row in &mut first.weights {
                if row.len() != LEGACY_FEATURES + embedding {
                    return Err("legacy dense dimensions differ".into());
                }
                let mut upgraded = vec![0.; FEATURES + embedding];
                upgraded[..LEGACY_FEATURES].copy_from_slice(&row[..LEGACY_FEATURES]);
                upgraded[FEATURES..].copy_from_slice(&row[LEGACY_FEATURES..]);
                *row = upgraded;
            }
        }
        if self
            .trees
            .iter()
            .flat_map(|t| &t.nodes)
            .any(|n| matches!(n, TreeNode::Split{feature,..} if *feature>=LEGACY_FEATURES))
        {
            return Err("legacy tree feature is outside schema".into());
        }
        self.feature_names = FEATURE_NAMES.iter().map(|s| s.to_string()).collect();
        self.schema = if self.spatial.is_some() {
            spatial::SPATIAL_SCHEMA
        } else {
            SCHEMA
        };
        self.validate()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema
            != (if self.spatial.is_some() {
                spatial::SPATIAL_SCHEMA
            } else {
                SCHEMA
            })
            || self.feature_names != FEATURE_NAMES
        {
            return Err("model feature schema mismatch".into());
        }
        if let Some(encoder) = &self.spatial {
            encoder.validate()?;
            if !self.trees.is_empty() {
                return Err("cannot combine trees and spatial encoder".into());
            }
        }
        if !self.trees.is_empty() {
            if !self.layers.is_empty() || self.trees.len() > 1024 {
                return Err("ambiguous or oversized tree ensemble".into());
            }
            for tree in &self.trees {
                tree.validate()?;
            }
            return Ok(());
        }
        if self.layers.is_empty() || self.layers.len() > 8 {
            return Err("invalid layer count".into());
        }
        let mut width = FEATURES
            + if self.spatial.is_some() {
                4 * spatial::EMBEDDING
            } else {
                0
            };
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
    pub fn score(&self, input: &[f32]) -> f64 {
        if !self.trees.is_empty() {
            return self.trees.iter().map(|tree| tree.score(input)).sum();
        }
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
        f64::from(x[0])
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
            if let Some(spatial) = choices.spatial {
                d.boards.push(spatial.board);
                d.indices.extend(spatial.indices);
            }
            d.features.extend(choices.features);
            d.offsets.push(d.features.len() as i64);
            d.labels.push(index as i64);
        } else {
            d.missing += 1;
        }
    });
}
pub(crate) fn needs_spatial() -> bool {
    SESSION.with(|s| {
        let s = s.borrow();
        s.data.as_ref().is_some_and(|d| d.spatial)
            || s.model.as_ref().is_some_and(|m| m.spatial.is_some())
    })
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
        let inputs = m.spatial.as_ref().map(|encoder| {
            let spatial = choices
                .spatial
                .as_ref()
                .expect("spatial observation required");
            encoder.inputs(&spatial.board, &spatial.indices, &choices.features)
        });
        // Match torch.argmax: retain the first candidate on equal logits.
        let mut best = (0, f64::NEG_INFINITY);
        for (index, features) in choices.features.iter().enumerate() {
            let score = m.score(
                inputs
                    .as_ref()
                    .map_or(features.as_slice(), |x| x[index].as_slice()),
            );
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
            trees: vec![],
            spatial: None,
            layers: vec![Layer {
                weights: vec![vec![0.; FEATURES]],
                bias: vec![0.],
            }],
        };
        let choices = Choices {
            spatial: None,
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
