//! Shared axial-grid observation and small seven-position hex convolutions.
use super::{FEATURE_NAMES, FEATURES, features};
use crate::{
    ai::{context::BattleView, orders::*},
    data::*,
};
use hex_command_battle::RuntimeCommand;
use serde::{Deserialize, Serialize};

pub const SPATIAL_SCHEMA: u32 = 19;
pub const ENCODED_FEATURES: usize = 8 * EMBEDDING;
pub const DUTY_CHANNELS: usize = 62;
pub const EMBEDDING: usize = 16;
pub const LEGACY_CHANNELS: usize = 53;
pub const CAMPAIGN_CHANNELS: usize = LEGACY_CHANNELS + super::duties::CHANNEL_NAMES.len();
pub const VISIBILITY_CHANNELS: usize = CAMPAIGN_CHANNELS + super::visibility::CHANNEL_NAMES.len();
pub const HOLD_CHANNELS: usize = VISIBILITY_CHANNELS + super::duties::HOLD_CHANNEL_NAMES.len();
pub const CHANNELS: usize = HOLD_CHANNELS + super::duties::OPPONENT_HOLD_CHANNEL_NAMES.len();
pub const ACTOR_CHANNELS: [usize; 14] = [68, 69, 70, 76, 78, 38, 39, 40, 44, 45, 46, 47, 48, 49];
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
    names.extend(
        super::visibility::CHANNEL_NAMES
            .into_iter()
            .map(String::from),
    );
    names.extend(
        super::duties::HOLD_CHANNEL_NAMES
            .into_iter()
            .map(String::from),
    );
    names.extend(
        super::duties::OPPONENT_HOLD_CHANNEL_NAMES
            .into_iter()
            .map(String::from),
    );
    names
}
#[derive(Clone)]
pub struct ActorContext {
    pub actor_site: usize,
    pub values: Vec<f32>, // Actor-relative planes, in ACTOR_CHANNELS order.
}
#[derive(Clone)]
pub struct SpatialBoard {
    pub height: usize,
    pub width: usize,
    pub contexts: Vec<ActorContext>,
    pub values: Vec<f32>, // CHW; channel zero is the valid-tile mask.
}
#[derive(Clone)]
pub struct Spatial {
    pub board: SpatialBoard,
    pub indices: Vec<[i64; 3]>, // actor, destination, target; -1 means absent.
}
pub fn observe(
    g: &BattleView<'_, '_>,
    orders: &TacticalOrders,
    commands: &[RuntimeCommand],
) -> Spatial {
    let duties = super::duties::Duties::new(g, orders);
    observe_with_duties(g, orders, commands, &duties)
}
pub(super) fn observe_with_duties(
    g: &BattleView<'_, '_>,
    orders: &TacticalOrders,
    commands: &[RuntimeCommand],
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
        let at = g.at(tile);
        let cover = at.map(|u| g.concealment_at(u, tile)).unwrap_or_default();
        let sight = at.and_then(|u| g.sight_range(u));
        for (channel, value) in [
            (65, f32::from(g.terrain_remembered(tile))),
            (66, f32::from(g.tile_in_sight(tile))),
            (67, f32::from(g.possible_enemy_hide_place(tile))),
            (71, f32::from(g.tile_insight_of_enemy(tile))),
            (72, sight.map_or(0., |r| r as f32 / 20.)),
            (73, f32::from(at.is_some() && sight.is_none())),
            (74, f32::from(cover.available)),
            (75, cover.reveal_distance as f32 / 20.),
            (
                77,
                at.map_or(0., |u| g.friendly_detector_count(u) as f32 / 20.),
            ),
        ] {
            values[channel * cells + site] = value;
        }
        for (channel, value) in [
            (41, g.is_keep(tile)),
            (42, g.owns(tile, g.observer_faction())),
            (
                43,
                g.owners()[tile].as_deref().is_some_and(|owner| {
                    g.faction_relation(g.observer_faction(), owner) == Relation::Hostile
                }),
            ),
        ] {
            values[channel * cells + site] = f32::from(value);
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
    let mut contexts = Vec::new();
    let mut actors = std::collections::BTreeSet::new();
    for c in commands {
        let id = match c {
            RuntimeCommand::Move { unit, .. }
            | RuntimeCommand::Attack { unit, .. }
            | RuntimeCommand::Advance { unit, .. } => unit.0,
            _ => continue,
        };
        if !actors.insert(id) {
            continue;
        }
        let actor = g.unit(id).unwrap();
        let mut planes = vec![0.; ACTOR_CHANNELS.len() * cells];
        for (tile, &site) in sites.iter().enumerate() {
            let cost = g.step_cost(actor, tile);
            planes[5 * cells + site] = if cost == u16::MAX {
                -1.
            } else {
                f32::from(cost) / 10.
            };
            planes[6 * cells + site] = f32::from(cost != u16::MAX);
            planes[7 * cells + site] = g.terrain_healing_at(actor, tile) as f32 / 10.;
            g.with_capabilities(actor, tile, |caps| {
                for (index, (category, kind)) in [Category::Melee, Category::Ranged]
                    .into_iter()
                    .flat_map(|category| {
                        [DamageType::Blade, DamageType::Pierce, DamageType::Impact]
                            .map(move |kind| (category, kind))
                    })
                    .enumerate()
                {
                    planes[(8 + index) * cells + site] = caps
                        .neural_unit_features(g.terrain_definition(tile), category, kind)
                        .terrain as f32
                        / 10.;
                }
            });
            let cover = g.concealment_at(actor, tile);
            planes[site] = f32::from(cover.available);
            planes[cells + site] = cover.reveal_distance as f32 / 20.;
            planes[2 * cells + site] = f32::from(g.can_observe_terrain(actor, tile));
            if let Some(unit) = g.at(tile) {
                planes[3 * cells + site] = f32::from(g.directly_detects(actor, unit));
                planes[4 * cells + site] = f32::from(g.directly_detects(unit, actor));
            }
        }
        contexts.push(ActorContext {
            actor_site: sites[actor.tile],
            values: planes,
        });
    }
    let indices = commands
        .iter()
        .map(|command| {
            let (id, dest, target) = match command {
                RuntimeCommand::Move { unit, tile } => (unit.0, Some(tile.0), None),
                RuntimeCommand::Attack { unit, target, .. } => {
                    (unit.0, None, g.unit(target.0).map(|u| u.tile))
                }
                RuntimeCommand::Advance { unit, .. } => (unit.0, None, None),
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
        board: SpatialBoard {
            height,
            width,
            values,
            contexts,
        },
        indices,
    }
}
impl SpatialBoard {
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
        let mut actors = std::collections::BTreeSet::new();
        for context in &self.contexts {
            if context.actor_site >= cells
                || self.values[context.actor_site] != 1.
                || !actors.insert(context.actor_site)
                || context.values.len() != ACTOR_CHANNELS.len() * cells
                || context.values.iter().any(|v| !v.is_finite())
                || context.values.chunks_exact(cells).any(|plane| {
                    plane
                        .iter()
                        .zip(&self.values[..cells])
                        .any(|(&v, &m)| m == 0. && v != 0.)
                })
            {
                return Err("invalid actor spatial context".into());
            }
        }
        Ok(())
    }
    pub fn with_context(&self, context: &ActorContext) -> Self {
        let cells = self.height * self.width;
        let mut board = Self {
            height: self.height,
            width: self.width,
            values: self.values.clone(),
            contexts: vec![],
        };
        for (plane, &channel) in ACTOR_CHANNELS.iter().enumerate() {
            board.values[channel * cells..(channel + 1) * cells]
                .copy_from_slice(&context.values[plane * cells..(plane + 1) * cells]);
        }
        board
    }
    pub fn validate_indices(&self, indices: &[[i64; 3]]) -> Result<(), String> {
        let cells = self.height * self.width;
        if !self.contexts.is_empty()
            && indices
                .iter()
                .any(|ix| ix[0] >= 0 && !self.contexts.iter().any(|c| c.actor_site as i64 == ix[0]))
        {
            return Err("missing actor spatial context".into());
        }
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
pub struct SpatialEncoder {
    pub channel_names: Vec<String>,
    pub convolutions: Vec<Convolution>,
}
impl SpatialEncoder {
    pub fn validate(&self) -> Result<(), String> {
        if self.channel_names != channel_names() || self.convolutions.len() != 4 {
            return Err("spatial encoder schema mismatch".into());
        }
        for (i, c) in self.convolutions.iter().enumerate() {
            let input = if i % 2 == 0 { CHANNELS } else { EMBEDDING };
            if c.input_channels != input
                || c.output_channels != EMBEDDING
                || c.weights.len() != input * EMBEDDING * 7
                || c.bias.len() != EMBEDDING
                || c.weights.iter().chain(&c.bias).any(|v| !v.is_finite())
            {
                return Err("invalid spatial convolution".into());
            }
        }
        Ok(())
    }
    pub fn encode(&self, board: &SpatialBoard) -> Vec<f32> {
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
        for conv in &self.convolutions[..2] {
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
    fn coarse(&self, board: &SpatialBoard) -> (Vec<f32>, Vec<f32>, usize) {
        let height = board.height.div_ceil(2);
        let width = board.width.div_ceil(2);
        let cells = height * width;
        let original = board.height * board.width;
        let mask: Vec<f32> = (0..cells)
            .map(|site| {
                let row = (site / width) * 2;
                let col = (site % width) * 2;
                (0..2).any(|dr| {
                    (0..2).any(|dq| {
                        row + dr < board.height
                            && col + dq < board.width
                            && board.values[(row + dr) * board.width + col + dq] == 1.
                    })
                }) as u8 as f32
            })
            .collect();
        let mut input = board.values.clone();
        for (i, conv) in self.convolutions[2..].iter().enumerate() {
            let stride = if i == 0 { 2 } else { 1 };
            let (ih, iw, ic) = if i == 0 {
                (board.height, board.width, original)
            } else {
                (height, width, cells)
            };
            let mut output = vec![0.; EMBEDDING * cells];
            for out in 0..EMBEDDING {
                for site in 0..cells {
                    if mask[site] == 0. {
                        continue;
                    }
                    let mut sum = 0.;
                    for channel in 0..conv.input_channels {
                        for (k, (dr, dq)) in KERNEL.iter().enumerate() {
                            let row = (site / width * stride + usize::from(i == 0)) as i32 + dr;
                            let col = (site % width * stride) as i32 + dq;
                            if row >= 0 && col >= 0 && row < ih as i32 && col < iw as i32 {
                                sum += input[channel * ic + row as usize * iw + col as usize]
                                    * conv.weights[(out * conv.input_channels + channel) * 7 + k];
                            }
                        }
                    }
                    output[out * cells + site] = (sum + conv.bias[out]).max(0.);
                }
            }
            input = output;
        }
        (input, mask, width)
    }
    pub fn inputs(
        &self,
        board: &SpatialBoard,
        indices: &[[i64; 3]],
        features: &[[f32; FEATURES]],
    ) -> Vec<Vec<f32>> {
        if !board.contexts.is_empty() {
            let mut inputs = vec![Vec::new(); features.len()];
            for actor in
                std::iter::once(-1).chain(board.contexts.iter().map(|c| c.actor_site as i64))
            {
                let selected: Vec<_> = indices
                    .iter()
                    .enumerate()
                    .filter(|(_, ix)| ix[0] == actor)
                    .map(|(i, _)| i)
                    .collect();
                if selected.is_empty() {
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
                for (i, row) in selected.into_iter().zip(self.inputs(&plain, &ix, &x)) {
                    inputs[i] = row;
                }
            }
            return inputs;
        }
        let embedding = self.encode(board);
        let (coarse, coarse_mask, coarse_width) = self.coarse(board);
        let coarse_cells = coarse_mask.len();
        let coarse_count: f32 = coarse_mask.iter().sum();
        let coarse_pool: Vec<_> = coarse
            .chunks_exact(coarse_cells)
            .map(|p| p.iter().sum::<f32>() / coarse_count)
            .collect();
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
                for &tile in tiles {
                    let site = if tile < 0 {
                        0
                    } else {
                        (tile as usize / board.width / 2) * coarse_width
                            + tile as usize % board.width / 2
                    };
                    for c in 0..EMBEDDING {
                        x.push(if tile < 0 {
                            0.
                        } else {
                            coarse[c * coarse_cells + site]
                        });
                    }
                }
                x.extend_from_slice(&coarse_pool);
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
        let rules = Catalogue::load().unwrap();
        let game = crate::logic::BattleSession::new(&rules, "mirror");
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
