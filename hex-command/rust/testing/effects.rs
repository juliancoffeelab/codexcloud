//! Author focused effect fixtures through the same JSON boundary as game content.
use crate::data::*;
pub fn healing(
    bundle: &mut JsonBundle,
    id: &str,
    profile: Option<(i32, u16, bool, HealingTargets)>,
) {
    crate::catalogue_fixture::effects(bundle, id, |effects| {
        effects.retain(|effect| !matches!(effect.effect, EffectValue::Healing { .. }));
        if let Some((power, radius, include_self, targets)) = profile {
            effects.push(ScopedEffect {
                scope: EffectScope {
                    radius,
                    include_self,
                    targets,
                },
                stacking: StackingRule {
                    group: "healing".into(),
                    combine: EffectCombine::Max,
                },
                effect: EffectValue::Healing { power },
            });
        }
    });
}
pub fn thorns(bundle: &mut JsonBundle, id: &str, power: i32) {
    reflect(bundle, id, power, false);
}
pub fn thorn_aura(bundle: &mut JsonBundle, id: &str, power: i32) {
    reflect(bundle, id, power, true);
}
fn reflect(bundle: &mut JsonBundle, id: &str, power: i32, aura: bool) {
    crate::catalogue_fixture::effects(bundle, id, |effects| {
        effects.retain(|effect| {
            !matches!(effect.effect, EffectValue::Thorns { .. })
                || (effect.scope.radius > 0) != aura
        });
        if power > 0 {
            effects.push(ScopedEffect {
                scope: EffectScope {
                    radius: if aura { 1 } else { 0 },
                    include_self: !aura,
                    targets: HealingTargets::Allies,
                },
                stacking: StackingRule {
                    group: if aura {
                        "external-thorns"
                    } else {
                        "personal-thorns"
                    }
                    .into(),
                    combine: EffectCombine::Max,
                },
                effect: EffectValue::Thorns {
                    damage_per_strike: power,
                    incoming_category: Category::Melee,
                },
            });
        }
    });
}
pub fn concealment(bundle: &mut JsonBundle, id: &str, rules: Vec<ConcealmentRule>) {
    crate::catalogue_fixture::effects(bundle, id, |effects| {
        effects.retain(|effect| !matches!(effect.effect, EffectValue::Concealment { .. }));
        effects.extend(rules.into_iter().map(|rule| ScopedEffect {
            scope: EffectScope {
                radius: 0,
                include_self: true,
                targets: HealingTargets::Allies,
            },
            stacking: StackingRule {
                group: "concealment".into(),
                combine: EffectCombine::Union,
            },
            effect: EffectValue::Concealment { rule },
        }));
    });
}
pub fn healing_kind(
    bundle: &mut JsonBundle,
    kind: usize,
    profile: Option<(i32, u16, bool, HealingTargets)>,
) {
    let id = crate::catalogue_fixture::id(bundle, kind);
    healing(bundle, &id, profile);
}
pub fn thorns_kind(bundle: &mut JsonBundle, kind: usize, power: i32) {
    let id = crate::catalogue_fixture::id(bundle, kind);
    thorns(bundle, &id, power);
}
pub fn thorn_aura_kind(bundle: &mut JsonBundle, kind: usize, power: i32) {
    let id = crate::catalogue_fixture::id(bundle, kind);
    thorn_aura(bundle, &id, power);
}
