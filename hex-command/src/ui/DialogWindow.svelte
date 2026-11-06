<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    title,
    children,
    dialog = $bindable(),
    body = $bindable(),
    id,
    width = "420px",
    tall = false,
    onBack,
    backLabel = "Back to Settings",
    onBeforeClose,
    className = "",
  }: {
    title: string;
    children: Snippet;
    dialog?: HTMLDialogElement;
    body?: HTMLDivElement;
    id?: string;
    width?: string;
    tall?: boolean;
    onBack?: () => void;
    backLabel?: string;
    onBeforeClose?: () => void;
    className?: string;
  } = $props();
  let heading: HTMLHeadingElement;
  export function focusHeading() {
    heading.focus({ preventScroll: true });
  }
  function close() {
    onBeforeClose?.();
    dialog?.close();
  }
  function backdrop(event: MouseEvent) {
    if (event.target !== dialog || !dialog) return;
    const bounds = dialog.getBoundingClientRect();
    if (
      event.clientX < bounds.left ||
      event.clientX > bounds.right ||
      event.clientY < bounds.top ||
      event.clientY > bounds.bottom
    )
      close();
  }
</script>

<dialog
  bind:this={dialog}
  {id}
  class={`dialog-window ${className}`}
  class:tall
  style:--window-width={width}
  aria-label={title}
  onclick={backdrop}
  oncancel={() => onBeforeClose?.()}
>
  <header class:with-back={!!onBack}>
    {#if onBack}
      <button class="window-icon" onclick={onBack} aria-label={backLabel}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m14 5-7 7 7 7" /></svg>
      </button>
    {/if}
    <h2 bind:this={heading} tabindex="-1">{title}</h2>
    <button class="window-icon" onclick={close} aria-label={`Close ${title.toLowerCase()}`}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18" /></svg>
    </button>
  </header>
  <div class="window-content" bind:this={body}>{@render children()}</div>
</dialog>

<style>
  .dialog-window {
    width: calc(100% - 32px);
    max-width: var(--window-width);
    max-height: calc(100dvh - 48px);
    padding: 0;
    overflow: hidden;
    background: var(--surface);
  }
  .dialog-window[open] {
    display: flex;
    flex-direction: column;
  }
  .dialog-window.tall {
    height: min(720px, calc(100dvh - 48px));
  }
  header {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 78px;
    padding: 16px 12px 16px 20px;
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }
  header.with-back {
    padding-left: 8px;
  }
  h2 {
    flex: 1;
    min-width: 0;
    margin: 0;
    font:
      27px/1.12 Georgia,
      serif;
    outline: none;
  }
  .window-icon {
    display: grid;
    place-items: center;
    flex: 0 0 42px;
    width: 42px;
    height: 42px;
    min-height: 42px;
    padding: 0;
    border: 0;
    background: transparent;
    color: #b9c0ad;
  }
  .window-icon:hover {
    background: #ffffff08;
    color: var(--text);
  }
  svg {
    width: 20px;
    height: 20px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .window-content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 0 20px 20px;
    scrollbar-width: thin;
    scrollbar-color: var(--line) transparent;
  }
  @media (max-width: 540px) {
    .dialog-window {
      width: calc(100% - 24px);
      max-height: calc(100dvh - 24px);
    }
    .dialog-window.tall {
      height: calc(100dvh - 24px);
    }
  }
</style>
