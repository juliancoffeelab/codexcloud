//! Shared mechanical valuation for recruits and free, earned promotions.
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};
use std::{cell::RefCell, rc::Rc};

struct Approach {
    weapon: usize,
    routes: Option<Rc<Vec<u16>>>,
    offense: RefCell<Option<OffensiveAverage>>,
}
// Scalar identity excludes placement and covers every mutable combat input.
#[derive(Clone, Copy, PartialEq, Eq)]
struct CombatIdentity {
    id: u32,
    kind: usize,
    hp: i32,
    side: TurnPhase,
    faction: NameKey,
    inventory: InventoryId,
}
impl CombatIdentity {
    fn new(unit: &Unit) -> Self {
        Self {
            id: unit.id,
            kind: unit.kind,
            hp: unit.hp,
            side: unit.side,
            faction: unit.faction,
            inventory: unit.inventory,
        }
    }
    fn matches(&self, unit: &Unit) -> bool {
        self.id == unit.id
            && self.kind == unit.kind
            && self.hp == unit.hp
            && self.side == unit.side
            && self.faction == unit.faction
            && self.inventory == unit.inventory
    }
}

// One assessment observes one immutable battle and candidate. Placement changes
// terrain, legal replies and reflection. Cache those exchange inputs while the
// caller retains origin order, floating-point sums and weapon tie breaks.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct DefenseKey {
    weapon: usize,
    terrain: usize,
    origin_terrain: usize,
    reply_class: usize,
    reflection: i32,
}
impl DefenseKey {
    // Oversized authored catalogs use the uncached calculation, preserving the
    // full index domain. Ordinary rows fit losslessly in two machine words.
    #[inline]
    fn packed(self, enemy: usize) -> Option<u128> {
        if enemy > u32::MAX as usize
            || self.weapon > u16::MAX as usize
            || self.terrain > u16::MAX as usize
            || self.origin_terrain > u16::MAX as usize
            || self.reply_class > u16::MAX as usize
        {
            return None;
        }
        Some(
            (enemy as u128)
                | ((self.weapon as u128) << 32)
                | ((self.terrain as u128) << 48)
                | ((self.origin_terrain as u128) << 64)
                | ((self.reply_class as u128) << 80)
                | ((self.reflection as u32 as u128) << 96),
        )
    }
}
// Legal-response classes depend on range boundaries, not duplicate weapon
// counts. A stack bitset represents every u8 range without a collected Vec.
#[derive(Clone, Copy, Default)]
struct ReplyRanges([u64; 4]);
impl ReplyRanges {
    fn insert(&mut self, range: u8) {
        self.0[usize::from(range) / 64] |= 1u64 << (range % 64);
    }
    #[inline]
    fn class(&self, distance: u16) -> usize {
        let distance = usize::from(distance).min(256);
        let word = distance / 64;
        let shift = distance % 64;
        let mut count = 0;
        for i in 0..word {
            count += self.0[i].count_ones() as usize;
        }
        if shift != 0 {
            count += (self.0[word] & ((1u64 << shift) - 1)).count_ones() as usize;
        }
        count
    }
}
// Consecutive equal exchange classes have the same additions in the same
// order, even when raw geometry splits a run at an irrelevant range boundary.
struct OriginClasses<'a> {
    runs: &'a [hex_command_battle::AttackOriginRun],
    ranges: &'a ReplyRanges,
    next: usize,
}
impl Iterator for OriginClasses<'_> {
    type Item = (usize, usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let first = self.runs.get(self.next)?;
        let terrain = first.terrain;
        let class = self.ranges.class(first.distance);
        let mut count = first.count;
        self.next += 1;
        while let Some(run) = self.runs.get(self.next) {
            if run.terrain != terrain || self.ranges.class(run.distance) != class {
                break;
            }
            count += run.count;
            self.next += 1;
        }
        Some((terrain, class, count))
    }
}
struct DefenseAverage {
    weapon: usize,
    terrain: usize,
    reflection: i32,
    runs: Rc<Vec<hex_command_battle::AttackOriginRun>>,
    value: ExchangeValue,
}
impl DefenseAverage {
    fn matches(
        &self,
        weapon: usize,
        terrain: usize,
        reflection: i32,
        runs: &[hex_command_battle::AttackOriginRun],
        ranges: &ReplyRanges,
    ) -> bool {
        self.weapon == weapon
            && self.terrain == terrain
            && self.reflection == reflection
            && (OriginClasses {
                runs: &self.runs,
                ranges,
                next: 0,
            })
            .eq(OriginClasses {
                runs,
                ranges,
                next: 0,
            })
    }
}
struct DefenseCache {
    identity: CombatIdentity,
    reachable: hex_command_battle::Reachability,
    last: Option<(DefenseKey, ExchangeValue)>,
    averages: [Option<DefenseAverage>; 2],
    next_average: usize,
}
impl DefenseCache {
    fn new(
        game: &BattleView<'_, '_>,
        enemy: &Unit,
        _unit: &Unit,
        identity: CombatIdentity,
    ) -> Self {
        Self {
            identity,
            reachable: game.reachability(enemy.kind, enemy.tile),
            last: None,
            averages: [None, None],
            next_average: 0,
        }
    }
    #[inline(always)]
    fn exchange(
        &mut self,
        key: DefenseKey,
        calculate: impl FnOnce() -> ExchangeValue,
    ) -> ExchangeValue {
        if let Some((last, value)) = self.last {
            if last == key {
                return value;
            }
        }
        let value = calculate();
        self.last = Some((key, value));
        value
    }
}
#[derive(Clone, Copy)]
struct ExchangeValue {
    value: f64,
    damage: f64,
    response: f64,
    strikes: f64,
}

struct OffensiveAverage {
    source: usize,
    count: usize,
    identity: CombatIdentity,
    value: f64,
    leader_damage: f64,
}
impl OffensiveAverage {
    fn matches(&self, source: usize, identity: &CombatIdentity) -> bool {
        self.source == source && self.identity == *identity
    }
}
impl ExchangeValue {
    #[inline]
    fn from_forecast(forecast: &hex_command_battle::exchange::ExchangeForecast) -> Self {
        let response = forecast.damage[1];
        Self {
            value: forecast.effective[0] + forecast.healing[0] + forecast.death[1] * 18.0
                - response * 0.7
                - forecast.death[0] * 40.0,
            damage: forecast.damage[0],
            response,
            strikes: forecast.strikes[0],
        }
    }
}
// Before four troops are visible, blend the known legal enemy roster into the
// matchup estimate. This prior prevents opening purchases from countering only
// a leader. Observed troops progressively replace it, including after losses.

struct Support {
    healing: f64,
    thorns: f64,
    routes: Rc<Vec<u16>>,
}
#[derive(serde::Serialize)]
pub(super) struct Score {
    pub offense: f64,
    pub effective_hp: f64,
    pub access: f64,
    pub retaliation: f64,
    pub support: f64,
    pub thorns: f64,
    pub mobility: f64,
    pub captures: f64,
    pub leader_pressure: f64,
    pub movement_bonus: f64,
    pub healing: f64,
    pub aura_thorns: f64,
    pub total: f64,
}

// Scalar scoring specializes away diagnostic-only fields and the large struct
// return; reports retain the same valuation and complete component breakdown.
impl From<Score> for f64 {
    #[inline]
    fn from(score: Score) -> Self {
        score.total
    }
}

