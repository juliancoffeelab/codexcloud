<script lang="ts">
  import type { Unit } from '../game/types';
  import { TYPES } from '../game/engine';
  import { artSprite, type ArtStyle } from '../presentation/art';
  let { unit, artStyle }: { unit: Unit; artStyle: ArtStyle } = $props();
  const info = $derived(TYPES[unit.type]);
</script>
<div class={`combatant ${unit.side}`}><span class="unit-side">{unit.side === 'player' ? 'Your unit' : 'Enemy unit'}</span>{@html artSprite(artStyle, 'units', unit.type, 0, 0, 72, 90, `portrait ${unit.side === 'enemy' ? 'enemy-unit' : ''}`, unit.side)}<h2>{info.name}</h2><div class="health-numbers"><b>{unit.hp}</b><span> / {info.hp} HP</span></div><div class="health-track" role="img" aria-label={`${unit.hp} of ${info.hp} current health`}><span class="health-left" style={`width:${unit.hp / info.hp * 100}%`}></span></div></div>
