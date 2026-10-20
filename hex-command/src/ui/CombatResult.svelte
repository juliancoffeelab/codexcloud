<script lang="ts">
  import type { BattleView } from "../controller/view";
  import { TYPES } from "../game/catalog";
  import { iconMarkup } from "../presentation/icons";

  let {
    combat,
    shown,
    complete,
  }: {
    combat: NonNullable<BattleView["combat"]>;
    shown: number;
    complete: boolean;
  } = $props();

  let expanded = $state<string | null>(null);
  const player = $derived(combat.units[0].side === "player" ? 0 : 1);
  const opponent = $derived(player === 0 ? 1 : 0);
  const finalHealth = $derived(combat.strikes.at(-1)!.health);
  const playerChange = $derived(finalHealth[player] - combat.units[player].hp);
  const opponentChange = $derived(finalHealth[opponent] - combat.units[opponent].hp);
  const loss = (amount: number) => (amount ? `−${amount}` : "0");
  const entries = $derived.by(() => {
    const rows: { id: string; good: boolean; text: string; details: string[] }[] = [];
    for (const [index, hit] of combat.strikes.slice(0, shown).entries()) {
      const actor = combat.units[0].side === hit.side ? 0 : 1;
      const previous = index
        ? combat.strikes[index - 1]!.health
        : ([combat.units[0].hp, combat.units[1].hp] as const);
      const yours = hit.side === "player";
      let equation = `${hit.raw}`;
      if (hit.modifier)
        equation += hit.modifier < 0 ? ` − ${-hit.modifier} armor` : ` + ${hit.modifier} weakness`;
      if (hit.terrain)
        equation += hit.terrain < 0 ? ` − ${-hit.terrain} cover` : ` + ${hit.terrain} exposure`;
      if (hit.raw + hit.modifier + hit.terrain < 0) equation = `max(0, ${equation})`;
      rows.push({
        id: `hit-${index}`,
        good: yours,
        text: `You ${yours ? "deal" : "take"} ${loss(hit.effective)} HP`,
        details: [
          `${hit.weapon}: rolled ${hit.raw} from ${hit.rollMin}–${hit.rollMax}.`,
          `${equation} = ${hit.damage} damage`,
          `min(${previous[1 - actor]} HP, ${hit.damage} damage) = ${hit.effective} HP removed`,
        ],
      });
      if (hit.healed)
        rows.push({
          id: `drain-${index}`,
          good: yours,
          text: `${yours ? "You heal" : "Opponent heals"} +${hit.healed} HP`,
          details: [
            `min(floor(${hit.effective} × ${hit.drain}%), ${TYPES[combat.units[actor].type].hp - previous[actor]} missing HP) = ${hit.healed} HP restored`,
          ],
        });
      if (hit.thorns)
        rows.push({
          id: `thorns-${index}`,
          good: !yours,
          text: `You ${yours ? "take" : "deal"} ${loss(hit.reflected)} HP · thorns`,
          details: [
            `min(${previous[actor] + hit.healed} HP, ${hit.thorns} thorns) = ${hit.reflected} HP removed`,
          ],
        });
    }
    return rows;
  });
</script>

<ol class="exchange-log" role="log" aria-label="Combat exchange">
  {#each entries as entry (entry.id)}
    <li class="exchange-event" class:good={entry.good} class:bad={!entry.good}>
      <div class="exchange-line">
        <strong>{entry.text}</strong>
        <button
          class="icon-button exchange-inspect"
          aria-label={`Inspect calculation: ${entry.text}`}
          aria-expanded={expanded === entry.id}
          aria-controls={`exchange-${entry.id}`}
          onclick={() => (expanded = expanded === entry.id ? null : entry.id)}
        >
          {@html iconMarkup("search")}
        </button>
      </div>
      <div id={`exchange-${entry.id}`} class="exchange-calculation" hidden={expanded !== entry.id}>
        {#each entry.details as detail}<p class="damage-equation">{detail}</p>{/each}
      </div>
    </li>
  {/each}
  {#if complete}
    {#if finalHealth[opponent] === 0}<li class="exchange-outcome good">Your opponent died</li>{/if}
    {#if finalHealth[player] === 0}<li class="exchange-outcome bad">You died</li>{/if}
    {#if finalHealth[0] > 0 && finalHealth[1] > 0}<li class="exchange-outcome">
        Exchange ended
      </li>{/if}
    <li class="exchange-totals">
      {#if playerChange > 0}
        <span class="good">You healed +{playerChange} HP</span>
      {:else}
        <span class="bad">You took {loss(-playerChange)} HP</span>
      {/if}
      {#if opponentChange > 0}
        <span class="bad">Opponent healed +{opponentChange} HP</span>
      {:else}
        <span class="good">You dealt {loss(-opponentChange)} HP</span>
      {/if}
    </li>
  {/if}
</ol>
