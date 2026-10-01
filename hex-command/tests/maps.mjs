import assert from "node:assert/strict";
import { MAPS } from "../dist/maps.mjs";

const key = ({ col, row }) => `${col},${row}`;
const axial = ({ col, row }) => [col - Math.floor(row / 2), row];
const adjacent = (a, b) => {
  const [aq, ar] = axial(a), [bq, br] = axial(b);
  return (Math.abs(aq - bq) + Math.abs(ar - br) + Math.abs(aq + ar - bq - br)) / 2 === 1;
};
const board = (map) => {
  const overrides = new Map(map.terrain.map((t) => [key(t), t]));
  return Array.from({ length: map.cols * map.rows }, (_, i) => {
    const col = i % map.cols, row = Math.floor(i / map.cols);
    return { col, row, terrain: "plains", ...overrides.get(`${col},${row}`) };
  });
};
const connected = (tiles, start) => {
  const seen = new Set([key(start)]), queue = [start];
  while (queue.length) {
    const tile = queue.shift();
    for (const next of tiles) {
      if (!seen.has(key(next)) && adjacent(tile, next)) {
        seen.add(key(next));
        queue.push(next);
      }
    }
  }
  return seen;
};

for (const [id, map] of Object.entries(MAPS)) {
  assert.equal(map.id, id);
  assert.equal(new Set(map.terrain.map(key)).size, map.terrain.length, "No competing terrain overrides");
  for (const t of map.terrain) {
    assert(t.col >= 0 && t.col < map.cols && t.row >= 0 && t.row < map.rows);
    assert(["forest", "hill", "water", "bridge", "village", "keep"].includes(t.terrain));
  }
  const tiles = board(map), passable = tiles.filter((t) => t.terrain !== "water");
  const keeps = passable.filter((t) => t.terrain === "keep");
  assert.equal(keeps.length, 2);
  assert.equal(connected(passable, keeps[0]).size, passable.length, "Every land tile is accessible");
  assert.equal(map.units.length, 6);
  assert.equal(new Set(map.units.map(key)).size, 6);
  for (const side of ["player", "enemy"]) {
    assert.deepEqual(map.units.filter((u) => u.side === side).map((u) => u.type), ["commander", "soldier", "scout"]);
    const commander = map.units.find((u) => u.side === side && u.type === "commander");
    assert(keeps.some((t) => t.owner === side && key(t) === key(commander)));
    for (const unit of map.units.filter((u) => u.side === side)) {
      assert(passable.some((t) => key(t) === key(unit)));
    }
  }
}

const mirror = MAPS.mirror;
assert.equal(mirror.cols * mirror.rows, 77);
assert.deepEqual(Object.fromEntries(["forest", "hill", "water", "village", "keep"].map((kind) => [kind, mirror.terrain.filter((t) => t.terrain === kind).length])), { forest: 7, hill: 8, water: 8, village: 4, keep: 2 });

const river = MAPS.river, tiles = board(river);
assert.equal(tiles.length, 135);
assert.equal(river.terrain.filter((t) => t.terrain === "village").length, 6);
assert(tiles.filter((t) => t.col === 7).every((t) => ["water", "bridge"].includes(t.terrain)), "River reaches both map edges");
const bridges = tiles.filter((t) => t.terrain === "bridge");
assert.deepEqual(bridges.map((t) => t.row), [1, 4, 7]);
const land = tiles.filter((t) => !["water", "bridge"].includes(t.terrain));
const left = land.find((t) => t.col === 1 && t.row === 4);
const leftBank = connected(land, left);
assert.equal(leftBank.size, 7 * 9, "Removing bridges separates banks");
assert(land.filter((t) => t.col > 7).every((t) => !leftBank.has(key(t))), "No route around the river");
for (const bridge of bridges) {
  assert.equal(connected([...land, bridge], left).size, land.length + 1, "Each bridge independently connects both banks");
  assert(tiles.some((t) => t.col === 6 && t.terrain !== "water" && adjacent(t, bridge)));
  assert(tiles.some((t) => t.col === 8 && t.terrain !== "water" && adjacent(t, bridge)));
}
console.log("Map layouts and river crossing connectivity passed.");
