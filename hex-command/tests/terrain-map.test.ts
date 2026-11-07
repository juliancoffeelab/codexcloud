import assert from "node:assert/strict";
import { test } from "vitest";
import { terrainMapKey } from "../src/presentation/terrain-map";
import type { Artwork } from "../src/presentation/art";

test("Terrain cache keys ignore ownership but include every terrain input", () => {
  const tile = { id: "0,0", col: 0, row: 0, terrain: "road" as const, owner: null };
  const game = { mapId: "mirror" as const, cols: 1, rows: 1, tiles: [tile] };
  const key = terrainMapKey(game);
  const other = { ...tile, id: "1,0", col: 1 };
  assert.equal(
    terrainMapKey({ ...game, tiles: [tile, other] }),
    terrainMapKey({ ...game, tiles: [other, tile] }),
  );
  assert.equal(terrainMapKey({ ...game, tiles: [{ ...tile, owner: "player" }] }), key);
  for (const changed of [
    { ...game, mapId: "river-valley" as const },
    { ...game, cols: 2 },
    { ...game, rows: 2 },
    { ...game, tiles: [{ ...tile, col: 1 }] },
    { ...game, tiles: [{ ...tile, row: 1 }] },
    { ...game, tiles: [{ ...tile, id: "1,0" }] },
    { ...game, tiles: [{ ...tile, terrain: "forest" as const }] },
    { ...game, tiles: [{ ...tile, floor: "snow-forest" }] },
    { ...game, tiles: [{ ...tile, connections: ["e" as const, "w" as const] }] },
    { ...game, tiles: [{ ...tile, deadEnds: ["e" as const] }] },
  ])
    assert.notEqual(terrainMapKey(changed), key);
});

test("Static scenes reuse geometry through ownership and unit changes, but invalidate terrain and artwork", async () => {
  const { TerrainViewCache } = await import("../src/presentation/terrain-map");
  const tile = {
    id: "0,0",
    col: 0,
    row: 0,
    terrain: "plains" as const,
    owner: null,
  };
  const game = { mapId: "mirror" as const, cols: 1, rows: 1, tiles: [tile] };
  const artwork: Artwork = { frames: {} };
  const cache = new TerrainViewCache();
  const first = cache.prepare(game, artwork);
  assert.equal(cache.prepare({ ...game, tiles: [{ ...tile, owner: "player" }] }, artwork), first);
  assert.equal(
    cache.prepare(game, { frames: { player_sample: { url: "unit.png", width: 64, height: 64 } } }),
    first,
    "Unit artwork must not invalidate static terrain",
  );
  const geometry = first.tiles.get(tile.id)!;
  const frame = { url: "native.png", width: 64, height: 64 };
  const prepared = cache.prepare(game, { frames: { [geometry.frameKey]: frame } });
  assert.equal(prepared.key, first.key);
  assert.equal(prepared.tiles.get(tile.id)?.points, geometry.points);
  assert.equal(prepared.tiles.get(tile.id)?.terrainFrame, frame);
  for (const change of [
    { terrain: "forest" as const },
    { floor: "snow-forest" },
    { terrain: "road" as const, connections: ["e" as const] },
    { col: 1 },
  ]) {
    const changed = cache.prepare({ ...game, tiles: [{ ...tile, ...change }] }, artwork);
    assert.notEqual(changed.key, first.key);
  }
  assert.notEqual(cache.prepare({ ...game, mapId: "river-valley" }, artwork).key, first.key);
});
