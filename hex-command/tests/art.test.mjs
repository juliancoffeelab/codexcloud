import { test, vi } from "vitest";
import assert from "node:assert/strict";
import * as Art from "@frontend/Art";
import * as Geometry from "@frontend/Geometry";
import * as Plan from "./plan-adapter.mjs";
import {
  pixelSprites,
  pixelUnitModels,
  pixelFloorColors,
  pixelTerrainLayouts,
  pixelTerrainGeometry,
} from "../src/presentation/pixel-art.mjs";
import { TYPES, TERRAIN } from "./game-fixture.mjs";
import { required } from "./helpers.mjs";
test("Artwork and terrain names never resolve inherited object properties", () => {
  for (const key of ["constructor", "__proto__", "toString", "hasOwnProperty"]) {
    assert.equal(Art.find({ frames: {} }, key), undefined);
    assert.equal(Art.frame_key("units", key, "player"), `player_${key}`);
    assert.equal(Art.frame_key("terrain", "road", "player", key), Art.frame_key("terrain", "road"));
    assert.throws(() =>
      Plan.prepare(
        "test",
        1,
        1,
        [{ id: "0,0", col: 0, row: 0, terrain: key, owner: null }],
        Art.current(),
      ),
    );
  }
});
test("Adjacent hexes share both edge vertices without gaps on either row parity", () => {
  const vertices = (col, row) => {
    const center = Geometry.center(col, row);
    // Both the lattice spacing and its origin must survive native bitmap baking.
    for (const value of [center.x, center.y]) {
      const native = (value * 64) / Geometry.sprite_size;
      assert.ok(Math.abs(native - Math.round(native)) < 1e-9);
    }
    return Geometry.hex_points(center.x, center.y)
      .split(" ")
      .map((point) => point.split(",").map(Number));
  };
  for (const row of [4, 5]) {
    const odd = row % 2;
    const origin = vertices(4, row);
    for (const [col, nextRow] of [
      [5, row],
      [4 + odd, row + 1],
      [3 + odd, row + 1],
      [3, row],
      [3 + odd, row - 1],
      [4 + odd, row - 1],
    ]) {
      const neighbor = vertices(col, nextRow);
      assert.equal(
        origin.filter(([x, y]) => neighbor.some(([nx, ny]) => Math.hypot(x - nx, y - ny) < 1e-8))
          .length,
        2,
      );
    }
  }
});
test("Published artwork is native URL metadata without fetching or extraction", async () => {
  vi.resetModules();
  vi.stubGlobal(
    "Image",
    class {
      constructor() {
        throw Error("Unexpected eager fetch");
      }
    },
  );
  vi.stubGlobal("document", {
    createElement() {
      throw Error("Unexpected canvas extraction");
    },
  });
  const fetch = vi.fn(() => {
    throw Error("Unexpected image fetch");
  });
  vi.stubGlobal("fetch", fetch);
  try {
    const art = await import("@frontend/Art");
    const artwork = art.current();
    assert.equal(art.current(), artwork);

    const frames = art.frames(artwork),
      second = art.frames(artwork);
    assert.notEqual(frames, second, "Each dictionary belongs to its caller");
    assert.equal(Object.getPrototypeOf(frames), null);

    assert.equal(Object.keys(frames).length, Object.keys(pixelSprites).length);
    const firstKey = required(Object.keys(frames)[0]),
      original = art.find(artwork, firstKey);
    delete frames[firstKey];
    assert.equal(art.find(artwork, firstKey), original);
    frames[firstKey] = { url: "replacement.png", width: 64, height: 64 };
    frames.constructor = frames[firstKey];
    assert.equal(art.find(artwork, firstKey), original);
    assert.equal(second[firstKey], original);
    assert.equal(art.find(artwork, "constructor"), undefined);
    for (const [key, path] of Object.entries(pixelSprites)) {
      const frame2 = art.find(artwork, key);

      const geometry = Object.entries(pixelTerrainGeometry).find(([kind]) =>
        key.startsWith(`terrain_${kind}_`),
      )?.[1];
      assert.equal(frame2.width, geometry?.width ?? 64);
      assert.equal(frame2.height, geometry?.height ?? 64);
      assert(frame2.url.endsWith(path));
    }
    const frame = art.unit_frame(artwork, "player", "swordsman");
    assert(frame.url.includes("assets/sprites/"));
    assert.equal(fetch.mock.calls.length, 0);
  } finally {
    vi.unstubAllGlobals();
  }
});
test("Playable units and aliases resolve to independent native sprites", () => {
  for (const type of Object.keys(TYPES)) {
    for (const side of ["player", "enemy"]) {
      const model = pixelUnitModels[type] ?? (type === "bonearcher" ? "bone_archer" : type);
      const frame = Art.unit_frame(Art.current(), side, type);
      assert.equal(frame, Art.find(Art.current(), `${side}_${model}`));
      assert(frame.url.endsWith(pixelSprites[`${side}_${model}`]));
      assert.equal(frame.width, 64);
      assert.equal(frame.height, 64);
    }
  }
  assert.equal(
    Art.portrait_frame(Art.current(), "worker"),
    Art.find(Art.current(), "portrait_worker"),
  );
  assert.equal(Art.unit_frame(Art.current(), "player", "missing"), void 0);
});
test("Every supported floor selects complete isolated terrain images", () => {
  for (const terrain of ["road_9", "forest_layout_0", "ford_9_layout_0", "village_0", "grove"]) {
    for (const floor of Object.keys(pixelFloorColors)) {
      const key = Art.frame_key("terrain", terrain, "player", floor);
      const frame = Art.frame(Art.current(), "terrain", terrain, "player", floor);
      assert.equal(frame, Art.find(Art.current(), key));
      assert(frame.url.endsWith(pixelSprites[key]));
      assert.equal(frame.width, 64);
      assert.equal(frame.height, 64);
    }
  }
  assert.equal(Art.frame_key("terrain", "grove", "player", "unknown"), "terrain_sacred_grove");
});
test("Map, preview and native bake dimensions share exact hex spacing", () => {
  assert.equal(Geometry.sprite_size, 70);
  assert.equal(Geometry.sprite_size * Geometry.native_pixel_scale, 64);
  assert.equal(Geometry.column_spacing, (70 * 56) / 64);
  assert.equal(Geometry.row_spacing, 52.5);
  assert.deepEqual(Geometry.dimensions(12, 8), {
    width: 758 + Geometry.column_spacing,
    height: 452 + Geometry.row_spacing,
  });
  assert.deepEqual(Geometry.preview_dimensions(12, 8), {
    width: 85 + Geometry.column_spacing * 11.5,
    height: 84 + Geometry.row_spacing * 7,
  });
});
test("Every supported terrain has a dedicated portrait with authored overflow metadata", () => {
  for (const terrain of Object.keys(TERRAIN)) {
    const frame = required(Art.terrain_portrait(Art.current(), terrain));
    const geometry = pixelTerrainGeometry[terrain];
    assert.equal(frame.width, geometry?.width ?? 64);
    assert.equal(frame.height, geometry?.height ?? 64);
  }
  const castle = required(Art.terrain_portrait(Art.current(), "castle-wall"));
  assert.equal(castle.anchor_x, 40);
  assert.equal(castle.anchor_y, 48);
});
