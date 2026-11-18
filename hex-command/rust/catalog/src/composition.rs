//! Pure numeric composition over explicit current inputs. No recipient selection occurs here.
use crate::*;

#[cfg(feature = "simulation")]
#[path = "diagnostics.rs"]
pub mod diagnostics;

pub struct UnitInputs<'a> {
    /// Current authored vision-group contribution; battle selects group membership.
    pub vision_group_sight: &'a SightContribution,
    pub equipment: &'a [&'a ItemDefinition],
    pub gained_abilities: &'a [&'a AbilityDefinition],
    /// Battle-selected occurrences, including eligible effects emitted by this unit itself.
    pub selected_effects: &'a [EffectHandle<'a>],
    /// Current replacement map/weather contribution, not the initial map snapshot.
    pub map_modifiers: &'a StatModifiers,
    /// Active event grants on this recipient, selected by battle.
    pub perception_contributions: &'a [&'a PerceptionEffect],
}
/// Complete incoming occurrences for queries whose only numeric inputs are
/// those selected effects. Eligibility is supplied by Battle, grouping by Catalogue.
pub struct SelectedEffects<'a> {
    effects: &'a [EffectHandle<'a>],
}
impl<'a> SelectedEffects<'a> {
    #[inline]
    pub fn new(effects: &'a [EffectHandle<'a>]) -> Self {
        Self { effects }
    }
    #[inline]
    pub fn healing(&self) -> i32 {
        UnitCapabilities::effect_total_from(self.effects.iter().copied(), |value| match value {
            EffectValue::Healing { power } => Some(*power),
            _ => None,
        })
    }
    #[inline]
    fn reflection(&self, category: Category) -> i32 {
        UnitCapabilities::effect_total_from(self.effects.iter().copied(), |value| match value {
            EffectValue::Thorns {
                damage_per_strike,
                incoming_category,
            } if *incoming_category == category => Some(*damage_per_strike),
            _ => None,
        })
    }
    #[inline]
    pub fn reflected_damage(&self, attack: AttackHandle<'_>) -> i32 {
        self.reflection(attack.category()) * i32::from(attack.source.count)
    }
    /// Forecast value for the supplied eligible incoming occurrences.
    #[inline]
    pub fn meta_thorn_score(&self, category: Category) -> i32 {
        self.reflection(category)
    }
}
/// Forecast-only owned emission/attack assessment. No receiver eligibility or
/// opponent protection is inferred, and no source fields are exposed.
pub struct OwnedAssessment {
    attack: f64,
    total: [f64; 2],
    healing: i32,
}
impl OwnedAssessment {
    /// Largest owned mean damage per strike before target protection.
    #[inline]
    pub fn meta_attack_score(&self) -> f64 {
        self.attack
    } // TODO: make it total expected damage
    #[inline]
    pub fn meta_total_attack_score(&self, category: Option<Category>) -> f64 {
        match category {
            Some(category) => self.total[category.index()],
            None => self.total[0].max(self.total[1]),
        }
    }
    #[inline]
    pub fn meta_healing_score(&self) -> f64 {
        f64::from(self.healing)
    }
}
impl UnitDefinition {
    pub fn assess(
        &self,
        equipment: &[&ItemDefinition],
        gained: &[&AbilityDefinition],
    ) -> OwnedAssessment {
        let mut attack = 0f64;
        let mut total = [0f64; 2];
        for weapon in self.attacks(equipment) {
            let mean = f64::from(weapon.source.damage);
            attack = attack.max(mean);
            total[weapon.category().index()] =
                total[weapon.category().index()].max(mean * f64::from(weapon.source.count));
        }
        let healing =
            UnitCapabilities::effect_total_from(self.effects(equipment, gained), |e| match e {
                EffectValue::Healing { power } => Some(*power),
                _ => None,
            });
        OwnedAssessment {
            attack,
            total,
            healing,
        }
    }
}
pub struct UnitCapabilities<'a> {
    unit: &'a UnitDefinition,
    inputs: UnitInputs<'a>,
}
/// Exact attack values for the current composed unit, before target protection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackDescription {
    pub damage: i32,
    pub spread: i32,
    pub count: u8,
    pub drain: u8,
}
/// Signed damage adjustments for the current unit, terrain, category and damage type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtectionBreakdown {
    pub armour: i32,
    pub terrain: i32,
}
/// Opponent-independent values used by the current neural feature schema.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuralAttackFeatures {
    pub mean_damage: f64,
    pub spread: i32,
    pub strike_count: u8,
    pub drain_percent: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeuralUnitFeatures {
    pub armour: i32,
    pub terrain: i32,
    pub reflection_per_strike: i32,
    pub owned_thorn_aura_bound: i32,
    pub owned_healing_bound: i32,
}
pub struct ResolvedSight {
    pub range: Option<u16>,
    pub penetrates: Vec<ConcealmentKind>,
}
/// Opaque identity of one strike's final HP-damage distribution. It excludes
/// drain/reflection and protection partition, which do not affect this distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrikeDistributionKey {
    low: i32,
    high: i32,
    modifier: i64,
}
impl std::hash::Hash for StrikeDistributionKey {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(u64::from(self.low as u32) | (u64::from(self.high as u32) << 32));
        state.write_i64(self.modifier);
    }
}
/// A composed uniform raw-roll distribution. Owns no weapon or battle state and allocates no storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AttackOutcomes {
    low: i32,
    high: i32,
    armour: i32,
    terrain: i32,
    count: u8,
    drain: u8,
    reflection: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageOutcome {
    pub raw: i32,
    pub after_armour: i32,
    pub damage: i32,
}
impl AttackOutcomes {
    #[inline]
    pub fn distribution_key(&self) -> StrikeDistributionKey {
        StrikeDistributionKey {
            low: self.low,
            high: self.high,
            modifier: i64::from(self.armour) + i64::from(self.terrain),
        }
    }

