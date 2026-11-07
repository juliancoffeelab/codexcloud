<script lang="ts">
  import { errorDetails } from "../error-details";
  import type { FrameImage } from "../presentation/art";
  import type { HudArtworkCache } from "../presentation/hud-artwork";
  let { frame, cache }: { frame: Readonly<FrameImage>; cache: HudArtworkCache } = $props();
  let host: HTMLDivElement;
  let loading = $state(false);
  let error = $state<string>();
  let attempt = $state(0);

  $effect(() => {
    const node = host;
    const current = frame;
    attempt;
    let active = true;
    const show = (image: HTMLImageElement) => {
      if (!active) return;
      image.className = "terrain-card-art pixel-art";
      image.alt = "";
      image.width = 80;
      image.height = 80;
      node.replaceChildren(image);
      loading = false;
    };
    error = undefined;
    const ready = cache.ready(current);
    if (ready) show(ready);
    else {
      node.replaceChildren();
      loading = true;
      void cache
        .load(current)
        .then(show)
        .catch((reason) => {
          if (!active) return;
          loading = false;
          error = errorDetails(reason);
        });
    }
    return () => {
      active = false;
    };
  });
</script>

<div class="preview-image" bind:this={host} aria-hidden="true"></div>
{#if loading}
  <span class="preview-status" role="status">Loading preview…</span>
{:else if error}
  <div class="preview-status" role="alert" title={error}>
    Preview could not load.
    <button
      onclick={() => {
        void cache.retry(frame).catch(() => {});
        attempt++;
      }}
    >
      Retry
    </button>
  </div>
{/if}

<style>
  .preview-image {
    width: 80px;
    height: 80px;
  }
  .preview-image :global(img) {
    display: block;
    width: 80px;
    height: 80px;
    object-fit: contain;
  }
  .preview-status {
    display: block;
    width: 80px;
    font:
      12px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
  }
  .preview-status button {
    display: block;
    margin-top: 4px;
  }
</style>
