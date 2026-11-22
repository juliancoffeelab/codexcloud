//! Prepared native inference; serialized models retain their portable weight layout.
use super::{DecisionContext, FEATURES, Model, recurrent::MEMORY, spatial};
use std::collections::HashMap;
struct MemoryEntry {
    map: String,
    round: u32,
    decision: u64,
    generation: u64,
    before: [f32; MEMORY],
    after: [f32; MEMORY],
}
mod convolution;

pub struct Evaluator {
    model: Model,
    weights: Vec<Vec<f32>>,
    spatial: Option<convolution::ConvolutionEncoder>,
    input: Vec<f32>,
    output: Vec<f32>,
    memories: HashMap<String, MemoryEntry>,
}

impl Evaluator {
    pub fn new(mut model: Model) -> Result<Self, String> {
        model.upgrade()?;
        let weights = model
            .layers
            .iter()
            .map(|layer| layer.weights.iter().flatten().copied().collect())
            .collect();
        let spatial = model
            .spatial
            .as_ref()
            .map(convolution::ConvolutionEncoder::new);
        Ok(Self {
            model,
            weights,
            spatial,
            input: Vec::new(),
            output: Vec::new(),
            memories: HashMap::new(),
        })
    }

    pub fn is_spatial(&self) -> bool {
        self.model.spatial.is_some()
    }

    pub fn reset_memory(&mut self) {
        self.memories.clear();
    }
    pub fn decision_scores(
        &mut self,
        context: &DecisionContext,
        features: &[[f32; FEATURES]],
        board: Option<(&spatial::SpatialBoard, &[[i64; 3]])>,
    ) -> Vec<f64> {
        let previous = self
            .memories
            .get(&context.observer)
            .map_or([0.; MEMORY], |m| {
                if m.map != context.map
                    || m.generation != context.generation
                    || context.round < m.round
                    || context.decision < m.decision
                {
                    [0.; MEMORY]
                } else if context.decision == m.decision {
                    m.before
                } else {
                    m.after
                }
            });
        let (scores, next) = self.recurrent_scores(features, board, &previous);
        self.memories.insert(
            context.observer.clone(),
            MemoryEntry {
                map: context.map.clone(),
                round: context.round,
                decision: context.decision,
                generation: context.generation,
                before: previous,
                after: next,
            },
        );
        scores
    }
    pub fn scores(
        &mut self,
        features: &[[f32; FEATURES]],
        board: Option<(&spatial::SpatialBoard, &[[i64; 3]])>,
    ) -> Vec<f64> {
        self.recurrent_scores(features, board, &[0.; MEMORY]).0
    }
    /// Score one decision's candidates. Spatial arrays must be validated by the caller.
    pub fn recurrent_scores(
        &mut self,
        features: &[[f32; FEATURES]],
        board: Option<(&spatial::SpatialBoard, &[[i64; 3]])>,
        previous: &[f32; MEMORY],
    ) -> (Vec<f64>, [f32; MEMORY]) {
        if features.is_empty() {
            return (Vec::new(), *previous);
        }
        if !self.model.trees.is_empty() {
            return (
                features.iter().map(|row| self.model.score(row)).collect(),
                *previous,
            );
        }
        self.input.clear();
        let mut summary = [0.; spatial::EMBEDDING];
        if let Some(encoder) = &mut self.spatial {
            let (board, indices) = board.expect("spatial observation required");
            assert_eq!(
                indices.len(),
                features.len(),
                "candidate index count mismatch"
            );
            encoder.inputs(board, indices, features, &mut self.input, &mut summary);
        } else {
            self.input.extend(features.iter().flatten().copied());
        }
        let memory = self
            .model
            .recurrent
            .as_ref()
            .map_or(*previous, |g| g.step(&summary, previous));
        if self.model.recurrent.is_some() {
            self.output.clear();
            for row in self
                .input
                .chunks_exact(FEATURES + spatial::ENCODED_FEATURES)
            {
                self.output.extend_from_slice(row);
                self.output.extend_from_slice(&memory);
            }
            std::mem::swap(&mut self.input, &mut self.output);
        }
        let count = features.len();
        for (i, layer) in self.model.layers.iter().enumerate() {
            let width = layer.weights[0].len();
            let outputs = layer.bias.len();
            assert_eq!(
                self.input.len(),
                count * width,
                "dense input shape mismatch"
            );
            self.output.resize(count * outputs, 0.);
            // SAFETY: x is count×width, weights are outputs×width, and y is
            // count×outputs. Transposing the weight strides gives width×outputs.
            // All buffers are contiguous, initialized, disjoint and live for the call.
            unsafe {
                gemm::gemm(
                    count,
                    outputs,
                    width,
                    self.output.as_mut_ptr(),
                    1,
                    outputs as isize,
                    false,
                    self.input.as_ptr(),
                    1,
                    width as isize,
                    self.weights[i].as_ptr(),
                    width as isize,
                    1,
                    0.,
                    1.,
                    false,
                    false,
                    false,
                    gemm::Parallelism::None,
                );
            }
            for row in self.output.chunks_exact_mut(outputs) {
                for (value, bias) in row.iter_mut().zip(&layer.bias) {
                    *value += bias;
                    if i + 1 < self.model.layers.len() {
                        *value = value.max(0.);
                    }
                }
            }
            std::mem::swap(&mut self.input, &mut self.output);
        }
        (self.input.iter().copied().map(f64::from).collect(), memory)
    }
}
