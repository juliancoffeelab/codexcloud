<script lang="ts">
  import { cellKey } from "../game/hex";
  import type { BattleView } from "../controller/view";
  import ItemList from "./ItemList.svelte";
  let { view }: { view: Pick<BattleView, "interactionGame" | "inspectedTile"> } = $props();
  const items = $derived(
    view.interactionGame.groundItems
      .filter((ground) => cellKey(ground.tile) === view.inspectedTile)
      .map((ground) => ground.item),
  );
</script>

<section
  id="ground-items-panel"
  class="panel"
  aria-label="Items on the ground"
  hidden={!items.length}
>
  <h2>On the ground</h2>
  <ItemList {items} />
</section>

<style>
  section {
    border-top: 0;
  }
  h2 {
    margin: 0 0 8px;
  }
</style>
