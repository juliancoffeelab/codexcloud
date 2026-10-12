use super::{
    Policy, TacticalRole,
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
                    remaining_attacks: self.rules.units[kind]
                        .attacks
                        .iter()
                        .map(|a| a.count)
                        .collect(),
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
        let shares = mixed_shares(pool.iter().map(|&k| recruit_role(self, k)));
        let represented: usize = UnitRole::ALL
            .into_iter()
            .filter(|&r| shares[r.index()] > 0.0)
            .map(|r| counts[r.index()])
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
                remaining_attacks: r.attacks.iter().map(|a| a.count).collect(),
                experience: None,
                name: None,
                role: None,
                retreating: None,
                rebuilding: None,
            };
            let attack_category = if category.is_ranged() {
                Category::Ranged
            } else {
                Category::Melee
            };
            let effectiveness = if opponents.is_empty() {
                r.attacks
                    .iter()
                    .filter(|a| a.category == attack_category)
                    .map(|a| a.damage * i32::from(a.count))
                    .sum::<i32>() as f64
            } else {
                opponents
                    .iter()
                    .map(|target| {
                        r.attacks
                            .iter()
                            .enumerate()
                            .filter(|(_, w)| w.category == attack_category)
                            .map(|(w, a)| self.damage(&recruit, target, w) * i32::from(a.count))
                            .sum::<i32>() as f64
                    })
                    .sum::<f64>()
                    / opponents.len() as f64
            };
            let efficiency = (r.hp as f64 * 0.35 + effectiveness * 1.5) / r.cost as f64;
            let score = if policy == Policy::Rush {
                efficiency * 5.0 + r.hp as f64 * 0.15
            } else {
                efficiency * 2.0
            };
            choices.push((
                kind,
                if policy == Policy::Mixed {
                    deficiency
                } else {
                    0.0
                },
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
                let command = game.choose_recruit(Policy::Mixed, 1, None, false).unwrap();
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
