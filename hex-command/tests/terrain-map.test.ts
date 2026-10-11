import assert from "node:assert/strict";
import { test, vi } from "vitest";
import * as Terrain from "@frontend/Terrain";
import * as Plan from "@frontend/Terrain_plan";
import * as Art from "@frontend/Art";
import type { State, Tile } from "../rust/pkg/hex_command";
import { mockSpriteCanvas, required } from "./helpers";

const tile = (col = 0, row = 0): Tile => ({
  id: `${col},${row}`,
  col,
  row,
  terrain: "plains",
  owner: null,
});
const state = (tiles = [tile()], cols = 1, rows = 1): State => ({
  mapId: "mirror",
  mapRevision: 1,
  cols,
  rows,
  round: 1,
  turn: "player",
  gold: { player: 0, enemy: 0 },
  winner: null,
  nextId: 1,
  log: [],
  tiles,
  units: [],
});
const key = (game: State) => Plan.key(game.mapId, game.cols, game.rows, game.tiles);
function sprite(game: State): Terrain.Sprite {
  const dimensions = Plan.dimensions(
    Plan.prepare(game.mapId, game.cols, game.rows, game.tiles, Art.current()),
  );
  return {
    url: "map.png",
    width: dimensions.width,
    height: dimensions.height,
    displayWidth: dimensions.display_width,
    displayHeight: dimensions.display_height,
  };
}
function artwork(frames: Record<string, Art.Frame>): Art.Artwork {
  return { frames } as unknown as Art.Artwork;
}
/** Different authored layouts deliberately share one decoder URL in this fixture. */
function uniformArtwork(): Art.Artwork {
  const frame: Art.Frame = { url: "shared.png", width: 64, height: 64 };
  return artwork(
    Object.fromEntries(Object.keys(Art.frames(Art.current())).map((key) => [key, frame])),
  );
}
class ControlledImage {
  static requests: ControlledImage[] = [];
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  src = "";
  constructor() {
    ControlledImage.requests.push(this);
  }
  removeAttribute(name: string) {
    assert.equal(name, "src");
  }
}
interface Recorded {
  draws: { source: string; coordinates: number[] }[];
  scales: number[][];
  paths: number[][][];
  clips: number;
  smoothing: boolean;
}
function recording(bitmap: Terrain.Bitmap): Recorded {
  let canvas: HTMLCanvasElement | undefined;
  const host = {
    replaceChildren(value: HTMLCanvasElement) {
      canvas = value;
    },
  };
  Terrain.mount(host as unknown as Element, bitmap);
  return JSON.parse(
    Buffer.from(required(canvas).toDataURL().split(",")[1]!, "base64").toString(),
  ) as Recorded;
}

test("Static keys ignore ownership and order, and include every authored terrain input", () => {
  const game = state();
  const original = key(game);
  assert.equal(key({ ...game, tiles: [{ ...tile(), owner: "player" }] }), original);
  const other = tile(1);
  assert.equal(key({ ...game, tiles: [tile(), other] }), key({ ...game, tiles: [other, tile()] }));
  for (const changed of [
    { ...game, mapId: "river" },
    { ...game, cols: 2 },
    { ...game, rows: 2 },
    ...[
      { col: 1 },
      { row: 1 },
      { id: "1,0" },
      { terrain: "forest" },
      { floor: "snow-forest" },
      { connections: ["e", "w"] },
      { deadEnds: ["e"] },
    ].map((change) => ({ ...game, tiles: [{ ...tile(), ...change } as Tile] })),
  ])
    assert.notEqual(key(changed), original);
});

test("Authored PNG previews validate metadata and overlay only changed cells", () => {
  const game = state([tile(), tile(1)], 2);
  const image = sprite(game);
  const owner = Terrain.create({ [key(game)]: image, broken: image });
  const first = Terrain.prepare(owner, game, Art.current());
  const exact = required(Terrain.preview(owner, first));
  assert.equal(Terrain.sprite(exact), image);
  assert.deepEqual(Terrain.changed(exact), []);
  const edited = { ...game, tiles: [tile(), { ...tile(1), terrain: "forest" }] };
  const preview = required(Terrain.preview(owner, Terrain.prepare(owner, edited, Art.current())));
  assert.equal(Terrain.sprite(preview), image);
  assert.deepEqual(Terrain.changed(preview), ["1,0"]);
  assert.equal(Terrain.changed_contains(preview, "1,0"), true);
  assert.equal(Terrain.changed_contains(preview, "0,0"), false);
  for (const change of [{ mapId: "river" }, { cols: 3 }, { rows: 2 }])
    assert.equal(
      Terrain.preview(owner, Terrain.prepare(owner, { ...game, ...change }, Art.current())),
      undefined,
    );
  for (const malformed of [
    { ...image, width: 0 },
    { ...image, height: NaN },
    { ...image, displayWidth: 70 },
    { ...image, url: "" },
  ]) {
    const invalid = Terrain.create({ [key(game)]: malformed });
    assert.equal(
      Terrain.preview(invalid, Terrain.prepare(invalid, game, Art.current())),
      undefined,
    );
  }
  Terrain.close(owner);
});

