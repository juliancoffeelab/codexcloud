//! Recruitment selection and placement, independent of tactical planning.
use super::{
    TacticalRole,
    objectives::TileOrder,
    roles::{recruit_role, role},
    tactics::jitter,
};
use crate::{ai::context::BattleView, data::*};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecruiterKind {
    Rotation,
    Frontline,
    Balanced,
    Situational,
}
pub(crate) trait Recruiter {
    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &crate::ai::orders::RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, CommandError>;
}
impl Recruiter for RecruiterKind {
    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &crate::ai::orders::RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, CommandError> {
        match self {
            Self::Rotation => rotation::Rotation.recruit(context, orders),
            Self::Frontline => frontline::Frontline.recruit(context, orders),
            Self::Balanced => balanced::Balanced.recruit(context, orders),
            Self::Situational => situational::Situational.recruit(context, orders),
        }
    }
}
fn drive(
    kind: RecruiterKind,
    context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
    orders: &crate::ai::orders::RecruitmentOrders,
) -> Result<crate::ai::orders::RecruitmentReport, CommandError> {
    use crate::ai::orders::*;
    let mut report = RecruitmentReport::default();
    while context
        .budget
        .is_none_or(|limit| report.recruited.len() < limit)
    {
        let mandatory = orders
            .requirements
            .iter()
            .any(|requirement| requirement.deficit(context.battle()) > 0);
        let Some(command) = kind.choose_orders(context.battle(), context.seed, orders) else {
            break;
        };
        let id = UnitId(context.battle().next_id());
        context.emit(command)?;
        report.recruited.push(id);
        if mandatory {
            report.mandatory.push(id);
        }
    }
    for requirement in &orders.requirements {
        let missing = requirement.deficit(context.battle());
        if missing > 0 {
            let RecruitRequirement::EnsureUnits { selector, .. } = requirement;
            report.requirements.push(RecruitmentShortfall {
                selector: selector.clone(),
                missing,
                reason: PlanningReason::RecruitmentUnavailable,
            });
        }
    }
    Ok(report)
}

