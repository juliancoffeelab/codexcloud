//! Offline role teachers. No map IDs, hidden opponents, RNG or persistent
//! completion flags: Choices can describe tentative branches of a turn plan.
use super::*;
use hex_command_battle::RuntimeCommand;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

fn unit(command: &RuntimeCommand) -> Option<u32> {
    match command {
        RuntimeCommand::Move { unit, .. }
        | RuntimeCommand::Attack { unit, .. }
        | RuntimeCommand::Advance { unit, .. } => Some(unit.0),
        _ => None,
    }
}
fn distance(a: usize, b: usize, width: usize) -> f32 {
    let (ar, aq) = ((a / width) as i32, (a % width) as i32);
    let (br, bq) = ((b / width) as i32, (b % width) as i32);
    ((ar - br).abs() + (aq - bq).abs() + (ar + aq - br - bq).abs()) as f32 / 2.
}
fn route(c: &Choices, actor: usize, goal: usize, risk: &[f32], alpha: f32) -> Vec<f32> {
    let b = &c.spatial.as_ref().unwrap().board;
    let n = b.width * b.height;
    let ctx = b.contexts.iter().find(|x| x.actor_site == actor).unwrap();
    let cost = ACTOR_CHANNELS.iter().position(|&x| x == 38).unwrap();
    let pass = ACTOR_CHANNELS.iter().position(|&x| x == 39).unwrap();
    let mut d = vec![f32::INFINITY; n];
    let mut queue = BinaryHeap::new();
    d[goal] = 0.;
    queue.push(Reverse((0u32, goal)));
    while let Some(Reverse((price, at))) = queue.pop() {
        if price as f32 > d[at] * 100. + 1. {
            continue;
        }
        let edge =
            ctx.values[cost * n + at].max(0.1) + alpha * if at == goal { 0. } else { risk[at] };
        let (r, q) = ((at / b.width) as i32, (at % b.width) as i32);
        for &(dr, dq) in KERNEL.iter().skip(1) {
            let (rr, qq) = (r + dr, q + dq);
            if rr < 0 || qq < 0 || rr >= b.height as i32 || qq >= b.width as i32 {
                continue;
            }
            let p = rr as usize * b.width + qq as usize;
            if b.values[p] < 0.5 || ctx.values[pass * n + p] < 0.5 {
                continue;
            }
            let next = d[at] + edge;
            if next + 0.0001 < d[p] {
                d[p] = next;
                queue.push(Reverse(((next * 100.).round() as u32, p)));
            }
        }
    }
    d
}
pub(super) fn controller(
    spec: &TacticalTeacher,
    rules: &Catalogue,
    config: &MatchConfig,
) -> Result<DecisionController, String> {
    let MatchConfig::Campaign(config) = config else {
        return Err("tactical teacher requires a campaign".into());
    };
    let chapters = rules.campaign(&config.campaign).chapters();
    if chapters.len() != 1 {
        return Err("tactical teacher requires a single chapter".into());
    }
    let chapter = &chapters[0];
    let map = rules.map(chapter.map_id(), chapter.map_revision());
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
    let site = |hex: &Hex| -> Result<usize, String> {
        if !map
            .tiles()
            .iter()
            .any(|t| t.col == hex.col && t.row == hex.row)
        {
            return Err("tactical teacher tile absent".into());
        }
        Ok((hex.row - min_r) as usize * (max_q - min_q + 1) as usize
            + (hex.col - hex.row.div_euclid(2) - min_q) as usize)
    };
    let goal = site(&spec.goal)?;
    let home = site(&spec.home)?;
    let leader = spec.leader;
    let (staging, healer_staging) = match &spec.plan {
        TacticalTeacherPlan::Refuge { staging, .. } => (site(staging)?, home),
        TacticalTeacherPlan::Relief { staging, .. } => (site(staging)?, home),
        TacticalTeacherPlan::Recovery {
            retreat,
            healer_staging,
            ..
        } => (site(retreat)?, site(healer_staging)?),
    };
    let plan = spec.plan.clone();
    Ok(Box::new(move |c: &Choices| {
        let b = &c.spatial.as_ref().unwrap().board;
        let n = b.width * b.height;
        let mut sites = HashMap::new();
        let mut actors = HashMap::new();
        for (i, command) in c.commands.iter().enumerate() {
            if let Some(id) = unit(command) {
                let at = c.spatial.as_ref().unwrap().indices[i][0];
                if at >= 0 && b.values[n + at as usize] > 0.5 {
                    sites.insert(id, at as usize);
                    actors.entry(id).or_insert(i);
                }
            }
        }
        let foes: Vec<_> = (0..n).filter(|&p| b.values[2 * n + p] > 0.5).collect();
        let marked: Vec<_> = foes
            .iter()
            .copied()
            .filter(|&p| b.values[51 * n + p] > 0.5)
            .collect();
        let mut risk = vec![0.; n];
        for &enemy in &foes {
            let melee = b.values[18 * n + enemy] * 10. * b.values[19 * n + enemy] * 5.;
            let ranged = b.values[26 * n + enemy] * 10. * b.values[27 * n + enemy] * 5.;
            let reach = (b.values[20 * n + enemy] * 5.)
                .max(b.values[28 * n + enemy] * 5.)
                .max(1.);
            let movement = b.values[6 * n + enemy] * 10.;
            for (p, threat) in risk.iter_mut().enumerate() {
                let d = distance(p, enemy, b.width);
                if d <= reach + movement {
                    *threat += melee.max(ranged) * if d <= reach { 1. } else { 0.5 };
                }
            }
        }
        let recovery = match &plan {
            TacticalTeacherPlan::Recovery {
                protected,
                healer,
                recovery_fraction,
                ..
            } => Some((*protected, *healer, *recovery_fraction)),
            _ => None,
        };
        let wounded = recovery.is_some_and(|(id, _, fraction)| {
            sites
                .get(&id)
                .is_some_and(|&p| b.values[34 * n + p] < fraction)
        });
        let holder = match &plan {
            TacticalTeacherPlan::Refuge { holder, .. } => Some(*holder),
            _ => None,
        };
        let holder_site = holder
            .and_then(|id| sites.get(&id).copied())
            .or_else(|| {
                (0..n).find(|&p| {
                    b.values[n + p] > 0.5
                        && (b.values[55 * n + p] > 0.5 || b.values[59 * n + p] > 0.5)
                })
            })
            .unwrap_or(staging);
        let ready = (0..n)
            .filter(|&p| {
                b.values[n + p] > 0.5
                    && p != holder_site
                    && b.values[37 * n + p] < 0.5
                    && b.values[34 * n + p] > 0.6
                    && distance(p, goal, b.width) <= 3.
            })
            .count()
            >= 3
            && foes.iter().all(|&e| distance(e, holder_site, b.width) > 5.)
            && foes
                .iter()
                .filter(|&&e| distance(e, goal, b.width) <= 4.)
                .count()
                <= 1;
        let mut goals = HashMap::new();
        let mut routes = HashMap::new();
        for &id in actors.keys() {
            let at = sites[&id];
            let mut target = goal;
            let mut alpha = 0.;
            if id == leader {
                target = home;
            } else {
                match &plan {
                    TacticalTeacherPlan::Refuge { holder, .. } => {
                        alpha = 0.02;
                        target = if id == *holder {
                            if ready || at == goal { goal } else { staging }
                        } else {
                            foes.iter()
                                .min_by_key(|&&e| distance(e, holder_site, b.width) as u32)
                                .copied()
                                .unwrap_or(goal)
                        };
                    }
                    TacticalTeacherPlan::Relief {
                        advance_round,
                        risk_cost,
                        ..
                    } => {
                        if c.context.round < *advance_round {
                            target = staging;
                        }
                        alpha = *risk_cost;
                    }
                    TacticalTeacherPlan::Recovery {
                        protected, healer, ..
                    } => {
                        if id == *protected && wounded {
                            target = staging;
                        } else if id == *healer {
                            target = if wounded {
                                healer_staging
                            } else {
                                sites.get(protected).copied().unwrap_or(staging)
                            };
                        } else {
                            target = (if marked.is_empty() { &foes } else { &marked })
                                .iter()
                                .min_by_key(|&&p| distance(at, p, b.width) as u32)
                                .copied()
                                .unwrap_or(goal);
                        }
                    }
                }
            }
            // Relief also uses the risk-aware route for the commander.
            if let TacticalTeacherPlan::Relief { risk_cost, .. } = &plan {
                alpha = *risk_cost;
            }
            if holder.is_some() {
                alpha = 0.02;
            }
            goals.insert(id, target);
            routes.insert(id, route(c, at, target, &risk, alpha));
        }
        let mut best = (0, 0f32);
        for (i, (command, f)) in c.commands.iter().zip(&c.features).enumerate().skip(1) {
            let Some(id) = unit(command) else {
                continue;
            };
            let ix = c.spatial.as_ref().unwrap().indices[i];
            if ix[0] < 0 {
                continue;
            }
            let at = ix[0] as usize;
            let critical =
                id == leader || holder == Some(id) || recovery.is_some_and(|(p, _, _)| p == id);
            let score = if f[3] > 0.5 {
                300.
            } else if f[2] > 0.5 {
                if (critical && f[27] > f[9] * 0.45)
                    || (!critical && f[27] >= f[9] && f[28] < 0.8)
                    || (wounded && recovery.is_some_and(|(p, _, _)| p == id))
                {
                    continue;
                }
                let marked_bonus = if ix[2] >= 0 && b.values[51 * n + ix[2] as usize] > 0.5 {
                    if recovery.is_some() { 260. } else { 60. }
                } else {
                    0.
                };
                let guard_bonus = if holder.is_some()
                    && ix[2] >= 0
                    && distance(ix[2] as usize, holder_site, b.width) <= 4.
                {
                    30.
                } else {
                    0.
                };
                100. + 120. * f[28]
                    + 35. * (f[26] - if critical { f[27] } else { 0.35 * f[27] })
                    + marked_bonus
                    + guard_bonus
            } else if f[1] > 0.5 && ix[1] >= 0 {
                let to = ix[1] as usize;
                if holder == Some(id) && at == goal {
                    continue;
                }
                let d = &routes[&id];
                if !d[at].is_finite() || !d[to].is_finite() {
                    continue;
                }
                if matches!(plan, TacticalTeacherPlan::Relief { .. }) && to == goal {
                    10000.
                } else {
                    let gain = d[at] - d[to];
                    let safer = risk[at] - risk[to];
                    if gain <= 0.01 && !(critical && safer > 4.) {
                        continue;
                    }
                    // Any genuine progress remains positive even on a long route.
                    let mut score = 10. + 80. * gain + f[21] * 8.;
                    if critical {
                        score += safer * 0.8;
                    }
                    if recovery.is_some_and(|(p, _, _)| p == id) && f[8] < 0.85 {
                        score += 60.;
                    }
                    if holder == Some(id) {
                        score += 60.;
                    }
                    if id == leader && risk[to] > f[9] * 30. * 0.7 {
                        score -= 100.;
                    }
                    if to == goals[&id] {
                        score += 4.;
                    }
                    score
                }
            } else {
                continue;
            };
            if score > best.1 {
                best = (i, score);
            }
        }
        Some(best.0)
    }))
}
