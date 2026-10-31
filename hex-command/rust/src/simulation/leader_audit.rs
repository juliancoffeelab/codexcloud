//! Observational leader and army diagnostics, computed only at real phase boundaries.
//! Exposure sums independent enemies' best legal damage after movement. It is a
//! conservative forecast, not a prediction of coordinated tactics or proof of a
//! mission-compatible alternative. Alternatives hold every other unit fixed.
use crate::{ai::audit_leader_exposure, data::*, logic::Game};
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
    pub tile: String,
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
    pub min_tile: String,
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
    pub side: Side,
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
fn strength(game: &Game<'_>, side: Side) -> ArmyStrength {
    let mut total = ArmyStrength::default();
    for unit in game
        .units
        .iter()
        .filter(|u| u.side == side && !u.is_leader())
    {
        let rule = &game.rules.units[unit.kind];
        total.units += 1;
        total.cost += rule.cost;
        total.hp += unit.hp.max(0);
        total.max_hp += rule.hp;
        total.hp_value +=
            f64::from(rule.cost) * f64::from(unit.hp.max(0)) / f64::from(rule.hp.max(1));
    }
    total
}
fn state(game: &Game<'_>, unit: &Unit) -> LeaderState {
    LeaderState {
        tile: game.board.ids[unit.tile].clone(),
        hp: unit.hp,
        max_hp: game.rules.units[unit.kind].hp,
    }
}
pub(crate) fn begin(game: &Game<'_>) -> Vec<Pending> {
    game.units
        .iter()
        .filter(|u| u.is_leader())
        .map(|leader| {
            let own_phase = leader.side == game.turn;
            let alternatives = if own_phase {
                let mut tiles = vec![leader.tile];
                tiles.extend(game.reachable(leader).iter().map(|&(tile, _)| tile));
                tiles.sort_unstable();
                tiles.dedup();
                audit_leader_exposure(game, leader, tiles)
            } else {
                vec![]
            };
            let options = game.recruitment_options(leader.side);
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
                    side: leader.side,
                    before: state(game, leader),
                    after: None,
                    army_before: strength(game, leader.side),
                    army_after: ArmyStrength::default(),
                    opponent_before: strength(game, leader.side.other()),
                    opponent_after: ArmyStrength::default(),
                    recruitment_before: RecruitmentAvailability {
                        gold: game.gold[leader.side.index()],
                        on_owned_keep: game.terrain_is(leader.tile, Terrain::Keep)
                            && game.owners[leader.tile] == Some(leader.side),
                        legal_options,
                        affordable_options,
                    },
                    exposure: None,
                },
            }
        })
        .collect()
}
pub(crate) fn finish(game: &Game<'_>, pending: Vec<Pending>) -> Vec<LeaderAudit> {
    pending
        .into_iter()
        .map(|mut p| {
            let after = game.unit(p.leader.id);
            p.audit.after = after.map(|leader| state(game, leader));
            p.audit.army_after = strength(game, p.leader.side);
            p.audit.opponent_after = strength(game, p.leader.side.other());
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
                    min_tile: game.board.ids[min_tile].clone(),
                    reachable_tiles: p.alternatives.len(),
                    safe_tiles: p
                        .alternatives
                        .iter()
                        .filter(|&&(_, damage)| damage < p.leader.hp)
                        .count(),
                    chosen_tile_reachable: chosen.is_some(),
                    after_damage: after
                        .map(|leader| audit_leader_exposure(game, leader, [leader.tile])[0].1),
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
        let mut rules = Catalog::load().unwrap();
        let original = Game::new(&rules, "mirror");
        let leader = original
            .units
            .iter()
            .find(|u| u.side == original.turn && u.is_leader())
            .unwrap()
            .clone();
        let enemy = original
            .units
            .iter()
            .find(|u| u.side != leader.side)
            .unwrap()
            .clone();
        drop(original);
        rules.units[enemy.kind].r#move = 1;
        for attack in &mut rules.units[enemy.kind].attacks {
            attack.damage = 100;
            attack.count = 1;
            attack.category = Category::Melee;
            attack.range = 1;
        }
        let mut game = Game::new(&rules, "mirror");
        game.units = vec![leader.clone(), enemy.clone()];
        let mut fixture = None;
        for tile in 0..game.board.ids.len() {
            if tile == leader.tile || game.cost(enemy.kind, tile) == u16::MAX {
                continue;
            }
            game.units[1].tile = tile;
            game.roster_changed();
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
        game.units[0].tile = chosen;
        game.roster_changed();
        let audit = finish(&game, pending);
        let own = audit.iter().find(|a| a.side == leader.side).unwrap();
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
        let rules = Catalog::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let leader = game
            .units
            .iter()
            .find(|u| u.side == game.turn && u.is_leader())
            .unwrap()
            .clone();
        let pending = begin(&game);
        let unchanged = game.state();
        let audit = finish(&game, pending);
        assert_eq!(
            serde_json::to_value(game.state()).unwrap(),
            serde_json::to_value(unchanged).unwrap()
        );
        let own = audit.iter().find(|a| a.side == game.turn).unwrap();
        let exposure = own.exposure.as_ref().unwrap();
        assert!(exposure.reachable_tiles >= 1);
        assert!(exposure.min_damage <= exposure.before_damage);
        assert!(exposure.chosen_tile_reachable);
        let pending = begin(&game);
        game.units
            .retain(|u| u.id != leader.id && u.side != leader.side);
        game.roster_changed();
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
