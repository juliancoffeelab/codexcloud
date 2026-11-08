//! Shared attack recommendation policy used by Classic and the display host.
use crate::{ai::context::BattleView, data::*};

pub(crate) fn best_attack(
    view: &BattleView<'_, '_>,
    unit: &Unit,
    target: &Unit,
    from: usize,
) -> Option<usize> {
    let mut best = None;
    let mut score = f64::NEG_INFINITY;
    for attack in view.forecast_attack_options(unit, target, from) {
        let result = view.exchange(unit, target, attack, from);
        let value = result.effective[0] + result.healing[0] + result.death[1] * 18.0
            - result.damage[1] * 0.7
            - result.death[0] * 40.0;
        if value > score {
            score = value;
            best = Some(attack);
        }
    }
    best
}

/// Decorate mechanical inspection output without putting scoring in the engine.
#[cfg(any(feature = "wasm", test))]
pub(crate) fn inspect(
    view: &BattleView<'_, '_>,
    mut result: crate::protocol::BattleInspection,
) -> crate::protocol::BattleInspection {
    for actions in &mut result.units {
        let Some(actor) = actions
            .unit
            .strip_prefix('u')
            .and_then(|id| id.parse().ok())
            .and_then(|id| view.unit(id))
        else {
            continue;
        };
        // Mechanical inspection emits each target's legal attacks contiguously.
        // Visit those groups once rather than scanning every attack for every unit.
        let mut offset = 0;
        while offset < actions.attacks.len() {
            let target_id = &actions.attacks[offset].target;
            let count = actions.attacks[offset..]
                .iter()
                .take_while(|detail| &detail.target == target_id)
                .count();
            if let Some(target) = target_id
                .strip_prefix('u')
                .and_then(|id| id.parse().ok())
                .and_then(|id| view.unit(id))
            {
                let recommended = best_attack(view, actor, target, actor.tile);
                for (detail, index) in actions.attacks[offset..offset + count]
                    .iter_mut()
                    .zip(view.forecast_attack_options(actor, target, actor.tile))
                {
                    detail.recommended = recommended == Some(index);
                }
            }
            offset += count;
        }
    }
    result
}
