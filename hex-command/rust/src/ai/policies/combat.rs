//! Shared attack recommendation policy used by Classic and the display host.
use crate::{ai::context::BattleView, data::*};

pub(crate) fn best_attack(view: &BattleView<'_, '_>, unit: &Unit, target: &Unit, from: usize) -> Option<usize> {
    let mut best = None;
    let mut score = f64::NEG_INFINITY;
    for attack in view.attack_options(unit, target, from) {
        let result = view.exchange(unit, target, attack, from);
        let value = result.effective[0] + result.healing[0] + result.death[1] * 18.0
            - result.damage[1] * 0.7 - result.death[0] * 40.0;
        if value > score { score = value; best = Some(attack); }
    }
    best
}

/// Decorate mechanical inspection output without putting scoring in the engine.
#[cfg(any(feature = "wasm", test))]
pub(crate) fn inspect(session: &crate::logic::Game<'_>, revision: u32) -> crate::protocol::BattleInspection {
    let mut result = session.inspect(revision);
    let view = session.view();
    for actions in &mut result.units {
        let Some(actor) = actions.unit.strip_prefix('u').and_then(|id|id.parse().ok()).and_then(|id|view.unit(id)) else { continue; };
        for target in view.units() {
            let recommended = best_attack(view, actor, target, actor.tile);
            let target_id = format!("u{}", target.id);
            for (detail,index) in actions.attacks.iter_mut().filter(|detail|detail.target == target_id)
                .zip(view.attack_options(actor,target,actor.tile)) {
                detail.recommended = recommended == Some(index);
            }
        }
    }
    result
}
