//! PyTorch GRUCell layout: reset, update, candidate; reset follows hidden bias.
use serde::{Deserialize, Serialize};
pub const MEMORY: usize = 64;
#[derive(Clone, Deserialize, Serialize)]
pub struct RecurrentLayer {
    pub weight_ih: Vec<f32>,
    pub weight_hh: Vec<f32>,
    pub bias_ih: Vec<f32>,
    pub bias_hh: Vec<f32>,
}
impl RecurrentLayer {
    pub(super) fn initialized() -> Self {
        let mut seed = 732u64;
        let mut values = |count| {
            (0..count)
                .map(|_| {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    ((seed >> 32) as u32 as f32 / u32::MAX as f32 * 2. - 1.) / 8.
                })
                .collect()
        };
        Self {
            weight_ih: values(3 * MEMORY * 16),
            weight_hh: values(3 * MEMORY * MEMORY),
            bias_ih: values(3 * MEMORY),
            bias_hh: values(3 * MEMORY),
        }
    }
    pub(super) fn validate(&self) -> Result<(), String> {
        for (v, n) in [
            (&self.weight_ih, 3 * MEMORY * 16),
            (&self.weight_hh, 3 * MEMORY * MEMORY),
            (&self.bias_ih, 3 * MEMORY),
            (&self.bias_hh, 3 * MEMORY),
        ] {
            if v.len() != n || v.iter().any(|v| !v.is_finite()) {
                return Err("invalid GRU weights".into());
            }
        }
        Ok(())
    }
    pub fn step(&self, input: &[f32; 16], previous: &[f32; MEMORY]) -> [f32; MEMORY] {
        let mut x = [0.; 3 * MEMORY];
        let mut h = [0.; 3 * MEMORY];
        for i in 0..3 * MEMORY {
            x[i] = self.bias_ih[i]
                + input
                    .iter()
                    .zip(&self.weight_ih[i * 16..(i + 1) * 16])
                    .map(|(a, b)| a * b)
                    .sum::<f32>();
            h[i] = self.bias_hh[i]
                + previous
                    .iter()
                    .zip(&self.weight_hh[i * MEMORY..(i + 1) * MEMORY])
                    .map(|(a, b)| a * b)
                    .sum::<f32>();
        }
        std::array::from_fn(|i| {
            let r = 1. / (1. + (-(x[i] + h[i])).exp());
            let z = 1. / (1. + (-(x[MEMORY + i] + h[MEMORY + i])).exp());
            let n = (x[2 * MEMORY + i] + r * h[2 * MEMORY + i]).tanh();
            (1. - z) * n + z * previous[i]
        })
    }
}
