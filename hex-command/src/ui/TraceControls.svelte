<script lang="ts">
  import type { BattleView } from "../controller/view";
  import {
    subscribeTrace,
    startPerformanceTrace,
    stopPerformanceTrace,
    downloadPerformanceTrace,
    type TraceState,
  } from "../presentation/performance-trace";
  let { view, onStart }: { view: BattleView; onStart: () => void } = $props();
  let trace = $state<TraceState>({ status: "empty", events: 0, dropped: 0 });
  let error = $state("");
  $effect(() =>
    subscribeTrace((value) => {
      trace = value;
    }),
  );
  function context() {
    return {
      mapId: view.game.mapId,
      cols: view.game.cols,
      rows: view.game.rows,
      round: view.game.round,
      turn: view.game.turn,
      snapshot: view.game.snapshot(),
      lighting: view.game.presentation.lighting,
      fog: view.game.presentation.fog,
    };
  }
  function start() {
    error = "";
    try {
      startPerformanceTrace(context);
      onStart();
    } catch (cause) {
      error = String(cause);
    }
  }
  async function download() {
    error = "";
    try {
      await downloadPerformanceTrace();
    } catch (cause) {
      error = String(cause);
    }
  }
</script>

<p>
  Record timings while you play, then download the JSON trace to attach here. Recording stops after
  ten minutes or 250,000 events. Recordings stay in memory until reload. No recording is uploaded
  automatically.
</p>
<div class="trace-buttons">
  <button onclick={start} disabled={trace.status === "recording" || trace.status === "stopping"}>
    {trace.status === "ready" ? "Start new trace" : "Start recording trace"}
  </button>
  <button onclick={() => void stopPerformanceTrace()} disabled={trace.status !== "recording"}>
    Stop recording
  </button>
  <button
    onclick={() => void download()}
    disabled={trace.status === "empty" || trace.status === "stopping"}
  >
    {trace.status === "recording" ? "Stop and download trace" : "Download trace"}
  </button>
</div>
<p role="status">
  {trace.status === "recording"
    ? "Recording"
    : trace.status === "stopping"
      ? "Finishing worker data…"
      : trace.status === "ready"
        ? "Trace ready"
        : "Not recording"} · {trace.events.toLocaleString()} events{trace.reason
    ? ` · ${trace.reason}`
    : ""}{trace.dropped ? ` · ${trace.dropped} events dropped` : ""}
</p>
{#if error}<p role="alert">{error}</p>{/if}
<p>
  Includes action/engine timings, browser frame intervals, map-worker stages and optional GPU
  timers. GPU timers depend on browser support; frame intervals are not physical display FPS.
</p>

<style>
  .trace-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
