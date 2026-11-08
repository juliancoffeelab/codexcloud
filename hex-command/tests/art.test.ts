import { test, vi } from "vitest";
import assert from "node:assert/strict";
import { currentArtwork, artFrame, artSprite, dialoguePortrait } from "../src/presentation/art";
import {
  pixelSprites,
  pixelUnitModels,
  pixelFloorColors,
  pixelTerrainLayouts,
} from "../src/presentation/pixel-art";
import { terrainTileView } from "../src/presentation/terrain-map";
import { terrainArtNames, terrainProps } from "../src/ui/battlefield-view";
import { terrainLayout } from "../src/presentation/terrain-layout";
import { TYPES } from "./game-fixture";
import { hexPoints, tileCenter, battlefieldSpriteSize } from "../src/ui/geometry";

test("Adjacent hexes share both edge vertices without gaps on either row parity", () => {
  const vertices = (col: number, row: number) => {
    const center = tileCenter({ col, row });
    // Both the lattice spacing and its origin must survive native bitmap baking.
    for (const value of [center.x, center.y]) {
      const native = (value * 64) / battlefieldSpriteSize;
      assert.ok(Math.abs(native - Math.round(native)) < 1e-9);
    }
    return hexPoints(center.x, center.y)
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
test("Published artwork is immutable URL metadata and does not fetch or extract images", () => {
  vi.stubGlobal(
    "Image",
    class {
      constructor() {
        throw new Error("Unexpected eager fetch");
      }
    },
  );
  vi.stubGlobal("document", {
    createElement() {
      throw new Error("Unexpected canvas extraction");
    },
  });
  try {
    const artwork = currentArtwork();
    assert.equal(currentArtwork(), artwork);
    assert(Object.isFrozen(artwork.frames));
    const frame = artFrame(artwork, "units", "swordsman")!;
    assert(Object.isFrozen(frame));
    assert.equal(frame.width, 64);
    assert.equal(frame.height, 64);
    assert(frame.url.includes("assets/sprites/"));
    const svg = artSprite(artwork, "units", "swordsman", 0, 0, 70, 70);
    assert(svg.includes(`href="${frame.url}"`));
    assert(svg.includes('viewBox="0 0 64 64"'));
    assert(!svg.includes("data:image"));
    assert(!svg.includes("overflow"));
  } finally {
    vi.unstubAllGlobals();
  }
});

test("Playable units and artwork aliases resolve to independent native sprites", () => {
  for (const type of Object.keys(TYPES))
    for (const side of ["player", "enemy"] as const) {
      const model = pixelUnitModels[type] ?? (type === "bonearcher" ? "bone_archer" : type);
      const frame = artFrame(currentArtwork(), "units", type, side)!;
      assert.equal(frame, currentArtwork().frames[`${side}_${model}`]);
      assert(frame.url.endsWith(pixelSprites[`${side}_${model}`]!));
      assert.equal(frame.width, 64);
      assert.equal(frame.height, 64);
    }
  assert(
    dialoguePortrait(currentArtwork(), "worker", 0, 0, 90, 120).includes('viewBox="0 0 64 64"'),
  );
  assert.equal(battlefieldSpriteSize, 70);
});

test("Terrain floor, arrangement and orientation resolve directly to complete PNGs, including explicit defaults", () => {
  for (const terrain of ["road", "forest", "ford", "village"] as const) {
    for (const floor of Object.keys(pixelFloorColors)) {
      const tile = {
        id: "3,4",
        col: 3,
        row: 4,
        terrain,
        floor,
        owner: null,
        ...(terrain === "road" ? { connections: ["e", "w", "se"] as const } : {}),
        ...(terrain === "village" ? { connections: [] } : {}),
        ...(terrain === "ford" ? { connections: ["nw", "se"] as const } : {}),
      };
      const result = terrainTileView(tile, "sample", currentArtwork());
      const frame = artFrame(
        currentArtwork(),
        "terrain",
        terrainLayout(tile, "sample").name,
        "player",
        floor,
      )!;
      assert.equal(result.terrainFrame, frame);
      assert(frame.url.endsWith(pixelSprites[result.frameKey]!));
      assert.equal(frame.width, 64);
      assert.equal(frame.height, 64);
    }
  }
  assert.equal(artFrame(currentArtwork(), "units", "missing"), undefined);
});

test("Prop arrangements stay stable per map tile and vary between locations", () => {
  for (const terrain of ["forest", "hill", "plains", "ford"] as const) {
    const choices = new Set<string>();
    for (let col = 0; col < 16; col++) {
      const tile = {
        terrain,
        id: `${col},3`,
        ...(terrain === "ford" ? { connections: ["e", "w"] as const } : {}),
      };
      const first = terrainProps(tile, "sample");
      assert.equal(terrainProps(tile, "sample"), first);
      choices.add(JSON.stringify(first));
    }
    assert(choices.size > 1);
  }
  assert.deepEqual(
    terrainProps({ terrain: "road", id: "1,1", connections: ["e", "w"] }, "sample"),
    [],
  );
});

test("Directional props select arrangements for the same orientation as the base frame", () => {
  for (const connections of [
    ["e", "w"],
    ["nw", "se"],
    ["ne", "sw"],
  ] as const) {
    const tile = {
      terrain: "ford" as const,
      id: "4,2",
      connections: [...connections],
    };
    const frame = terrainArtNames(tile)[0]!;
    const props = terrainProps(tile, "sample");
    assert(pixelTerrainLayouts[frame]!.includes(props));
    assert.equal(terrainProps(tile, "sample"), props);
  }
});