    #[inline]
    pub(crate) fn with_protection(self, protection: ProtectionBreakdown, reflection: i32) -> Self {
        Self {
            armour: protection.armour,
            terrain: protection.terrain,
            reflection,
            ..self
        }
        .checked()
    }
    fn from_attack(attack: AttackHandle<'_>, armour: i32, terrain: i32, reflection: i32) -> Self {
        let weapon = attack.source;
        AttackOutcomes {
            low: weapon
                .damage
                .checked_sub(weapon.spread)
                .expect("raw roll lower bound fits i32"),
            high: weapon
                .damage
                .checked_add(weapon.spread)
                .expect("raw roll upper bound fits i32"),
            armour,
            terrain,
            count: weapon.count,
            drain: weapon.drain,
            reflection,
        }
        .checked()
    }
    #[inline]
    fn checked(self) -> Self {
        // Validate arithmetic at construction; a canonical memoization key
        // then needs no repeated range checks, even after a cache hit.
        let _ = self.bounds();
        self
    }
    #[inline]
    fn outcome(&self, raw: i32) -> DamageOutcome {
        let after_armour = i64::from(raw) + i64::from(self.armour);
        DamageOutcome {
            raw,
            after_armour: i32::try_from(after_armour.max(0))
                .expect("composed armour damage fits i32"),
            damage: i32::try_from((after_armour + i64::from(self.terrain)).max(0))
                .expect("composed damage fits i32"),
        }
    }
    #[inline]
    pub fn bounds(&self) -> (DamageOutcome, DamageOutcome) {
        (self.outcome(self.low), self.outcome(self.high))
    }
    /// Fill single-strike damage probabilities, indexed from `bounds().0.damage`.
    /// The caller supplies exactly one bucket per inclusive damage value. Existing
    /// contents are overwritten. Positive damage buckets each represent one raw roll;
    /// the zero bucket combines all raw rolls prevented by protection.
    #[inline]
    pub fn fill_probabilities(&self, probabilities: &mut [f64]) {
        let modifier = i64::from(self.armour) + i64::from(self.terrain);
        let minimum = (i64::from(self.low) + modifier).max(0);
        let maximum = (i64::from(self.high) + modifier).max(0);
        assert_eq!(probabilities.len(), (maximum - minimum + 1) as usize);
        let width = i64::from(self.high) - i64::from(self.low) + 1;
        let probability = 1.0 / width as f64;
        probabilities.fill(probability);
        if minimum == 0 {
            let zero_rolls = i64::from(self.high).min(-modifier) - i64::from(self.low) + 1;
            probabilities[0] = zero_rolls as f64 * probability;
        }
    }
    /// Draws uniformly using caller-owned random words. Rejection preserves unbiased rolls.
    pub fn roll(&self, mut next_u32: impl FnMut() -> u32) -> DamageOutcome {
        let width = u32::try_from(i64::from(self.high) - i64::from(self.low) + 1)
            .expect("raw roll width fits u32");
        let threshold = width.wrapping_neg() % width;
        loop {
            let sample = next_u32();
            if sample >= threshold {
                let raw = i64::from(self.low) + i64::from(sample % width);
                return self.outcome(i32::try_from(raw).expect("raw roll fits i32"));
            }
        }
    }
    #[inline]
    pub fn mean(&self) -> f64 {
        let low = i64::from(self.low);
        let high = i64::from(self.high);
        let modifier = i64::from(self.armour) + i64::from(self.terrain);
        let first_positive = low.max(1 - modifier);
        if first_positive > high {
            return 0.0;
        }
        let count = high - first_positive + 1;
        (first_positive + high + 2 * modifier) as f64 * count as f64
            / (2.0 * (high - low + 1) as f64)
    }
    /// Probability that one strike removes at least `hp` before battle applies current HP.
    #[inline]
    pub fn kill_probability(&self, hp: i32) -> f64 {
        if hp <= 0 {
            return 1.0;
        }
        let low = i64::from(self.low);
        let high = i64::from(self.high);
        let first = low.max(i64::from(hp) - i64::from(self.armour) - i64::from(self.terrain));
        (high - first + 1).max(0) as f64 / (high - low + 1) as f64
    }
    #[inline]
    pub fn strike_count(&self) -> u8 {
        self.count
    }
    #[inline]
    pub fn drain_percent(&self) -> u8 {
        self.drain
    }
    #[inline]
    pub fn protection(&self) -> ProtectionBreakdown {
        ProtectionBreakdown {
            armour: self.armour,
            terrain: self.terrain,
        }
    }
    #[inline]
    pub fn reflection_per_strike(&self) -> i32 {
        self.reflection
    }
}
impl<'a> UnitCapabilities<'a> {
    pub(crate) fn combat_preparation_key(
        &self,
        instance_identity: usize,
    ) -> crate::prepared_combat::CombatPreparationKey {
        assert_eq!(
            self.unit.instance_identity, instance_identity,
            "prepared unit belongs to Catalogue"
        );
        for item in self.inputs.equipment {
            assert_eq!(
                item.instance_identity, instance_identity,
                "prepared equipment belongs to Catalogue"
            );
        }
        crate::prepared_combat::CombatPreparationKey::new(
            self.unit as *const UnitDefinition as usize,
            self.inputs
                .equipment
                .iter()
                .map(|i| *i as *const ItemDefinition as usize)
                .collect(),
            self.inputs.map_modifiers.clone(),
            self.inputs
                .selected_effects
                .iter()
                .filter_map(|e| match e.value() {
                    EffectValue::StatModifiers { modifiers } => Some(modifiers.clone()),
                    _ => None,
                })
                .collect(),
        )
    }
    pub(crate) fn unprotected_attack(&self, attack: AttackHandle<'_>) -> AttackOutcomes {
        self.assert_attack(attack);
        AttackOutcomes::from_attack(attack, 0, 0, 0)
    }

