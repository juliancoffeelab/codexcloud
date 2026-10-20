<script lang="ts" module>
  import type { MapDefinition, UnitType } from "../game/types";
  import type { Immutable } from "../game/immutable";
  export interface Choice {
    value: string;
    label: string;
    sprite?: UnitType;
    map?: Immutable<MapDefinition>;
  }
</script>

<script lang="ts">
  import { artSprite, currentArtwork } from "../presentation/art";
  import MapThumbnail from "./MapThumbnail.svelte";
  let {
    label,
    choices,
    value = $bindable(""),
    disabled = false,
    side = "player",
    onchange = () => {},
  }: {
    label: string;
    choices: readonly Choice[];
    value?: string;
    disabled?: boolean;
    side?: "player" | "enemy";
    onchange?: () => void;
  } = $props();
  let dialog: HTMLDialogElement;
  let trigger: HTMLButtonElement;
  let expanded = $state(false);
  const selected = $derived(choices.find((choice) => choice.value === value));
  function open() {
    expanded = true;
    dialog.showModal();
    dialog.querySelector<HTMLButtonElement>('[aria-selected="true"]')?.focus();
  }
  function choose(choice: Choice) {
    value = choice.value;
    onchange();
    dialog.close();
  }
  function navigate(event: KeyboardEvent) {
    const rows = [...dialog.querySelectorAll<HTMLButtonElement>('[role="option"]')];
    if (!rows.length || !["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const i = rows.indexOf(document.activeElement as HTMLButtonElement);
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? rows.length - 1
          : (i + (event.key === "ArrowDown" ? 1 : -1) + rows.length) % rows.length;
    rows[next]?.focus();
  }
  function backdrop(event: MouseEvent) {
    if (event.target !== dialog) return;
    const b = dialog.getBoundingClientRect();
    if (
      event.clientX < b.left ||
      event.clientX > b.right ||
      event.clientY < b.top ||
      event.clientY > b.bottom
    )
      dialog.close();
  }
</script>

<button
  class="choice-trigger"
  type="button"
  bind:this={trigger}
  aria-label={label}
  aria-haspopup="dialog"
  aria-expanded={expanded}
  {disabled}
  onclick={open}
>
  <span>{selected?.label ?? value}</span>
  <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
    <path d="m4 6 4 4 4-4" fill="none" stroke="currentColor" stroke-width="2" />
  </svg>
</button>
<dialog
  class="choice-dialog"
  aria-label={label}
  bind:this={dialog}
  onclick={backdrop}
  onkeydown={navigate}
  onclose={() => {
    expanded = false;
    trigger.focus();
  }}
>
  <header class="setup-heading">
    <h2>{label}</h2>
    <button class="setup-close" aria-label="Close picker" onclick={() => dialog.close()}>×</button>
  </header>
  <div class="choice-list" role="listbox" aria-label={label}>
    {#each choices as choice}
      <button
        class="choice-row"
        role="option"
        aria-selected={choice.value === value}
        onclick={() => choose(choice)}
      >
        {#if choice.sprite}{@html artSprite(
            currentArtwork(),
            "units",
            choice.sprite,
            0,
            0,
            64,
            64,
            "",
            side,
          )}{/if}
        {#if choice.map}<MapThumbnail map={choice.map} />{/if}
        <span>{choice.label}</span>
        <span class="choice-tick" aria-hidden="true">{choice.value === value ? "✓" : ""}</span>
      </button>
    {/each}
  </div>
</dialog>
