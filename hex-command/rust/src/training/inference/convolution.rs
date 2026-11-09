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
    arch: pulp::Arch,
    height: usize,
    width: usize,
    mask: Vec<f32>,
    sites: Vec<usize>,
    neighbors: Vec<[usize; 7]>,
    first: Vec<f32>,
    embedding: Vec<f32>,
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
        Self {
            layers,
            arch: pulp::Arch::new(),
            height: 0,
            width: 0,
            mask: Vec::new(),
            sites: Vec::new(),
            neighbors: Vec::new(),
            first: Vec::new(),
            embedding: Vec::new(),
        }
    }

    pub(super) fn inputs(
        &mut self,
        board: &Board,
        indices: &[[i64; 3]],
        features: &[[f32; FEATURES]],
        input: &mut Vec<f32>,
    ) {
        let cells = board.height * board.width;
        if self.height != board.height
            || self.width != board.width
            || self.mask != board.values[..cells]
        {
            self.height = board.height;
            self.width = board.width;
            self.mask.clear();
            self.mask.extend_from_slice(&board.values[..cells]);
            self.sites.clear();
            self.sites
                .extend((0..cells).filter(|&site| board.values[site] == 1.));
            self.neighbors.clear();
            self.neighbors.extend(self.sites.iter().map(|&site| {
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
            }));
            // Invalid cells must be zero after a shape or mask change. With the
            // same geometry, kernels overwrite every valid cell on each call.
            self.first.resize(cells * EMBEDDING, 0.);
            self.first.fill(0.);
            self.embedding.resize(cells * EMBEDDING, 0.);
            self.embedding.fill(0.);
        }
        dispatch(
            self.arch,
            Kernel::<true> {
                layer: &self.layers[0],
                input: &board.values,
                output: &mut self.first,
                cells,
                sites: &self.sites,
                neighbors: &self.neighbors,
            },
        );
        dispatch(
            self.arch,
            Kernel::<false> {
                layer: &self.layers[1],
                input: &self.first,
                output: &mut self.embedding,
                cells,
                sites: &self.sites,
                neighbors: &self.neighbors,
            },
        );
        let mut pooled = [0.; EMBEDDING];
        for tile in self.embedding.chunks_exact(EMBEDDING) {
            for (sum, value) in pooled.iter_mut().zip(tile) {
                *sum += value;
            }
        }
        let count = self.sites.len() as f32;
        for sum in &mut pooled {
            *sum /= count;
        }
        input.reserve(features.len() * (FEATURES + 4 * EMBEDDING));
        for (features, tiles) in features.iter().zip(indices) {
            input.extend_from_slice(features);
            for &tile in tiles {
                if tile < 0 {
                    input.extend_from_slice(&[0.; EMBEDDING]);
                } else {
                    input.extend_from_slice(
                        &self.embedding[tile as usize * EMBEDDING..(tile as usize + 1) * EMBEDDING],
                    );
                }
            }
            input.extend_from_slice(&pooled);
        }
    }
}

#[inline]
fn dispatch<K: WithSimd>(arch: pulp::Arch, kernel: K) -> K::Output {
    // Host-specific builds select a concrete SIMD type before entering the
    // kernel. try_new folds to Some when the complete feature set is enabled.
    #[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
    if let Some(simd) = pulp::x86::V4::try_new() {
        return Simd::vectorize(simd, kernel);
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    if let Some(simd) = pulp::x86::V3::try_new() {
        return Simd::vectorize(simd, kernel);
    }
    arch.dispatch(kernel)
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
        let (bias, _) = S::as_simd_f32s(&self.layer.bias);
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
                    let (weights, _) =
                        S::as_simd_f32s(&self.layer.weights[offset..offset + EMBEDDING]);
                    for (sum, &weight) in sums[..vectors].iter_mut().zip(weights) {
                        // Separate multiply/add retain the scalar accumulation order.
                        *sum = simd.add_f32s(*sum, simd.mul_f32s(value, weight));
                    }
                }
            }
            let a = site * EMBEDDING;
            let (output, _) = S::as_mut_simd_f32s(&mut self.output[a..a + EMBEDDING]);
            for ((out, sum), &bias) in output.iter_mut().zip(&sums[..vectors]).zip(bias) {
                *out = simd.max_f32s(simd.add_f32s(*sum, bias), zero);
            }
        }
    }
}
