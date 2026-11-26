//! Fixed seven-neighbor convolutions, vectorized across the sixteen outputs.
use super::super::{
    FEATURES,
    spatial::{self, EMBEDDING, KERNEL, SpatialBoard},
};
#[cfg(not(target_arch = "wasm32"))]
use pulp::{Simd, WithSimd};

struct ConvolutionLayer {
    channels: usize,
    weights: Vec<f32>, // input channel, kernel position, output channel
    bias: Vec<f32>,
}

pub(super) struct ConvolutionEncoder {
    layers: Vec<ConvolutionLayer>,
    #[cfg(not(target_arch = "wasm32"))]
    arch: pulp::Arch,
    stride: usize,
    coarse: Option<Box<ConvolutionEncoder>>,
    next_neighbors: Vec<[usize; 7]>,
    height: usize,
    width: usize,
    mask: Vec<f32>,
    sites: Vec<usize>,
    neighbors: Vec<[usize; 7]>,
    first: Vec<f32>,
    embedding: Vec<f32>,
}

impl ConvolutionEncoder {
    pub(super) fn new(encoder: &spatial::SpatialEncoder) -> Self {
        let mut result = Self::from_layers(&encoder.convolutions[..2], 1);
        result.coarse = Some(Box::new(Self::from_layers(&encoder.convolutions[2..], 2)));
        result
    }
    fn from_layers(convolutions: &[spatial::Convolution], stride: usize) -> Self {
        let layers = convolutions
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
                ConvolutionLayer {
                    channels: conv.input_channels,
                    weights,
                    bias: conv.bias.clone(),
                }
            })
            .collect();
        Self {
            layers,
            #[cfg(not(target_arch = "wasm32"))]
            arch: pulp::Arch::new(),
            stride,
            coarse: None,
            next_neighbors: Vec::new(),
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
        board: &SpatialBoard,
        indices: &[[i64; 3]],
        features: &[[f32; FEATURES]],
        input: &mut Vec<f32>,
        summary: &mut [f32; EMBEDDING],
    ) {
        if !board.contexts.is_empty() {
            let stride = FEATURES + spatial::ENCODED_FEATURES;
            let offset = input.len();
            input.resize(offset + features.len() * stride, 0.);
            for actor in
                std::iter::once(-1).chain(board.contexts.iter().map(|c| c.actor_site as i64))
            {
                let selected: Vec<_> = indices
                    .iter()
                    .enumerate()
                    .filter(|(_, ix)| ix[0] == actor)
                    .map(|(i, _)| i)
                    .collect();
                if selected.is_empty() && actor != -1 {
                    continue;
                }
                let context = board.contexts.iter().find(|c| c.actor_site as i64 == actor);
                let plain =
                    context
                        .map(|c| board.with_context(c))
                        .unwrap_or_else(|| SpatialBoard {
                            contexts: vec![],
                            ..board.clone()
                        });
                let ix: Vec<_> = selected.iter().map(|&i| indices[i]).collect();
                let x: Vec<_> = selected.iter().map(|&i| features[i]).collect();
                let mut rows = Vec::new();
                let mut pooled = [0.; EMBEDDING];
                self.inputs(&plain, &ix, &x, &mut rows, &mut pooled);
                if actor == -1 {
                    *summary = pooled;
                }
                for (i, row) in selected.into_iter().zip(rows.chunks_exact(stride)) {
                    input[offset + i * stride..offset + (i + 1) * stride].copy_from_slice(row);
                }
            }
            return;
        }
        self.encode_board(board);
        let pooled = self.pooled();
        *summary = pooled;
        if let Some(coarse) = &mut self.coarse {
            coarse.encode_board(board);
        }
        let coarse_pool = self.coarse.as_ref().map(|c| c.pooled());
        input.reserve(features.len() * (FEATURES + spatial::ENCODED_FEATURES));
        for (features, tiles) in features.iter().zip(indices) {
            input.extend_from_slice(features);
            append(input, &self.embedding, &pooled, tiles, board.width, 1);
            if let Some(coarse) = &self.coarse {
                append(
                    input,
                    &coarse.embedding,
                    coarse_pool.as_ref().unwrap(),
                    tiles,
                    board.width,
                    2,
                );
            }
        }
    }
    fn encode_board(&mut self, board: &SpatialBoard) {
        let original_cells = board.height * board.width;
        let height = board.height.div_ceil(self.stride);
        let width = board.width.div_ceil(self.stride);
        let cells = height * width;
        if self.height != board.height
            || self.width != board.width
            || self.mask != board.values[..original_cells]
        {
            self.height = board.height;
            self.width = board.width;
            self.mask = board.values[..original_cells].to_vec();
            self.sites = (0..cells)
                .filter(|&site| {
                    let row = site / width * self.stride;
                    let col = site % width * self.stride;
                    (0..self.stride).any(|dr| {
                        (0..self.stride).any(|dq| {
                            row + dr < board.height
                                && col + dq < board.width
                                && board.values[(row + dr) * board.width + col + dq] == 1.
                        })
                    })
                })
                .collect();
            self.neighbors = self
                .sites
                .iter()
                .map(|&site| {
                    neighbors(
                        site / width * self.stride + usize::from(self.stride == 2),
                        site % width * self.stride,
                        board.height,
                        board.width,
                    )
                })
                .collect();
            self.next_neighbors = self
                .sites
                .iter()
                .map(|&site| neighbors(site / width, site % width, height, width))
                .collect();
            self.first = vec![0.; cells * EMBEDDING];
            self.embedding = vec![0.; cells * EMBEDDING];
        }
        let first = Kernel::<true> {
            layer: &self.layers[0],
            input: &board.values,
            output: &mut self.first,
            cells: original_cells,
            sites: &self.sites,
            neighbors: &self.neighbors,
        };
        #[cfg(not(target_arch = "wasm32"))]
        dispatch(self.arch, first);
        #[cfg(target_arch = "wasm32")]
        first.scalar();
        let second = Kernel::<false> {
            layer: &self.layers[1],
            input: &self.first,
            output: &mut self.embedding,
            cells,
            sites: &self.sites,
            neighbors: &self.next_neighbors,
        };
        #[cfg(not(target_arch = "wasm32"))]
        dispatch(self.arch, second);
        #[cfg(target_arch = "wasm32")]
        second.scalar();
    }
    fn pooled(&self) -> [f32; EMBEDDING] {
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
        pooled
    }
}

