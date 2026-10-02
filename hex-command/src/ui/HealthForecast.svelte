<script lang="ts">
  import { TYPES } from '../game/engine';
  import type { Unit } from '../game/types';
  let { unit, damage, label, impact }: { unit: Unit; damage: number; label: string; impact: string } = $props();
  const maximum = $derived(TYPES[unit.type].hp);
  const after = $derived(Math.max(0, unit.hp - damage));
</script>
<span class={`attack-health ${unit.side}`} data-forecast={unit.side}><span class="forecast-label"><span>{unit.hp} → <b class={after === 0 ? 'fatal' : ''}>{after}</b> / {maximum} HP</span></span><span class="health-track" role="img" aria-label={`${label}: ${unit.hp} to ${after} HP after this attack`}><span class="health-lost" style={`width:${unit.hp / maximum * 100}%`}></span><span class="health-left" style={`width:${after / maximum * 100}%`}></span></span><span class="forecast-impact">{impact} {damage ? `−${damage}` : 0} HP</span></span>