    #[inline]
    pub(crate) fn new(unit: &'a UnitDefinition, inputs: UnitInputs<'a>) -> Self {
        Self { unit, inputs }
    }
    #[inline]
    pub fn effects(&self) -> impl Iterator<Item = EffectHandle<'a>> + Clone + '_ {
        self.unit
            .effects(self.inputs.equipment, self.inputs.gained_abilities)
    }
    #[inline]
    pub fn attacks(&self) -> impl Iterator<Item = AttackHandle<'a>> + Clone + '_ {
        self.unit.attacks(self.inputs.equipment)
    }
    #[inline]
    fn reflection(&self, category: Category) -> i32 {
        SelectedEffects {
            effects: self.inputs.selected_effects,
        }
        .reflection(category)
    }
    #[inline]
    fn owned_healing_bound(&self) -> i32 {
        Self::effect_total_from(self.effects(), |effect| match effect {
            EffectValue::Healing { power } => Some(*power),
            _ => None,
        })
    }
    /// Final maximum accumulated path cost for the current unit capabilities.
    #[inline]
    pub fn path_cost_limit(&self) -> usize {
        self.unit.movement
    }
    #[inline]
    pub fn permits_path_cost(&self, total: u32) -> bool {
        usize::try_from(total).expect("path cost fits address space") <= self.path_cost_limit()
    }
    #[inline]
    fn modifiers(&self) -> impl Iterator<Item = &StatModifiers> {
        std::iter::once(&self.unit.modifiers)
            .chain(std::iter::once(self.inputs.map_modifiers))
            .chain(
                self.inputs
                    .selected_effects
                    .iter()
                    .filter_map(|effect| match effect.value() {
                        EffectValue::StatModifiers { modifiers } => Some(modifiers),
                        _ => None,
                    }),
            )
    }
    #[inline]
    pub fn step_cost(&self, terrain: &TerrainDefinition) -> Option<u16> {
        let mut cost = i32::from(terrain.source.cost);
        let mut passable = terrain.source.passable;
        for m in self
            .unit
            .racial
            .terrain_modifiers
            .get(&terrain.source.role)
            .into_iter()
            .chain(
                self.modifiers()
                    .filter_map(|m| m.terrain_modifiers.get(&terrain.source.role)),
            )
        {
            cost = cost
                .checked_add(m.cost)
                .expect("movement contribution overflow");
            if let Some(value) = m.passable {
                passable = value;
            }
        }
        passable.then(|| u16::try_from(cost).expect("composed movement cost fits u16"))
    }
    pub fn sight(&self, target_illumination: Illumination) -> ResolvedSight {
        let mut sight = self.inputs.vision_group_sight.clone();
        sight
            .stack(&self.unit.racial.sight)
            .expect("racial sight fits numeric limits");
        for m in self.modifiers() {
            sight
                .stack(&m.sight)
                .expect("sight stack fits numeric limits");
        }
        for item in self.inputs.equipment {
            sight
                .stack(&item.source.sight)
                .expect("equipment sight fits numeric limits");
        }
        for effect in self.inputs.selected_effects {
            if let EffectValue::Sight {
                range,
                unlimited,
                sight_modifiers,
                penetrates,
            } = effect.value()
            {
                sight
                    .stack(&SightContribution {
                        range: *range,
                        unlimited: *unlimited,
                        sight_modifiers: *sight_modifiers,
                        penetrates: penetrates.clone(),
                    })
                    .expect("effect sight fits numeric limits");
            }
        }
        for grant in self.inputs.perception_contributions {
            for kind in &grant.penetrates {
                if !sight.penetrates.contains(kind) {
                    sight.penetrates.push(*kind)
                }
            }
        }
        let adjustment = match target_illumination {
            Illumination::Lit => sight.sight_modifiers.lit,
            Illumination::Unlit => sight.sight_modifiers.unlit,
            Illumination::Darkness => sight.sight_modifiers.darkness,
        };
        let range = if sight.unlimited {
            None
        } else {
            Some(
                u16::try_from((sight.range + i32::from(adjustment)).max(0))
                    .expect("composed sight fits u16"),
            )
        };
        ResolvedSight {
            range,
            penetrates: sight.penetrates,
        }
    }
    /// Predicates remain intact. Battle determines which rules conceal a unit at a location.
    pub fn concealment_rules(&self) -> impl Iterator<Item = &ConcealmentRule> + Clone {
        let rules = self
            .inputs
            .selected_effects
            .iter()
            .filter_map(|e| match e.value() {
                EffectValue::Concealment { rule } => Some(rule),
                _ => None,
            })
            .chain(
                self.inputs
                    .perception_contributions
                    .iter()
                    .flat_map(|p| &p.concealment),
            );
        let previous = rules.clone();
        rules.enumerate().filter_map(move |(index, rule)| {
            (!previous.clone().take(index).any(|earlier| earlier == rule)).then_some(rule)
        })
    }
    #[inline]
    fn effect_total_from<'e>(
        effects: impl Iterator<Item = EffectHandle<'e>> + Clone,
        select: impl Fn(&EffectValue) -> Option<i32>,
    ) -> i32 {
        Self::effect_total_with_groups(effects, &select, |_, _, _| {})
    }
    fn effect_total_with_groups<'e>(
        effects: impl Iterator<Item = EffectHandle<'e>> + Clone,
        select: &impl Fn(&EffectValue) -> Option<i32>,
        mut group_result: impl FnMut(&str, &EffectCombine, i32),
    ) -> i32 {
        let mut total = 0i32;
        for (index, effect) in effects.clone().enumerate() {
            if select(effect.value()).is_none() {
                continue;
            }
            let rule = effect.stacking();
            if effects.clone().take(index).any(|previous| {
                previous.stacking().group == rule.group && select(previous.value()).is_some()
            }) {
                continue;
            }
            let mut combined: Option<i32> = None;
            for occurrence in effects.clone() {
                if occurrence.stacking().group != rule.group {
                    continue;
                }
                if let Some(value) = select(occurrence.value()) {
                    assert_eq!(
                        occurrence.stacking().combine,
                        rule.combine,
                        "authored group uses a consistent operation"
                    );
                    combined = Some(match (&rule.combine, combined) {
                        (EffectCombine::Union, _) => panic!("numeric effect requires Add or Max"),
                        (_, None) => value,
                        (EffectCombine::Add, Some(previous)) => {
                            previous.checked_add(value).expect("effect group overflow")
                        }
                        (EffectCombine::Max, Some(previous)) => previous.max(value),
                    });
                }
            }
            let combined = combined.expect("selected numeric group contains an occurrence");
            group_result(&rule.group, &rule.combine, combined);
            total = total.checked_add(combined).expect("effect total overflow");
        }
        total
    }
    #[inline]
    pub fn healing(&self) -> i32 {
        SelectedEffects {
            effects: self.inputs.selected_effects,
        }
        .healing()
    }
    #[inline]
    fn defense(
        &self,
        terrain: &TerrainDefinition,
        category: Category,
        kind: DamageType,
    ) -> (i32, i32) {
        fn environmental(m: &EnvironmentalModifiers, category: Category, kind: DamageType) -> i32 {
            (match category {
                Category::Melee => m.melee,
                Category::Ranged => m.ranged,
            }) + match kind {
                DamageType::Blade => m.blade,
                DamageType::Pierce => m.pierce,
                DamageType::Impact => m.impact,
            }
        }
        let armour = Self::armour_from(
            &self.unit.racial.damage_modifiers,
            self.modifiers(),
            self.inputs.equipment,
            kind,
        );
        let mut environment = environmental(&terrain.source.damage_modifiers, category, kind);
        if let Some(m) = self.unit.racial.terrain_modifiers.get(&terrain.source.role) {
            environment += environmental(&m.damage_modifiers, category, kind)
        }
        for modifiers in self.modifiers() {
            if let Some(m) = modifiers.terrain_modifiers.get(&terrain.source.role) {
                environment += environmental(&m.damage_modifiers, category, kind)
            }
        }
        (armour, environment)
    }
    #[inline]
    fn armour_from<'m>(
        racial: &DamageModifiers,
        modifiers: impl IntoIterator<Item = &'m StatModifiers>,
        equipment: &[&ItemDefinition],
        kind: DamageType,
    ) -> i32 {
        let mut armour = racial.get(kind);
        for modifiers in modifiers {
            armour += modifiers.damage_modifiers.get(kind);
        }
        for item in equipment {
            if let Some(armor) = &item.source.armor {
                armour += armor.get(kind);
            }
        }
        armour
    }
    #[inline]
    pub fn attack_description(&self, attack: AttackHandle<'_>) -> AttackDescription {
        self.assert_attack(attack);
        AttackDescription {
            damage: attack.source.damage,
            spread: attack.source.spread,
            count: attack.source.count,
            drain: attack.source.drain,
        }
    }
    #[inline]
    pub fn protection(
        &self,
        terrain: &TerrainDefinition,
        category: Category,
        damage_type: DamageType,
    ) -> ProtectionBreakdown {
        let (armour, terrain) = self.defense(terrain, category, damage_type);
        ProtectionBreakdown { armour, terrain }
    }
    // TODO: figure out if neural needs this
    #[inline]
    pub fn neural_pure_attack_features(&self, attack: AttackHandle<'_>) -> NeuralAttackFeatures {
        let description = self.attack_description(attack);
        NeuralAttackFeatures {
            mean_damage: f64::from(description.damage),
            spread: description.spread,
            strike_count: description.count,
            drain_percent: description.drain,
        }
    }
    // TODO: figure out if neural needs this
    pub fn neural_unit_features(
        &self,
        terrain: &TerrainDefinition,
        category: Category,
        damage_type: DamageType,
    ) -> NeuralUnitFeatures {
        let (armour, terrain) = self.defense(terrain, category, damage_type);
        let reflection_per_strike = self.reflection(category);
        let aura_effects = self.effects().filter(|effect| effect.scope().radius > 0);
        let owned_thorn_aura_bound = Self::effect_total_from(aura_effects, |effect| match effect {
            EffectValue::Thorns {
                damage_per_strike,
                incoming_category,
            } if *incoming_category == category => Some(*damage_per_strike),
            _ => None,
        });
        NeuralUnitFeatures {
            armour,
            terrain,
            reflection_per_strike,
            owned_thorn_aura_bound,
            owned_healing_bound: self.owned_healing_bound(),
        }
    }
    #[inline]
    fn assert_attack(&self, attack: AttackHandle<'_>) {
        assert!(
            self.unit
                .attacks
                .iter()
                .chain(
                    self.inputs
                        .equipment
                        .iter()
                        .flat_map(|item| &item.source.attacks)
                )
                .any(|source| std::ptr::eq(source, attack.source)),
            "attack belongs to current composed equipment or unit"
        );
    }
    /// The same operation resolves the selected retaliation with attacker and defender reversed.
    #[inline]
    pub fn attack_outcomes(
        &self,
        attack: AttackHandle<'_>,
        defender: &UnitCapabilities<'_>,
        defender_terrain: &TerrainDefinition,
    ) -> AttackOutcomes {
        self.assert_attack(attack);
        let (armour, terrain) =
            defender.defense(defender_terrain, attack.category(), attack.damage_type());
        AttackOutcomes::from_attack(
            attack,
            armour,
            terrain,
            defender.reflection(attack.category()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outcome_queries_preserve_clipping_probabilities_and_raw_rolls() {
        for armour in -5..=2 {
            for terrain in -3..=2 {
                let outcomes = AttackOutcomes {
                    low: 1,
                    high: 5,
                    armour,
                    terrain,
                    count: 2,
                    drain: 25,
                    reflection: 3,
                };
                let damage: Vec<_> = (1..=5).map(|raw| (raw + armour + terrain).max(0)).collect();
                assert_eq!(outcomes.mean(), f64::from(damage.iter().sum::<i32>()) / 5.0);
                for hp in 0..=10 {
                    let expected = damage.iter().filter(|&&d| d >= hp).count() as f64 / 5.0;
                    assert!((outcomes.kill_probability(hp) - expected).abs() < 1e-12);
                }
                assert_eq!(
                    outcomes.bounds().0,
                    DamageOutcome {
                        raw: 1,
                        after_armour: (1 + armour).max(0),
                        damage: damage[0]
                    }
                );
                assert_eq!(outcomes.bounds().1.damage, damage[4]);
            }
        }
        let outcomes = AttackOutcomes {
            low: 1,
            high: 3,
            armour: -2,
            terrain: 0,
            count: 1,
            drain: 0,
            reflection: 0,
        };
        assert_eq!(outcomes.kill_probability(1), 1.0 / 3.0);
        let mut words = [0, 1].into_iter();
        let sampled = outcomes.roll(|| words.next().expect("only one rejection"));
        assert_eq!(
            sampled,
            DamageOutcome {
                raw: 2,
                after_armour: 0,
                damage: 0
            }
        );
        assert!(words.next().is_none());
    }
}
