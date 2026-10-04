import type { Coordinates, MapDefinition } from "../src/game/types.ts";
import type { Immutable } from "../src/game/immutable.ts";
import { required, TestStorage } from "./helpers.ts";
import assert from "node:assert/strict";
import { MAPS, LEGACY_RIVER } from "../src/game/maps.ts";
import { DEVELOPER_TERRAIN_SAMPLES, DEVELOPER_UNIT_SAMPLES } from "../src/game/developer-map.ts";

const key = ({ col, row }: Coordinates) => `${col},${row}`;
const axial = ({ col, row }: Coordinates): [number, number] => [col - Math.floor(row / 2), row];
const adjacent = (a: Coordinates, b: Coordinates) => {
  const [aq, ar] = axial(a),
    [bq, br] = axial(b);
  return (Math.abs(aq - bq) + Math.abs(ar - br) + Math.abs(aq + ar - bq - br)) / 2 === 1;
};
const board = (map: Immutable<MapDefinition>) => {
  const overrides = new Map(map.terrain.map((t) => [key(t), t]));
  return Array.from({ length: map.cols * map.rows }, (_, i) => {
    const col = i % map.cols,
      row = Math.floor(i / map.cols);
    return { col, row, terrain: "plains", ...overrides.get(`${col},${row}`) };
  });
};
const connected = (tiles: Coordinates[], start: Coordinates) => {
  const seen = new Set([key(start)]),
    queue = [start];
  while (queue.length) {
    const tile = required(queue.shift());
    for (const next of tiles) {
      if (!seen.has(key(next)) && adjacent(tile, next)) {
        seen.add(key(next));
        queue.push(next);
      }
    }
  }
  return seen;
};

for (const [id, map] of Object.entries(MAPS).filter(([id]) => id === "mirror" || id === "river")) {
  assert.equal(map.id, id);
  assert.equal(
    new Set(map.terrain.map(key)).size,
    map.terrain.length,
    "No competing terrain overrides",
  );
  for (const t of map.terrain) {
    assert(t.col >= 0 && t.col < map.cols && t.row >= 0 && t.row < map.rows);
    assert(["forest", "hill", "water", "bridge", "ford", "village", "keep"].includes(t.terrain));
  }
  const tiles = board(map),
    passable = tiles.filter((t) => t.terrain !== "water");
  const keeps = passable.filter((t) => t.terrain === "keep");
  assert.equal(keeps.length, 2);
  assert.equal(
    connected(passable, required(keeps[0])).size,
    passable.length,
    "Every land tile is accessible",
  );
  assert.equal(map.units.length, 6);
  assert.equal(new Set(map.units.map(key)).size, 6);
  for (const side of ["player", "enemy"]) {
    assert.deepEqual(
      map.units.filter((u) => u.side === side).map((u) => u.type),
      ["commander", "soldier", "scout"],
    );
    const commander = map.units.find((u) => u.side === side && u.type === "commander");
    assert(keeps.some((t) => t.owner === side && key(t) === key(required(commander))));
    for (const unit of map.units.filter((u) => u.side === side)) {
      assert(passable.some((t) => key(t) === key(unit)));
    }
  }
}

const mirror = MAPS.mirror;
assert.equal(mirror.cols * mirror.rows, 77);
assert.deepEqual(
  Object.fromEntries(
    ["forest", "hill", "water", "village", "keep"].map((kind) => [
      kind,
      mirror.terrain.filter((t) => t.terrain === kind).length,
    ]),
  ),
  { forest: 7, hill: 8, water: 8, village: 4, keep: 2 },
);

const river = MAPS.river,
  tiles = board(river);
assert.equal(tiles.length, 135);
assert.equal(river.terrain.filter((t) => t.terrain === "village").length, 6);
assert.equal(river.revision, 2);
assert.equal(mirror.revision, 1);
const riverTiles = tiles.filter((t) => ["water", "bridge", "ford"].includes(t.terrain));
assert.equal(Math.min(...riverTiles.map((t) => t.row)), 0, "River reaches north edge");
assert.equal(Math.max(...riverTiles.map((t) => t.row)), river.rows - 1, "River reaches south edge");
assert.equal(
  connected(riverTiles, required(riverTiles[0])).size,
  riverTiles.length,
  "River bends continuously",
);
assert(new Set(riverTiles.map((t) => t.col)).size >= 5, "River visibly bends across columns");
const crossings = tiles.filter((t) => ["bridge", "ford"].includes(t.terrain));
assert.deepEqual(
  crossings.map((t) => [t.terrain, t.col, t.row]),
  [
    ["bridge", 7, 3],
    ["ford", 5, 7],
  ],
);
const land = tiles.filter((t) => !["water", "bridge", "ford"].includes(t.terrain));
const playerKeep = required(land.find((t) => t.terrain === "keep" && t.owner === "player"));
const enemyKeep = required(land.find((t) => t.terrain === "keep" && t.owner === "enemy"));
const leftBank = connected(land, playerKeep),
  rightBank = connected(land, enemyKeep);
