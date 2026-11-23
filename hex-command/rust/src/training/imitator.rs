//! Explicit pretraining-only reference policies; never used by PPO.
use super::{
    Choices, DecisionController,
    spatial::{ACTOR_CHANNELS, KERNEL},
};
use crate::{data::*, simulation::MatchConfig};
fn path_controller(goal: usize) -> DecisionController {
    Box::new(move |c: &Choices| {
        let b = &c.spatial.as_ref().unwrap().board;
        let cells = b.height * b.width;
        let mut distances = std::collections::HashMap::<i64, Vec<f32>>::new();
        for ix in &c.spatial.as_ref().unwrap().indices {
            let actor = ix[0];
            if actor < 0 || distances.contains_key(&actor) {
                continue;
            }
            let ctx = b
                .contexts
                .iter()
                .find(|x| x.actor_site == actor as usize)
                .unwrap();
            let cost_index = ACTOR_CHANNELS.iter().position(|&x| x == 38).unwrap();
            let pass_index = ACTOR_CHANNELS.iter().position(|&x| x == 39).unwrap();
            let mut d = vec![f32::INFINITY; cells];
            let mut used = vec![false; cells];
            d[goal] = 0.;
            for _ in 0..cells {
                let Some(at) = (0..cells)
                    .filter(|&p| !used[p] && d[p].is_finite())
                    .min_by(|&a, &bb| d[a].total_cmp(&d[bb]))
                else {
                    break;
                };
                used[at] = true;
                let r = (at / b.width) as i32;
                let q = (at % b.width) as i32;
                let price = ctx.values[cost_index * cells + at];
                for &(dr, dq) in KERNEL.iter().skip(1) {
                    let rr = r + dr;
                    let qq = q + dq;
                    if rr < 0 || qq < 0 || rr >= b.height as i32 || qq >= b.width as i32 {
                        continue;
                    }
                    let p = rr as usize * b.width + qq as usize;
                    if b.values[p] < 0.5 || ctx.values[pass_index * cells + p] < 0.5 {
                        continue;
                    }
                    let next = d[at] + price.max(0.01);
                    if next < d[p] {
                        d[p] = next;
                    }
                }
            }
            distances.insert(actor, d);
        }
        let mut best = (0, 0.0f32);
        for (i, f) in c.features.iter().enumerate().skip(1) {
            if f[10] > 0.5 {
                let actor = c.spatial.as_ref().unwrap().indices[i][0];
                if actor >= 0
                    && b.values[41 * cells + actor as usize] > 0.5
                    && b.values[42 * cells + actor as usize] > 0.5
                {
                    continue;
                }
                // Rebuild from a friendly settlement rather than exposing the only commander.
                if f[1] > 0.5 && f[142] > 0.5 && f[143] > 0.5 {
                    let score = 3.0 - f[22];
                    if score > best.1 {
                        best = (i, score);
                    }
                }
                continue;
            }
            let ix = c.spatial.as_ref().unwrap().indices[i];
            let score = if f[3] > 0.5 {
                80.
            } else if f[2] > 0.5 && f[26] > 0.0 && f[27] * 30.0 < f[9] * 30.0 {
                50. + 30. * (f[26] - 0.35 * f[27]) + 20. * f[28]
            } else if f[1] > 0.5 && ix[0] >= 0 && ix[1] >= 0 {
                let d = &distances[&ix[0]];
                let from = d[ix[0] as usize];
                let to = d[ix[1] as usize];
                if from.is_finite() && to.is_finite() && to + 0.0001 < from {
                    10. + 50. * (from - to) - to + 0.1 * f[8]
                } else {
                    0.
                }
            } else {
                0.
            };
            if score > best.1 {
                best = (i, score);
            }
        }
        Some(best.0)
    })
}

/// Named offline teachers operate only on the same legal, observed choices as the learner.
pub fn controller(
    spec: &PretrainImitator,
    rules: &Catalogue,
    config: &MatchConfig,
) -> Result<Option<DecisionController>, String> {
    if !spec.valid() {
        return Err("unknown pretrain_imitator".into());
    }
    match spec {
        PretrainImitator::Named(name) if name == "objective_progress" => {
            Ok(Some(Box::new(|c: &Choices| {
                let mut best = (0, 0.0f32);
                for (i, f) in c.features.iter().enumerate().skip(1) {
                    if f[10] > 0.5 {
                        continue;
                    }
                    let score = if f[3] > 0.5 {
                        80.0
                    } else if f[2] > 0.5 && f[26] > f[27] {
                        50.0 + 30.0 * (f[26] - f[27]) + 20.0 * f[28]
                    } else if f[1] > 0.5 && f[157] > 0.5 && f[156] > 0.0001 {
                        10.0 + 50.0 * f[156] - f[154]
                    } else {
                        0.0
                    };
                    if score > best.1 {
                        best = (i, score);
                    }
                }
                Some(best.0)
            })))
        }
        PretrainImitator::Named(_) | PretrainImitator::Blocks { .. } => Ok(None),
        PretrainImitator::Path { path_to } => {
            let MatchConfig::Campaign(c) = config else {
                return Err("path_to requires a campaign".into());
            };
            let chapters = rules.campaign(&c.campaign).chapters();
            if chapters.len() != 1 {
                return Err("path_to requires a single-chapter campaign".into());
            }
            let chapter = &chapters[0];
            let map = rules.map(chapter.map_id(), chapter.map_revision());
            if !map
                .tiles()
                .iter()
                .any(|t| t.col == path_to.col && t.row == path_to.row)
            {
                return Err("path_to tile absent".into());
            }
            let min_r = map.tiles().iter().map(|t| t.row).min().unwrap();
            let min_q = map
                .tiles()
                .iter()
                .map(|t| t.col - t.row.div_euclid(2))
                .min()
                .unwrap();
            let max_q = map
                .tiles()
                .iter()
                .map(|t| t.col - t.row.div_euclid(2))
                .max()
                .unwrap();
            let goal = (path_to.row - min_r) as usize * (max_q - min_q + 1) as usize
                + (path_to.col - path_to.row.div_euclid(2) - min_q) as usize;
            Ok(Some(path_controller(goal)))
        }
    }
}
