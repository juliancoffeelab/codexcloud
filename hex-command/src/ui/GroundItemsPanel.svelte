<script lang="ts">
  import type { BattleView } from "../controller/view";
  import ItemList from "./ItemList.svelte";
  let { view }: { view: BattleView } = $props();
  const items = $derived(
    view.game.groundItems
      .filter((ground) => ground.tile === view.inspectedTile)
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
