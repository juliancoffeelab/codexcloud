import { test, vi } from "vitest";
import assert from "node:assert/strict";
import * as Art from "@frontend/Art";
import * as Geometry from "@frontend/Geometry";
import * as Plan from "@frontend/Terrain_plan";
import {
  pixelSprites,
  pixelUnitModels,
  pixelFloorColors,
  pixelTerrainLayouts,
} from "../src/presentation/pixel-art";
import { TYPES, TERRAIN } from "./game-fixture";
import { required } from "./helpers";

test("Adjacent hexes share both edge vertices without gaps on either row parity", () => {
  const vertices = (col: number, row: number) => {
    const center = Geometry.center(col, row);
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
      const neighbor = vertices(col!, nextRow!);
      assert.equal(
        origin.filter(([x, y]) =>
          neighbor.some(([nx, ny]) => Math.hypot(x! - nx!, y! - ny!) < 1e-8),
        ).length,
        2,
      );
    }
  }
});

test("Published artwork is immutable native URL metadata without fetching or extraction", async () => {
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
    assert(Object.isFrozen(artwork));
    assert(Object.isFrozen(art.frames(artwork)));
    assert.equal(Object.keys(art.frames(artwork)).length, Object.keys(pixelSprites).length);
    for (const [key, path] of Object.entries(pixelSprites)) {
      const frame = art.find(artwork, key)!;
      assert(Object.isFrozen(frame));
      assert.equal(frame.width, 64);
      assert.equal(frame.height, 64);
      assert(frame.url.endsWith(path));
    }
    const frame = art.unit_frame(artwork, "player", "swordsman")!;
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
      const frame = Art.unit_frame(Art.current(), side, type)!;
      assert.equal(frame, Art.find(Art.current(), `${side}_${model}`));
      assert(frame.url.endsWith(pixelSprites[`${side}_${model}`]!));
      assert.equal(frame.width, 64);
      assert.equal(frame.height, 64);
    }
  }
  assert.equal(
    Art.portrait_frame(Art.current(), "worker"),
    Art.find(Art.current(), "portrait_worker"),
  );
  assert.equal(Art.unit_frame(Art.current(), "player", "missing"), undefined);
});

test("Every supported floor selects complete isolated terrain images", () => {
  for (const terrain of ["road", "forest", "ford", "village", "grove"]) {
    for (const floor of Object.keys(pixelFloorColors)) {
      const key = Art.frame_key("terrain", terrain, "player", floor);
      const frame = Art.frame(Art.current(), "terrain", terrain, "player", floor)!;
      assert.equal(frame, Art.find(Art.current(), key));
      assert(frame.url.endsWith(pixelSprites[key]!));
      assert.equal(frame.width, 64);
      assert.equal(frame.height, 64);
    }
  }
  assert.equal(Art.frame_key("terrain", "grove", "player", "unknown"), "terrain_sacred_grove");
});

test("Map, preview and native bake dimensions share exact hex spacing", () => {
  assert.equal(Geometry.sprite_size, 70);
  assert.equal(Geometry.sprite_size * Geometry.native_pixel_scale, 64);
  assert.equal(Geometry.column_spacing, (70 * Math.sqrt(3)) / 2);
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

test("Every supported terrain has a complete native canonical inspection frame", () => {
  for (const terrain of Object.keys(TERRAIN)) {
    const name = Plan.canonical_name(terrain);
    const frame = required(Art.terrain_frame(Art.current(), undefined, name));
    assert.equal(frame.width, 64);
    assert.equal(frame.height, 64);
    const key = Art.frame_key("terrain", name);
    assert(frame.url.endsWith(required(pixelSprites[key])));
    const base = name.replace(/_layout_\d+$/, "");
    if (pixelTerrainLayouts[base]?.length) assert.equal(name, `${base}_layout_0`);
    else assert.equal(name, base);
  }
  assert.equal(Plan.canonical_name("village").replace(/_layout_\d+$/, ""), "village_0");
  for (const terrain of ["road", "bridge", "ford"])
    assert.equal(Plan.canonical_name(terrain).replace(/_layout_\d+$/, ""), terrain);
});