test("Immutable static scenes retain identity across unit, ownership, ordering and unused-art changes", () => {
  const game = state([tile(), tile(1)], 2);
  const owner = Terrain.create({});
  const frames = { ...Art.frames(Art.current()) };
  const art = artwork(frames);
  const first = Terrain.prepare(owner, game, art);
  const changed = { ...game, tiles: [{ ...tile(1), owner: "player" as const }, tile()], round: 7 };
  assert.equal(Terrain.prepare(owner, changed, art), first);
  assert.equal(
    Terrain.prepare(
      owner,
      game,
      artwork({ ...frames, player_unused: { url: "unused.png", width: 64, height: 64 } }),
    ),
    first,
  );
  const cell = required(Terrain.get_tile(first, "0,0"));
  const replacement = { url: "replacement.png", width: 64, height: 64 };
  const artKey = Art.frame_key("terrain", cell.name, undefined, cell.floor);
  const updated = Terrain.prepare(owner, game, artwork({ ...frames, [artKey]: replacement }));
  assert.notEqual(updated, first);
  assert.equal(Terrain.key(updated), Terrain.key(first));
  assert.equal(required(Terrain.get_tile(updated, "0,0")).points, cell.points);
  assert.equal(required(Terrain.get_tile(updated, "0,0")).frame, replacement);
  Terrain.close(owner);
});

test("Fallback shares one bitmap promise, paints native clipping, retries failures and cancels stale decoders", async () => {
  ControlledImage.requests = [];
  vi.stubGlobal("Image", ControlledImage);
  const restore = mockSpriteCanvas();
  const owner = Terrain.create({});
  const art = uniformArtwork();
  try {
    const game = state([tile(), tile(1)], 2);
    const scene = Terrain.prepare(owner, game, art);
    Terrain.activate(owner, scene);
    const first = Terrain.bake(owner, scene);
    assert.equal(Terrain.bake(owner, scene), first);
    assert.equal(ControlledImage.requests.length, 1);
    required(ControlledImage.requests[0]).onerror?.();
    assert.equal(await first, undefined);
    const retry = Terrain.bake(owner, scene);
    assert.notEqual(retry, first);
    required(ControlledImage.requests[1]).onload?.();
    const bitmap = required(await retry);
    assert(Object.isFrozen(bitmap));
    assert(Terrain.valid_bitmap(scene, bitmap));
    const dimensions = Terrain.bitmap_size(bitmap);
    assert.equal(dimensions.width, sprite(game).width);
    const painted = recording(bitmap);
    assert.equal(painted.smoothing, false);
    assert.equal(painted.clips, 2);
    assert.deepEqual(painted.scales, [[64 / 70, 64 / 70]]);
    assert.equal(painted.paths[0]?.length, 6);
    assert.deepEqual(painted.draws[0]?.coordinates, [8, 7, 70, 70]);
    assert.equal(Terrain.bake(owner, scene), retry);
    const next = Terrain.prepare(owner, { ...game, mapId: "river" }, art);
    Terrain.activate(owner, next);
    const obsolete = Terrain.bake(owner, next);
    const pending = required(ControlledImage.requests[2]);
    Terrain.activate(owner, Terrain.prepare(owner, game, art));
    assert.equal(pending.onload, null);
    assert.equal(pending.onerror, null);
    assert.equal(await obsolete, undefined);
    assert(!Terrain.valid_bitmap(next, bitmap));
    const replayScene = Terrain.prepare(owner, game, art);
    Terrain.close(owner);
    Terrain.activate(owner, replayScene);
    const replay = Terrain.bake(owner, replayScene);
    required(ControlledImage.requests[3]).onload?.();
    assert(
      Terrain.valid_bitmap(replayScene, required(await replay)),
      "Effect cleanup/setup can reuse its memoized scene",
    );
  } finally {
    Terrain.close(owner);
    restore();
    vi.unstubAllGlobals();
  }
});