/// Immutable observations for one selection. Candidates under the same
/// commander share relation classification, legal-future queries and units.
pub(super) struct AssessmentScene {
    observed: Vec<usize>,
    allies: Vec<usize>,
    units: Vec<Unit>,
    weights: Vec<f64>,
}
impl AssessmentScene {
    pub(super) fn new(
        game: &BattleView<'_, '_>,
        observer: &Unit,
        predict: bool,
        future_enemies: f64,
    ) -> Self {
        let mut observed = Vec::new();
        let mut allies = Vec::new();
        for (index, unit) in game.units().iter().enumerate() {
            match game.unit_relation(observer, unit) {
                Relation::Hostile => observed.push(index),
                Relation::Allied => {
                    if unit.id != observer.id {
                        allies.push(index);
                    }
                }
                Relation::Neutral => {}
            }
        }
        let enemies = || observed.iter().map(|&index| &game.units()[index]);
        let mut units = Vec::new();
        let mut weights_by_unit = Vec::new();
        if predict {
            for leader in enemies().filter(|e| e.is_leader()) {
                let actual = enemies()
                    .filter(|e| !e.is_leader() && game.same_faction(leader, e))
                    .count();
                let commanders = enemies()
                    .filter(|e| e.is_leader() && game.same_faction(leader, e))
                    .count();
                let options = game.recruitment_options_for(leader);
                let forecast_weight = (future_enemies - actual as f64).max(0.0)
                    / options.len().max(1) as f64
                    / commanders as f64;
                if forecast_weight > 0.0 {
                    for kind in options {
                        let mut forecast = (*leader).clone();
                        forecast.inventory = InventoryId::default();
                        forecast.kind = kind;
                        forecast.hp = game.rules().mechanics.units[kind].hp;
                        forecast.id = 0;
                        units.push(forecast);
                        weights_by_unit.push(forecast_weight);
                    }
                }
            }
        }
        Self {
            observed,
            allies,
            units,
            weights: weights_by_unit,
        }
    }
}

