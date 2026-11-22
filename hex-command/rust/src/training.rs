//! Native training contract. Numeric observations never expose authored scripts.
use crate::{
    ai::{PlannerKind, context::BattleView, orders::*},
    data::*,
};
use hex_command_battle::RuntimeCommand;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
pub const SCHEMA: u32 = 16;
mod duties;
mod features;
#[cfg(feature = "simulation")]
pub mod imitator;
mod inference;
mod recurrent;
pub mod spatial;
mod visibility;
pub use features::{
    CAMPAIGN_FEATURES, Choices, DIPLOMACY_FEATURES, DUTY_FEATURES, FEATURE_NAMES, FEATURES,
    IDLE_FEATURES, LEGACY_FEATURES, VISIBILITY_FEATURES, candidates,
};
pub use inference::Evaluator;
pub use recurrent::RecurrentLayer;
#[derive(Default)]
pub struct Dataset {
    pub features: Vec<[f32; FEATURES]>,
    pub offsets: Vec<i64>,
    pub labels: Vec<i64>,
    pub missing: usize,
    pub limit: usize,
    pub phase: Option<TurnPhase>,
    pub spatial: bool,
    pub boards: Vec<spatial::SpatialBoard>,
    pub indices: Vec<[i64; 3]>,
    pub game: usize,
    pub decision_contexts: Vec<CollectedDecision>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct DenseLayer {
    pub weights: Vec<Vec<f32>>,
    pub bias: Vec<f32>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Model {
    #[serde(default)]
    pub recurrent: Option<RecurrentLayer>,
    pub schema: u32,
    pub feature_names: Vec<String>,
    #[serde(default)]
    pub layers: Vec<DenseLayer>,
    #[serde(default)]
    pub trees: Vec<Tree>,
    #[serde(default)]
    pub spatial: Option<spatial::SpatialEncoder>,
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
        let (old_width, old_channels) = match (self.schema, self.spatial.is_some()) {
            (2, false) | (3, true) => (LEGACY_FEATURES, spatial::LEGACY_CHANNELS),
            (4, false) | (5, true) => (DUTY_FEATURES, spatial::DUTY_CHANNELS),
            (6, false) | (7, true) => (DIPLOMACY_FEATURES, spatial::CAMPAIGN_CHANNELS),
            (8, false) | (9, true) => (CAMPAIGN_FEATURES, spatial::CAMPAIGN_CHANNELS),
            (10, false) | (11, true) => (VISIBILITY_FEATURES, spatial::VISIBILITY_CHANNELS),
            (12, false) => (IDLE_FEATURES, spatial::CHANNELS),
            (13, true) => (IDLE_FEATURES, spatial::VISIBILITY_CHANNELS),
            (14, true) => (IDLE_FEATURES, spatial::HOLD_CHANNELS),
            (15, true) => (IDLE_FEATURES, spatial::CHANNELS),
            (17, true) | (18, true) => (FEATURES, spatial::CHANNELS),
            _ => return self.validate(),
        };
        if self.feature_names != FEATURE_NAMES[..old_width] {
            return Err("legacy model feature names differ".into());
        }
        let legacy_coarse = self.schema != 18;
        let embedding = if self.spatial.is_some() {
            if legacy_coarse {
                4 * spatial::EMBEDDING
            } else {
                spatial::ENCODED_FEATURES
            }
        } else {
            0
        };
        if let Some(encoder) = &mut self.spatial {
            let names = spatial::channel_names();
            if encoder.channel_names != names[..old_channels]
                || encoder.convolutions.len() != if legacy_coarse { 2 } else { 4 }
            {
                return Err("legacy spatial schema differs".into());
            }
            let first = &mut encoder.convolutions[0];
            if first.input_channels != old_channels
                || first.output_channels != spatial::EMBEDDING
                || first.weights.len() != spatial::EMBEDDING * old_channels * 7
            {
                return Err("legacy convolution dimensions differ".into());
            }
            let mut weights = vec![0.; spatial::EMBEDDING * spatial::CHANNELS * 7];
            for output in 0..spatial::EMBEDDING {
                let old = output * old_channels * 7;
                let new = output * spatial::CHANNELS * 7;
                weights[new..new + old_channels * 7]
                    .copy_from_slice(&first.weights[old..old + old_channels * 7]);
            }
            first.weights = weights;
            first.input_channels = spatial::CHANNELS;
            encoder.channel_names = names;
            // Deterministic nonzero initialization; appended head weights remain
            // zero, preserving legacy scores without creating a dead branch.
            if legacy_coarse {
                let mut seed = 731u64;
                for channels in [spatial::CHANNELS, spatial::EMBEDDING] {
                    let scale = 1. / (channels as f32 * 7.).sqrt();
                    let weights = (0..spatial::EMBEDDING * channels * 7)
                        .map(|_| {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            ((seed >> 32) as u32 as f32 / u32::MAX as f32 * 2. - 1.) * scale
                        })
                        .collect();
                    encoder.convolutions.push(spatial::Convolution {
                        input_channels: channels,
                        output_channels: spatial::EMBEDDING,
                        weights,
                        bias: vec![0.; spatial::EMBEDDING],
                    });
                }
            }
        }
        if self.spatial.is_some() {
            self.recurrent = Some(RecurrentLayer::initialized());
        }
        if let Some(first) = self.layers.first_mut() {
            for row in &mut first.weights {
                if row.len() != old_width + embedding {
                    return Err("legacy dense dimensions differ".into());
                }
                let mut upgraded = vec![
                    0.;
                    FEATURES
                        + if self.spatial.is_some() {
                            spatial::ENCODED_FEATURES + recurrent::MEMORY
                        } else {
                            0
                        }
                ];
                upgraded[..old_width].copy_from_slice(&row[..old_width]);
                upgraded[FEATURES..FEATURES + embedding].copy_from_slice(&row[old_width..]);
                *row = upgraded;
            }
        }
        if self
            .trees
            .iter()
            .flat_map(|t| &t.nodes)
            .any(|n| matches!(n, TreeNode::Split{feature,..} if *feature>=old_width))
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
        if self.spatial.is_none() && self.recurrent.is_some() {
            return Err("dense model cannot contain a GRU".into());
        }
        if let Some(encoder) = &self.spatial {
            self.recurrent
                .as_ref()
                .ok_or("spatial model requires recurrent weights")?
                .validate()?;
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
                spatial::ENCODED_FEATURES + recurrent::MEMORY
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
/// A disconnected controller cancels its native rollout.
pub type DecisionController = Box<dyn FnMut(&Choices) -> Option<usize>>;
#[derive(Default)]
struct TrainingSession {
    data: Option<Dataset>,
    model: Option<Evaluator>,
    controller: Option<DecisionController>,
    history: std::collections::HashMap<String, TurnHistory>,
    generation: u64,
}
thread_local! { static SESSION: RefCell<TrainingSession> = RefCell::new(TrainingSession::default()); }
struct Restore(Option<TrainingSession>);
impl Drop for Restore {
    fn drop(&mut self) {
        SESSION.with(|s| *s.borrow_mut() = self.0.take().unwrap());
    }
}
pub fn scoped<T>(
    data: Option<Dataset>,
    model: Option<Model>,
    controller: Option<DecisionController>,
    f: impl FnOnce() -> T,
) -> (T, Option<Dataset>) {
    let old = SESSION.with(|s| {
        s.replace(TrainingSession {
            data,
            model: model.map(|model| Evaluator::new(model).expect("valid learned model required")),
            controller,
            history: Default::default(),
            generation: 0,
        })
    });
    let _restore = Restore(Some(old));
    let result = f();
    let data = SESSION.with(|s| s.borrow_mut().data.take());
    (result, data)
}
/// Per-observer tactical history. Recruitment does not count as a planner action.
#[derive(Default)]
struct TurnHistory {
    map: String,
    round: u32,
    idle: u32,
    acted: bool,
    decision: u64,
    units: std::collections::HashMap<u32, UnitHistory>,
}
#[derive(Default)]
struct UnitHistory {
    last_action: usize,
    moved_round: Option<u32>,
    attacked_round: Option<u32>,
    move_origin: Option<usize>,
}
fn command_actor(command: &RuntimeCommand) -> Option<u32> {
    match command {
        RuntimeCommand::Move { unit, .. }
        | RuntimeCommand::Attack { unit, .. }
        | RuntimeCommand::Advance { unit, .. } => Some(unit.0),
        _ => None,
    }
}
impl UnitHistory {
    fn encode(&self, round: u32, command: &RuntimeCommand) -> [f32; 9] {
        let mut x = [0.; 9];
        x[0] = 1.;
        x[1 + self.last_action] = 1.;
        if let Some(last) = self.moved_round {
            x[4] = 1.;
            x[5] = round.saturating_sub(last) as f32 / 10.;
        }
        if let Some(last) = self.attacked_round {
            x[6] = 1.;
            x[7] = round.saturating_sub(last) as f32 / 10.;
        }
        if let RuntimeCommand::Move { tile, .. } = command {
            x[8] = self
                .move_origin
                .as_ref()
                .is_some_and(|origin| origin == &tile.0) as u8 as f32;
        }
        x
    }
}
impl TurnHistory {
    fn observe(&mut self, map: &str, round: u32) -> [f32; 2] {
        if self.map != map || round < self.round {
            *self = Self {
                map: map.to_owned(),
                round,
                ..Self::default()
            };
        } else if round > self.round {
            self.idle = if self.acted {
                0
            } else {
                self.idle.saturating_add(1)
            };
            self.acted = false;
            self.round = round;
        }
        [self.idle as f32 / 10., f32::from(self.acted)]
    }
}
fn history_features(g: &BattleView<'_, '_>) -> [f32; 2] {
    SESSION.with(|s| {
        s.borrow_mut()
            .history
            .entry(g.observer_faction().to_owned())
            .or_default()
            .observe(g.map_id(), g.round())
    })
}
fn unit_history_features(g: &BattleView<'_, '_>, command: &RuntimeCommand) -> [f32; 9] {
    SESSION.with(|s| {
        let s = s.borrow();
        command_actor(command)
            .and_then(|id| s.history.get(g.observer_faction())?.units.get(&id))
            .map_or([0.; 9], |h| h.encode(g.round(), command))
    })
}
#[derive(Clone, Default, Serialize)]
pub struct DecisionContext {
    pub observer: String,
    pub map: String,
    pub round: u32,
    pub decision: u64,
    pub generation: u64,
}
#[derive(Serialize)]
pub struct CollectedDecision {
    pub game: usize,
    pub context: DecisionContext,
}
fn decision_context(g: &BattleView<'_, '_>) -> DecisionContext {
    SESSION.with(|s| DecisionContext {
        observer: g.observer_faction().to_owned(),
        map: g.map_id().to_owned(),
        round: g.round(),
        generation: s.borrow().generation,
        decision: s
            .borrow()
            .history
            .get(g.observer_faction())
            .map_or(0, |h| h.decision),
    })
}
pub(crate) fn reset_history() {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        s.generation += 1;
        s.history.clear();
        if let Some(model) = &mut s.model {
            model.reset_memory();
        }
    });
}
fn mark_history(g: &BattleView<'_, '_>, command: &RuntimeCommand) {
    history_features(g);
    SESSION.with(|s| {
        s.borrow_mut()
            .history
            .get_mut(g.observer_faction())
            .unwrap()
            .decision += 1;
    });
    if *command != RuntimeCommand::EndTurn {
        SESSION.with(|s| {
            let mut s = s.borrow_mut();
            let history = s.history.get_mut(g.observer_faction()).unwrap();
            history.acted = true;
            if let Some(id) = command_actor(command) {
                let unit = history.units.entry(id).or_default();
                match command {
                    RuntimeCommand::Move { .. } => {
                        unit.last_action = 0;
                        unit.moved_round = Some(g.round());
                        unit.move_origin = g.unit(id).map(|u| u.tile);
                    }
                    RuntimeCommand::Attack { .. } => {
                        unit.last_action = 1;
                        unit.attacked_round = Some(g.round());
                    }
                    RuntimeCommand::Advance { .. } => unit.last_action = 2,
                    _ => unreachable!(),
                }
            }
        });
    }
}
pub(crate) fn record(g: &BattleView<'_, '_>, orders: &TacticalOrders, command: &RuntimeCommand) {
    let enabled = SESSION.with(|s| {
        s.borrow().data.as_ref().is_some_and(|d| {
            d.labels.len() < d.limit && d.phase.is_none_or(|side| side == g.turn())
        })
    });
    if !matches!(
        command,
        RuntimeCommand::Move { .. }
            | RuntimeCommand::Attack { .. }
            | RuntimeCommand::Advance { .. }
            | RuntimeCommand::EndTurn
    ) {
        return;
    }
    if !enabled {
        mark_history(g, command);
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
            d.decision_contexts.push(CollectedDecision {
                game: d.game,
                context: choices.context,
            });
        } else {
            d.missing += 1;
        }
    });
    mark_history(g, command);
}
pub(crate) fn needs_spatial() -> bool {
    SESSION.with(|s| {
        let s = s.borrow();
        s.data.as_ref().is_some_and(|d| d.spatial)
            || s.model.as_ref().is_some_and(Evaluator::is_spatial)
    })
}
pub(crate) fn configured() -> bool {
    SESSION.with(|s| {
        let s = s.borrow();
        s.model.is_some() || s.controller.is_some()
    })
}
pub(crate) fn select(choices: &Choices) -> Option<usize> {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(c) = s.controller.as_mut() {
            return c(choices);
        }
        let m = s
            .model
            .as_mut()
            .expect("Learned planner requires a model or controller");
        let scores = m.decision_scores(
            &choices.context,
            &choices.features,
            choices
                .spatial
                .as_ref()
                .map(|s| (&s.board, s.indices.as_slice())),
        );
        // Match torch.argmax: retain the first candidate on equal logits.
        let mut best = (0, f64::NEG_INFINITY);
        for (index, score) in scores.into_iter().enumerate() {
            if score > best.1 {
                best = (index, score);
            }
        }
        Some(best.0)
    })
}
pub fn learned_blocks(mut blocks: crate::ai::Blocks) -> crate::ai::Blocks {
    blocks.planner = PlannerKind::Learned;
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_logits_follow_pytorch_argmax() {
        let model = Model {
            recurrent: None,
            schema: SCHEMA,
            feature_names: FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
            trees: vec![],
            spatial: None,
            layers: vec![DenseLayer {
                weights: vec![vec![0.; FEATURES]],
                bias: vec![0.],
            }],
        };
        let choices = Choices {
            context: DecisionContext::default(),
            spatial: None,
            commands: vec![RuntimeCommand::EndTurn; 3],
            features: vec![[0.; FEATURES]; 3],
        };
        let (index, _) = scoped(None, Some(model), None, || select(&choices));
        assert_eq!(index, Some(0));
    }
    #[test]
    fn candidates_execute_on_mechanical_forecasts() {
        let rules = Catalogue::load().unwrap();
        let mut game = crate::logic::BattleSession::new(&rules, "mirror");
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
                    .execute_runtime(command.clone())
                    .unwrap_or_else(|e| panic!("candidate {command:?}: {e}"));
            }
            let index = (step * 31) % choices.commands.len();
            game.execute_runtime(choices.commands[index].clone())
                .unwrap();
        }
    }
    #[test]
    #[cfg(feature = "simulation")]
    fn recording_preserves_complete_teacher_trace() {
        use crate::simulation::{ArenaConfig, Armies, run_arena};
        let rules = Catalogue::load().unwrap();
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
            first: TurnPhase::Player,
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
