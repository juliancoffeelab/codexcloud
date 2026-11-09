//! Fixed seven-neighbor convolutions, vectorized across the sixteen outputs.
use super::super::{
    FEATURES,
    spatial::{self, Board, EMBEDDING, KERNEL},
};
use pulp::{Simd, WithSimd};

struct Layer {
    channels: usize,
    weights: Vec<f32>, // input channel, kernel position, output channel
    bias: Vec<f32>,
}

pub(super) struct Encoder {
    layers: Vec<Layer>,
}

impl Encoder {
    pub(super) fn new(encoder: &spatial::Encoder) -> Self {
        let layers = encoder
            .convolutions
            .iter()
            .map(|conv| {
                let mut weights = vec![0.; conv.weights.len()];
                for channel in 0..conv.input_channels {
                    for k in 0..7 {
                        for out in 0..EMBEDDING {
                            weights[(channel * 7 + k) * EMBEDDING + out] =
                                conv.weights[(out * conv.input_channels + channel) * 7 + k];
                        }
                    }
                }
                Layer {
                    channels: conv.input_channels,
                    weights,
                    bias: conv.bias.clone(),
                }
            })
            .collect();
        Self { layers }
    }

    pub(super) fn inputs(
        &self,
        board: &Board,
        indices: &[[i64; 3]],
        features: &[[f32; FEATURES]],
    ) -> Vec<Vec<f32>> {
        let cells = board.height * board.width;
        let sites: Vec<_> = (0..cells)
            .filter(|&site| board.values[site] == 1.)
            .collect();
        let neighbors: Vec<_> = sites
            .iter()
            .map(|&site| {
                KERNEL.map(|(dr, dq)| {
                    let row = (site / board.width) as i32 + dr;
                    let col = (site % board.width) as i32 + dq;
                    if row < 0 || col < 0 || row >= board.height as i32 || col >= board.width as i32
                    {
                        usize::MAX
                    } else {
                        row as usize * board.width + col as usize
                    }
                })
            })
            .collect();
        let mut first = vec![0.; cells * EMBEDDING];
        let mut embedding = vec![0.; cells * EMBEDDING];
        let arch = pulp::Arch::new();
        arch.dispatch(Kernel::<true> {
            layer: &self.layers[0],
            input: &board.values,
            output: &mut first,
            cells,
            sites: &sites,
            neighbors: &neighbors,
        });
        arch.dispatch(Kernel::<false> {
            layer: &self.layers[1],
            input: &first,
            output: &mut embedding,
            cells,
            sites: &sites,
            neighbors: &neighbors,
        });
        let mut pooled = [0.; EMBEDDING];
        for tile in embedding.chunks_exact(EMBEDDING) {
            for (sum, value) in pooled.iter_mut().zip(tile) {
                *sum += value;
            }
        }
        let count = sites.len() as f32;
        for sum in &mut pooled {
            *sum /= count;
        }
        features
            .iter()
            .zip(indices)
            .map(|(features, tiles)| {
                let mut row = features.to_vec();
                for &tile in tiles {
                    if tile < 0 {
                        row.extend_from_slice(&[0.; EMBEDDING]);
                    } else {
                        row.extend_from_slice(
                            &embedding[tile as usize * EMBEDDING..(tile as usize + 1) * EMBEDDING],
                        );
                    }
                }
                row.extend_from_slice(&pooled);
                row
            })
            .collect()
    }
}

struct Kernel<'a, const CHANNEL_MAJOR: bool> {
    layer: &'a Layer,
    input: &'a [f32],
    output: &'a mut [f32],
    cells: usize,
    sites: &'a [usize],
    neighbors: &'a [[usize; 7]],
}

impl<const CHANNEL_MAJOR: bool> WithSimd for Kernel<'_, CHANNEL_MAJOR> {
    type Output = ();

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) {
        let lanes = size_of::<S::f32s>() / size_of::<f32>();
        assert!(lanes <= EMBEDDING && EMBEDDING % lanes == 0);
        let vectors = EMBEDDING / lanes;
        let zero = simd.splat_f32s(0.);
        for (&site, neighbors) in self.sites.iter().zip(self.neighbors) {
            let mut sums = [zero; EMBEDDING];
            for channel in 0..self.layer.channels {
                for (k, &neighbor) in neighbors.iter().enumerate() {
                    if neighbor == usize::MAX {
                        continue;
                    }
                    let value = self.input[if CHANNEL_MAJOR {
                        channel * self.cells + neighbor
                    } else {
                        neighbor * EMBEDDING + channel
                    }];
                    let value = simd.splat_f32s(value);
                    let offset = (channel * 7 + k) * EMBEDDING;
                    for (v, sum) in sums[..vectors].iter_mut().enumerate() {
                        let a = offset + v * lanes;
                        let weight = simd.partial_load_f32s(&self.layer.weights[a..a + lanes]);
                        // Separate multiply/add retain the scalar accumulation order.
                        *sum = simd.add_f32s(*sum, simd.mul_f32s(value, weight));
                    }
                }
            }
            for (v, sum) in sums[..vectors].iter().enumerate() {
                let a = v * lanes;
                let bias = simd.partial_load_f32s(&self.layer.bias[a..a + lanes]);
                let value = simd.max_f32s(simd.add_f32s(*sum, bias), zero);
                let a = site * EMBEDDING + a;
                simd.partial_store_f32s(&mut self.output[a..a + lanes], value);
            }
        }
    }
}
