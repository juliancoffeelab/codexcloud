//! Immutable browser query publication; no work is added to simulation turns.
use super::battle::Game;
use crate::{data::*, protocol::*};

impl Game<'_> {
    fn damage_detail(&self, attack: &Weapon, defender: &Unit, tile: usize) -> DamageDetail {
        let rule = &self.rules.units[defender.kind];
        let modifier = rule.damage_modifiers.get(attack.damage_type);
        let environment = rule.environment[self.terrain[tile]][attack.category.index()]
            [attack.damage_type.index()];
        DamageDetail {
            count: attack.count,
            total: (attack.damage + modifier + environment).max(0) * i32::from(attack.count),
            base: attack.damage,
            damage_type: attack.damage_type,
            modifier,
            modified: attack.damage + modifier,
            protection: (-environment).max(0),
            exposure: environment.max(0),
            terrain: self.rules.terrain_names[self.terrain[tile]].clone(),
            damage: (attack.damage + modifier + environment).max(0),
        }
    }

    pub(crate) fn inspect(&self, revision: u32) -> BattleInspection {
        let units = self
            .units
            .iter()
            .map(|unit| {
                let mut attacks = Vec::new();
                for target in &self.units {
                    let recommended = self.best_attack(unit, target, unit.tile);
                    for weapon in self.attack_options(unit, target, unit.tile) {
                        let attack = self.rules.units[unit.kind].attacks[weapon].clone();
                        let (damage, retaliation, response) =
                            self.preview(unit, target, weapon, unit.tile);
                        let response = response
                            .map(|index| self.rules.units[target.kind].attacks[index].clone());
                        let retaliation_detail = response
                            .as_ref()
                            .map(|r| self.damage_detail(r, unit, unit.tile));
                        attacks.push(AttackDetails {
                            recommended: recommended == Some(weapon),
                            target: format!("u{}", target.id),
                            damage,
                            retaliation,
                            damage_detail: self.damage_detail(&attack, target, target.tile),
                            attack,
                            response,
                            retaliation_detail,
                        });
                    }
                }
                UnitActions {
                    unit: format!("u{}", unit.id),
                    reachable: self
                        .reachable(unit)
                        .iter()
                        .copied()
                        .map(|(tile, cost)| ReachableTile {
                            tile: TileId(self.board.ids[tile].clone()),
                            cost,
                        })
                        .collect(),
                    attacks,
                    promotions: self
                        .promotion_options(unit)
                        .into_iter()
                        .map(|kind| self.rules.units[kind].id.clone())
                        .collect(),
                    recruitment: self
                        .recruitment_options(unit.side)
                        .into_iter()
                        .map(|kind| {
                            (
                                self.rules.units[kind].id.clone(),
                                self.placement_tiles(unit, kind, true)
                                    .into_iter()
                                    .map(|tile| self.board.ids[tile].clone())
                                    .collect(),
                            )
                        })
                        .collect(),
                }
            })
            .collect();
        BattleInspection {
            revision,
            units,
            recruitment: [Side::Player, Side::Enemy]
                .into_iter()
                .map(|side| {
                    (
                        if side == Side::Player {
                            "player".into()
                        } else {
                            "enemy".into()
                        },
                        self.recruitment_options(side)
                            .into_iter()
                            .map(|kind| self.rules.units[kind].id.clone())
                            .collect(),
                    )
                })
                .collect(),
            costs: self
                .rules
                .units
                .iter()
                .map(|rule| {
                    (
                        rule.id.clone(),
                        self.rules
                            .terrain_names
                            .iter()
                            .cloned()
                            .zip(rule.costs.iter().copied())
                            .collect(),
                    )
                })
                .collect(),
            neighbors: self
                .board
                .ids
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    (
                        id.clone(),
                        self.board.adjacent[index]
                            .iter()
                            .map(|&next| self.board.ids[next].clone())
                            .collect(),
                    )
                })
                .collect(),
            income: Gold {
                player: self.income(Side::Player),
                enemy: self.income(Side::Enemy),
            },
        }
    }
}
