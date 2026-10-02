import { TYPES, DAMAGE_TYPES } from './engine.mjs?v=9d86854f39f3';

const reach = attack => attack.range === 1 ? '1 hex' : `1–${attack.range} hexes`;
export function resistanceMarkup(unit) {
  const values = TYPES[unit.type].resistances;
  return `<div class="resistances"><h3>Resistances</h3><div class="resistance-list">${Object.entries(DAMAGE_TYPES).map(([type, name]) => {
    const amount = values[type] || 0;
    return `<span class="resistance ${amount > 0 ? 'resistant' : amount < 0 ? 'vulnerable' : ''}">${name}: <b>${amount ? `${Math.abs(amount)}% ${amount > 0 ? 'less' : 'more'}` : 'normal'}</b></span>`;
  }).join('')}</div><p class="resistance-hint">Damage received, before terrain cover.</p></div>`;
}
export function arsenalMarkup(unit) {
  return `<div class="arsenal"><h3>Attacks</h3>${TYPES[unit.type].attacks.map(attack =>
    `<p><span class="arsenal-weapon"><strong>${attack.name}</strong> · ${attack.damage} ${DAMAGE_TYPES[attack.damageType].toLowerCase()}</span><span class="arsenal-reach">${attack.category === 'melee' ? 'Melee' : 'Ranged'} · ${reach(attack)}</span></p>`).join('')}</div>${resistanceMarkup(unit)}`;
}
export function attackChoicesMarkup(game, attacker, target, selectedId) {
  return `<fieldset class="attack-choices"><legend>Choose an attack</legend>${TYPES[attacker.type].attacks.map(attack => {
    const available = game.canAttackFrom(attacker, attacker.tile, target, attack.id);
    const result = available ? game.preview(attacker, target, attack.id) : null;
    return `<label class="attack-choice${attack.id === selectedId ? ' chosen' : ''}${available ? '' : ' unavailable'}"><input type="radio" name="attack-choice" value="${attack.id}" ${attack.id === selectedId ? 'checked' : ''} ${available ? '' : 'disabled'} /><span><strong>${attack.name}</strong><span class="weapon-kind">${DAMAGE_TYPES[attack.damageType]} · ${attack.category} · ${reach(attack)}</span><span class="weapon-outcome">${result ? `Deal ${result.damage} · take ${result.retaliation}${result.retaliation >= attacker.hp ? ' · you fall' : result.damage >= target.hp ? ' · enemy falls' : ''}` : 'Out of reach or blocked'}</span></span></label>`;
  }).join('')}</fieldset>`;
}
export function damageExplanation(detail) {
  const parts = [`${detail.base} base`];
  if (detail.resistance) parts.push(`${Math.abs(detail.resistance)}% ${DAMAGE_TYPES[detail.damageType].toLowerCase()} ${detail.resistance > 0 ? 'resistance' : 'weakness'}`);
  if (detail.protection) parts.push(`−${detail.protection} cover`);
  if (detail.exposure) parts.push(`+${detail.exposure} shallow water`);
  return `${parts.join(' · ')} = ${detail.damage} damage`;
}