assert(!leftBank.has(key(enemyKeep)), "Removing both crossings separates keeps");
assert.equal(leftBank.size + rightBank.size, land.length, "Exactly two accessible banks remain");
assert.notEqual(leftBank.size, rightBank.size, "Banks have different shapes and areas");
for (const crossing of crossings) {
  assert.equal(
    connected([...land, crossing], playerKeep).size,
    land.length + 1,
    "Each crossing independently connects both banks",
  );
  assert(land.some((t) => leftBank.has(key(t)) && adjacent(t, crossing)));
  assert(land.some((t) => rightBank.has(key(t)) && adjacent(t, crossing)));
}
assert.notEqual(playerKeep.row, enemyKeep.row, "Keeps use different approaches");
assert.notDeepEqual(
  river.terrain
    .filter((t) => t.terrain === "village" && leftBank.has(key(t)))
    .map((t) => [14 - t.col, t.row])
    .sort(),
  river.terrain
    .filter((t) => t.terrain === "village" && rightBank.has(key(t)))
    .map((t) => [t.col, t.row])
    .sort(),
  "Village approaches are asymmetric",
);
assert.deepEqual(
  LEGACY_RIVER.terrain.filter((t) => t.terrain === "bridge").map((t) => [t.col, t.row]),
  [
    [7, 1],
    [7, 4],
    [7, 7],
  ],
);
assert.deepEqual(
  LEGACY_RIVER.terrain.filter((t) => t.terrain === "village").map((t) => [t.col, t.row]),
  [
    [4, 1],
    [4, 4],
    [4, 7],
    [10, 1],
    [10, 4],
    [10, 7],
  ],
);
assert(!Object.values(MAPS).includes(LEGACY_RIVER), "Legacy layout is not a new-game choice");
console.log("Map layouts and river crossing connectivity passed.");

// Opening village routes no longer require pushing through forest.
const { Game, TERRAIN, TYPES } = await import("../src/game/engine.ts");
const opening = new Game("river");
assert(opening.reachable("u2").has("4,1"), "starting soldier can reach the northern village");
assert(opening.reachable("u3").has("4,1"), "starting scout can reach the northern village");
assert(opening.reachable("u3").has("2,7"), "starting scout can reach the southern village");
for (const id of ["2,1", "3,1", "3,2", "4,2", "1,6", "2,6"])
  assert.equal(required(opening.tile(id)).terrain, "plains");
console.log("Opening river village routes are clear.");

// The inspection gallery must cover every current definition, once per side.
const showcase = new Game("developer");
assert.equal(showcase.cols, 6);
assert.equal(showcase.rows, 10);
assert.equal(showcase.campaign, undefined);
assert.deepEqual(
  DEVELOPER_TERRAIN_SAMPLES.map((sample) => sample.terrain).sort(),
  Object.keys(TERRAIN).sort(),
);
assert.equal(new Set(DEVELOPER_TERRAIN_SAMPLES.map(key)).size, 12);
for (const sample of DEVELOPER_TERRAIN_SAMPLES) {
  assert.equal(required(showcase.tile(key(sample))).terrain, sample.terrain);
  assert(sample.row < 2);
}
assert.equal(DEVELOPER_UNIT_SAMPLES.length, 2 * Object.keys(TYPES).length);
assert.equal(new Set(DEVELOPER_UNIT_SAMPLES.map(key)).size, showcase.units.length);
for (const side of ["player", "enemy"] as const) {
  assert.deepEqual(
    showcase.units
      .filter((unit) => unit.side === side)
      .map((unit) => unit.type)
      .sort(),
    Object.keys(TYPES).sort(),
  );
  assert.equal(
    showcase.units.filter((unit) => unit.side === side && unit.type === "commander").length,
    1,
  );
}
for (const sample of DEVELOPER_UNIT_SAMPLES) {
  const tile = required(showcase.tile(key(sample)));
  const unit = required(showcase.at(tile.id));
  assert.equal(unit.type, sample.type);
  assert.equal(unit.side, sample.side);
  assert.equal(unit.role, undefined);
  assert.equal(tile.terrain, "plains");
  assert(sample.row >= 3 && sample.col < showcase.cols && sample.row < showcase.rows);
}
assert(showcase.tiles.filter((tile) => tile.row === 2).every((tile) => !showcase.at(tile.id)));

// Developer snapshots remain noncampaign saves without relaxing ordinary validation.
const { loadBattle, saveBattle, SAVE_KEY, SAVE_VERSION } = await import("../src/game/save.ts");
globalThis.localStorage = new TestStorage();
saveBattle(showcase);
const loaded = loadBattle();
assert.deepEqual(loaded.game.snapshot(), showcase.snapshot());
assert.equal(loaded.pendingEnemy, false);
assert.equal(loaded.game.campaign, undefined);
const validSave = {
  version: SAVE_VERSION,
  pendingEnemy: false,
  state: { ...showcase.snapshot(), nextId: showcase.nextId, log: showcase.log },
};
for (const invalidate of [
  (save: typeof validSave) => {
    save.state.units[1]!.tile = save.state.units[0]!.tile;
  },
  (save: typeof validSave) => {
    save.state.units[0]!.tile = "3,0"; // The water sample remains impassable.
  },
  (save: typeof validSave) => {
    save.state.units = save.state.units.filter(
      (unit) => !(unit.side === "enemy" && unit.type === "commander"),
    );
  },
  (save: typeof validSave) => {
    save.state.campaign = Game.startCampaign("green-accord").campaign;
  },
  (save: typeof validSave) => {
    save.state.mapId = "green-border"; // A real campaign map still needs its campaign state.
  },
]) {
  const invalid = structuredClone(validSave);
  invalidate(invalid);
  localStorage.setItem(SAVE_KEY, JSON.stringify(invalid));
  assert.equal(loadBattle().game.mapId, "mirror");
}
console.log("Developer showcase coverage and save validation passed.");
