<script lang="ts">
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

  function attackWithWeapon(weaponId: string): void {
    actions.attackSelected(weaponId);
  }
</script>

<section id="unit-panel" class="panel" aria-label="Selected unit" hidden={panel.kind === "hidden"}>
  {#if panel.kind === "outcome"}
    <div class="unit-title">
      <h2>{panel.title}</h2>
    </div>
    <p>{panel.description}</p>
    <button id="play-again" class="primary" onclick={onNewBattle}>Play again</button>
  {:else if panel.kind === "combat"}
    <div class="duel">
      <UnitCard unit={panel.attacker} artStyle={view.artStyle} />
      <span class="duel-divider" aria-hidden="true">⚔</span>
      <UnitCard unit={panel.defender} artStyle={view.artStyle} />
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
    <UnitDetails details={panel.details} artStyle={view.artStyle} />
  {/if}
</section>
