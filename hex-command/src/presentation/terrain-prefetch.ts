import type { TileView } from "../game/view";
import { artFrame, type Artwork } from "./art";
import { terrainLayout } from "./terrain-layout";

/** Warm the actual authored inspection variants present on the map. */
export function terrainInspectionUrls(
  artwork: Artwork,
  tiles: readonly Pick<TileView, "terrain" | "id" | "connections" | "deadEnds" | "wallCorners">[],
): string[] {
  return [
    ...new Set(
      tiles.flatMap((tile) => {
        const frame = artFrame(artwork, "terrain", terrainLayout(tile, "inspection").name);
        return frame ? [frame.url] : [];
      }),
    ),
  ];
}

/** Defer background work until after a paint, then use idle time when available. */
function afterPaint(task: () => void): () => void {
  let cancelIdle: (() => void) | undefined;
  const frame = requestAnimationFrame(() => {
    if (typeof requestIdleCallback === "function") {
      const idle = requestIdleCallback(task);
      cancelIdle = () => cancelIdleCallback(idle);
    } else {
      const timer = setTimeout(task, 0);
      cancelIdle = () => clearTimeout(timer);
    }
  });
  return () => {
    cancelAnimationFrame(frame);
    cancelIdle?.();
  };
}

async function loadSprite(url: string): Promise<void> {
  const image = new Image();
  image.fetchPriority = "low";
  image.decoding = "async";
  image.src = url;
  await image.decode();
}

/** Warm authored inspection images; the browser owns the image cache. */
export class TerrainTilePrefetcher {
  private queue: string[] = [];
  private loaded = new Set<string>();
  private loading = new Set<string>();
  private cancel?: () => void;

  constructor(
    private readonly defer = afterPaint,
    private readonly load = loadSprite,
  ) {}

  start(urls: readonly string[]): () => void {
    this.stop();
    const queue = [...new Set(urls)].filter(
      (url) => !this.loaded.has(url) && !this.loading.has(url),
    );
    this.queue = queue;
    this.schedule();
    return () => {
      if (this.queue === queue) this.stop();
    };
  }

  private stop() {
    this.cancel?.();
    this.cancel = undefined;
    this.queue = [];
  }

  private schedule() {
    if (this.cancel || !this.queue.length || this.loading.size >= 2) return;
    this.cancel = this.defer(() => {
      this.cancel = undefined;
      while (this.queue.length && this.loading.size < 2) {
        const url = this.queue.shift()!;
        this.loading.add(url);
        void this.load(url)
          .then(() => this.loaded.add(url))
          .catch(() => {
            // A visible inspection can retry normally; prefetch failures stay silent.
          })
          .finally(() => {
            this.loading.delete(url);
            this.schedule();
          });
      }
    });
  }
}
