//! Shared axial-grid observation and small seven-position hex convolutions.
use super::{FEATURE_NAMES, FEATURES, features};
use crate::{
    ai::{context::BattleView, orders::*},
    data::*,
};
use hex_command_battle::RuntimeCommand as Command;
use serde::{Deserialize, Serialize};

pub const SPATIAL_SCHEMA: u32 = 9;
pub const DUTY_CHANNELS: usize = 62;
pub const EMBEDDING: usize = 16;
pub const LEGACY_CHANNELS: usize = 53;
pub const CHANNELS: usize = LEGACY_CHANNELS + super::duties::CHANNEL_NAMES.len();
// (row, column) in axial coordinates q = col - floor(row/2), r = row.
pub const KERNEL: [(i32, i32); 7] = [(0, 0), (0, 1), (1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1)];
pub fn channel_names() -> Vec<String> {
    let mut names: Vec<String> = ["valid", "friendly", "hostile", "neutral"]
        .into_iter()
        .map(String::from)
        .collect();
    names.extend(
        FEATURE_NAMES[32..66]
            .iter()
            .map(|s| s.strip_prefix("actor_").unwrap().to_string()),
    );
    names.extend(
        [
            "terrain_cost",
            "terrain_passable",
            "terrain_healing",
            "keep",
            "friendly_owned",
            "enemy_owned",
            "terrain_melee_blade",
            "terrain_melee_pierce",
            "terrain_melee_impact",
            "terrain_ranged_blade",
            "terrain_ranged_pierce",
            "terrain_ranged_impact",
            "capture_goal",
            "defeat_goal",
            "visit_goal",
        ]
        .into_iter()
        .map(String::from),
    );
    names.extend(super::duties::CHANNEL_NAMES.into_iter().map(String::from));
    names
}
#[derive(Clone)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub values: Vec<f32>, // CHW; channel zero is the valid-tile mask.
}
#[derive(Clone)]
pub struct Spatial {
    pub board: Board,
    pub indices: Vec<[i64; 3]>, // actor, destination, target; -1 means absent.
}
pub fn observe(g: &BattleView<'_, '_>, orders: &TacticalOrders, commands: &[Command]) -> Spatial {
    let duties = super::duties::Duties::new(g, orders);
    observe_with_duties(g, orders, commands, &duties)
}
pub(super) fn observe_with_duties(
    g: &BattleView<'_, '_>,
    orders: &TacticalOrders,
    commands: &[Command],
    duties: &super::duties::Duties,
) -> Spatial {
    let axial: Vec<_> = g
        .board()
        .ids
        .iter()
        .map(|h| (h.row, h.col - h.row.div_euclid(2)))
        .collect();
    let min_r = axial.iter().map(|p| p.0).min().unwrap();
    let min_q = axial.iter().map(|p| p.1).min().unwrap();
    let height = (axial.iter().map(|p| p.0).max().unwrap() - min_r + 1) as usize;
    let width = (axial.iter().map(|p| p.1).max().unwrap() - min_q + 1) as usize;
    let cells = height * width;
    let sites: Vec<usize> = axial
        .iter()
        .map(|&(r, q)| (r - min_r) as usize * width + (q - min_q) as usize)
        .collect();
    let mut values = vec![0.; CHANNELS * cells];
    for (tile, &site) in sites.iter().enumerate() {
        values[site] = 1.;
        if let Some(u) = g.at(tile) {
            let friendly =
                g.faction_relation(g.observer_faction(), g.unit_faction(u)) == Relation::Allied;
            let hostile = g.hostile_to_faction(u, g.observer_faction());
            values[(if friendly {
                1
            } else if hostile {
                2
            } else {
                3
            }) * cells
                + site] = 1.;
            for (i, v) in features::unit_features(g, u).into_iter().enumerate() {
                values[(4 + i) * cells + site] = v;
            }
        }
        let (cost, passable, modifiers) = g.base_terrain_properties(tile);
        let terrain_values = [
            cost as f32 / 10.,
            passable as u8 as f32,
            g.terrain_healing(tile) as f32 / 10.,
            g.is_keep(tile) as u8 as f32,
            g.owns(tile, g.observer_faction()) as u8 as f32,
            g.owners()[tile].as_deref().is_some_and(|owner| {
                g.faction_relation(g.observer_faction(), owner) == Relation::Hostile
            }) as u8 as f32,
            modifiers[0] as f32 / 10.,
            modifiers[1] as f32 / 10.,
            modifiers[2] as f32 / 10.,
            modifiers[3] as f32 / 10.,
            modifiers[4] as f32 / 10.,
            modifiers[5] as f32 / 10.,
        ];
        for (i, v) in terrain_values.into_iter().enumerate() {
            values[(38 + i) * cells + site] = v;
        }
    }
    for goal in &orders.targets {
        let channel = match goal.kind {
            TargetKind::Capture => 50,
            TargetKind::Defeat => 51,
            TargetKind::Visit => 52,
        };
        for hex in &goal.tiles {
            if let Some(&tile) = g.board().indices.get(hex) {
                let value = &mut values[channel * cells + sites[tile]];
                *value = value.max(goal.weight as f32 / 10.);
            }
        }
    }
    duties.paint(g, orders, &sites, &mut values, cells);
    let indices = commands
        .iter()
        .map(|command| {
            let (id, dest, target) = match command {
                Command::Move { unit, tile } => (unit.0, Some(tile.0), None),
                Command::Attack { unit, target, .. } => {
                    (unit.0, None, g.unit(target.0).map(|u| u.tile))
                }
                Command::Advance { unit, .. } => (unit.0, None, None),
                _ => return [-1; 3],
            };
            let tile = g.unit(id).unwrap().tile;
            [
                sites[tile] as i64,
                sites[dest.unwrap_or(tile)] as i64,
                target.map_or(-1, |t| sites[t] as i64),
            ]
        })
        .collect();
    Spatial {
        board: Board {
            height,
            width,
            values,
        },
        indices,
    }
}
impl Board {
    pub fn validate(&self) -> Result<(), String> {
        let cells = self
            .height
            .checked_mul(self.width)
            .ok_or("board dimensions overflow")?;
        if cells == 0
            || cells > 16384
            || self.values.len() != cells * CHANNELS
            || self.values.iter().any(|v| !v.is_finite())
        {
            return Err("invalid spatial board dimensions or values".into());
        }
        if self.values[..cells].iter().any(|&v| v != 0. && v != 1.)
            || !self.values[..cells].contains(&1.)
        {
            return Err("invalid spatial board mask".into());
        }
        for channel in self.values.chunks_exact(cells) {
            if channel
                .iter()
                .zip(&self.values[..cells])
                .any(|(&x, &mask)| mask == 0. && x != 0.)
            {
                return Err("nonzero board padding".into());
            }
        }
        Ok(())
    }
    pub fn validate_indices(&self, indices: &[[i64; 3]]) -> Result<(), String> {
        let cells = self.height * self.width;
        if indices
            .iter()
            .flatten()
            .any(|&i| i < -1 || (i >= 0 && (i as usize >= cells || self.values[i as usize] != 1.)))
        {
            return Err("invalid spatial tile index".into());
        }
        Ok(())
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Convolution {
    pub input_channels: usize,
    pub output_channels: usize,
    pub weights: Vec<f32>, // output, input, kernel-position
    pub bias: Vec<f32>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Encoder {
    pub channel_names: Vec<String>,
    pub convolutions: Vec<Convolution>,
}
impl Encoder {
    pub fn validate(&self) -> Result<(), String> {
        if self.channel_names != channel_names() || self.convolutions.len() != 2 {
            return Err("spatial encoder schema mismatch".into());
        }
        let mut input = CHANNELS;
        for c in &self.convolutions {
            if c.input_channels != input
                || c.output_channels != EMBEDDING
                || c.weights.len() != input * EMBEDDING * 7
                || c.bias.len() != EMBEDDING
                || c.weights.iter().chain(&c.bias).any(|v| !v.is_finite())
            {
                return Err("invalid spatial convolution".into());
            }
            input = EMBEDDING;
        }
        Ok(())
    }
    pub fn encode(&self, board: &Board) -> Vec<f32> {
        let cells = board.height * board.width;
        let sites: Vec<_> = (0..cells).filter(|&i| board.values[i] == 1.).collect();
        let neighbors: Vec<[Option<usize>; 7]> = sites
            .iter()
            .map(|&site| {
                KERNEL.map(|(dr, dq)| {
                    let r = (site / board.width) as i32 + dr;
                    let q = (site % board.width) as i32 + dq;
                    if r < 0 || q < 0 || r >= board.height as i32 || q >= board.width as i32 {
                        None
                    } else {
                        Some(r as usize * board.width + q as usize)
                    }
                })
            })
            .collect();
        let mut input = board.values.clone();
        for conv in &self.convolutions {
            let mut output = vec![0.; EMBEDDING * cells];
            for out in 0..EMBEDDING {
                for (&site, neighbors) in sites.iter().zip(&neighbors) {
                    let mut sum = 0.;
                    for channel in 0..conv.input_channels {
                        for (k, neighbor) in neighbors.iter().enumerate() {
                            if let Some(n) = neighbor {
                                sum += input[channel * cells + n]
                                    * conv.weights[(out * conv.input_channels + channel) * 7 + k];
                            }
                        }
                    }
                    output[out * cells + site] = (sum + conv.bias[out]).max(0.);
                }
            }
            input = output;
        }
        input
    }
    pub fn inputs(
        &self,
        board: &Board,
        indices: &[[i64; 3]],
        features: &[[f32; FEATURES]],
    ) -> Vec<Vec<f32>> {
        let embedding = self.encode(board);
        let cells = board.height * board.width;
        let count = board.values[..cells].iter().sum::<f32>();
        let pooled: Vec<_> = embedding
            .chunks_exact(cells)
            .map(|plane| plane.iter().sum::<f32>() / count)
            .collect();
        features
            .iter()
            .zip(indices)
            .map(|(features, tiles)| {
                let mut x = features.to_vec();
                for &tile in tiles {
                    for c in 0..EMBEDDING {
                        x.push(if tile < 0 {
                            0.
                        } else {
                            embedding[c * cells + tile as usize]
                        });
                    }
                }
                x.extend_from_slice(&pooled);
                x
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axial_kernel_matches_engine_adjacency_and_candidate_tiles() {
        let rules = Rules::load().unwrap();
        let game = crate::logic::Game::new(&rules, "mirror");
        let view = game.view();
        let commands = super::super::candidates(&view, &TacticalOrders::default()).commands;
        let spatial = observe(&view, &TacticalOrders::default(), &commands);
        spatial.board.validate().unwrap();
        spatial.board.validate_indices(&spatial.indices).unwrap();
        assert_eq!(channel_names().len(), CHANNELS);
        let axial: Vec<_> = view
            .board()
            .ids
            .iter()
            .map(|h| (h.row, h.col - h.row.div_euclid(2)))
            .collect();
        for (tile, &(r, q)) in axial.iter().enumerate() {
            let mut expected = view.board().adjacent[tile].clone();
            expected.sort_unstable();
            let mut found: Vec<_> = KERNEL[1..]
                .iter()
                .filter_map(|&(dr, dq)| axial.iter().position(|&(a, b)| a == r + dr && b == q + dq))
                .collect();
            found.sort_unstable();
            assert_eq!(found, expected);
        }
        assert_eq!(spatial.indices[0], [-1; 3]);
    }
}