fn neighbors(row: usize, col: usize, height: usize, width: usize) -> [usize; 7] {
    KERNEL.map(|(dr, dq)| {
        let r = row as i32 + dr;
        let c = col as i32 + dq;
        if r < 0 || c < 0 || r >= height as i32 || c >= width as i32 {
            usize::MAX
        } else {
            r as usize * width + c as usize
        }
    })
}
fn append(
    input: &mut Vec<f32>,
    embedding: &[f32],
    pooled: &[f32; EMBEDDING],
    tiles: &[i64; 3],
    width: usize,
    stride: usize,
) {
    for &tile in tiles {
        if tile < 0 {
            input.extend_from_slice(&[0.; EMBEDDING]);
        } else {
            let site = (tile as usize / width / stride) * width.div_ceil(stride)
                + tile as usize % width / stride;
            input.extend_from_slice(&embedding[site * EMBEDDING..(site + 1) * EMBEDDING]);
        }
    }
    input.extend_from_slice(pooled);
}

#[cfg(not(target_arch = "wasm32"))]
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
    layer: &'a ConvolutionLayer,
    input: &'a [f32],
    output: &'a mut [f32],
    cells: usize,
    sites: &'a [usize],
    neighbors: &'a [[usize; 7]],
}

#[cfg(not(target_arch = "wasm32"))]
impl<const CHANNEL_MAJOR: bool> WithSimd for Kernel<'_, CHANNEL_MAJOR> {
    type Output = ();

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) {
        // Exact accumulator counts let LLVM keep them in registers instead of
        // clearing sixteen vectors even on the one-vector AVX-512 path.
        match size_of::<S::f32s>() {
            4 => self.run::<S, 16>(simd),
            8 => self.run::<S, 8>(simd),
            16 => self.run::<S, 4>(simd),
            32 => self.run::<S, 2>(simd),
            64 => self.run::<S, 1>(simd),
            _ => unreachable!("unsupported SIMD width"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<const CHANNEL_MAJOR: bool> Kernel<'_, CHANNEL_MAJOR> {
    #[inline(always)]
    fn run<S: Simd, const VECTORS: usize>(self, simd: S) {
        let lanes = size_of::<S::f32s>() / size_of::<f32>();
        assert_eq!(VECTORS * lanes, EMBEDDING);
        let zero = simd.splat_f32s(0.);
        let (bias, _) = S::as_simd_f32s(&self.layer.bias);
        for (&site, neighbors) in self.sites.iter().zip(self.neighbors) {
            let mut sums = [zero; VECTORS];
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
                    for (sum, &weight) in sums.iter_mut().zip(weights) {
                        // Separate multiply/add retain the scalar accumulation order.
                        *sum = simd.add_f32s(*sum, simd.mul_f32s(value, weight));
                    }
                }
            }
            let a = site * EMBEDDING;
            let (output, _) = S::as_mut_simd_f32s(&mut self.output[a..a + EMBEDDING]);
            for ((out, sum), &bias) in output.iter_mut().zip(&sums).zip(bias) {
                *out = simd.max_f32s(simd.add_f32s(*sum, bias), zero);
            }
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl<const CHANNEL_MAJOR: bool> Kernel<'_, CHANNEL_MAJOR> {
    fn scalar(self) {
        for (&site, neighbors) in self.sites.iter().zip(self.neighbors) {
            let mut sums = [0f32; EMBEDDING];
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
                    let offset = (channel * 7 + k) * EMBEDDING;
                    for (out, sum) in sums.iter_mut().enumerate() {
                        *sum += value * self.layer.weights[offset + out];
                    }
                }
            }
            for (out, sum) in sums.into_iter().enumerate() {
                self.output[site * EMBEDDING + out] = (sum + self.layer.bias[out]).max(0.);
            }
        }
    }
}

