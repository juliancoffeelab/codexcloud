//! Prepared native inference; serialized models retain their portable weight layout.
use super::{FEATURES, Model, spatial};
mod convolution;

pub struct Evaluator {
    model: Model,
    weights: Vec<Vec<f32>>,
    spatial: Option<convolution::Encoder>,
}

impl Evaluator {
    pub fn new(mut model: Model) -> Result<Self, String> {
        model.upgrade()?;
        let weights = model
            .layers
            .iter()
            .map(|layer| layer.weights.iter().flatten().copied().collect())
            .collect();
        let spatial = model.spatial.as_ref().map(convolution::Encoder::new);
        Ok(Self {
            model,
            weights,
            spatial,
        })
    }

    pub fn is_spatial(&self) -> bool {
        self.model.spatial.is_some()
    }

    /// Score one decision's candidates. Spatial arrays must be validated by the caller.
    pub fn scores(
        &mut self,
        features: &[[f32; FEATURES]],
        board: Option<(&spatial::Board, &[[i64; 3]])>,
    ) -> Vec<f64> {
        if features.is_empty() {
            return Vec::new();
        }
        if !self.model.trees.is_empty() {
            return features.iter().map(|row| self.model.score(row)).collect();
        }
        let mut x: Vec<f32> = if let Some(encoder) = &self.spatial {
            let (board, indices) = board.expect("spatial observation required");
            encoder
                .inputs(board, indices, features)
                .into_iter()
                .flatten()
                .collect()
        } else {
            features.iter().flatten().copied().collect()
        };
        let count = features.len();
        for (i, layer) in self.model.layers.iter().enumerate() {
            let width = layer.weights[0].len();
            let outputs = layer.bias.len();
            let mut y = vec![0.; count * outputs];
            // SAFETY: x is count×width, weights are outputs×width, and y is
            // count×outputs. Transposing the weight strides gives width×outputs.
            // All buffers are contiguous, initialized, disjoint and live for the call.
            unsafe {
                gemm::gemm(
                    count,
                    outputs,
                    width,
                    y.as_mut_ptr(),
                    1,
                    outputs as isize,
                    false,
                    x.as_ptr(),
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
            for row in y.chunks_exact_mut(outputs) {
                for (value, bias) in row.iter_mut().zip(&layer.bias) {
                    *value += bias;
                    if i + 1 < self.model.layers.len() {
                        *value = value.max(0.);
                    }
                }
            }
            x = y;
        }
        x.into_iter().map(f64::from).collect()
    }
}
