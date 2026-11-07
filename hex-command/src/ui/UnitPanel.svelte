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
  const playback = $derived(view.combatPlayback);
  const result = $derived(playback?.combat);
  const frozen = $derived(playback && !playback.complete);
  const health = $derived(playback?.combat.strikes[playback.shown - 1]?.health);
  const shown = $derived(playback?.shown ?? 0);
  const complete = $derived(playback?.complete ?? false);
  const display = $derived(playback?.before ?? view);
  const panel = $derived(
    playback
      ? {
          kind: "combat" as const,
          attacker: playback.combat.units[0],
          defender: playback.combat.units[1],
          detailsResetKey: "playback",
        }
      : unitPanelView(display),
  );

  let panelElement: HTMLElement | undefined;
  const inspection = $derived([display.selected, display.target, display.inspectedTile].join("|"));

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
    if (!frozen) actions.attackSelected(weaponId);
  }
</script>

<section
  bind:this={panelElement}
  id="unit-panel"
  class="panel"
  class:resolving={!!frozen}
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
        faction={display.game.faction(panel.attacker.id)}
        artwork={display.artwork}
        hp={health?.[0]}
      />
      <span class="duel-divider" aria-hidden="true">⚔</span>
      <UnitCard
        unit={panel.defender}
        faction={display.game.faction(panel.defender.id)}
        artwork={display.artwork}
        hp={health?.[1]}
      />
    </div>
    {#if result}
      <CombatResult combat={result} {shown} {complete} />
    {:else}
      {#key panel.detailsResetKey}
        <AttackChoices
          game={display.game}
          attacker={panel.attacker}
          target={panel.defender}
          onAttack={attackWithWeapon}
          busy={!!frozen}
        />
      {/key}
    {/if}
  {:else if panel.kind === "details"}
    <UnitDetails
      details={panel.details}
      artwork={view.artwork}
      onAdvance={(type) => actions.advanceUnit(panel.details.unit.id, type)}
      onDrop={panel.details.faction.controllable &&
      display.game.turn === "player" &&
      !display.game.winner
        ? (item) => actions.dropItem(panel.details.unit.id, item)
        : undefined}
    />
  {/if}
</section>
