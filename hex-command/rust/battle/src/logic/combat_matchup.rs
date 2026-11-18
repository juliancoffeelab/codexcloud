//! A borrowed evaluation batch: resolved Catalogue composition, Battle geometry.
use super::{
    BattlePosition,
    exchange::{self, ExchangeForecast, Strikes},
    geometry::AttackOriginRun,
};
use crate::data::*;
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
    sync::Arc,
};

pub struct CombatMatchup<'a, 'catalog> {
    game: &'a BattlePosition<'catalog>,
    actors: [Unit; 2],
    prepared: Option<[Arc<PreparedCombat>; 2]>,
    fixed_reflection: bool,
    reflection: [OnceCell<i32>; 2],
    exchanges: RefCell<[Option<((usize, usize, usize, i32), Rc<ExchangeForecast>)>; 16]>,
}
impl<'a, 'catalog> CombatMatchup<'a, 'catalog> {
    pub(crate) fn new(
        game: &'a BattlePosition<'catalog>,
        actors: [Unit; 2],
        prepared: Option<[Arc<PreparedCombat>; 2]>,
    ) -> Self {
        // Only the incoming attacker changes origin in this batch. If it
        // emits no thorns, every selected reflection provider stays fixed.
        let fixed_reflection = !game
            .definitions
            .owned(&actors[0], &game.runtime.inventories)
            .effects
            .iter()
            .any(|e| matches!(e.value(), EffectValue::Thorns { .. }));
        Self {
            game,
            actors,
            prepared,
            fixed_reflection,
            reflection: std::array::from_fn(|_| OnceCell::new()),
            exchanges: RefCell::new(std::array::from_fn(|_| None)),
        }
    }
    /// Forecast-only incoming reflection; other providers remain at their
    /// current positions. An emitting attacker retains per-origin selection.
    pub fn meta_thorn_score_at(&self, category: Category, origin: usize) -> i32 {
        let calculate = || {
            self.game.with_selected_domain_effects_at(
                &self.actors[1],
                self.actors[1].tile,
                &self.game.units,
                &[(self.actors[0], origin)],
                super::effect_index::EffectKind::Thorns,
                true,
                |_, _, _| true,
                |effects| effects.meta_thorn_score(category),
            )
        };
        if self.fixed_reflection {
            *self.reflection[category.index()].get_or_init(calculate)
        } else {
            calculate()
        }
    }
    #[inline]
    pub fn at_tile(&self, weapon: usize, tile: usize, reflection: i32) -> Rc<ExchangeForecast> {
        self.at_origin(
            weapon,
            &AttackOriginRun {
                tile,
                terrain: self.game.runtime.terrain[tile],
                distance: self.game.board.distance(tile, self.actors[1].tile),
                count: 1,
            },
            reflection,
        )
    }
    #[inline]
    pub fn at_origin(
        &self,
        weapon: usize,
        origin: &AttackOriginRun,
        reflection: i32,
    ) -> Rc<ExchangeForecast> {
        let [unit, target] = &self.actors;
        let Some([attacker, defender]) = &self.prepared else {
            return self
                .game
                .exchange_with_thorns(unit, target, weapon, origin.tile, reflection);
        };
        let category = attacker
            .attacks()
            .nth(weapon)
            .expect("prepared attack index")
            .0;
        let reply_class = defender
            .attacks()
            .filter(|&(reply_category, range)| {
                reply_category == category && u16::from(range) < origin.distance
            })
            .count();
        let key = (
            weapon,
            self.game.terrain[origin.tile],
            reply_class,
            reflection,
        );
        // Direct mapping bounds storage and lookup to one stack slot. Exact
        // keys are still checked, so collisions only recompute a forecast.
        let slot =
            (key.0.wrapping_mul(3) ^ key.1 ^ (key.1 >> 4) ^ key.2.wrapping_mul(5) ^ key.3 as usize)
                & 15;
        if let Some((cached, forecast)) = &self.exchanges.borrow()[slot] {
            if *cached == key {
                return Rc::clone(forecast);
            }
        }
        let attack = attacker.attack_outcomes(
            weapon,
            defender,
            self.game.terrain_definition(target.tile),
            reflection,
        );
        let attack_strikes = Strikes {
            outcomes: attack,
            count: attack.strike_count(),
            maximum_hp: self.game.definitions.unit(unit.kind).max_hp,
            reflection,
        };
        let mut response = None;
        let mut response_mean = 0.0;
        if self.game.hostile(target, unit) {
            for (slot, (reply_category, range)) in defender.attacks().enumerate() {
                if reply_category != category || u16::from(range) < origin.distance {
                    continue;
                }
                if response.is_none() {
                    response = Some(slot);
                    continue;
                }
                if response_mean == 0.0 {
                    let previous = defender.attack_outcomes(
                        response.unwrap(),
                        attacker,
                        self.game.terrain_definition(origin.tile),
                        0,
                    );
                    response_mean = super::damage::distribution(previous).mean;
                }
                let reply = defender.attack_outcomes(
                    slot,
                    attacker,
                    self.game.terrain_definition(origin.tile),
                    0,
                );
                let mean = super::damage::distribution(reply).mean;
                if mean.total_cmp(&response_mean).is_gt() {
                    response = Some(slot);
                    response_mean = mean;
                }
            }
        }
        let reply = response.map_or(
            Strikes {
                count: 0,
                reflection: 0,
                ..attack_strikes
            },
            |slot| {
                let outcomes = defender.attack_outcomes(
                    slot,
                    attacker,
                    self.game.terrain_definition(origin.tile),
                    0,
                );
                Strikes {
                    outcomes,
                    count: outcomes.strike_count(),
                    maximum_hp: self.game.definitions.unit(target.kind).max_hp,
                    reflection: 0,
                }
            },
        );
        let result = exchange::forecast([unit.hp, target.hp], [attack_strikes, reply]);
        self.exchanges.borrow_mut()[slot] = Some((key, Rc::clone(&result)));
        result
    }
}
