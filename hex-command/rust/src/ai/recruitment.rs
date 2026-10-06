use super::{
    Policy, RecruitRole, TacticalRole,
    objectives::TileOrder,
    roles::{recruit_role, role},
    tactics::jitter,
};
use crate::{data::*, logic::Game};
impl Game<'_> {
    pub(super) fn choose_recruit(
        &self,
        policy: Policy,
        seed: u32,
        goals: Option<&[Objective]>,
        restrained: bool,
    ) -> Option<Command> {
        let side = self.turn;
        let goals = goals.map_or_else(|| self.objectives(side), |g| g.to_vec());
        let lanes = self.withdrawal_lanes(side, restrained);
        let Some(leader) = self
            .units
            .iter()
            .find(|u| u.side == side && self.rules.units[u.kind].leader)
        else {
            return None;
        };
        let pool = self.recruitment_options(side);
        let reserved: std::collections::HashMap<_, _> = pool
            .iter()
            .map(|&kind| {
                let unit = Unit {
                    id: 0,
                    kind,
                    tile: leader.tile,
                    side,
                    hp: self.rules.units[kind].hp,
                    moved: true,
                    attacked: true,
                    experience: None,
                    name: None,
                    role: None,
                    retreating: None,
                    rebuilding: None,
                };
                (kind, self.reserved(&unit, &goals))
            })
            .collect();
        let placement_allowed =
            |kind: usize, tile: usize| !reserved[&kind].contains(&tile) && !lanes.contains(&tile);
        if policy == Policy::Classic {
            if pool.is_empty() {
                return None;
            }
            let archer = self.kind("archer");
            let swordsman = self.kind("swordsman");
            let kind =
                if self.campaign.is_some() || !pool.contains(&archer) || !pool.contains(&swordsman)
                {
                    pool[(self.round as usize - 1) % pool.len()]
                } else if self
                    .units
                    .iter()
                    .any(|u| u.side == side && self.rules.units[u.kind].family == Family::Archer)
                {
                    swordsman
                } else {
                    archer
                };
            let mut tiles: Vec<_> = self
                .placement_tiles(&leader, kind, true)
                .into_iter()
                .filter(|&t| placement_allowed(kind, t))
                .collect();
            let order = TileOrder::new(self, &leader, false);
            tiles.sort_by(|&a, &b| order.compare(self, a, b));
            if let Some(&tile) = tiles.first() {
                return Some(super::recruit_command(self, leader.id, kind, tile));
            }
            return None;
        }
        let allies: Vec<_> = self
            .units
            .iter()
            .filter(|u| u.side == side && !self.rules.units[u.kind].leader)
            .collect();
        let opponents: Vec<_> = self.units.iter().filter(|u| u.side != side).collect();
        let mut counts = [0; 4];
        for ally in &allies {
            counts[recruit_role(self, ally.kind).index()] += 1;
        }
        let shares = [0.35, 0.15, 0.35, 0.15];
        let total_share: f64 = RecruitRole::ALL
            .into_iter()
            .filter(|&r| pool.iter().any(|&k| recruit_role(self, k) == r))
            .map(|r| shares[r.index()])
            .sum();
        let mut choices = vec![];
        for kind in pool {
            if policy == Policy::Rush && role(self, kind) != TacticalRole::Frontline {
                continue;
            }
            if !self
                .placement_tiles(&leader, kind, policy != Policy::Mixed)
                .iter()
                .any(|&t| placement_allowed(kind, t))
            {
                continue;
            }
            let r = &self.rules.units[kind];
            let category = recruit_role(self, kind);
            let deficiency = shares[category.index()] / total_share * (allies.len() + 1) as f64
                - counts[category.index()] as f64;
            let recruit = Unit {
                id: 0,
                kind,
                side,
                tile: leader.tile,
                hp: r.hp,
                moved: true,
                attacked: true,
                experience: None,
                name: None,
                role: None,
                retreating: None,
                rebuilding: None,
            };
            let effectiveness = if opponents.is_empty() {
                r.damage as f64
            } else {
                opponents
                    .iter()
                    .map(|target| {
                        r.attacks
                            .iter()
                            .enumerate()
                            .filter(|(_, w)| {
                                w.category
                                    == if category.is_ranged() {
                                        Category::Ranged
                                    } else {
                                        Category::Melee
                                    }
                            })
                            .map(|(w, _)| self.damage(&recruit, target, w))
                            .max()
                            .unwrap_or(0) as f64
                    })
                    .sum::<f64>()
                    / opponents.len() as f64
            };
            let efficiency = (r.hp as f64 * 0.35 + effectiveness * 1.5) / r.cost as f64;
            let score = if policy == Policy::Rush {
                efficiency * 5.0 + r.hp as f64 * 0.15
            } else {
                deficiency * 8.0
                    + efficiency * 2.0
                    + if category == RecruitRole::Frontline
                        && counts[RecruitRole::Frontline.index()] == 0
                    {
                        6.0
                    } else {
                        0.0
                    }
                    + if category == RecruitRole::Healer && counts[RecruitRole::Healer.index()] == 0
                    {
                        3.0
                    } else {
                        0.0
                    }
            };
            choices.push((
                kind,
                score + jitter(seed, &format!("recruit:{}:{}", self.round, r.id)),
            ));
        }
        choices.sort_by(|a, b| b.1.total_cmp(&a.1));
        let Some(&(preferred, _)) = choices.first() else {
            return None;
        };
        if self.rules.units[preferred].cost > self.gold[side.index()] && self.income(side) > 0 {
            if (recruit_role(self, preferred) == RecruitRole::Frontline
                && counts[RecruitRole::Frontline.index()] == 0)
                || self.rules.units[preferred].cost - self.gold[side.index()]
                    <= self.income(side) * 2
            {
                return None;
            }
        }
        let Some(&(kind, _)) = choices
            .iter()
            .find(|&&(k, _)| self.rules.units[k].cost <= self.gold[side.index()])
        else {
            return None;
        };
        let mut tiles: Vec<_> = self
            .placement_tiles(&leader, kind, true)
            .into_iter()
            .filter(|&t| placement_allowed(kind, t))
            .collect();
        let proximity = |t| {
            opponents
                .iter()
                .map(|u| self.board.distance(t, u.tile))
                .min()
                .unwrap_or(0)
        };
        tiles.sort_by(|&a, &b| {
            if role(self, kind) == TacticalRole::Frontline {
                proximity(a).cmp(&proximity(b))
            } else {
                proximity(b).cmp(&proximity(a))
            }
        });
        Some(super::recruit_command(self, leader.id, kind, tiles[0]))
    }
}
