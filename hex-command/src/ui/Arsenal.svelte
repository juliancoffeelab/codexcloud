<script lang="ts">
  import { TYPES, DAMAGE_TYPES } from '../game/engine';
  import type { Unit, DamageType } from '../game/types';
  import { iconMarkup } from '../presentation/icons';
  let { unit }: { unit: Unit } = $props();
  const damageTypes = Object.entries(DAMAGE_TYPES) as [DamageType, string][];
</script>
<div class="arsenal"><h3>Attacks</h3><table class="weapon-table" aria-label="Unit attacks"><thead><tr><th scope="col">Attack</th><th scope="col">Damage</th><th scope="col">Range</th></tr></thead><tbody>{#each TYPES[unit.type].attacks as attack}<tr><th scope="row"><span class="weapon-name">{@html iconMarkup(attack.damageType)}<strong>{attack.name}</strong></span><span class="weapon-detail weapon-category">{attack.category === 'melee' ? 'Melee' : 'Ranged'}</span></th><td class="weapon-damage"><b>{attack.damage}</b> {DAMAGE_TYPES[attack.damageType]}</td><td>{attack.range === 1 ? '1 hex range' : `1–${attack.range} hex range`}</td></tr>{/each}</tbody></table></div><div class="resistances"><h3>Damage received</h3><div class="resistance-list">{#each damageTypes as [type, name]}{@const amount = TYPES[unit.type].resistances[type] || 0}<span class={`resistance ${amount > 0 ? 'resistant' : amount < 0 ? 'vulnerable' : ''}`}><span>{name}</span><b>{100 - amount}%</b></span>{/each}</div><p class="resistance-hint">Base damage multiplier, rounded before terrain cover.</p></div>
