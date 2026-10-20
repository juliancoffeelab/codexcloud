<script lang="ts">
  import { tick, onDestroy } from "svelte";
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
  let frozen = $state.raw<BattleView | null>(null);
  let health = $state.raw<readonly [number, number] | undefined>();
  let result = $state.raw<BattleView["combat"]>(null);
  let shown = $state(0);
  let complete = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  const display = $derived(frozen ?? view);
  const panel = $derived(unitPanelView(display));

  function finishAttack() {
    clearTimeout(timer);
    frozen = null;
    health = undefined;
    result = null;
    shown = 0;
    complete = false;
  }
  onDestroy(finishAttack);

  $effect(() => {
    if (
      frozen &&
      (view.selected !== frozen.selected ||
        view.inspectedTile !== frozen.inspectedTile ||
        view.game.mapId !== frozen.game.mapId)
    ) {
      finishAttack();
    }
  });
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
    if (frozen) return;
    const before = view;
    frozen = before;
    actions.attackSelected(weaponId);
    void tick().then(() => {
      if (frozen !== before) return;
      const combat = view.combat;
      if (!combat || combat === before.combat) {
        finishAttack();
        return;
      }
      result = combat;
      if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
        health = combat.strikes.at(-1)?.health;
        shown = combat.strikes.length;
        complete = true;
        return;
      }
      let index = 0;
      const advance = () => {
        const hit = combat.strikes[index++];
        if (!hit) {
          timer = undefined;
          complete = true;
          return;
        }
        health = hit.health;
        shown = index;
        timer = setTimeout(advance, 550);
      };
      timer = setTimeout(advance, 200);
    });
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
      <UnitCard unit={panel.attacker} artwork={display.artwork} hp={health?.[0]} />
      <span class="duel-divider" aria-hidden="true">⚔</span>
      <UnitCard unit={panel.defender} artwork={display.artwork} hp={health?.[1]} />
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
    />
  {/if}
</section>
