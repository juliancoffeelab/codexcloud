<script lang="ts">
  import type { HealthForecastView } from "./combat-view";

  let { forecast }: { forecast: HealthForecastView } = $props();
</script>

<span class={`attack-health ${forecast.side}`} data-forecast={forecast.side}>
  <span class="forecast-label">
    <span>
      {forecast.currentHp} →
      <b class={forecast.remainingClass}>{forecast.remainingLabel}</b>
      / {forecast.maximumHp} HP
    </span>
  </span>
  <span class="health-track" role="img" aria-label={forecast.healthLabel}>
    <span
      class="forecast-loss"
      style={`left:${forecast.maximumPercent}%;width:${Math.max(0, forecast.currentPercent - forecast.maximumPercent)}%`}
    ></span>
    <span
      class="forecast-possible"
      style={`left:${Math.min(forecast.minimumPercent, forecast.currentPercent)}%;width:${Math.min(forecast.maximumPercent, forecast.currentPercent) - Math.min(forecast.minimumPercent, forecast.currentPercent)}%`}
    ></span>
    <span
      class="forecast-certain"
      style={`width:${Math.min(forecast.minimumPercent, forecast.currentPercent)}%`}
    ></span>
    <span
      class="forecast-healing"
      style={`left:${forecast.currentPercent}%;width:${Math.max(0, forecast.maximumPercent - forecast.currentPercent)}%`}
    ></span>
    <span
      class="forecast-current"
      style={`left:${forecast.currentPercent}%`}
      title={`Current HP: ${forecast.currentHp}`}
    ></span>
  </span>
  <span class="forecast-impact">
    {#each forecast.impactLabel.split(/(\([^)]*\))/) as part}
      {#if part.startsWith("(")}
        <span class="keep-together">{part}</span>
      {:else}{part}{/if}
    {/each}
  </span>
</span>