#[cfg(test)]
mod portable_tests {
    use super::*;
    fn compare<const CHANNEL_MAJOR: bool>(channels: usize) {
        let cells = 9;
        let layer = ConvolutionLayer {
            channels,
            weights: (0..channels * 7 * EMBEDDING)
                .map(|i| (i as f32 % 23. - 11.) / 37.)
                .collect(),
            bias: (0..EMBEDDING).map(|i| i as f32 / 100.).collect(),
        };
        let input: Vec<f32> = (0..cells * channels)
            .map(|i| (i as f32 % 17. - 8.) / 13.)
            .collect();
        let sites = vec![0, 1, 3, 4, 7, 8];
        let neighbors: Vec<_> = sites
            .iter()
            .map(|&i| super::neighbors(i / 3, i % 3, 3, 3))
            .collect();
        let mut scalar = vec![0.; cells * EMBEDDING];
        let mut native = scalar.clone();
        Kernel::<CHANNEL_MAJOR> {
            layer: &layer,
            input: &input,
            output: &mut scalar,
            cells,
            sites: &sites,
            neighbors: &neighbors,
        }
        .scalar();
        dispatch(
            pulp::Arch::new(),
            Kernel::<CHANNEL_MAJOR> {
                layer: &layer,
                input: &input,
                output: &mut native,
                cells,
                sites: &sites,
                neighbors: &neighbors,
            },
        );
        assert_eq!(scalar, native);
    }
    #[test]
    fn portable_convolution_matches_native_layouts_and_edges() {
        compare::<true>(3);
        compare::<false>(16);
    }
}