test("Fallback image decoding stays bounded and unmount settles pending work", async () => {
  ControlledImage.requests = [];
  vi.stubGlobal("Image", ControlledImage);
  const restore = mockSpriteCanvas();
  const owner = Terrain.create({});
  try {
    const terrains = ["plains", "road", "water", "forest", "hill", "village"];
    const game = state(
      terrains.map((terrain, col) => ({ ...tile(col), terrain })),
      6,
    );
    const scene = Terrain.prepare(owner, game, Art.current());
    Terrain.activate(owner, scene);
    const pending = Terrain.bake(owner, scene);
    assert.equal(ControlledImage.requests.length, 4);
    for (const image of ControlledImage.requests.slice()) image.onload?.();
    await Promise.resolve();
    await Promise.resolve();
    assert(ControlledImage.requests.length <= 8);
    assert(ControlledImage.requests.length > 4);
    Terrain.close(owner);
    assert.equal(await pending, undefined);
    Terrain.close(owner);
  } finally {
    Terrain.close(owner);
    restore();
    vi.unstubAllGlobals();
  }
});

test("Large fallback yields between drawing batches and cancelled yields settle without publishing", async () => {
  vi.useFakeTimers();
  class ReadyImage {
    onload?: () => void;
    onerror?: () => void;
    set src(_value: string) {
      this.onload?.();
    }
    removeAttribute() {}
  }
  vi.stubGlobal("Image", ReadyImage);
  const restore = mockSpriteCanvas();
  const owner = Terrain.create({});
  const art = uniformArtwork();
  try {
    const game = state(
      Array.from({ length: 65 }, (_, row) => tile(0, row)),
      1,
      65,
    );
    const scene = Terrain.prepare(owner, game, art);
    let published = false;
    Terrain.activate(owner, scene);
    const ready = Terrain.bake(owner, scene).then((bitmap) => {
      published = true;
      return bitmap;
    });
    for (let index = 0; index < 8; index++) await Promise.resolve();
    assert.equal(published, false);
    assert.equal(vi.getTimerCount(), 1);
    await vi.runAllTimersAsync();
    assert(await ready);
    const next = Terrain.prepare(owner, { ...game, mapId: "river" }, art);
    Terrain.activate(owner, next);
    const cancelled = Terrain.bake(owner, next);
    for (let index = 0; index < 8; index++) await Promise.resolve();
    assert.equal(vi.getTimerCount(), 1);
    Terrain.close(owner);
    assert.equal(await cancelled, undefined);
    assert.equal(vi.getTimerCount(), 0);
  } finally {
    Terrain.close(owner);
    restore();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  }
});

test("Selected-map warming retains the latest image, retries failures and ignores invalid manifests", () => {
  ControlledImage.requests = [];
  vi.stubGlobal("Image", ControlledImage);
  try {
    const game = state();
    const image = sprite(game);
    const manifest = { [key(game)]: image };
    Terrain.preload_manifest(manifest, "mirror");
    Terrain.preload_manifest(manifest, "mirror");
    assert.equal(ControlledImage.requests.length, 1);
    required(ControlledImage.requests[0]).onerror?.();
    Terrain.preload_manifest(manifest, "mirror");
    assert.equal(ControlledImage.requests.length, 2);
    Terrain.preload_manifest({}, "mirror");
    assert.equal(required(ControlledImage.requests[1]).onerror, null);
  } finally {
    vi.unstubAllGlobals();
  }
});

test("An abandoned render leaves the committed bake running; activation cancels it even for an exact PNG", async () => {
  ControlledImage.requests = [];
  vi.stubGlobal("Image", ControlledImage);
  const restore = mockSpriteCanvas();
  const game = state();
  const nextGame = { ...game, mapId: "river" };
  const owner = Terrain.create({ [key(nextGame)]: sprite(nextGame) });
  const art = Art.current();
  try {
    const current = Terrain.prepare(owner, game, art);
    Terrain.activate(owner, current);
    const committed = Terrain.bake(owner, current);
    const image = required(ControlledImage.requests[0]);
    const abandoned = Terrain.prepare(owner, nextGame, art);
    assert(image.onload, "Speculative preparation cannot detach active image handlers");
    assert.equal(Terrain.bake(owner, current), committed);
    assert.equal(
      await Terrain.bake(owner, abandoned),
      undefined,
      "An uncommitted scene cannot replace active work",
    );
    assert.equal(
      Terrain.prepare(owner, game, art),
      current,
      "Returning to the committed view preserves scene identity",
    );
    const next = Terrain.prepare(owner, nextGame, art);
    assert(Terrain.preview(owner, next));
    Terrain.activate(owner, next);
    assert.equal(image.onload, null);
    assert.equal(await committed, undefined);
    assert.equal(
      await Terrain.bake(owner, current),
      undefined,
      "A stale effect cannot reactivate the previous scene",
    );
  } finally {
    Terrain.close(owner);
    restore();
    vi.unstubAllGlobals();
  }
});

