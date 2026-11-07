<script lang="ts">
  import { tick } from "svelte";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { unitPanelView } from "./unit-panel-view";
  import UnitCard from "./UnitCard.svelte";
  import UnitDetails from "./UnitDetails.svelte";
  import AttackChoices from "./AttackChoices.svelte";
  import CombatResult from "./CombatResult.svelte";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
    onNewBattle: () => void;
  }

  let { view, actions, onNewBattle }: Props = $props();
  const panel = $derived(unitPanelView(view));
  const playback = $derived(panel.kind === "combat" ? panel.playback : undefined);
  const result = $derived(playback?.combat);
  const resolving = $derived(!!playback && !playback.complete);
  const health = $derived(playback?.combat.strikes[playback.shown - 1]?.health);
  const shown = $derived(playback?.shown ?? 0);
  const complete = $derived(playback?.complete ?? false);

  let panelElement: HTMLElement | undefined;
  const inspection = $derived([view.selected, view.target, view.inspectedTile].join("|"));

  $effect(() => {
    inspection;
    const node = panelElement;
    if (!node) return;
    let current = true;
    void tick().then(() => {
      if (!current || !matchMedia("(max-width:1000px)").matches) return;
      const attack = node.querySelector<HTMLElement>(".attack-actions");
      attack?.scrollIntoView({ block: "nearest" });
    });
    return () => {
      current = false;
    };
  });

  function attackWithWeapon(weaponId: string): void {
    if (!resolving) actions.attackSelected(weaponId);
  }
</script>

<section
  bind:this={panelElement}
  id="unit-panel"
  class="panel"
  class:resolving
  aria-label="Selected unit"
  hidden={panel.kind === "hidden"}
>
  {#if panel.kind === "outcome"}
    <div class="unit-title">
      <h2>{panel.title}</h2>
    </div>
    <p>{panel.description}</p>
    <button id="play-again" class="primary" onclick={onNewBattle}>Play again</button>
  {:else if panel.kind === "combat"}
    <div class="duel">
      <UnitCard
        unit={panel.attacker}
        faction={panel.game.faction(panel.attacker.id)}
        artwork={view.artwork}
        hp={health?.[0]}
      />
      <span class="duel-divider" aria-hidden="true">⚔</span>
      <UnitCard
        unit={panel.defender}
        faction={panel.game.faction(panel.defender.id)}
        artwork={view.artwork}
        hp={health?.[1]}
      />
    </div>
    {#if result}
      <CombatResult combat={result} {shown} {complete} />
    {:else}
      {#key panel.detailsResetKey}
        <AttackChoices
          game={panel.game}
          attacker={panel.attacker}
          target={panel.defender}
          onAttack={attackWithWeapon}
          busy={resolving}
        />
      {/key}
    {/if}
  {:else if panel.kind === "details"}
    <UnitDetails
      details={panel.details}
      artwork={view.artwork}
      onAdvance={(type) => actions.advanceUnit(panel.details.unit.id, type)}
      onDrop={panel.details.faction.controllable && view.game.turn === "player" && !view.game.winner
        ? (item) => actions.dropItem(panel.details.unit.id, item)
        : undefined}
    />
  {/if}
</section>
