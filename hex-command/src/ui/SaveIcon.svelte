<script lang="ts">
  let {
    name,
  }: { name: "load" | "details" | "gold" | "villages" | "objectives" | "units" | "trash" } =
    $props();
  const paths = {
    load: '<path d="M5 4v16l14-8L5 4z"/>',
    details: '<circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/>',
    gold: '<circle cx="12" cy="12" r="9"/><path d="M14 8h-3a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4h-3m2-10v12"/>',
    villages: '<path d="m3 11 9-8 9 8M5 10v11h14V10M10 21v-7h4v7"/>',
    objectives: '<path d="M5 21V3m0 0h14l-4 5 4 5H5"/>',
    units: '<circle cx="12" cy="6.5" r="3.5"/><path d="M5 21v-3a7 7 0 0 1 14 0v3"/>',
    trash: '<path d="M3 7h18M9 7V3h6v4M5 7l1 14h12l1-14M10 11v6M14 11v6"/>',
  };
</script>

<svg class:filled={name === "load"} viewBox="0 0 24 24" aria-hidden="true" focusable="false">
  {@html paths[name]}
</svg>

<style>
  svg {
    width: var(--save-icon-size, 14px);
    height: var(--save-icon-size, 14px);
    flex-shrink: 0;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .filled {
    fill: currentColor;
    stroke: none;
  }
</style>
