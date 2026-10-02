import { TYPES, DAMAGE_TYPES, TERRAIN } from './engine.mjs?v=9d86854f39f3';
import { iconMarkup } from './icons.mjs?v=a28758f5c35e';

const reach = attack => attack.range === 1 ? '1 hex range' : `1–${attack.range} hex range`;
const category = attack => attack.category === 'melee' ? 'Melee' : 'Ranged';
export function resistanceMarkup(unit) {
  const values = TYPES[unit.type].resistances;
  return `<div class="resistances"><h3>Damage received</h3><div class="resistance-list">${Object.entries(DAMAGE_TYPES).map(([type, name]) => {
    const amount = values[type] || 0;
    return `<span class="resistance ${amount > 0 ? 'resistant' : amount < 0 ? 'vulnerable' : ''}"><span>${name}</span><b>${100 - amount}%</b></span>`;
  }).join('')}</div><p class="resistance-hint">Base damage multiplier, rounded before terrain cover.</p></div>`;
}
export function arsenalMarkup(unit) {
  return `<div class="arsenal"><h3>Attacks</h3><table class="weapon-table" aria-label="Unit attacks"><thead><tr><th scope="col">Attack</th><th scope="col">Damage</th><th scope="col">Range</th></tr></thead><tbody>${TYPES[unit.type].attacks.map(attack =>
    `<tr><th scope="row"><span class="weapon-name">${iconMarkup(attack.damageType)}<strong>${attack.name}</strong></span><span class="weapon-detail weapon-category">${category(attack)}</span></th><td class="weapon-damage"><b>${attack.damage}</b> ${DAMAGE_TYPES[attack.damageType]}</td><td>${reach(attack)}</td></tr>`).join('')}</tbody></table></div>${resistanceMarkup(unit)}`;
}
function healthForecast(unit, damage, label, impact) {
  const maximum = TYPES[unit.type].hp, after = Math.max(0, unit.hp - damage);
  return `<span class="attack-health ${unit.side}" data-forecast="${unit.side}"><span class="forecast-label"><span>${unit.hp} → <b class="${after === 0 ? 'fatal' : ''}">${after}</b> / ${maximum} HP</span></span><span class="health-track" role="img" aria-label="${label}: ${unit.hp} to ${after} HP after this attack"><span class="health-lost" style="width:${unit.hp/maximum*100}%"></span><span class="health-left" style="width:${after/maximum*100}%"></span></span><span class="forecast-impact">${impact} ${damage ? `−${damage}` : 0} HP</span></span>`;
}
function attackDescription(label, attack, detail, tile) {
  const terrain = detail.protection || detail.exposure ? `, against terrain: ${TERRAIN[tile.terrain].name.toLowerCase()}` : '';
  return `${label}: ${attack.damage} ${DAMAGE_TYPES[attack.damageType].toLowerCase()}, ${attack.name.toLowerCase()}, ${attack.category}${terrain}`;
}
export function attackChoicesMarkup(game, attacker, target) {
  return `<div class="attack-actions" aria-label="Attack actions"><h3>Attack</h3>${TYPES[attacker.type].attacks.map(attack => {
    const available = game.canAttackFrom(attacker, attacker.tile, target, attack.id);
    const result = available ? game.previewDetails(attacker, target, attack.id) : null;
    const warning = result && (result.retaliation >= attacker.hp ? ', you fall' : result.damage >= target.hp ? ', enemy falls' : '');
    const detailsId = `attack-details-${attack.id}`;
    const explanation = result ? `<div id="${detailsId}" class="combat-explanation" hidden><div class="attack-calculation"><p>${attackDescription('Your attack', attack, result.damageDetail, game.tile(target.tile))}</p><p class="damage-equation">${damageExplanation(result.damageDetail)}</p></div><div class="attack-calculation"><p>${result.response ? attackDescription('Their attack', result.response, result.retaliationDetail, game.tile(attacker.tile)) : 'Their attack: none'}</p><p class="damage-equation">${result.response ? damageExplanation(result.retaliationDetail) : result.damage >= target.hp ? '0 (enemy defeated)' : '0 (out of range)'}</p></div></div>` : '';
    return `<div class="attack-action-row"><div class="attack-box"><button class="attack-action" data-attack="${attack.id}" aria-label="Attack with ${attack.name}${result ? `: deal ${result.damage} damage, take ${result.retaliation} damage${warning}` : ' — out of reach or blocked'}" ${available ? '' : 'disabled'}><span class="attack-heading"><strong>${attack.name}</strong>${result ? '' : '<span>Out of reach or blocked</span>'}</span>${result ? `<span class="attack-forecasts">${healthForecast(attacker, result.retaliation, 'You', 'Take')}${healthForecast(target, result.damage, 'Enemy', 'Deal')}</span>` : ''}</button>${result ? `<button class="attack-inspect" data-inspect="${attack.id}" aria-label="Inspect ${attack.name.toLowerCase()} damage calculation" aria-expanded="false" aria-controls="${detailsId}">${iconMarkup('search')}</button>` : ''}</div>${explanation}</div>`;
  }).join('')}</div>`;
}
export function damageExplanation(detail) {
  const multiplier = 100 - detail.resistance;
  let equation = `round(${detail.base} × ${multiplier}%)`;
  if (detail.protection) equation += ` − ${detail.protection} (cover)`;
  if (detail.exposure) equation += ` + ${detail.exposure} (shallow water)`;
  if (detail.resisted - detail.protection + detail.exposure < 1) equation = `max(1, ${equation})`;
  return `${equation} = ${detail.damage}`;
}
