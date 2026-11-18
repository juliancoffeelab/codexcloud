//! Observational leader and army diagnostics, computed only at real phase boundaries.
//! Exposure sums independent enemies' best legal damage after movement. It is a
//! conservative forecast, not a prediction of coordinated tactics or proof of a
//! mission-compatible alternative. Alternatives hold every other unit fixed.
use crate::{ai::audit_leader_exposure, data::*, logic::BattleSession};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArmyStrength {
    pub units: usize,
    pub cost: i32,
    pub hp: i32,
    pub max_hp: i32,
    /// Sum of each non-leader's cost multiplied by its remaining HP fraction.
    pub hp_value: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderState {
    pub tile: Hex,
    pub hp: i32,
    pub max_hp: i32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentAvailability {
    pub gold: i32,
    pub on_owned_keep: bool,
    /// Recruit kinds with an available placement, regardless of affordability.
    pub legal_options: usize,
    pub affordable_options: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exposure {
    pub before_damage: i32,
    pub chosen_tile_damage: Option<i32>,
    pub min_damage: i32,
    pub min_tile: Hex,
    pub reachable_tiles: usize,
    pub safe_tiles: usize,
    pub chosen_tile_reachable: bool,
    /// Recomputed after all real friendly actions, rather than mixing snapshots.
    pub after_damage: Option<i32>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderAudit {
    pub leader: String,
    #[serde(rename = "side")]
    pub phase: TurnPhase,
    pub before: LeaderState,
    pub after: Option<LeaderState>,
    pub army_before: ArmyStrength,
    pub army_after: ArmyStrength,
    pub opponent_before: ArmyStrength,
    pub opponent_after: ArmyStrength,
    pub recruitment_before: RecruitmentAvailability,
    /// Only present on this leader's own phase; enemy-turn rows still retain
    /// before/after army strength and the exact boundary on which death occurred.
    pub exposure: Option<Exposure>,
}
pub(crate) struct Pending {
    leader: Unit,
    audit: LeaderAudit,
    alternatives: Vec<(usize, i32)>,
}
fn strength(game: &BattleSession<'_>, leader: &Unit, opponent: bool) -> ArmyStrength {
    let mut total = ArmyStrength::default();
    for unit in game.authoritative_units().iter().filter(|u| {
        !u.is_leader()
            && game.available_combatant(u)
            && if opponent {
                game.hostile(leader, u)
            } else {
                game.allied(leader, u)
            }
    }) {
        let rule = game.unit_definition(unit.kind);
        total.units += 1;
        let price = game.recruit_cost(unit, unit.kind);
        total.cost += price;
        total.hp += unit.hp.max(0);
        total.max_hp += rule.max_hp;
        total.hp_value +=
            f64::from(price) * f64::from(unit.hp.max(0)) / f64::from(rule.max_hp.max(1));
    }
    total
}
fn state(game: &BattleSession<'_>, unit: &Unit) -> LeaderState {
    LeaderState {
        tile: game.board().ids[unit.tile].clone(),
        hp: unit.hp,
        max_hp: game.unit_definition(unit.kind).max_hp,
    }
}
pub(crate) fn begin(game: &BattleSession<'_>) -> Vec<Pending> {
    game.authoritative_units()
        .iter()
        .filter(|u| u.is_leader())
        .map(|leader| {
            let own_phase = leader.phase == game.turn();
            let alternatives = if own_phase {
                let mut scoped = game.fork();
                scoped.set_active_faction(Some(game.unit_faction(leader).to_owned()));
                let mut tiles = vec![leader.tile];
                tiles.extend(scoped.reachable(leader).iter().map(|&(tile, _)| tile));
                tiles.sort_unstable();
                tiles.dedup();
                audit_leader_exposure(scoped.phase_forecasts(), leader, tiles)
            } else {
                vec![]
            };
            let options = game.recruitment_options_for(leader);
            let legal_options = options
                .iter()
                .filter(|&&kind| !game.placement_tiles(leader, kind, false).is_empty())
                .count();
            let affordable_options = options
                .iter()
                .filter(|&&kind| !game.placement_tiles(leader, kind, true).is_empty())
                .count();
            Pending {
                leader: leader.clone(),
                alternatives,
                audit: LeaderAudit {
                    leader: format!("u{}", leader.id),
                    phase: leader.phase,
                    before: state(game, leader),
                    after: None,
                    army_before: strength(game, leader, false),
                    army_after: ArmyStrength::default(),
                    opponent_before: strength(game, leader, true),
                    opponent_after: ArmyStrength::default(),
                    recruitment_before: RecruitmentAvailability {
                        gold: game.gold_for_unit(leader),
                        on_owned_keep: game.is_keep(leader.tile)
                            && game.owners()[leader.tile].as_deref()
                                == Some(game.unit_faction(leader)),
                        legal_options,
                        affordable_options,
                    },
                    exposure: None,
                },
            }
        })
        .collect()
}
pub(crate) fn finish(game: &BattleSession<'_>, pending: Vec<Pending>) -> Vec<LeaderAudit> {
    pending
        .into_iter()
        .map(|mut p| {
            let after = game.authoritative_unit(p.leader.id);
            p.audit.after = after.map(|leader| state(game, leader));
            p.audit.army_after = strength(game, &p.leader, false);
            p.audit.opponent_after = strength(game, &p.leader, true);
            if let Some(&(min_tile, min_damage)) = p
                .alternatives
                .iter()
                .min_by_key(|&&(tile, damage)| (damage, tile))
            {
                let chosen = after.and_then(|leader| {
                    p.alternatives
                        .iter()
                        .find(|&&(tile, _)| tile == leader.tile)
                });
                p.audit.exposure = Some(Exposure {
                    before_damage: p
                        .alternatives
                        .iter()
                        .find(|&&(tile, _)| tile == p.leader.tile)
                        .unwrap()
                        .1,
                    chosen_tile_damage: chosen.map(|&(_, damage)| damage),
                    min_damage,
                    min_tile: game.board().ids[min_tile].clone(),
                    reachable_tiles: p.alternatives.len(),
                    safe_tiles: p
                        .alternatives
                        .iter()
                        .filter(|&&(_, damage)| damage < p.leader.hp)
                        .count(),
                    chosen_tile_reachable: chosen.is_some(),
                    after_damage: after.map(|leader| {
                        audit_leader_exposure(game.phase_forecasts(), leader, [leader.tile])[0].1
                    }),
                });
            }
            p.audit
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exposed_destination_is_compared_with_a_reachable_nonlethal_alternative() {
        let mut source = crate::catalogue_fixture::bundle();
        let rules = crate::catalogue_fixture::frozen(&source);
        let original = BattleSession::new(&rules, "mirror");
        let leader = original
            .units()
            .iter()
            .find(|u| u.phase == original.turn() && u.is_leader())
            .unwrap()
            .clone();
        let enemy = original
            .units()
            .iter()
            .find(|u| {
                u.faction
                    != original
                        .units()
                        .iter()
                        .find(|x| x.id == leader.id)
                        .unwrap()
                        .faction
            })
            .unwrap()
            .clone();
        drop(original);
        crate::catalogue_fixture::unit_kind(&mut source, enemy.kind, |unit| {
            unit["move"] = serde_json::json!(1);
            for attack in unit["attacks"].as_array_mut().unwrap() {
                attack["damage"] = serde_json::json!(100);
                attack["count"] = serde_json::json!(1);
                attack["category"] = serde_json::json!(Category::Melee);
                attack["range"] = serde_json::json!(1);
            }
        });
        let rules = crate::catalogue_fixture::frozen(&source);
        let mut game = BattleSession::new(&rules, "mirror");
        crate::tests::edit(&mut game, |state| {
            state
                .units
                .retain(|u| u.id == format!("u{}", leader.id) || u.id == format!("u{}", enemy.id))
        });
        let mut fixture = None;
        for tile in 0..game.board().ids.len() {
            if tile == leader.tile || game.step_cost(game.unit(enemy.id).unwrap(), tile) == u16::MAX
            {
                continue;
            }
            let target = game.board().ids[tile].clone();
            crate::tests::edit(&mut game, |state| state.units[1].tile = target);
            let pending = begin(&game);
            let own = pending.iter().find(|p| p.leader.id == leader.id).unwrap();
            if own
                .alternatives
                .iter()
                .any(|&(_, damage)| damage < leader.hp)
            {
                if let Some(&(chosen, _)) = own
                    .alternatives
                    .iter()
                    .find(|&&(site, damage)| site != leader.tile && damage >= leader.hp)
                {
                    fixture = Some((pending, chosen));
                    break;
                }
            }
        }
        let (pending, chosen) =
            fixture.expect("fixture has a safe alternative and an exposed destination");
        let target = game.board().ids[chosen].clone();
        crate::tests::edit(&mut game, |state| state.units[0].tile = target);
        let audit = finish(&game, pending);
        let own = audit.iter().find(|a| a.phase == leader.phase).unwrap();
        let exposure = own.exposure.as_ref().unwrap();
        assert!(exposure.chosen_tile_reachable);
        assert!(exposure.safe_tiles > 0);
        assert!(exposure.min_damage < leader.hp);
        assert!(exposure.chosen_tile_damage.unwrap() >= leader.hp);
        assert!(exposure.after_damage.unwrap() >= leader.hp);
        assert!(own.after.is_some()); // Forecast exposure is not an observed death.
    }
    #[test]
    fn boundaries_record_army_attrition_and_leader_death_without_a_verdict() {
        let rules = Catalogue::load().unwrap();
        let mut game = BattleSession::new(&rules, "mirror");
        let leader = game
            .units()
            .iter()
            .find(|u| u.phase == game.turn() && u.is_leader())
            .unwrap()
            .clone();
        let pending = begin(&game);
        let unchanged = game.state();
        let audit = finish(&game, pending);
        assert_eq!(
            serde_json::to_value(game.state()).unwrap(),
            serde_json::to_value(unchanged).unwrap()
        );
        let own = audit.iter().find(|a| a.phase == game.turn()).unwrap();
        let exposure = own.exposure.as_ref().unwrap();
        assert!(exposure.reachable_tiles >= 1);
        assert!(exposure.min_damage <= exposure.before_damage);
        assert!(exposure.chosen_tile_reachable);
        let pending = begin(&game);
        let faction = game.unit_faction(&leader).to_owned();
        crate::tests::edit(&mut game, |state| {
            state
                .units
                .retain(|u| u.id != format!("u{}", leader.id) && u.faction != faction)
        });
        let audit = finish(&game, pending);
        let dead = audit
            .iter()
            .find(|a| a.leader == format!("u{}", leader.id))
            .unwrap();
        assert!(dead.after.is_none());
        assert_eq!(dead.army_after.units, 0);
        assert_eq!(dead.army_after.hp_value, 0.0);
        assert!(dead.army_before.units > 0);
    }
}
