<script lang="ts">
  import type { Item } from "../../rust/pkg/hex_command";
  import { pixelSprites } from "../presentation/pixel-art";

  let {
    items,
    onDrop,
  }: {
    items: readonly Readonly<Pick<Item, "id" | "name" | "description">>[];
    onDrop?: (item: string) => void;
  } = $props();
</script>

<ul class="item-list">
  {#each items as item, index (`${item.id}-${index}`)}
    <li class="item-row">
      <span class="item-icon" aria-hidden="true">
        {#if pixelSprites[`terrain_prop_items_${item.id}`]}
          <img
            class="pixel-art"
            src={pixelSprites[`terrain_prop_items_${item.id}`]}
            alt=""
            width="64"
            height="64"
          />
        {/if}
      </span>
      <div class="item-copy">
        <div class="item-heading">
          <strong>{item.name}</strong>
          {#if onDrop}
            <button
              class="item-drop"
              aria-label={`Drop ${item.name}`}
              onclick={() => onDrop?.(item.id)}
            >
              Drop
            </button>
          {/if}
        </div>
        <p>{item.description}</p>
      </div>
    </li>
  {/each}
</ul>

<style>
  .item-list {
    list-style: none;
    padding: 0;
    margin: 0;
    font:
      14px/1.5 system-ui,
      sans-serif;
  }
  .item-row {
    display: grid;
    grid-template-columns: 64px minmax(0, 1fr);
    align-items: start;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--line);
  }
  .item-icon,
  .item-icon img {
    display: block;
    width: 64px;
    height: 64px;
  }
  .item-copy {
    min-width: 0;
  }
  .item-copy strong {
    display: block;
    color: var(--text);
    font-weight: 600;
  }
  .item-copy p {
    margin: 3px 0 0;
    color: var(--muted);
  }
  .item-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .item-drop {
    flex: 0 0 auto;
    min-height: 32px;
    padding: 4px 10px;
    font-size: 13px;
  }
</style>