test("Used artwork replacements bypass exact and overlay PNGs and paint the replacement, while canonical-only changes preserve the scene", async () => {
  ControlledImage.requests = [];
  vi.stubGlobal("Image", ControlledImage);
  const restore = mockSpriteCanvas();
  const game = state([{ ...tile(), terrain: "forest", floor: "snow-forest" }]);
  const owner = Terrain.create({ [key(game)]: sprite(game) });
  const base = Art.current();
  try {
    const original = Terrain.prepare(owner, game, base);
    assert(Terrain.preview(owner, original));
    const frames = { ...Art.frames(base) };
    const canonicalKey = Art.frame_key("terrain", Plan.canonical_name("forest"));
    const canonicalOnly = artwork({
      ...frames,
      [canonicalKey]: { url: "canonical-only.png", width: 64, height: 64 },
      player_unused: { url: "unused.png", width: 64, height: 64 },
    });
    assert.equal(Terrain.prepare(owner, game, canonicalOnly), original);
    assert(Terrain.preview(owner, original));
    const used = required(Terrain.get_tile(original, "0,0"));
    const usedKey = Art.frame_key("terrain", used.name, undefined, used.floor);
    const changedArt = artwork({
      ...frames,
      [usedKey]: { url: "replacement-terrain.png", width: 64, height: 64 },
    });
    const changed = Terrain.prepare(owner, game, changedArt);
    assert.equal(Terrain.preview(owner, changed), undefined);
    Terrain.activate(owner, changed);
    const painted = Terrain.bake(owner, changed);
    const request = required(ControlledImage.requests[0]);
    assert.equal(request.src, "replacement-terrain.png");
    request.onload?.();
    const bitmap = required(await painted);
    assert.equal(recording(bitmap).draws[0]?.source, "replacement-terrain.png");
    const edited = { ...game, tiles: [{ ...required(game.tiles[0]), floor: "autumn-forest" }] };
    const normalEdited = Terrain.prepare(owner, edited, base);
    assert(Terrain.preview(owner, normalEdited));
    const editedCell = required(Terrain.get_tile(normalEdited, "0,0"));
    const editedKey = Art.frame_key("terrain", editedCell.name, undefined, editedCell.floor);
    const overridden = Terrain.prepare(
      owner,
      edited,
      artwork({
        ...frames,
        [editedKey]: { url: "replacement-overlay.png", width: 64, height: 64 },
      }),
    );
    assert.equal(
      Terrain.preview(owner, overridden),
      undefined,
      "Changed-cell overlays cannot retain incompatible artwork pixels",
    );
  } finally {
    Terrain.close(owner);
    restore();
    vi.unstubAllGlobals();
  }
});

test("Successful map warming releases image handlers, reuses only its URL and ignores stale completion after a newer map", () => {
  ControlledImage.requests = [];
  vi.stubGlobal("Image", ControlledImage);
  const first = state();
  const second = { ...first, mapId: "river" };
  const owner = Terrain.create({
    [key(first)]: sprite(first),
    [key(second)]: { ...sprite(second), url: "river.png" },
  });
  try {
    Terrain.preload(owner, "mirror");
    const old = required(ControlledImage.requests[0]);
    const staleLoad = required(old.onload);
    const staleError = required(old.onerror);
    Terrain.preload(owner, "river");
    const current = required(ControlledImage.requests[1]);
    assert.equal(old.onload, null);
    assert.equal(old.onerror, null);
    staleLoad();
    staleError();
    Terrain.preload(owner, "river");
    assert.equal(ControlledImage.requests.length, 2, "Stale completion must retain the newer warm");
    const lateError = required(current.onerror);
    required(current.onload)();
    assert.equal(current.onload, null);
    assert.equal(current.onerror, null);
    lateError();
    Terrain.preload(owner, "river");
    assert.equal(
      ControlledImage.requests.length,
      2,
      "Success and a late error must not reload the same URL",
    );
    Terrain.close(owner);
    Terrain.preload(owner, "river");
    assert.equal(ControlledImage.requests.length, 3);
  } finally {
    Terrain.close(owner);
    vi.unstubAllGlobals();
  }
});
