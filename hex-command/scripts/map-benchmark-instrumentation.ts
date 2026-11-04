/** Browser-only completion probe. Never imported by the game or default tests. */
import { PhaserBattlefield } from "../src/presentation/phaser-battlefield";

export function installMapBenchmark() {
  let gl: WebGL2RenderingContext | undefined;
  let query: WebGLQuery | undefined;
  let extension: { TIME_ELAPSED_EXT: number; GPU_DISJOINT_EXT: number } | null = null;
  const original = PhaserBattlefield.prototype.render;
  PhaserBattlefield.prototype.render = function (...args) {
    gl = this.context;
    extension = gl.getExtension("EXT_disjoint_timer_query_webgl2");
    if (query) gl.deleteQuery(query);
    query = extension ? (gl.createQuery() ?? undefined) : undefined;
    if (query) gl.beginQuery(extension!.TIME_ELAPSED_EXT, query);
    try {
      return original.apply(this, args);
    } finally {
      if (query) gl.endQuery(extension!.TIME_ELAPSED_EXT);
    }
  };
  return {
    async completed(): Promise<number | null> {
      if (!gl) throw new Error("Benchmark received no rendered frame");
      const context = gl,
        sample = query,
        timer = extension;
      query = undefined;
      const fence = context.fenceSync(context.SYNC_GPU_COMMANDS_COMPLETE, 0);
      if (!fence) throw new Error("Benchmark could not create completion fence");
      context.flush();
      const deadline = performance.now() + 5000;
      try {
        while (true) {
          const state = context.clientWaitSync(fence, 0, 0);
          if (state === context.WAIT_FAILED || context.isContextLost())
            throw new Error("Benchmark GPU completion failed");
          if (state !== context.TIMEOUT_EXPIRED) break;
          if (performance.now() > deadline) throw new Error("Benchmark GPU completion timed out");
          await new Promise((resolve) => setTimeout(resolve, 1));
        }
        return sample &&
          timer &&
          !context.getParameter(timer.GPU_DISJOINT_EXT) &&
          context.getQueryParameter(sample, context.QUERY_RESULT_AVAILABLE)
          ? (context.getQueryParameter(sample, context.QUERY_RESULT) as number) / 1e6
          : null;
      } finally {
        context.deleteSync(fence);
        if (sample) context.deleteQuery(sample);
      }
    },
    dispose() {
      PhaserBattlefield.prototype.render = original;
      if (query) gl?.deleteQuery(query);
    },
  };
}