impl RecruiterKind {
    pub(super) fn choose_orders(
        self,
        game: &BattleView<'_, '_>,
        seed: u32,
        orders: &super::orders::RecruitmentOrders,
    ) -> Option<Command> {
        game.recruit_from_orders(self, seed, orders)
    }
    #[cfg(test)]
    pub(in crate::ai) fn choose(
        self,
        game: &crate::logic::Game<'_>,
        seed: u32,
        goals: Option<&[Objective]>,
        restrained: bool,
    ) -> Option<Command> {
        let workspace = crate::ai::PlanningWorkspace::new(
            seed,
            1,
            goals.map(<[Objective]>::to_vec),
            if restrained {
                crate::ai::Diplomacy::Restrained
            } else {
                crate::ai::Diplomacy::Pragmatic
            },
        );
        let orders = crate::ai::CampaignBlock::Mission.campaign_orders(game, &workspace);
        BattleView::new(game).recruit_from_orders(self, seed, &orders.recruitment)
    }
}
impl BattleView<'_, '_> {
    fn recruit_from_orders(
        &self,
        recruiter: RecruiterKind,
        seed: u32,
        orders: &super::orders::RecruitmentOrders,
    ) -> Option<Command> {
        let (rotation, frontline_only, balanced) = match recruiter {
            RecruiterKind::Rotation => (true, false, false),
            RecruiterKind::Frontline => (false, true, false),
            RecruiterKind::Balanced => (false, false, true),
            RecruiterKind::Situational => return situational::choose(self, seed, orders),
        };
        let side = self.turn();
        let Some(leader) = self
            .units()
            .iter()
            .find(|u| u.side == side && self.rules().units[u.kind].leader)
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
                    pool[(self.round() as usize - 1) % pool.len()]
                } else if self
                    .units()
                    .iter()
                    .any(|u| u.side == side && self.rules().units[u.kind].family == Family::Archer)
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
            .units()
            .iter()
            .filter(|u| u.side == side && !self.rules().units[u.kind].leader)
            .collect();
        let opponents: Vec<_> = self.units().iter().filter(|u| u.side != side).collect();
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
            let r = &self.rules().units[kind];
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
            let copies = allies
                .iter()
                .filter(|u| self.rules().units[u.kind].family == r.family)
                .count();
            let efficiency = (r.hp as f64 * 0.35 + effectiveness * 1.5)
                / r.cost as f64
                / if balanced {
                    1.0 + copies as f64 * 0.25
                } else {
                    1.0
                };
            let score = if frontline_only {
                efficiency * 5.0 + r.hp as f64 * 0.15
            } else {
                efficiency * 2.0
            };
            choices.push((
                kind,
                if balanced { deficiency } else { 0.0 },
                score + jitter(seed, &format!("recruit:{}:{}", self.round(), r.id)),
            ));
        }
        choices.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| b.2.total_cmp(&a.2)));
        let Some(&(preferred, _, _)) = choices.first() else {
            return None;
        };
        if self.rules().units[preferred].cost > self.gold()[side.index()] && self.income(side) > 0 {
            if (recruit_role(self, preferred) == UnitRole::Warrior
                && counts[UnitRole::Warrior.index()] == 0)
                || self.rules().units[preferred].cost - self.gold()[side.index()]
                    <= self.income(side) * 2
            {
                return None;
            }
        }
        let Some(&(kind, _, _)) = choices
            .iter()
            .find(|&&(k, _, _)| self.rules().units[k].cost <= self.gold()[side.index()])
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
                .map(|u| self.board().distance(t, u.tile))
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
    fn explicit_arena_pool_rotates_through_expensive_units_instead_of_legacy_two_unit_mix() {
        let rules = Rules::load().unwrap();
        let mut game = crate::logic::Game::new(&rules, "mirror");
        game.units.retain(|u| rules.units[u.kind].leader);
        game.roster_changed();
        let pool = vec![
            rules.types["swordsman"],
            rules.types["archer"],
            rules.types["rider"],
        ];
        game.pools = Some([pool.clone(), pool]);
        game.round = 3;
        game.gold[Side::Player.index()] = rules.units[rules.types["rider"]].cost;
        let workspace =
            crate::ai::PlanningWorkspace::new(7, 1, None, crate::ai::Diplomacy::Pragmatic);
        let orders = crate::ai::CampaignBlock::Arena.campaign_orders(&game, &workspace);
        assert!(orders.recruitment.rotate_pool);
        let selected = RecruiterKind::Rotation
            .choose_orders(&BattleView::new(&game), 7, &orders.recruitment)
            .unwrap();
        fn recruited(command: Command) -> Option<String> {
            match command {
                Command::Recruit { kind, .. } => Some(kind.0),
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Advance { .. }
                | Command::SetUnitIntent { .. }
                | Command::EndTurn
                | Command::DismissDialogue
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::RecallMission { .. }
                | Command::RetryMission => None,
            }
        }
        assert_eq!(recruited(selected).as_deref(), Some("rider"));
    }

    #[test]
    fn absent_roles_redistribute_without_counting_duplicate_unit_choices() {
        let shares =
            mixed_shares([UnitRole::Warrior, UnitRole::Warrior, UnitRole::Ranger].into_iter());
        assert_eq!(
            shares,
            mixed_shares([UnitRole::Warrior, UnitRole::Ranger].into_iter())
        );
        assert_eq!(shares[UnitRole::Flank.index()], 0.0);
        assert_eq!(shares[UnitRole::Healer.index()], 0.0);
        assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(
            (shares[UnitRole::Warrior.index()] / shares[UnitRole::Ranger.index()]
                - UnitRole::Warrior.weight() / UnitRole::Ranger.weight())
            .abs()
                < 1e-12
        );
        assert_eq!(mixed_shares(std::iter::empty()), [0.0; 4]);
    }
}

mod balanced;
mod frontline;
mod rotation;
mod situational;
