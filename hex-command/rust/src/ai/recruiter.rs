//! Recruitment selection and placement, independent of tactical planning.
use super::{
    TacticalRole,
    objectives::TileOrder,
    roles::{recruit_role, role},
    tactics::jitter,
};
use crate::{data::*, logic::Game};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Recruiter {
    Rotation,
    Frontline,
    Balanced,
}
impl Recruiter {
    pub(super) fn choose_orders(
        self,
        game: &Game<'_>,
        seed: u32,
        orders: &super::orders::RecruitmentOrders,
    ) -> Option<Command> {
        game.recruit_from_orders(self, seed, orders)
    }
    #[cfg(test)]
    pub(super) fn choose(
        self,
        game: &Game<'_>,
        seed: u32,
        goals: Option<&[Objective]>,
        restrained: bool,
    ) -> Option<Command> {
        game.choose_recruit(self, seed, goals, restrained)
    }
}
impl Game<'_> {
    #[cfg(test)]
    pub(super) fn choose_recruit(
        &self,
        recruiter: Recruiter,
        seed: u32,
        goals: Option<&[Objective]>,
        restrained: bool,
    ) -> Option<Command> {
        let workspace = super::PlanningWorkspace::new(
            seed,
            1,
            goals.map(<[Objective]>::to_vec),
            if restrained {
                super::Diplomacy::Restrained
            } else {
                super::Diplomacy::Pragmatic
            },
        );
        let orders = super::CampaignBlock::Objectives.campaign_orders(self, &workspace);
        self.recruit_from_orders(recruiter, seed, &orders.recruitment)
    }
    fn recruit_from_orders(
        &self,
        recruiter: Recruiter,
        seed: u32,
        orders: &super::orders::RecruitmentOrders,
    ) -> Option<Command> {
        fn behavior(recruiter: Recruiter) -> (bool, bool, bool) {
            match recruiter {
                Recruiter::Rotation => (true, false, false),
                Recruiter::Frontline => (false, true, false),
                Recruiter::Balanced => (false, false, true),
            }
        }
        let (rotation, frontline_only, balanced) = behavior(recruiter);
        let side = self.turn;
        let Some(leader) = self
            .units
            .iter()
            .find(|u| u.side == side && self.rules.units[u.kind].leader)
        else {
            return None;
        };
        let required = orders
            .requirements
            .iter()
            .find(|requirement| requirement.deficit(self) > 0);
        let pool: Vec<_> = self
            .recruitment_options(side)
            .into_iter()
            .filter(|&kind| required.is_none_or(|requirement| requirement.accepts(self, kind)))
            .collect();
        let reserved: rustc_hash::FxHashSet<_> = orders
            .reserved
            .iter()
            .map(|tile| self.site(&tile.0))
            .collect();
        let placement_allowed = |_: usize, tile: usize| !reserved.contains(&tile);
        if rotation && required.is_none() {
            if pool.is_empty() {
                return None;
            }
            let archer = self.kind("archer");
            let swordsman = self.kind("swordsman");
            let kind =
                if orders.rotate_pool || !pool.contains(&archer) || !pool.contains(&swordsman) {
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
        let shares = mixed_shares(pool.iter().map(|&k| recruit_role(self, k)));
        let represented: usize = UnitRole::ALL
            .into_iter()
            .filter(|&r| shares[r.index()] > 0.0)
            .map(|r| counts[r.index()])
            .sum();
        let mut choices = vec![];
        for kind in pool {
            if frontline_only && required.is_none() && role(self, kind) != TacticalRole::Frontline {
                continue;
            }
            if !self
                .placement_tiles(&leader, kind, !balanced)
                .iter()
                .any(|&t| placement_allowed(kind, t))
            {
                continue;
            }
            let r = &self.rules.units[kind];
            let category = recruit_role(self, kind);
            let deficiency = shares[category.index()] * (represented + 1) as f64
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
                r.attacks
                    .iter()
                    .filter(|a| {
                        a.category
                            == if category.is_ranged() {
                                Category::Ranged
                            } else {
                                Category::Melee
                            }
                    })
                    .map(|a| a.damage * i32::from(a.count))
                    .max()
                    .unwrap_or(0) as f64
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
            let score = if frontline_only {
                efficiency * 5.0 + r.hp as f64 * 0.15
            } else {
                efficiency * 2.0
            };
            choices.push((
                kind,
                if balanced { deficiency } else { 0.0 },
                score + jitter(seed, &format!("recruit:{}:{}", self.round, r.id)),
            ));
        }
        choices.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| b.2.total_cmp(&a.2)));
        let Some(&(preferred, _, _)) = choices.first() else {
            return None;
        };
        if self.rules.units[preferred].cost > self.gold[side.index()] && self.income(side) > 0 {
            if (recruit_role(self, preferred) == UnitRole::Warrior
                && counts[UnitRole::Warrior.index()] == 0)
                || self.rules.units[preferred].cost - self.gold[side.index()]
                    <= self.income(side) * 2
            {
                return None;
            }
        }
        let Some(&(kind, _, _)) = choices
            .iter()
            .find(|&&(k, _, _)| self.rules.units[k].cost <= self.gold[side.index()])
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

/// Redistribute the authored role proportions over the current roster only.
fn mixed_shares(roles: impl Iterator<Item = UnitRole>) -> [f64; 4] {
    let mut shares = [0.0; 4];
    for role in roles {
        shares[role.index()] = role.weight();
    }
    let total: f64 = shares.iter().sum();
    if total > 0.0 {
        for share in &mut shares {
            *share /= total;
        }
    }
    shares
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_roles_redistribute_without_counting_duplicate_unit_choices() {
        let shares =
            mixed_shares([UnitRole::Warrior, UnitRole::Warrior, UnitRole::Ranger].into_iter());
        assert_eq!(shares, [0.6, 0.0, 0.4, 0.0]);
        let shares = mixed_shares(UnitRole::ALL.into_iter());
        assert!((shares[0] - 1.0 / 3.0).abs() < 1e-12);
        assert!(
            shares[1..]
                .iter()
                .all(|share| (share - 2.0 / 9.0).abs() < 1e-12)
        );
        assert_eq!(mixed_shares(std::iter::empty()), [0.0; 4]);
    }

    #[test]
    fn mixed_recruitment_keeps_the_authored_role_proportions() {
        let rules = Rules::load().unwrap();
        for (roster, purchases, expected) in [
            (
                vec!["reefguard", "scout", "harpooner", "wavemender"],
                9,
                [3, 2, 2, 2],
            ),
            (vec!["skeleton", "bonearcher"], 5, [3, 0, 2, 0]),
        ] {
            let mut game = Game::new(&rules, "mirror");
            game.units
                .retain(|u| u.side == Side::Player && rules.units[u.kind].leader);
            let home = game.units[0].tile;
            game.terrain.fill(rules.terrains["plains"]);
            game.terrain[home] = rules.terrains["keep"];
            game.owners[home] = Some(Side::Player);
            game.pools = Some([
                roster.iter().map(|kind| rules.types[*kind]).collect(),
                vec![],
            ]);
            game.gold = [1000, 0];
            game.roster_changed();
            for _ in 0..purchases {
                let command = game
                    .choose_recruit(Recruiter::Balanced, 1, None, false)
                    .unwrap();
                game.execute(command).unwrap();
                let tile = (0..game.terrain.len())
                    .find(|&t| game.at(t).is_none() && game.board.distance(t, home) > 2)
                    .unwrap();
                game.units.last_mut().unwrap().tile = tile;
                game.roster_changed();
            }
            let mut counts = [0; 4];
            for unit in game.units.iter().filter(|u| !rules.units[u.kind].leader) {
                counts[rules.units[unit.kind].role.index()] += 1;
            }
            assert_eq!(counts, expected);
        }
    }
}
