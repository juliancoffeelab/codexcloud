import { test, beforeEach, afterEach, vi } from "vitest";
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
  test(
    "Skirmish maps have connected land, legal armies and unique overrides — " +
      JSON.stringify([id, map]),
    async () => {
      assert.equal(map.id, id);
      assert.equal(
        new Set(map.terrain.map(key)).size,
        map.terrain.length,
        "No competing terrain overrides",
      );
      for (const t of map.terrain) {
        assert(t.col >= 0 && t.col < map.cols && t.row >= 0 && t.row < map.rows);
        assert(
          ["forest", "hill", "water", "bridge", "ford", "village", "keep"].includes(t.terrain),
        );
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
    },
  );
}

const mirror = MAPS.mirror;
test("Mirror map retains its published size and terrain density", async () => {
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
});

test("River forms two asymmetric banks linked by either crossing", async () => {
  const river = MAPS.river,
    tiles = board(river);
  assert.equal(tiles.length, 135);
  assert.equal(river.terrain.filter((t) => t.terrain === "village").length, 6);
  assert.equal(river.revision, 2);
  assert.equal(mirror.revision, 1);
  const riverTiles = tiles.filter((t) => ["water", "bridge", "ford"].includes(t.terrain));
  assert.equal(Math.min(...riverTiles.map((t) => t.row)), 0, "River reaches north edge");
  assert.equal(
    Math.max(...riverTiles.map((t) => t.row)),
    river.rows - 1,
    "River reaches south edge",
  );
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
    LEGACY_RIVER.terrain
      .filter((t) => t.terrain === "village")
      .map((t) => [t.col, t.row])
      .sort(([ac, ar], [bc, br]) => ac! - bc! || ar! - br!),
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
});

// Opening village routes no longer require pushing through forest.
const { Game, TERRAIN, TYPES } = await import("../src/game/engine.ts");
test("Opening river village routes remain clear", async () => {
  const opening = new Game("river");
  assert(opening.reachable("u2").has("4,1"), "starting soldier can reach the northern village");
  assert(opening.reachable("u3").has("4,1"), "starting scout can reach the northern village");
  assert(opening.reachable("u3").has("2,7"), "starting scout can reach the southern village");
  for (const id of ["2,1", "3,1", "3,2", "4,2", "1,6", "2,6"])
    assert.equal(required(opening.tile(id)).terrain, "plains");
});

// The developer map must cover every current definition, once per side.
test("Developer map covers every unit, side and terrain", async () => {
  const showcase = new Game("developer");
  assert.equal(showcase.cols, 6);
  assert.equal(showcase.rows, 3 + Math.ceil(Object.keys(TYPES).length / 2));
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
});

// These are ordinary units on an ordinary map: movement, combat and turns still work.
test("Developer units use ordinary movement, combat and turns", async () => {
  const developerBattle = new Game("developer");
  const soldier = required(
    developerBattle.units.find((unit) => unit.type === "soldier" && unit.side === "player"),
  );
  const enemySoldier = required(
    developerBattle.units.find((unit) => unit.type === "soldier" && unit.side === "enemy"),
  );
  developerBattle.move(soldier.id, "4,2");
  assert.equal(soldier.tile, "4,2");
  assert(developerBattle.targets(soldier.id).includes(enemySoldier));
  developerBattle.attack(soldier.id, enemySoldier.id, "sword");
  assert(enemySoldier.hp < TYPES.soldier.hp);
  assert(soldier.attacked);
  developerBattle.endTurn();
  assert.equal(developerBattle.turn, "enemy");
  assert.equal(developerBattle.winner, null);
});

// Developer snapshots remain noncampaign saves without relaxing ordinary validation.
const { loadBattle, saveBattle, SAVE_KEY, SAVE_VERSION } = await import("../src/game/save.ts");

