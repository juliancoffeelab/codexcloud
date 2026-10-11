<script lang="ts">
  import { tick } from "svelte";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { unitPanelView } from "./unit-panel-view";
  import UnitCard from "./UnitCard.svelte";
  import UnitDetails from "./UnitDetails.svelte";
  import AttackChoices from "./AttackChoices.svelte";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
    onNewBattle: () => void;
  }

  let { view, actions, onNewBattle }: Props = $props();
  const panel = $derived(unitPanelView(view));
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
    actions.attackSelected(weaponId);
  }
</script>

<section
  bind:this={panelElement}
  id="unit-panel"
  class="panel"
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
      <UnitCard unit={panel.attacker} artwork={view.artwork} />
      <span class="duel-divider" aria-hidden="true">⚔</span>
      <UnitCard unit={panel.defender} artwork={view.artwork} />
    </div>
    {#key panel.detailsResetKey}
      <AttackChoices
        game={view.game}
        attacker={panel.attacker}
        target={panel.defender}
        onAttack={attackWithWeapon}
      />
    {/key}
  {:else if panel.kind === "details"}
    <UnitDetails
      details={panel.details}
      artwork={view.artwork}
      onAdvance={(type) => actions.advanceUnit(panel.details.unit.id, type)}
    />
  {/if}
</section>
