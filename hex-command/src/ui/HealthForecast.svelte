<script lang="ts">
  import type { HealthForecastView } from "./combat-view";

  let { forecast }: { forecast: HealthForecastView } = $props();
</script>

<span class={`attack-health ${forecast.side}`} data-forecast={forecast.side}>
  <span class="forecast-label">
    <span>
      {forecast.currentHp} →
      <b class={forecast.remainingClass}>≈{forecast.remainingHp}</b>
      / {forecast.maximumHp} HP
    </span>
  </span>
  <span class="health-track" role="img" aria-label={forecast.healthLabel}>
    <span class="health-lost" style={`width:${forecast.currentPercent}%`}></span>
    <span class="health-left" style={`width:${forecast.remainingPercent}%`}></span>
  </span>
  <span class="forecast-impact">{forecast.impactLabel}</span>
</span>
