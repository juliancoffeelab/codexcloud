import { TYPES, DAMAGE_TYPES } from './engine.mjs?v=9d86854f39f3';
import { iconMarkup } from './icons.mjs?v=a28758f5c35e';

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
    `<p><span class="arsenal-weapon"><strong>${attack.name}</strong> · ${iconMarkup(attack.damageType)} ${attack.damage} ${DAMAGE_TYPES[attack.damageType].toLowerCase()}</span><span class="arsenal-reach">${iconMarkup('range')} ${attack.category === 'melee' ? 'Melee' : 'Ranged'} · ${reach(attack)}</span></p>`).join('')}</div>${resistanceMarkup(unit)}`;
}
export function attackChoicesMarkup(game, attacker, target) {
  return `<div class="attack-actions" aria-label="Attack actions"><h3>Attack</h3>${TYPES[attacker.type].attacks.map(attack => {
    const available = game.canAttackFrom(attacker, attacker.tile, target, attack.id);
    const result = available ? game.previewDetails(attacker, target, attack.id) : null;
    const warning = result && (result.retaliation >= attacker.hp ? ' · you fall' : result.damage >= target.hp ? ' · enemy falls' : '');
    const detailsId = `attack-details-${attack.id}`;
    const explanation = result ? `<div id="${detailsId}" class="combat-explanation" hidden><p>${DAMAGE_TYPES[attack.damageType]} · ${attack.category} · ${reach(attack)}</p><p><strong>${attack.name}</strong>: ${damageExplanation(result.damageDetail)}.</p><p>${result.response ? `Enemy replies with ${result.response.name.toLowerCase()}: ${damageExplanation(result.retaliationDetail)}.` : result.damage >= target.hp ? 'Enemy falls before replying.' : `No ${attack.category} retaliation in reach.`}</p>${resistanceMarkup(target)}</div>` : '';
    return `<div class="attack-action-row"><button class="attack-action" data-attack="${attack.id}" aria-label="Attack with ${attack.name}${result ? `: deal ${result.damage} damage, take ${result.retaliation} damage${warning}` : ' — out of reach or blocked'}" ${available ? '' : 'disabled'}><strong>${attack.name}</strong><span>${result ? ` · deal ${result.damage} · take ${result.retaliation}` : ' · out of reach or blocked'}</span></button>${result ? `<button class="attack-inspect" data-inspect="${attack.id}" aria-label="Inspect ${attack.name.toLowerCase()} damage calculation" aria-expanded="false" aria-controls="${detailsId}">${iconMarkup('search')}</button>` : ''}${explanation}</div>`;
  }).join('')}</div>`;
}
export function damageExplanation(detail) {
  const parts = [`${detail.base} base`];
  if (detail.resistance) parts.push(`${Math.abs(detail.resistance)}% ${DAMAGE_TYPES[detail.damageType].toLowerCase()} ${detail.resistance > 0 ? 'resistance' : 'weakness'}`);
  if (detail.protection) parts.push(`−${detail.protection} cover`);
  if (detail.exposure) parts.push(`+${detail.exposure} shallow water`);
  return `${parts.join(' · ')} = ${detail.damage} damage`;
}
