//! Recruitment selection and placement, independent of tactical planning.
use super::{
    TacticalRole,
    policies::objectives::TileOrder,
    roles::{recruit_role, role},
    tactics::jitter,
};
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecruiterKind {
    /// Fixed live-enemy behavior, independent of simulation-player changes.
    Classic,
    Rotation,
    Frontline,
    Balanced,
    Situational,
    Predictor,
    #[serde(rename = "aggressive-predictor")]
    AggressivePredictor,
}
pub(crate) trait Recruiter {
    fn kind(&self) -> Option<RecruiterKind> {
        None
    }
    fn promotions(&mut self, game: &BattleView<'_, '_>, seed: u32) -> Vec<(UnitId, UnitTypeId)> {
        promotion::choices(game, RecruiterKind::Classic, seed)
    }

    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &crate::ai::orders::RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, CommandError>;
}
impl Recruiter for RecruiterKind {
    fn kind(&self) -> Option<RecruiterKind> {
        Some(*self)
    }
    fn promotions(&mut self, game: &BattleView<'_, '_>, seed: u32) -> Vec<(UnitId, UnitTypeId)> {
        promotion::choices(game, *self, seed)
    }

    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &crate::ai::orders::RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, CommandError> {
        match self {
            Self::Classic => drive(Self::Classic, context, orders),
            Self::Rotation => rotation::Rotation.recruit(context, orders),
            Self::Frontline => frontline::Frontline.recruit(context, orders),
            Self::Balanced => balanced::Balanced.recruit(context, orders),
            Self::Situational => situational::Situational.recruit(context, orders),
            Self::Predictor => drive(Self::Predictor, context, orders),
            Self::AggressivePredictor => drive(Self::AggressivePredictor, context, orders),
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
        let Some(command) = kind.choose_orders(context.battle(), context.seed(), orders) else {
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
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn predicts(self) -> bool {
        match self {
            Self::Predictor | Self::AggressivePredictor => true,
            Self::Classic
            | Self::Rotation
            | Self::Frontline
            | Self::Balanced
            | Self::Situational => false,
        }
    }
    fn choose_orders(
        self,
        game: &BattleView<'_, '_>,
        seed: u32,
        orders: &super::orders::RecruitmentOrders,
    ) -> Option<Command> {
        game.recruit_from_orders(self, seed, orders)
    }
}
impl BattleView<'_, '_> {
    fn recruit_from_orders(
        &self,
        recruiter: RecruiterKind,
        seed: u32,
        orders: &super::orders::RecruitmentOrders,
    ) -> Option<Command> {
        match recruiter {
            RecruiterKind::Situational => return situational::choose(self, seed, orders),
            RecruiterKind::Predictor => {
                return situational::choose_with_forecast(self, seed, orders, true);
            }
            RecruiterKind::AggressivePredictor => {
                return situational::choose_with_weights(
                    self,
                    seed,
                    orders,
                    true,
                    self.rules().recruitment.aggressive_predictor,
                );
            }
            RecruiterKind::Classic
            | RecruiterKind::Rotation
            | RecruiterKind::Frontline
            | RecruiterKind::Balanced => {}
        }
        self.units()
            .iter()
            .filter(|u| self.belongs_to_phase(u, self.turn()) && u.is_leader())
            .find_map(|leader| self.recruit_for_leader(recruiter, seed, orders, leader))
    }
    fn recruit_for_leader(
        &self,
        recruiter: RecruiterKind,
        seed: u32,
        orders: &super::orders::RecruitmentOrders,
        leader: &Unit,
    ) -> Option<Command> {
        let (rotation, frontline_only, balanced) = match recruiter {
            RecruiterKind::Classic | RecruiterKind::Rotation => (true, false, false),
            RecruiterKind::Frontline => (false, true, false),
            RecruiterKind::Balanced => (false, false, true),
            RecruiterKind::Situational
            | RecruiterKind::Predictor
            | RecruiterKind::AggressivePredictor => {
                unreachable!("adaptive recruiters evaluate all commanders together")
            }
        };
        let side = self.turn();
        let required = orders
            .requirements
            .iter()
            .find(|requirement| requirement.deficit(self) > 0);
        let pool: Vec<_> = self
            .recruitment_options_for(&leader)
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
            let kind = if recruiter != RecruiterKind::Classic
                || orders.rotate_pool
                || !pool.contains(&archer)
                || !pool.contains(&swordsman)
            {
                pool[if recruiter == RecruiterKind::Classic {
                    (self.round() as usize - 1) % pool.len()
                } else {
                    // Fill the least represented role, then its least
                    // represented family. Ties give an opening cycle of flank,
                    // infantry, ranged, healer, skipping unavailable roles.
                    let cycle = [
                        UnitRole::Flank,
                        UnitRole::Warrior,
                        UnitRole::Ranger,
                        UnitRole::Healer,
                    ];
                    let allies: Vec<_> = self
                        .units()
                        .iter()
                        .filter(|u| self.belongs_to_phase(u, side) && !u.is_leader())
                        .collect();
                    pool.iter()
                        .enumerate()
                        .min_by_key(|(_, kind)| {
                            let role = recruit_role(self, **kind);
                            (
                                allies
                                    .iter()
                                    .filter(|u| recruit_role(self, u.kind) == role)
                                    .count(),
                                cycle.iter().position(|&r| r == role).unwrap(),
                                allies
                                    .iter()
                                    .filter(|u| {
                                        self.rules().units[u.kind].family
                                            == self.rules().units[**kind].family
                                    })
                                    .count(),
                            )
                        })
                        .map(|(index, _)| index)
                        .unwrap_or(0)
                }]
            } else if self.units().iter().any(|u| {
                self.belongs_to_phase(u, side)
                    && self.rules().units[u.kind].family == Family::Archer
            }) {
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
            tiles.order_by(|&a, &b| order.compare(self, a, b));
            if let Some(&tile) = tiles.first() {
                return Some(super::recruit_command(self, leader.id, kind, tile));
            }
            return None;
        }
        let allies: Vec<_> = self
            .units()
            .iter()
            .filter(|u| self.belongs_to_phase(u, side) && !u.is_leader())
            .collect();
        let opponents: Vec<_> = self
            .units()
            .iter()
            .filter(|u| self.hostile_to_phase(u, side))
            .collect();
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
        let mut choices: Vec<(usize, f64, f64)> = vec![];
        for kind in pool {
            if frontline_only && required.is_none() && role(self, kind) != TacticalRole::Frontline {
                continue;
            }
            if !self
                .placement_tiles(&leader, kind, recruiter == RecruiterKind::Classic)
                .iter()
                .any(|&t| placement_allowed(kind, t))
            {
                continue;
            }
            let r = &self.rules().units[kind];
            let category = recruit_role(self, kind);
            let deficiency = shares[category.index()] * (represented + 1) as f64
                - counts[category.index()] as f64;
            if balanced {
                choices.push((
                    kind,
                    if deficiency > 0.0 { 1.0 } else { 0.0 },
                    random(
                        seed,
                        &format!("balanced:{:?}:{}:{}", side, represented, r.id),
                    ),
                ));
                continue;
            }
            let recruit = Unit {
                faction: leader.faction.clone(),
                items: vec![],
                ai_group: None,
                id: 0,
                kind,
                side,
                tile: leader.tile,
                hp: r.hp,
                moved: true,
                attacked: true,
                experience: None,
                character: None,
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
            let efficiency = (r.hp as f64 * 0.35 + effectiveness * 1.5)
                / self.recruit_cost(&leader, kind).max(1) as f64;
            let score = if frontline_only {
                efficiency * 5.0 + r.hp as f64 * 0.15
            } else {
                efficiency * 2.0
            };
            choices.push((
                kind,
                0.0,
                score + jitter(seed, &format!("recruit:{}:{}", self.round(), r.id)),
            ));
        }
        choices.order_by(|a, b| {
            b.1.total_cmp(&a.1)
                .then_with(|| {
                    if balanced {
                        // Draw roles equally, independently of how many unit
                        // variants each role offers; then draw within the role.
                        let draw = |kind| {
                            random(
                                seed,
                                &format!(
                                    "balanced-role:{:?}:{}:{}",
                                    side,
                                    represented,
                                    recruit_role(self, kind).index()
                                ),
                            )
                        };
                        draw(b.0).total_cmp(&draw(a.0))
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .then_with(|| b.2.total_cmp(&a.2))
        });
        let Some(&(preferred, _, _)) = choices.first() else {
            return None;
        };
        // Choose the desired unit before checking the purse. With income, keep
        // its gold instead of continuously replacing it with cheap recruits.
        let kind = if self.recruit_cost(&leader, preferred) <= self.gold_for_unit(&leader) {
            preferred
        } else if self.income(side) > 0 {
            return None;
        } else {
            choices
                .iter()
                .find(|&&(kind, _, _)| {
                    self.recruit_cost(&leader, kind) <= self.gold_for_unit(&leader)
                })?
                .0
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
        tiles.order_by(|&a, &b| {
            if role(self, kind) == TacticalRole::Frontline {
                proximity(a).cmp(&proximity(b))
            } else {
                proximity(b).cmp(&proximity(a))
            }
        });
        Some(super::recruit_command(self, leader.id, kind, tiles[0]))
    }
}

/// Native fixed-position diagnostics. These are the same mechanical components
/// used by Situational, before the seeded tie-break. They never mutate the game.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn diagnostics(
    game: &crate::logic::Game<'_>,
    recruiter: RecruiterKind,
) -> Vec<serde_json::Value> {
    let view = BattleView::new(game);
    let mut rows = Vec::new();
    for unit in view
        .units()
        .iter()
        .filter(|u| view.belongs_to_phase(u, view.turn()))
    {
        let recruit = unit.is_leader();
        let options = if recruit {
            view.recruitment_options_for(unit)
        } else {
            view.rules().units[unit.kind]
                .advances_to
                .iter()
                .map(|id| view.kind(id))
                .collect()
        };
        for kind in options {
            let rule = &view.rules().units[kind];
            let mut candidate = unit.clone();
            candidate.kind = kind;
            candidate.hp = if recruit {
                rule.hp
            } else {
                (rule.hp - (view.rules().units[unit.kind].hp - unit.hp)).max(1)
            };
            if recruit {
                candidate.id = 0;
            }
            let tiles = if recruit {
                view.placement_tiles(unit, kind, false)
            } else {
                vec![unit.tile]
            };
            let Some(&tile) = tiles.first() else {
                continue;
            };
            candidate.tile = tile;
            let weights = if recruiter == RecruiterKind::AggressivePredictor {
                view.rules().recruitment.aggressive_predictor
            } else {
                view.rules().recruitment.standard
            };
            let assessment = evaluation::Assessment::with_weights(
                &view,
                &candidate,
                recruiter.predicts(),
                if recruit {
                    weights
                } else {
                    view.rules().recruitment.standard
                },
            );
            let copies = view
                .units()
                .iter()
                .filter(|u| {
                    view.allied(u, unit)
                        && !u.is_leader()
                        && view.rules().units[u.kind].family == rule.family
                })
                .count();
            let price = if recruit {
                view.recruit_cost(unit, kind)
            } else {
                rule.cost
            };
            let denominator = if recruit {
                price.max(1) as f64
                    * (1.0 + copies as f64 * weights.duplicate_penalty)
                    * (1.0
                        + f64::from((price - view.gold_for_unit(unit)).max(0))
                            / f64::from(view.income(unit.side).max(1))
                            * weights.wait_penalty)
            } else {
                1.0
            };
            let mut best = None;
            for tile in tiles {
                candidate.tile = tile;
                let score = if recruit {
                    assessment.recruitment_breakdown(&view, &candidate)
                } else {
                    assessment.breakdown(&view, &candidate)
                };
                if best
                    .as_ref()
                    .is_none_or(|(_, previous): &(usize, evaluation::Score)| {
                        score.total > previous.total
                    })
                {
                    best = Some((tile, score));
                }
            }
            if let Some((tile, score)) = best {
                rows.push(serde_json::json!({"choice":if recruit {"recruit"} else {"promotion"},"actor":unit.id,"base":view.rules().units[unit.kind].id,"unit":rule.id,"tile":view.board().ids[tile],"hp":candidate.hp,"cost":price,"affordable":price <= view.gold_for_unit(unit),"eligible":recruit || view.promotion_options(unit).contains(&kind),"denominator":denominator,"score":score.total/denominator,"components":score}));
            }
        }
    }
    rows
}

/// Seeded, well-mixed draw; the small deterministic planner jitter alone is
/// unsuitable for choosing uniformly between adjacent content identifiers.
fn random(seed: u32, key: &str) -> f64 {
    let hash = (jitter(seed, key) / 0.05 * u32::MAX as f64) as u32;
    crate::ai::planner::search::Random(hash).next()
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
    fn balanced_draws_both_kinds_but_only_within_the_needed_role() {
        let mut rules = Rules::load().unwrap();
        let a = rules.types["swordsman"];
        let b = rules.types["archer"];
        rules.units[a].role = UnitRole::Warrior;
        rules.units[b].role = UnitRole::Warrior;
        let mut game = crate::logic::Game::new(&rules, "mirror");
        game.units.retain(|u| u.is_leader());
        game.roster_changed();
        game.pools = Some([vec![a, b], vec![a, b]]);
        game.gold["player"] = 100;
        let orders = crate::ai::orders::RecruitmentOrders::default();
        let draw = |game: &crate::logic::Game<'_>, seed| {
            let result =
                RecruiterKind::Balanced.choose_orders(&BattleView::new(game), seed, &orders);
            assert_eq!(
                result,
                RecruiterKind::Balanced.choose_orders(&BattleView::new(game), seed, &orders)
            );
            let Command::Recruit { kind, .. } = result.unwrap() else {
                panic!("expected recruit");
            };
            kind.0
        };
        let outcomes: std::collections::BTreeSet<_> =
            (0..64).map(|seed| draw(&game, seed)).collect();
        assert_eq!(outcomes.len(), 2);
        drop(game);
        rules.units[b].role = UnitRole::Flank;
        let mut game = crate::logic::Game::new(&rules, "mirror");
        game.units.retain(|u| u.is_leader());
        game.roster_changed();
        game.pools = Some([vec![a, b], vec![a, b]]);
        game.gold["player"] = 100;
        let leader = game
            .units
            .iter()
            .find(|u| u.side == Side::Player)
            .unwrap()
            .clone();
        let tile = game.placement_tiles(&leader, a, false)[0];
        game.add(a, Side::Player, tile);
        // Both roles are under target, even though their deficits differ.
        let outcomes: std::collections::BTreeSet<_> =
            (0..64).map(|seed| draw(&game, seed)).collect();
        assert_eq!(outcomes.len(), 2);
        let tile = game.placement_tiles(&leader, a, false)[0];
        game.add(a, Side::Player, tile);
        // Infantry is now over target, so every draw must be a flanker.
        for seed in 0..16 {
            assert_eq!(draw(&game, seed), rules.units[b].id);
        }
    }

    #[test]
    fn rotation_fills_roles_in_cycle_order_before_repeating_and_skips_absent_roles() {
        let mut rules = Rules::load().unwrap();
        let kinds = ["scout", "swordsman", "archer", "adept"].map(|id| rules.types[id]);
        let roles = [
            UnitRole::Flank,
            UnitRole::Warrior,
            UnitRole::Ranger,
            UnitRole::Healer,
        ];
        for (&kind, role) in kinds.iter().zip(roles) {
            rules.units[kind].role = role;
        }
        for available in [kinds.to_vec(), vec![kinds[1], kinds[2]]] {
            let mut game = crate::logic::Game::new(&rules, "mirror");
            game.units.retain(|u| u.is_leader());
            game.roster_changed();
            let mut reversed = available.clone();
            reversed.reverse();
            game.pools = Some([reversed.clone(), reversed]);
            game.gold["player"] = 1000;
            for &expected in available.iter().cycle().take(available.len() * 2) {
                let command = RecruiterKind::Rotation
                    .choose_orders(
                        &BattleView::new(&game),
                        7,
                        &crate::ai::orders::RecruitmentOrders::default(),
                    )
                    .unwrap();
                let Command::Recruit { ref kind, .. } = command else {
                    panic!("expected recruit")
                };
                assert_eq!(kind.0, rules.units[expected].id);
                game.execute(command).unwrap();
                // Clear the keep so the next choice measures composition,
                // rather than running out of deployment tiles.
                let leader = game
                    .units
                    .iter()
                    .find(|u| u.side == Side::Player && u.is_leader())
                    .unwrap()
                    .clone();
                let deployment = game.placement_tiles(&leader, expected, false);
                let tile = (0..game.terrain.len())
                    .find(|tile| {
                        !deployment.contains(tile) && !game.units.iter().any(|u| u.tile == *tile)
                    })
                    .unwrap();
                game.units.last_mut().unwrap().tile = tile;
                game.roster_changed();
            }
        }
    }

    #[test]
    fn explicit_arena_pool_rotates_through_expensive_units_instead_of_legacy_two_unit_mix() {
        let rules = Rules::load().unwrap();
        let mut game = crate::logic::Game::new(&rules, "mirror");
        game.units.retain(|u| u.is_leader());
        game.roster_changed();
        let pool = vec![
            rules.types["swordsman"],
            rules.types["archer"],
            rules.types["rider"],
        ];
        game.pools = Some([pool.clone(), pool]);
        let leader = game
            .units
            .iter()
            .find(|u| u.side == Side::Player)
            .unwrap()
            .clone();
        let sites = game.placement_tiles(&leader, rules.types["swordsman"], false);
        game.add(rules.types["swordsman"], Side::Player, sites[0]);
        game.add(rules.types["archer"], Side::Player, sites[1]);
        game.round = 3;
        game.gold["player"] = rules.units[rules.types["rider"]].cost;
        let workspace =
            crate::ai::PlanningWorkspace::new(7, 1, None, crate::ai::Diplomacy::Unrestricted);
        let orders = crate::ai::CampaignBlock::Arena.campaign_orders(&game, &workspace);
        assert!(orders.recruitment.rotate_pool);
        // A selected mount survives a round boundary while saving, rather
        // than being skipped when the next roster entry comes around.
        game.gold["player"] = 0;
        assert!(
            RecruiterKind::Rotation
                .choose_orders(&BattleView::new(&game), 7, &orders.recruitment)
                .is_none()
        );
        game.round += 1;
        game.gold["player"] = rules.units[rules.types["rider"]].cost;
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
                | Command::DropItem { .. }
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

mod evaluation;
mod promotion;