test("Developer saves remain noncampaign and round-trip", async () => {
  const showcase = new Game("developer");
  saveBattle(showcase);
  const loaded = loadBattle();
  assert.deepEqual(loaded.game.snapshot(), showcase.snapshot());
  assert.equal(loaded.pendingEnemy, false);
  assert.equal(loaded.game.campaign, undefined);
});
for (const [type, destination] of [
  ["scout", "5,1"],
  ["soldier", "3,1"],
  ["soldier", "4,1"],
] as const) {
  test(
    "Moved developer sites persist ownership — " + JSON.stringify([type, destination]),
    async () => {
      const game = new Game("developer");
      const unit = required(
        game.units.find((unit) => unit.side === "player" && unit.type === type),
      );
      game.move(unit.id, destination);
      assert.equal(required(game.tile(destination)).owner, "player");
      saveBattle(game);
      assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
      assert.equal(required(loadBattle().game.tile(destination)).owner, "player");
    },
  );
}
test("Ordinary skirmishes do not publish site ownership overrides", async () => {
  assert.equal(new Game("mirror").snapshot().sites, undefined);
  assert.equal(new Game("river").snapshot().sites, undefined);
});
function validDeveloperSave() {
  const showcase = new Game("developer");
  const validSave = {
    version: SAVE_VERSION,
    pendingEnemy: false,
    state: { ...showcase.snapshot(), nextId: showcase.nextId, log: showcase.log },
  };
  return validSave;
}
test("Legacy developer saves gain neutral sites without resetting units", async () => {
  const showcase = new Game("developer");
  const validSave = validDeveloperSave();
  const legacySave = structuredClone(validSave);
  delete legacySave.state.sites;
  localStorage.setItem(SAVE_KEY, JSON.stringify(legacySave));
  const legacy = loadBattle().game;
  assert.equal(legacy.mapId, "developer");
  assert.deepEqual(legacy.units, showcase.units);
  assert(required(legacy.snapshot().sites).every((site) => site.owner === null));
});
test("Campaign saves still require site ownership", async () => {
  const campaign = Game.startCampaign("green-accord");
  const missingCampaignSites = {
    version: SAVE_VERSION,
    pendingEnemy: false,
    state: { ...campaign.snapshot(), nextId: campaign.nextId, log: campaign.log },
  };
  delete missingCampaignSites.state.sites;
  localStorage.setItem(SAVE_KEY, JSON.stringify(missingCampaignSites));
  assert.equal(loadBattle().game.mapId, "mirror", "Campaign sites remain mandatory");
});
const invalidCases: [string, (save: ReturnType<typeof validDeveloperSave>) => void][] = [
  [
    "duplicate unit position",
    (save) => {
      save.state.units[1]!.tile = save.state.units[0]!.tile;
    },
  ],
  [
    "unit on impassable water",
    (save) => {
      save.state.units[0]!.tile = "3,0";
    },
  ],
  [
    "missing enemy commander",
    (save) => {
      save.state.units = save.state.units.filter(
        (unit) => !(unit.side === "enemy" && TYPES[unit.type].family === "commander"),
      );
    },
  ],
  [
    "campaign attached to developer map",
    (save) => {
      save.state.campaign = Game.startCampaign("green-accord").campaign;
    },
  ],
  [
    "campaign map without campaign state",
    (save) => {
      save.state.mapId = "green-border";
    },
  ],
  [
    "missing site",
    (save) => {
      required(save.state.sites).pop();
    },
  ],
  [
    "duplicate site",
    (save) => {
      const sites = required(save.state.sites);
      sites[1]!.id = sites[0]!.id;
    },
  ],
  [
    "site on plains",
    (save) => {
      required(save.state.sites)[0]!.id = "2,2";
    },
  ],
  [
    "invalid site owner",
    (save) => {
      Object.assign(required(required(save.state.sites)[0]), { owner: "neutral" });
    },
  ],
];
test.each(invalidCases)("Invalid developer save is rejected: %s", (_label, invalidate) => {
  const invalid = validDeveloperSave();
  invalidate(invalid);
  localStorage.setItem(SAVE_KEY, JSON.stringify(invalid));
  assert.equal(loadBattle().game.mapId, "mirror");
});

beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
});

afterEach(() => {
  vi.unstubAllGlobals();
});