pub(super) struct Assessment {
    identity: RefCell<CombatIdentity>,
    weights: RecruitmentWeights,
    scene: Rc<AssessmentScene>,
    approaches: Vec<Approach>,
    weapon_count: usize,
    defenses: Vec<RefCell<Option<DefenseCache>>>,
    support: Vec<Support>,
    captures: Vec<(Rc<Vec<u16>>, f64)>,
    friendly_count: usize,
    exchanges: RefCell<rustc_hash::FxHashMap<u128, ExchangeValue>>,
}
impl Assessment {
    #[cfg(test)]
    pub(super) fn new(game: &BattleView<'_, '_>, unit: &Unit) -> Self {
        Self::with_forecast(game, unit, false)
    }
    pub(super) fn with_forecast(game: &BattleView<'_, '_>, unit: &Unit, predict: bool) -> Self {
        Self::with_weights(game, unit, predict, game.rules().recruitment.standard)
    }
    pub(super) fn with_weights(
        game: &BattleView<'_, '_>,
        unit: &Unit,
        predict: bool,
        weights: RecruitmentWeights,
    ) -> Self {
        let scene = Rc::new(AssessmentScene::new(
            game,
            unit,
            predict,
            weights.future_enemies,
        ));
        Self::with_prepared(game, unit, weights, scene)
    }
    pub(super) fn with_prepared(
        game: &BattleView<'_, '_>,
        unit: &Unit,
        weights: RecruitmentWeights,
        scene: Rc<AssessmentScene>,
    ) -> Self {
        let rule = &game.rules().mechanics.units[unit.kind];
        let enemies = || scene.observed.iter().map(|&index| &game.units()[index]);
        let allies = || scene.allies.iter().map(|&index| &game.units()[index]);
        let weapon_count = game.weapons(unit).count();
        let mut approaches =
            Vec::with_capacity((scene.observed.len() + scene.units.len()) * weapon_count);
        for enemy in enemies().chain(scene.units.iter()) {
            for (weapon, attack) in game.weapons(unit).enumerate() {
                let routes = (weights.leader_pressure > 0.0).then(|| {
                    let all = game.attack_origins(attack.category, attack.range);
                    let mut origins = Vec::new();
                    all[enemy.tile].for_each(|tile| {
                        if game.cost(unit.kind, tile) != u16::MAX {
                            origins.push(tile);
                        }
                    });
                    game.routes(unit.kind, &origins)
                });
                approaches.push(Approach {
                    weapon,
                    routes,
                    offense: RefCell::new(None),
                });
            }
        }
        // An existing provider cannot support itself. Treat remote providers'
        // future coverage as independent; current adjacent coverage is exact.
        let overlap_probability = (weights.support_recipients
            / scene.allies.len().saturating_sub(1).max(1) as f64
            * weights.healing_overlap)
            .min(1.0);
        let aura_overlap_probability = (weights.support_recipients
            / scene.allies.len().saturating_sub(1).max(1) as f64
            * weights.aura_overlap)
            .min(1.0);
        let mut support = Vec::new();
        if rule.healing > 0 || rule.thorn_aura > 0 {
            for ally in allies() {
                let ally_rule = &game.rules().mechanics.units[ally.kind];
                let mut covered = 0;
                let mut aura = 0;
                let mut remote_healing = Vec::new();
                let mut remote_auras = Vec::new();
                for other in allies() {
                    if other.id == ally.id {
                        continue;
                    }
                    let other_rule = &game.rules().mechanics.units[other.kind];
                    if game.board().distance(other.tile, ally.tile) == 1 {
                        covered = covered.max(other_rule.healing);
                        aura = aura.max(other_rule.thorn_aura);
                    } else {
                        if other_rule.healing > 0 {
                            remote_healing.push(other_rule.healing);
                        }
                        if other_rule.thorn_aura > 0 {
                            remote_auras.push(other_rule.thorn_aura);
                        }
                    }
                }
                // Anticipate support use without a nearby-enemy gate.
                let healing = expected_marginal_support(
                    (ally_rule.hp - ally.hp).max(0) as f64
                        + ally_rule.hp as f64 * weights.future_wounds,
                    covered,
                    rule.healing,
                    &mut remote_healing,
                    overlap_probability,
                );
                let extra_aura = expected_marginal_support(
                    f64::from(rule.thorn_aura),
                    aura,
                    rule.thorn_aura,
                    &mut remote_auras,
                    aura_overlap_probability,
                );
                // Classify the enemy's preferred exchange against this ally,
                // including the proposed aura, rather than any melee backup.
                let thorns = if extra_aura > 0.0 {
                    extra_aura
                        * expected_weighted_thorn_triggers(
                            enemies().chain(scene.units.iter()).enumerate().filter_map(
                                |(index, enemy)| {
                                    dominant_melee_strikes(
                                        game,
                                        enemy,
                                        ally,
                                        Some(aura.max(rule.thorn_aura)),
                                    )
                                    .map(|count| {
                                        (
                                            count,
                                            if index < scene.observed.len() {
                                                1.0
                                            } else {
                                                scene.weights[index - scene.observed.len()]
                                            },
                                        )
                                    })
                                },
                            ),
                            scene.allies.len() + 1,
                        )
                } else {
                    0.0
                };
                let goals: Vec<_> = game.board().adjacent[ally.tile]
                    .iter()
                    .copied()
                    .filter(|&t| game.cost(unit.kind, t) != u16::MAX)
                    .collect();
                support.push(Support {
                    healing,
                    thorns,
                    routes: game.routes(unit.kind, &goals),
                });
            }
        }
        let captures = game
            .owners()
            .iter()
            .enumerate()
            .filter(|(tile, _owner)| {
                game.captures(*tile, unit.side) && game.role_is(*tile, TerrainRole::Village)
            })
            .map(|(tile, _)| {
                let covered = allies()
                    .map(|ally| {
                        let route = game.routes(ally.kind, &[tile])[ally.tile];
                        if route == u16::MAX {
                            0.0
                        } else {
                            1.0 / (1.0
                                + f64::from(route)
                                    / game.rules().mechanics.units[ally.kind].r#move.max(1) as f64)
                        }
                    })
                    .fold(0.0, f64::max);
                (game.routes(unit.kind, &[tile]), covered)
            })
            .collect();
        let defenses = (0..scene.observed.len() + scene.units.len())
            .map(|_| RefCell::new(None))
            .collect();
        let friendly_count = scene.allies.len() + 1;
        let exchange_capacity = (scene.observed.len() + scene.units.len()).saturating_mul(8);
        Self {
            identity: RefCell::new(CombatIdentity::new(unit)),
            weights,
            scene,
            approaches,
            weapon_count,
            defenses,
            support,
            captures,
            friendly_count,
            exchanges: RefCell::new(rustc_hash::FxHashMap::with_capacity_and_hasher(
                exchange_capacity,
                Default::default(),
            )),
        }
    }
    pub(super) fn score(&self, game: &BattleView<'_, '_>, unit: &Unit) -> f64 {
        self.valuation(game, unit, false)
    }
    pub(super) fn recruitment_score(&self, game: &BattleView<'_, '_>, unit: &Unit) -> f64 {
        self.valuation(game, unit, true)
    }
    pub(super) fn recruitment_breakdown(&self, game: &BattleView<'_, '_>, unit: &Unit) -> Score {
        self.valuation(game, unit, true)
    }
    pub(super) fn breakdown(&self, game: &BattleView<'_, '_>, unit: &Unit) -> Score {
        self.valuation(game, unit, false)
    }
    fn valuation<O: From<Score>>(
        &self,
        game: &BattleView<'_, '_>,
        unit: &Unit,
        recruitment: bool,
    ) -> O {
        // Recruitment can reward village capture and configured mobility; earned
        // promotions retain village value. Enemy relevance never depends on distance.
        let rule = &game.rules().mechanics.units[unit.kind];
        let enemy_count = self.scene.observed.len();
        let enemies = || {
            self.scene
                .observed
                .iter()
                .map(|&index| &game.units()[index])
        };
        let identity = {
            let mut identity = self.identity.borrow_mut();
            if !identity.matches(unit) {
                *identity = CombatIdentity::new(unit);
                self.exchanges.borrow_mut().clear();
            }
            identity.clone()
        };
        let coefficients = self.weights;
        let mut leader_pressure: f64 = 0.0;
        let mut offense = 0.0;
        let mut retaliation = 0.0;
        let mut raw = 0.0;
        let mut incoming = 0.0;
        let mut access = 0.0;
        let mut weights = 0.0;
        let mut thorn_enemies = 0.0;
        let mut thorn_strikes = 0.0;
        let reachable = game.reachability(unit.kind, unit.tile);
        // All observed/forecast enemies are hostile: their hypothetical origins
        // cannot move an allied aura provider. Query the actual melee reflection
        // once for this placement and reuse its aura component across enemies.
        let defense_aura = enemies().chain(self.scene.units.iter()).find_map(|enemy| {
            game.weapons(enemy)
                .enumerate()
                .find(|(_, attack)| attack.category == Category::Melee)
                .map(|(weapon, attack)| {
                    game.thorn_damage(enemy, unit, weapon, enemy.tile) / i32::from(attack.count)
                        - rule.thorns
                })
        });
        for (index, enemy) in enemies().chain(self.scene.units.iter()).enumerate() {
            let weight = if index >= enemy_count {
                self.scene.weights[index - enemy_count]
            } else {
                1.0
            };
            let mut best_offense: f64 = 0.0;
            let mut reach: f64 = 0.0;
            for approach in
                &self.approaches[index * self.weapon_count..(index + 1) * self.weapon_count]
            {
                let route = approach
                    .routes
                    .as_ref()
                    .map_or(0, |routes| routes[unit.tile]);
                if route == u16::MAX {
                    continue;
                }
                let access = 1.0;
                let cached = recruitment
                    .then(|| {
                        approach
                            .offense
                            .borrow()
                            .as_ref()
                            .filter(|last| last.matches(reachable.source_key(), &identity))
                            .map(|last| (last.value, last.leader_damage, last.count))
                    })
                    .flatten();
                let (value, leader_damage, origin_count) = if let Some(cached) = cached {
                    cached
                } else {
                    let attack = game.weapon(unit, approach.weapon);
                    let runs = game.attack_origin_runs(
                        unit.kind,
                        unit.tile,
                        attack.category,
                        attack.range,
                        enemy.tile,
                    );
                    let origin_count: usize = runs.iter().map(|run| run.count).sum();
                    if origin_count == 0 {
                        continue;
                    }
                    let mut value = 0.0;
                    let mut leader_damage = 0.0;
                    let matchup = game.combat_matchup(unit, enemy);
                    for run in runs.iter() {
                        let reflection = game.thorn_damage(unit, enemy, approach.weapon, run.tile)
                            / i32::from(attack.count);
                        let forecast = matchup.at_origin(approach.weapon, run, reflection);
                        let damage = forecast.damage[0].round() as i32;
                        let response = forecast.damage[1].round() as i32;
                        let drain = damage.min(enemy.hp) as f64 * f64::from(attack.drain) / 100.0;
                        let exchange = forecast.effective[0] + drain * coefficients.drain
                            - response.min(unit.hp) as f64 * coefficients.exchange_retaliation
                            + forecast.death[1] * coefficients.kill
                            - if response as f64 >= unit.hp as f64 + drain {
                                unit.hp as f64
                            } else {
                                0.0
                            };
                        let delivered = damage.min(enemy.hp).max(0) as f64;
                        // Repeating each addition retains the original sum and
                        // tie breaks, while equivalent origins share a forecast.
                        for _ in 0..run.count {
                            value += exchange;
                            leader_damage += delivered;
                        }
                    }
                    value /= origin_count as f64;
                    if recruitment {
                        *approach.offense.borrow_mut() = Some(OffensiveAverage {
                            source: reachable.source_key(),
                            count: origin_count,
                            identity: identity.clone(),
                            value,
                            leader_damage,
                        });
                    }
                    (value, leader_damage, origin_count)
                };
                if recruitment && coefficients.leader_pressure > 0.0 && enemy.is_leader() {
                    let turns = f64::from(route) / rule.r#move.max(1) as f64;
                    let fraction = leader_damage / origin_count as f64 / enemy.hp.max(1) as f64;
                    leader_pressure = leader_pressure.max(fraction.min(1.0) / (1.0 + turns));
                }
                best_offense = best_offense.max(value * access);
                reach = reach.max(access);
            }
            offense += best_offense * weight;
            access += reach * weight;
            // A common unarmoured, terrain-neutral target takes the enemy's
            // strongest raw attack. This reference cannot change when armour,
            // retaliation or thorns makes that enemy switch weapons.
            let reference = game
                .weapons(enemy)
                .map(|a| a.damage * i32::from(a.count))
                .max()
                .unwrap_or(0);
            raw += f64::from(reference.max(1)) * weight;
            if let Some(preferred) = preferred_attack_cached(
                game,
                enemy,
                unit,
                defense_aura,
                Some(&self.defenses[index]),
                Some(&identity),
                Some((&self.exchanges, index)),
            ) {
                if rule.thorns > 0
                    && game.weapon(enemy, preferred.weapon).category == Category::Melee
                    && preferred.strikes > 0.0
                    && weight > 0.0
                {
                    thorn_enemies += weight;
                    thorn_strikes += preferred.strikes * weight;
                }
                let attack = game.weapon(enemy, preferred.weapon);
                incoming += preferred.damage * weight;
                // Own reflection is valued separately by army pressure.
                // Preserve ordinary retaliation and existing friendly aura.
                let response = preferred.response
                    - if attack.category == Category::Melee {
                        f64::from(rule.thorns) * preferred.strikes
                    } else {
                        0.0
                    };
                retaliation += response.max(0.0).min(enemy.hp as f64) * weight;
            }
            weights += weight;
        }
        if weights > 0.0 {
            offense /= weights;
            access /= weights;
            retaliation /= weights;
            raw /= weights;
            incoming /= weights;
        } else {
            offense = game
                .weapons(unit)
                .map(|a| (a.damage * i32::from(a.count)) as f64)
                .fold(0.0, f64::max);
            access = 1.0;
            raw = 1.0;
            incoming = 1.0;
        }
        // Effective HP in raw incoming damage, including per-strike armor. A
        // floor keeps complete immunity to one observed weapon finite.
        let durability = unit.hp.max(1) as f64 * raw / incoming.max(raw * 0.2).max(1.0);
        let support = support_coverage(
            self.support
                .iter()
                .filter_map(|s| (s.routes[unit.tile] != u16::MAX).then_some(s.healing + s.thorns)),
            coefficients.support_recipients,
        );
        let aura_thorns = support_coverage(
            self.support
                .iter()
                .filter_map(|s| (s.routes[unit.tile] != u16::MAX).then_some(s.thorns)),
            coefficients.support_recipients,
        );
        let healing = (support - aura_thorns).max(0.0);
        let captures = self
            .captures
            .iter()
            .filter(|(routes, _)| routes[unit.tile] != u16::MAX)
            .map(|(routes, covered)| {
                (1.0 / (1.0 + f64::from(routes[unit.tile]) / rule.r#move.max(1) as f64) - covered)
                    .max(0.0)
            })
            .sum::<f64>();
        let thorns = if thorn_enemies > 0.0 {
            f64::from(rule.thorns)
                * ((thorn_enemies / self.friendly_count.max(1) as f64).min(6.0) * thorn_strikes
                    / thorn_enemies)
        } else {
            0.0
        };
        // Recruitment buys a unit's future contributions: endurance helps an
        // effective attacker deliver them repeatedly. Adding HP independently
        // can instead buy durable units with little useful damage. Support
        // remains separate from sustained weapon contribution.
        // Earned promotions retain their existing valuation.
        let combat = if recruitment {
            // Linear endurance values sustained contribution. Normalize to
            // the former square-root score at 20 effective HP so retaliation
            // and support keep the same relative scale at that reference.
            let endurance = durability / coefficients.endurance_reference_hp.sqrt();
            offense * coefficients.offense * endurance
                + retaliation * access * coefficients.retaliation
        } else {
            offense * 1.5 + durability * access * 0.35 + retaliation * access * 0.5
        };
        let mobility = if recruitment && coefficients.speed > 0.0 {
            terrain_speed(game, unit)
        } else {
            0.0
        };
        let movement_bonus = if recruitment {
            movement_bonus(captures, mobility, leader_pressure, coefficients)
        } else {
            captures * 0.1
        };
        // Keep the old arithmetic when the support weights agree. The aura
        // adjustment exposes a separate coefficient without changing defaults.
        let total = combat * (1.0 + movement_bonus)
            + (support + thorns) * coefficients.healing
            + aura_thorns * (coefficients.aura_thorns - coefficients.healing)
            + thorns * (coefficients.intrinsic_thorns - coefficients.healing);
        Score {
            offense,
            effective_hp: durability,
            access,
            retaliation,
            support,
            thorns,
            mobility,
            captures,
            leader_pressure,
            movement_bonus,
            healing,
            aura_thorns,
            total,
        }
        .into()
    }
}

// Intrinsic speed uses typical traversable terrain cost, not distance to troops.
fn terrain_speed(game: &BattleView<'_, '_>, unit: &Unit) -> f64 {
    let costs: Vec<_> = (0..game.board().ids.len())
        .map(|t| game.cost(unit.kind, t))
        .filter(|&c| c != u16::MAX)
        .collect();
    let mean = costs.iter().map(|&x| f64::from(x)).sum::<f64>() / costs.len().max(1) as f64;
    ((game.rules().mechanics.units[unit.kind].r#move as f64 / mean.max(1.0) - 3.0) / 3.0)
        .clamp(0.0, 1.0)
}

// Speed and leader delivery describe alternative uses of the same mobility.
// Take their maximum, then add distinct marginal village value, with one cap.
fn movement_bonus(captures: f64, speed: f64, leader: f64, weights: RecruitmentWeights) -> f64 {
    let bonus = captures * weights.village_capture
        + (speed * weights.speed).max(leader * weights.leader_pressure);
    weights.movement_cap.map_or(bonus, |cap| bonus.min(cap))
}

// Forecast types carry fractional troop weights. Ranged preferences contribute
// no melee triggers; the six-attacker cap remains the same as observed pressure.
fn expected_weighted_thorn_triggers(
    strikes: impl Iterator<Item = (f64, f64)>,
    friendly_count: usize,
) -> f64 {
    let (enemies, strikes) = strikes
        .filter(|&(count, weight)| count > 0.0 && weight > 0.0)
        .fold((0.0, 0.0), |(enemies, strikes), (count, weight)| {
            (enemies + weight, strikes + count * weight)
        });
    if enemies == 0.0 {
        0.0
    } else {
        (enemies / friendly_count.max(1) as f64).min(6.0) * strikes / enemies
    }
}

// Representative legal origins let melee and ranged weapons compete even when
// the enemy is currently distant. As with engine best_attack, choose by damage,
// drain, retaliation, kills and survival, not raw damage or strike count alone.
// An aura override replaces actual adjacent aura in previews, so promotions do
// not count their old aura as well as their proposed new one.
fn dominant_melee_strikes(
    game: &BattleView<'_, '_>,
    enemy: &Unit,
    target: &Unit,
    aura: Option<i32>,
) -> Option<f64> {
    preferred_attack(game, enemy, target, aura).map(|preferred| {
        let attack = game.weapon(enemy, preferred.weapon);
        if attack.category == Category::Melee {
            preferred.strikes
        } else {
            0.0
        }
    })
}

struct PreferredAttack {
    weapon: usize,
    damage: f64,
    response: f64,
    strikes: f64,
}

#[inline]
fn prefer_attack(best: &mut Option<(f64, PreferredAttack)>, weapon: usize, average: ExchangeValue) {
    if best
        .as_ref()
        .is_none_or(|(score, _)| average.value > *score)
    {
        *best = Some((
            average.value,
            PreferredAttack {
                weapon,
                damage: average.damage,
                response: average.response,
                strikes: average.strikes,
            },
        ));
    }
}

fn preferred_attack(
    game: &BattleView<'_, '_>,
    enemy: &Unit,
    target: &Unit,
    aura: Option<i32>,
) -> Option<PreferredAttack> {
    preferred_attack_cached(game, enemy, target, aura, None, None, None)
}

fn preferred_attack_cached(
    game: &BattleView<'_, '_>,
    enemy: &Unit,
    target: &Unit,
    aura: Option<i32>,
    cache: Option<&RefCell<Option<DefenseCache>>>,
    identity: Option<&CombatIdentity>,
    exchanges: Option<(&RefCell<rustc_hash::FxHashMap<u128, ExchangeValue>>, usize)>,
) -> Option<PreferredAttack> {
    let mut cache = cache.map(|cache| cache.borrow_mut());
    if let Some(cache) = cache.as_mut() {
        let matches = cache.as_ref().is_some_and(|entry| {
            identity.map_or_else(
                || entry.identity.matches(target),
                |identity| entry.identity == *identity,
            )
        });
        if !matches {
            let identity = identity
                .cloned()
                .unwrap_or_else(|| CombatIdentity::new(target));
            **cache = Some(DefenseCache::new(game, enemy, target, identity));
        }
    }

    let matchup = game.combat_matchup(enemy, target);
    let mut best = None;
    let reachable = cache
        .as_ref()
        .map(|cache| cache.as_ref().unwrap().reachable.clone())
        .unwrap_or_else(|| game.reachability(enemy.kind, enemy.tile));
    let mut reply_ranges = [ReplyRanges::default(); 2];
    for reply in game.weapons(target) {
        reply_ranges[reply.category.index()].insert(reply.range);
    }
    for (weapon, attack) in game.weapons(enemy).enumerate() {
        let fixed_reflection = if let Some(aura) = aura {
            Some(if attack.category == Category::Melee {
                game.rules().mechanics.units[target.kind].thorns + aura
            } else {
                0
            })
        } else if !game.allied(enemy, target) {
            // A hostile attacker cannot also be an allied aura provider. Moving
            // its hypothetical origin therefore cannot change reflection.
            Some(game.thorn_damage(enemy, target, weapon, enemy.tile) / i32::from(attack.count))
        } else {
            None
        };
        let reflection_at = |origin| {
            fixed_reflection.unwrap_or_else(|| {
                game.thorn_damage(enemy, target, weapon, origin) / i32::from(attack.count)
            })
        };
        let grouped = fixed_reflection.map(|_| {
            game.attack_origin_runs(
                enemy.kind,
                enemy.tile,
                attack.category,
                attack.range,
                target.tile,
            )
        });
        if let (Some(cache), Some(runs), Some(reflection)) =
            (cache.as_mut(), grouped.as_ref(), fixed_reflection)
        {
            let cache = cache.as_mut().unwrap();
            if let Some((index, average)) =
                cache
                    .averages
                    .iter()
                    .enumerate()
                    .find_map(|(index, entry)| {
                        entry
                            .as_ref()
                            .filter(|entry| {
                                entry.matches(
                                    weapon,
                                    game.terrain()[target.tile],
                                    reflection,
                                    runs,
                                    &reply_ranges[attack.category.index()],
                                )
                            })
                            .map(|entry| (index, entry.value))
                    })
            {
                cache.next_average = index ^ 1;
                prefer_attack(&mut best, weapon, average);
                continue;
            }
        }
        // Opposite directions share category and line of fire. Only crossing a
        // retaliation range boundary changes the available replies. Cache each
        // exchange, then add it in the original order: even different origin
        // counts and terrain sequences retain their exact floating-point sums.
        let mut sums = [0.0; 4];
        let mut origin_count = 0usize;
        if fixed_reflection.is_some() {
            let reflection = fixed_reflection.unwrap();
            let runs = grouped.as_ref().unwrap();
            for run in runs.iter() {
                let calculate = || {
                    let forecast = matchup.at_origin(weapon, run, reflection);
                    ExchangeValue::from_forecast(&forecast)
                };
                let value = if let Some(cache) = cache.as_mut() {
                    let key = DefenseKey {
                        weapon,
                        terrain: game.terrain()[target.tile],
                        origin_terrain: run.terrain,
                        reply_class: reply_ranges[attack.category.index()].class(run.distance),
                        reflection,
                    };
                    cache.as_mut().unwrap().exchange(key, || {
                        if let Some((exchanges, packed)) = exchanges
                            .and_then(|(table, index)| key.packed(index).map(|key| (table, key)))
                        {
                            *exchanges
                                .borrow_mut()
                                .entry(packed)
                                .or_insert_with(calculate)
                        } else {
                            calculate()
                        }
                    })
                } else {
                    calculate()
                };
                origin_count += run.count;
                let values = [value.value, value.damage, value.response, value.strikes];
                // Independent lanes vectorize without reassociating additions
                // across origins: every lane keeps the original scalar order.
                for _ in 0..run.count {
                    for lane in 0..4 {
                        sums[lane] += values[lane];
                    }
                }
            }
        } else {
            let origins = game.attack_origins(attack.category, attack.range);
            origins[target.tile].for_each(|origin| {
                if !reachable.contains(origin) || game.cost(enemy.kind, origin) == u16::MAX {
                    return;
                }
                origin_count += 1;
                let forecast =
                    game.exchange_with_thorns(enemy, target, weapon, origin, reflection_at(origin));
                let value = ExchangeValue::from_forecast(&forecast);
                let values = [value.value, value.damage, value.response, value.strikes];
                for lane in 0..4 {
                    sums[lane] += values[lane];
                }
            });
        }
        if origin_count == 0 {
            continue;
        }
        let count = origin_count as f64;
        let average = ExchangeValue {
            value: sums[0] / count,
            damage: sums[1] / count,
            response: sums[2] / count,
            strikes: sums[3] / count,
        };
        if let (Some(cache), Some(runs), Some(reflection)) =
            (cache.as_mut(), grouped, fixed_reflection)
        {
            let cache = cache.as_mut().unwrap();
            cache.averages[cache.next_average] = Some(DefenseAverage {
                weapon,
                terrain: game.terrain()[target.tile],
                reflection,
                runs,
                value: average,
            });
            cache.next_average ^= 1;
        }
        prefer_attack(&mut best, weapon, average);
    }
    best.map(|(_, attack)| attack)
}

// With equal providers this is marginal_value * (1 - overlap_probability)^count.
// Descending strengths also handle upgrades correctly: stronger existing buffs
// erase the gain; weaker ones leave only the candidate's incremental benefit.
fn expected_marginal_support(
    demand: f64,
    covered: i32,
    candidate: i32,
    others: &mut [i32],
    overlap_probability: f64,
) -> f64 {
    others.order_unstable_by(|a, b| b.cmp(a));
    let mut uncovered_probability = 1.0;
    let mut value = 0.0;
    for &strength in others.iter() {
        value += uncovered_probability
            * overlap_probability
            * marginal_healing(demand, covered.max(strength), candidate);
        uncovered_probability *= 1.0 - overlap_probability;
    }
    value + uncovered_probability * marginal_healing(demand, covered, candidate)
}

// Recruitment estimates typical coverage, rather than summing mutually
// exclusive future positions. This is a prior, not a measured runtime statistic.
fn support_coverage(values: impl Iterator<Item = f64>, recipients: f64) -> f64 {
    let (sum, count) = values.fold((0.0, 0usize), |(sum, count), value| {
        (sum + value, count + 1)
    });
    if count == 0 {
        0.0
    } else {
        sum * recipients.min(count as f64) / count as f64
    }
}

fn marginal_healing(demand: f64, covered: i32, candidate: i32) -> f64 {
    (demand.min(f64::from(candidate)) - demand.min(f64::from(covered))).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::recruiter::fixtures;
    use crate::data::Hex;
    #[test]
    fn movement_controls_are_optional_separate_and_share_a_cap() {
        let (mut rules, target, enemy) = thorn_fixture();

        rules.units[target].r#move = 6;
        let (mut game, target_id, enemy_id) = thorn_battle(&rules, target, enemy);
        fixtures::units(&mut game, |units| {
            units
                .iter_mut()
                .find(|unit| unit.side == TurnPhase::Ai)
                .unwrap()
                .role = Some(Role::Leader);
        });
        let a = game.site(&Hex::new(1, 4));
        let b = game.site(&Hex::new(8, 4));
        fixtures::units(&mut game, |units| {
            units.iter_mut().find(|u| u.id == target_id).unwrap().tile = a;
        });
        fixtures::units(&mut game, |units| {
            units.iter_mut().find(|u| u.id == enemy_id).unwrap().tile = b;
        });
        fixtures::edit(&mut game, |state| {
            for tile in &mut state.tiles {
                tile.owner = Some("player".into());
            }
        });
        let view = BattleView::new(&game);
        let unit = game.unit(target_id).unwrap();
        let score = |weights| {
            Assessment::with_weights(&view, unit, false, weights).recruitment_breakdown(&view, unit)
        };
        let baseline = score(RecruitmentWeights::default());
        let speed = score(RecruitmentWeights {
            speed: 0.5,
            ..RecruitmentWeights::default()
        });
        let leader = score(RecruitmentWeights {
            leader_pressure: 2.0,
            ..RecruitmentWeights::default()
        });
        assert_eq!(baseline.movement_bonus, 0.0);
        assert!(speed.movement_bonus > 0.0 && leader.movement_bonus > 0.0);
        assert_eq!(speed.leader_pressure, 0.0);
        assert_eq!(leader.mobility, 0.0);
        assert_eq!(baseline.offense, leader.offense);
        assert_eq!(baseline.effective_hp, speed.effective_hp);
        let capped = score(RecruitmentWeights {
            speed: 100.0,
            leader_pressure: 100.0,
            movement_cap: Some(0.25),
            ..RecruitmentWeights::default()
        });
        assert_eq!(capped.movement_bonus, 0.25);
        assert_eq!(
            movement_bonus(
                0.0,
                1.0,
                1.0,
                RecruitmentWeights {
                    speed: 0.4,
                    leader_pressure: 0.4,
                    ..RecruitmentWeights::default()
                }
            ),
            0.4
        );
    }

    #[test]
    fn thorn_pressure_caps_attackers_before_strikes_and_ignores_ranged_enemies() {
        assert_eq!(
            expected_weighted_thorn_triggers([(3.0, 1.0); 20].into_iter(), 1),
            18.0
        );
        assert_eq!(
            expected_weighted_thorn_triggers([(1.0, 1.0)].into_iter(), 10),
            0.1
        );
        assert_eq!(
            expected_weighted_thorn_triggers([(0.0, 1.0); 20].into_iter(), 1),
            0.0
        );
        assert_eq!(
            expected_weighted_thorn_triggers(
                [(0.0, 1.0), (0.0, 1.0), (1.0, 1.0), (3.0, 1.0)].into_iter(),
                4
            ),
            1.0
        );
        assert_eq!(expected_weighted_thorn_triggers([].into_iter(), 1), 0.0);
    }

    fn thorn_fixture() -> (Rules, usize, usize) {
        let mut rules = Rules::load().unwrap();
        let target = rules.types["swordsman"];
        let enemy = rules.types["archer"];
        for kind in [target, enemy] {
            let rule = &mut rules.units[kind];
            rule.hp = 100;
            rule.thorns = 0;
            rule.thorn_aura = 0;
            rule.healing = 0;
            rule.attacks.truncate(1);
            rule.attacks[0].category = Category::Melee;
            rule.attacks[0].range = 1;
            rule.attacks[0].damage_type = DamageType::Blade;
            rule.attacks[0].damage = 2;
            rule.attacks[0].count = 1;
            rule.attacks[0].drain = 0;
            rule.damage_modifiers = DamageModifiers {
                blade: 0,
                pierce: 0,
                impact: 0,
            };
            for environment in &mut rule.environment {
                *environment = [[0; 3]; 2];
            }
        }
        rules.units[enemy].attacks[0].damage = 8;
        rules.units[enemy].attacks[0].count = 3;
        (rules, target, enemy)
    }

    fn thorn_battle(
        rules: &Rules,
        target: usize,
        enemy: usize,
    ) -> (crate::logic::Game<'_>, u32, u32) {
        let mut game = crate::logic::Game::new(rules, "mirror");
        fixtures::units(&mut game, |units| {
            units.clear();
        });
        let tile = game.site(&Hex::new(4, 4));
        let target = fixtures::add(&mut game, target, TurnPhase::Player, tile);
        let tile = game.site(&Hex::new(5, 4));
        let enemy = fixtures::add(&mut game, enemy, TurnPhase::Ai, tile);
        (game, target, enemy)
    }

    #[test]
    fn defensive_exchange_cache_preserves_placements_ranges_auras_and_wounds() {
        let (mut rules, target, enemy) = thorn_fixture();
        let mut ranged = rules.units[enemy].attacks[0].clone();
        ranged.id = "test-ranged".into();
        ranged.category = Category::Ranged;
        ranged.range = 2;
        rules.units[enemy].attacks.push(ranged.clone());
        rules.units[target].attacks.push(ranged.clone());
        ranged.id = "test-short-reply".into();
        ranged.range = 1;
        ranged.damage += 3;
        rules.units[target].attacks.push(ranged);
        rules.units[target].thorns = 2;
        rules.units[target].thorn_aura = 3;
        rules.units[target].environment[rules.terrains["forest"]][0][0] = -3;
        let (mut game, target_id, enemy_id) = thorn_battle(&rules, target, enemy);
        let tile = game.site(&Hex::new(4, 5));
        fixtures::add(&mut game, target, TurnPhase::Player, tile);
        let view = BattleView::new(&game);
        let enemy = game.unit(enemy_id).unwrap();
        let mut target = game.unit(target_id).unwrap().clone();
        let cache = RefCell::new(None);
        let signature = |attack: Option<PreferredAttack>| {
            attack.map(|a| {
                (
                    a.weapon,
                    a.damage.to_bits(),
                    a.response.to_bits(),
                    a.strikes.to_bits(),
                )
            })
        };
        // One cache spans changes in candidate HP, actual aura coverage and
        // proposed aura overrides. Each result is checked against engine queries
        // without the placement cache, including absent legal attack origins.
        for hp in [target.hp, target.hp - 17] {
            target.hp = hp;
            for tile in 0..game.board().ids.len() {
                target.tile = tile;
                for aura in [None, Some(7)] {
                    assert_eq!(
                        signature(preferred_attack_cached(
                            &view,
                            enemy,
                            &target,
                            aura,
                            Some(&cache),
                            None,
                            None,
                        )),
                        signature(preferred_attack(&view, enemy, &target, aura)),
                        "tile {tile}, hp {hp}, aura {aura:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn dominant_attack_accounts_for_ranged_alternatives_armor_and_proposed_aura() {
        let (mut rules, target, enemy) = thorn_fixture();
        let mut ranged = rules.units[enemy].attacks[0].clone();
        ranged.id = "test-ranged-alternative".into();
        ranged.category = Category::Ranged;
        ranged.range = 2;
        ranged.damage_type = DamageType::Pierce;
        ranged.damage = 20;
        ranged.count = 1;
        rules.units[enemy].attacks.push(ranged);
        {
            let (game, target, enemy) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let target = game.unit(target).unwrap();
            let enemy = game.unit(enemy).unwrap();
            assert!(
                (dominant_melee_strikes(&view, enemy, target, None).unwrap() - 3.0).abs() < 1e-9
            );
            // The buff makes melee a worse exchange, so the enemy avoids it.
            assert_eq!(
                dominant_melee_strikes(&view, enemy, target, Some(2)),
                Some(0.0)
            );
        }
        rules.units[target].damage_modifiers.blade = -2;
        let (game, target, enemy) = thorn_battle(&rules, target, enemy);
        let view = BattleView::new(&game);
        assert_eq!(
            dominant_melee_strikes(
                &view,
                game.unit(enemy).unwrap(),
                game.unit(target).unwrap(),
                None
            ),
            Some(0.0)
        );
    }

    #[test]
    fn recruitment_anticipates_legal_enemy_rosters_until_observed_troops_replace_them() {
        let (mut rules, target, enemy) = thorn_fixture();

        rules.units[target].attacks[0].category = Category::Ranged;
        rules.units[target].attacks[0].range = 2;
        rules.units[target].attacks[0].damage = 12;
        rules.units[target].attacks[0].spread = 0;
        let resistant = rules.types["scout"];
        let observed = rules.types["spearman"];
        for kind in [resistant, observed] {
            let mut rule = rules.units[enemy].clone();
            rule.id = rules.units[kind].id.clone();
            rules.units[kind] = rule;
            rules.route_classes[kind] = kind;
        }
        rules.units[resistant].damage_modifiers.blade = -8;
        let (mut game, target, _) = thorn_battle(&rules, target, enemy);
        fixtures::units(&mut game, |units| {
            units
                .iter_mut()
                .find(|unit| unit.side == TurnPhase::Ai)
                .unwrap()
                .role = Some(Role::Leader);
        });
        fixtures::edit(&mut game, |state| {
            for tile in &mut state.tiles {
                tile.owner = Some("player".into());
            }
        });
        let assess = |game: &crate::logic::Game<'_>| {
            let view = BattleView::new(game);
            let unit = game.unit(target).unwrap();
            let assessment = Assessment::with_forecast(&view, unit, true);
            (
                assessment.recruitment_breakdown(&view, unit).offense,
                Assessment::new(&view, unit).breakdown(&view, unit).total,
                Assessment::new(&view, unit)
                    .recruitment_breakdown(&view, unit)
                    .total,
            )
        };
        game.configure_arena(Some([vec![], vec![]]), None);
        let neutral = assess(&game);
        game.configure_arena(Some([vec![], vec![resistant]]), None);
        let anticipated = assess(&game);
        assert!(anticipated.0 < neutral.0);
        // Free promotions continue to consider the actual battlefield only.
        assert_eq!(anticipated.1, neutral.1);
        // Situational never reads the hidden future recruit pool.
        assert_eq!(anticipated.2, neutral.2);
        for site in [
            Hex::new(5, 3),
            Hex::new(6, 3),
            Hex::new(6, 4),
            Hex::new(6, 5),
        ] {
            let tile = game.site(&site);
            fixtures::add(&mut game, observed, TurnPhase::Ai, tile);
        }
        let observed_with_prior = assess(&game);
        game.configure_arena(Some([vec![], vec![]]), None);
        assert_eq!(observed_with_prior, assess(&game));
    }

    #[test]
    fn only_predictor_credits_thorns_against_future_melee_troops() {
        for aura in [false, true] {
            let (mut rules, target, enemy) = thorn_fixture();
            let melee = rules.types["scout"];
            rules.units[melee] = rules.units[enemy].clone();

            rules.units[enemy].attacks[0].category = Category::Ranged;
            rules.units[enemy].attacks[0].range = 2;
            if aura {
                rules.units[target].thorn_aura = 4;
            } else {
                rules.units[target].thorns = 4;
            }
            let (mut game, target_id, _) = thorn_battle(&rules, target, enemy);
            fixtures::units(&mut game, |units| {
                units
                    .iter_mut()
                    .find(|unit| unit.side == TurnPhase::Ai)
                    .unwrap()
                    .role = Some(Role::Leader);
            });
            if aura {
                let tile = game.site(&Hex::new(3, 4));
                fixtures::add(&mut game, target, TurnPhase::Player, tile);
            }
            game.configure_arena(Some([vec![], vec![melee]]), None);
            let view = BattleView::new(&game);
            let unit = game.unit(target_id).unwrap();
            let situational = Assessment::new(&view, unit).recruitment_breakdown(&view, unit);
            let predictor =
                Assessment::with_forecast(&view, unit, true).recruitment_breakdown(&view, unit);
            if aura {
                assert_eq!(situational.support, 0.0);
                assert!(predictor.support > 0.0);
            } else {
                assert_eq!(situational.thorns, 0.0);
                assert!(predictor.thorns > 0.0);
            }
        }
    }

    #[test]
    fn effective_hp_and_thorn_weapon_preferences_do_not_choose_the_nearest_terrain() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units[target].attacks[0].spread = 0;
        rules.units[enemy].attacks[0].spread = 0;
        let mut ranged = rules.units[enemy].attacks[0].clone();
        ranged.id = "test-ranged-alternative".into();
        ranged.category = Category::Ranged;
        ranged.range = 2;
        ranged.damage = 20;
        ranged.count = 1;
        rules.units[enemy].attacks.push(ranged);
        let forest = rules.terrains["forest"];
        rules.units[enemy].environment[forest][Category::Melee.index()]
            [DamageType::Blade.index()] = 10;
        let (mut game, target, enemy) = thorn_battle(&rules, target, enemy);
        fixtures::edit(&mut game, |state| {
            for tile in &mut state.tiles {
                tile.terrain = "plains".into();
            }
        });
        let tile = game.site(&Hex::new(3, 4));
        fixtures::terrain(&mut game, tile, forest);
        let mut health = Vec::new();
        for site in [Hex::new(1, 4), Hex::new(8, 4)] {
            let tile = game.site(&site);
            let index = game.index(enemy).unwrap();
            fixtures::units(&mut game, |units| {
                units[index].tile = tile;
            });
            let view = BattleView::new(&game);
            let defender = game.unit(target).unwrap();
            let attacker = game.unit(enemy).unwrap();
            assert_eq!(
                preferred_attack(&view, attacker, defender, None)
                    .unwrap()
                    .weapon,
                0
            );
            assert_eq!(
                preferred_attack(&view, attacker, defender, Some(1))
                    .unwrap()
                    .weapon,
                1
            );
            health.push(
                Assessment::new(&view, defender)
                    .recruitment_breakdown(&view, defender)
                    .effective_hp,
            );
        }
        assert!((health[0] - health[1]).abs() < 1e-9);
    }

    #[test]
    fn effective_hp_keeps_a_common_reference_when_armour_changes_preferred_weapon() {
        let (mut rules, target, enemy) = thorn_fixture();
        let mut ranged = rules.units[enemy].attacks[0].clone();
        ranged.id = "test-ranged-alternative".into();
        ranged.category = Category::Ranged;
        ranged.range = 2;
        ranged.damage_type = DamageType::Pierce;
        ranged.damage = 20;
        ranged.count = 1;
        rules.units[enemy].attacks.push(ranged);
        for (blade, expected_weapon, expected_hp) in [(0, 0, 100.0), (-2, 1, 120.0)] {
            rules.units[target].damage_modifiers.blade = blade;
            let (game, target, enemy) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            assert_eq!(
                preferred_attack(&view, game.unit(enemy).unwrap(), unit, None)
                    .unwrap()
                    .weapon,
                expected_weapon
            );
            let score = Assessment::new(&view, unit).breakdown(&view, unit);
            assert!((score.effective_hp - expected_hp).abs() < 1e-9);
        }
    }

    #[test]
    fn effective_hp_values_repeated_ranged_attacks_and_per_strike_armour() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units[target].hp = 40;
        rules.units[enemy].attacks[0].category = Category::Ranged;
        rules.units[enemy].attacks[0].range = 2;
        for (damage, count, armour, expected) in [
            (10, 1, 0, 40.0),
            (10, 1, -2, 50.0),
            (5, 2, -2, 40.0 * 10.0 / 6.0),
            (5, 2, -5, 200.0),
        ] {
            rules.units[enemy].attacks[0].damage = damage;
            rules.units[enemy].attacks[0].count = count;
            rules.units[target].damage_modifiers.blade = armour;
            let (game, target, _) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            let score = Assessment::new(&view, unit).breakdown(&view, unit);
            assert!((score.effective_hp - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn recruitment_values_endurance_together_with_damage_without_changing_promotions() {
        let mut scores = Vec::new();
        // The old additive score prefers the much tougher, weak attacker.
        // Recruiting should instead value the useful damage it can sustain.
        for (hp, damage) in [(100, 2), (30, 8)] {
            let (mut rules, target, enemy) = thorn_fixture();
            rules.units[target].hp = hp;
            let attack = &mut rules.units[target].attacks[0];
            attack.category = Category::Ranged;
            attack.range = 2;
            attack.damage = damage;
            attack.spread = 0;
            let (game, target, _) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            let assessment = Assessment::new(&view, unit);
            scores.push((
                assessment.score(&view, unit),
                assessment.recruitment_breakdown(&view, unit).total,
            ));
        }
        assert!(scores[0].0 > scores[1].0);
        assert!(scores[1].1 > scores[0].1);
    }

    #[test]
    fn recruitment_values_sustained_damage_linearly_with_endurance() {
        let mut scores = Vec::new();
        for hp in [40, 80] {
            let (mut rules, target, enemy) = thorn_fixture();
            rules.units[target].hp = 80;
            let attack = &mut rules.units[target].attacks[0];
            attack.category = Category::Ranged;
            attack.range = 2;
            attack.damage = 8;
            attack.spread = 0;
            let (mut game, target_id, _) = thorn_battle(&rules, target, enemy);
            fixtures::units(&mut game, |units| {
                units.iter_mut().find(|u| u.id == target_id).unwrap().hp = hp;
            });
            fixtures::edit(&mut game, |state| {
                for tile in &mut state.tiles {
                    tile.owner = Some("player".into());
                }
            });
            let view = BattleView::new(&game);
            let unit = game.unit(target_id).unwrap();
            let score = Assessment::new(&view, unit).recruitment_breakdown(&view, unit);
            assert_eq!(score.retaliation, 0.0);
            assert_eq!(score.captures, 0.0);
            scores.push(score.total);
        }
        assert!(scores[0] > 0.0);
        assert!((scores[1] - 2.0 * scores[0]).abs() < 1e-9);
    }

    #[test]
    fn recruitment_combat_and_future_healing_ignore_enemy_proximity() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units[target].attacks[0].category = Category::Ranged;
        rules.units[target].attacks[0].range = 2;
        rules.units[target].attacks[0].damage = 12;
        rules.units[target].healing = 4;
        rules.units[target].damage_modifiers.blade = -2;
        let other = rules.types["scout"];
        rules.units[other] = rules.units[enemy].clone();
        rules.units[other].damage_modifiers.blade = -8;
        rules.units[other].attacks[0].damage = 20;
        rules.units[other].attacks[0].count = 1;
        rules.units[other].attacks[0].damage_type = DamageType::Pierce;
        let (mut game, target, enemy) = thorn_battle(&rules, target, enemy);
        let plains = rules.terrains["plains"];
        fixtures::edit(&mut game, |state| {
            for tile in &mut state.tiles {
                tile.terrain = rules.terrain_names[plains].clone();
            }
        });
        fixtures::edit(&mut game, |state| {
            for tile in &mut state.tiles {
                tile.owner = Some("player".into());
            }
        });
        let ally_tile = game.site(&Hex::new(3, 4));
        let ally_kind = game.unit(target).unwrap().kind;
        fixtures::add(&mut game, ally_kind, TurnPhase::Player, ally_tile);
        let other_tile = game.site(&Hex::new(8, 5));
        let other = fixtures::add(&mut game, other, TurnPhase::Ai, other_tile);
        let mut samples = Vec::new();
        for (first, second) in [
            (Hex::new(5, 4), Hex::new(8, 5)),
            (Hex::new(8, 4), Hex::new(5, 5)),
            (Hex::new(8, 3), Hex::new(8, 5)),
        ] {
            for (id, site) in [(enemy, first), (other, second)] {
                let tile = game.site(&site);
                let index = game.index(id).unwrap();
                fixtures::units(&mut game, |units| {
                    units[index].tile = tile;
                });
            }
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            samples.push(Assessment::new(&view, unit).recruitment_breakdown(&view, unit));
        }
        assert!(samples[0].support > 0.0);
        assert_eq!(samples[0].captures, 0.0);
        for sample in &samples[1..] {
            assert!((samples[0].offense - sample.offense).abs() < 1e-9);
            assert!((samples[0].effective_hp - sample.effective_hp).abs() < 1e-9);
            assert!((samples[0].support - sample.support).abs() < 1e-9);
            assert!((samples[0].total - sample.total).abs() < 1e-9);
        }
    }

    #[test]
    fn recruitment_rewards_speed_only_for_village_reach() {
        for healing in [0, 4] {
            let mut samples = Vec::new();
            for movement in [1, 6] {
                let (mut rules, target, enemy) = thorn_fixture();
                rules.units[target].r#move = movement;
                rules.units[target].healing = healing;
                let (mut game, target_id, enemy_id) = thorn_battle(&rules, target, enemy);
                let start = game.site(&Hex::new(1, 4));
                let end = game.site(&Hex::new(8, 4));
                fixtures::units(&mut game, |units| {
                    for unit in units {
                        unit.tile = if unit.id == target_id { start } else { end };
                    }
                });
                assert_eq!(game.unit(enemy_id).unwrap().tile, end);
                if healing > 0 {
                    let tile = game.site(&Hex::new(6, 4));
                    let ally = fixtures::add(&mut game, target, TurnPhase::Player, tile);
                    fixtures::units(&mut game, |units| {
                        units.iter_mut().find(|u| u.id == ally).unwrap().hp = 50;
                    });
                }
                fixtures::edit(&mut game, |state| {
                    for tile in &mut state.tiles {
                        tile.owner = Some("player".into());
                    }
                });
                let (recruit, promotion) = {
                    let view = BattleView::new(&game);
                    let unit = game.unit(target_id).unwrap();
                    let assessment = Assessment::new(&view, unit);
                    (
                        assessment.recruitment_breakdown(&view, unit),
                        assessment.score(&view, unit),
                    )
                };
                assert_eq!(recruit.mobility, 0.0);
                assert_eq!(recruit.access, 1.0);
                assert_eq!(recruit.captures, 0.0);
                let village = (0..game.terrain().len())
                    .find(|&tile| {
                        game.role_is(tile, TerrainRole::Village)
                            && game.board().distance(start, tile) > 2
                    })
                    .unwrap();
                fixtures::edit(&mut game, |state| state.tiles[village].owner = None);
                let view = BattleView::new(&game);
                let unit = game.unit(target_id).unwrap();
                let capture = Assessment::new(&view, unit).recruitment_breakdown(&view, unit);
                samples.push((recruit, promotion, capture));
            }
            assert!((samples[0].0.total - samples[1].0.total).abs() < 1e-9);
            assert!((samples[1].1 - samples[0].1).abs() < 1e-9);
            if healing == 0 {
                assert!(samples[1].2.captures > samples[0].2.captures);
                assert!(samples[1].2.total > samples[0].2.total);
            } else {
                assert!(samples[0].0.support > 0.0);
                assert_eq!(samples[0].0.support, samples[1].0.support);
            }
        }
    }

    #[test]
    fn intrinsic_thorns_use_pressure_without_also_counting_as_weapon_retaliation() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units[target].thorns = 3;
        let (game, target, _) = thorn_battle(&rules, target, enemy);
        let unit = game.unit(target).unwrap();
        let view = BattleView::new(&game);
        let score = Assessment::new(&view, unit).breakdown(&view, unit);
        assert!((score.thorns - 9.0).abs() < 1e-9);
        assert!((score.retaliation - 2.0).abs() < 1e-9);
        let mut recruit = unit.clone();
        recruit.id = 0;
        // A new recruit joins the existing army; it is not omitted from F.
        let score = Assessment::new(&view, &recruit).breakdown(&view, &recruit);
        assert!((score.thorns - 4.5).abs() < 1e-9);
    }

    #[test]
    fn expected_coverage_does_not_sum_exclusive_future_positions() {
        assert_eq!(support_coverage([].into_iter(), 1.0), 0.0);
        assert_eq!(support_coverage([4.0].into_iter(), 1.0), 4.0);
        assert_eq!(support_coverage([4.0; 2].into_iter(), 1.0), 4.0);
        assert_eq!(support_coverage([4.0; 8].into_iter(), 1.0), 4.0);
    }

    #[test]
    fn overlapping_providers_have_diminishing_value_scaled_by_army_size() {
        // Six possible recipients: each provider covers a given ally with p=.25.
        let first = expected_marginal_support(12.0, 0, 4, &mut [], 0.25);
        let second = expected_marginal_support(12.0, 0, 4, &mut [4], 0.25);
        let third = expected_marginal_support(12.0, 0, 4, &mut [4, 4], 0.25);
        assert_eq!((first, second, third), (4.0, 3.0, 2.25));
        // A larger company gives a second provider more uncovered recipients.
        assert!(expected_marginal_support(12.0, 0, 4, &mut [4], 0.1) > second);
    }

    #[test]
    fn overlap_credits_strength_upgrades_and_respects_actual_coverage() {
        assert_eq!(expected_marginal_support(12.0, 3, 3, &mut [], 0.25), 0.0);
        assert_eq!(expected_marginal_support(12.0, 3, 5, &mut [], 0.25), 2.0);
        assert_eq!(expected_marginal_support(12.0, 0, 5, &mut [3], 0.5), 3.5);
        assert_eq!(expected_marginal_support(12.0, 0, 3, &mut [5], 1.0), 0.0);
        assert_eq!(expected_marginal_support(4.0, 0, 5, &mut [3], 1.0), 1.0);
        // Multiple strengths: highest covering provider determines the increment.
        assert_eq!(
            expected_marginal_support(12.0, 0, 5, &mut [3, 5], 0.5),
            1.75
        );
    }

    #[test]
    fn strongest_only_healing_credits_only_the_increment() {
        assert_eq!(marginal_healing(12.0, 3, 3), 0.0);
        assert_eq!(marginal_healing(12.0, 3, 5), 2.0);
        assert_eq!(marginal_healing(4.0, 3, 5), 1.0);
        assert_eq!(marginal_healing(4.0, 5, 3), 0.0);
    }
}
