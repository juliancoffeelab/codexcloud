import { historicalCharacters, historicalTiles } from "./legacy-save";
import { Game } from "./game-fixture";
import { CAMPAIGNS } from "../src/save/migrations/legacy/catalog";
import { importWaveMarker } from "../src/save/migrations/legacy/snapshot";
import assert from "node:assert/strict";
import { beforeEach, test, vi } from "vitest";
import { LiveGame, nativeValue } from "../src/game/live-game";
import { loadBattle } from "../src/save/load";
import { migrateSave } from "../src/save/migrations";
import historicalLeaderTypes from "../src/save/migrations/legacy/data/type-leaders-v5.json";
import { migrateV1ToV2 } from "../src/save/migrations/v1-to-v2";
import { SAVE_VERSION } from "../src/save/schema";
import { SAVE_KEY } from "../src/save/storage";
import { saveBattle } from "../src/save/write";
import { createTestFactory } from "./native-fixture";
import { TestStorage } from "./helpers";
import { MAPS } from "../src/game/maps";

beforeEach(() => vi.stubGlobal("localStorage", new TestStorage()));

for (const [cols, rows] of [
  [6, 20],
  [13, 23],
] as const) {
  test(`Developer ${cols}×${rows} saves grow without resetting the battle`, () => {
    const factory = createTestFactory();
    const game = LiveGame.create(factory, { kind: "scenario", map: "developer" });
    const checkpoint = structuredClone(
      nativeValue(game.checkpoint()),
    ) as import("../rust/pkg/hex_command").SavedGame;
    const state = checkpoint.state;
    Object.assign(state, { mapRevision: 1, cols, rows, round: 5 });
    state.tiles = state.tiles.filter((tile) => tile.col < cols && tile.row < rows);
    state.gold.player = 23;
    state.units[0]!.hp -= 1;
    state.units[1]!.moved = true;
    Object.assign(state.tiles[0]!, {
      terrain: "road",
      connections: ["e", "w"],
      floor: "green-forest",
    });
    state.tiles.find((tile) => tile.terrain === "village")!.owner = "player";
    checkpoint.seed = 42;
    const old = { version: 3, checkpoint: historicalTiles(checkpoint), pendingEnemy: true };
    const before = structuredClone(old);
    const migrated = migrateSave(old);
    assert(migrated.migrated);
    assert.deepEqual(old, before, "Migration stays pure");
    const expanded = migrated.save.checkpoint.state;
    assert.equal(expanded.cols, MAPS.developer.cols);
    assert.equal(expanded.rows, MAPS.developer.rows);
    assert.equal(expanded.mapRevision, MAPS.developer.revision);
    assert.equal(expanded.tiles.length, expanded.cols * expanded.rows);
    for (const tile of state.tiles)
      assert.deepEqual(
        expanded.tiles.find((candidate) => candidate.id === tile.id),
        tile,
      );
    assert.deepEqual(
      expanded.units,
      state.units.map((unit) =>
        unit.role === undefined && historicalLeaderTypes.includes(unit.type)
          ? { ...unit, role: "leader" }
          : unit,
      ),
    );
    assert.deepEqual(expanded.gold, state.gold);
    assert.equal(expanded.round, 5);
    assert.equal(migrated.save.checkpoint.seed, 42);
    assert.equal(migrated.save.pendingEnemy, true);
    assert.equal(migrateSave(migrated.save).migrated, false);
    localStorage.setItem(SAVE_KEY, JSON.stringify(old));
    const restored = loadBattle(factory);
    assert.equal(restored.notice, undefined);
    assert.equal(restored.game.mapId, "developer");
    assert.deepEqual(nativeValue(restored.game.checkpoint()), migrated.save.checkpoint);
    assert.equal(JSON.parse(localStorage.getItem(SAVE_KEY)!).version, SAVE_VERSION);
    assert.equal(localStorage.getItem(`${SAVE_KEY}-unrestored`), null);
    const incomplete = structuredClone(old);
    incomplete.checkpoint.state.tiles.pop();
    assert.throws(() => migrateSave(incomplete), /Incomplete historical developer map/);
    const duplicate = structuredClone(old);
    duplicate.checkpoint.state.tiles[1] = duplicate.checkpoint.state.tiles[0]!;
    assert.throws(() => migrateSave(duplicate), /Invalid historical developer tile/);
    restored.game.dispose();
    game.dispose();
  });
}

test("Version-three ordinary battles receive only an envelope update", () => {
  const current = checkpoint();
  const old = { version: 3, checkpoint: historicalTiles(current), pendingEnemy: false };
  assert.deepEqual(migrateSave(old).save, { ...old, checkpoint: current, version: SAVE_VERSION });
});

test("Live WASM catalog contains no historical save metadata", () => {
  const result = createTestFactory().catalog();
  assert(result.ok);
  const catalog = result.value;
  assert(!catalog.maps.some((map) => map.id === "ashen-refuge" && map.revision === 1));
  for (const unit of catalog.units) assert.equal("terrainModifiers" in unit, false);
  for (const campaign of catalog.campaigns)
    for (const mission of campaign.missions) {
      assert.equal("legacyDialogue" in mission, false);
      for (const wave of mission.waves) {
        assert.equal("legacyIndex" in wave, false);
        assert.equal("previouslyDeployedBeforeRevision" in wave, false);
      }
    }
});

function checkpoint() {
  const factory = createTestFactory();
  const game = LiveGame.create(factory, { kind: "campaign", id: "borrowed-tide" });
  const saved = structuredClone(
    nativeValue(game.checkpoint()),
  ) as import("../rust/pkg/hex_command").SavedGame;
  game.dispose();
  return saved;
}
function oldSnapshot(
  input:
    import("../rust/pkg/hex_command").State | import("../src/game/types").GameState = checkpoint()
    .state,
) {
  const {
    tiles,
    cols: _cols,
    rows: _rows,
    ...state
  } = historicalCharacters(input) as import("../rust/pkg/hex_command").State;
  return {
    version: 1,
    campaignDeploymentFormat: "named-waves-v1",
    pendingEnemy: false,
    state: {
      ...state,
      villages: tiles
        .filter((tile) => tile.terrain === "village")
        .map(({ id, owner }) => ({ id, owner })),
      sites: tiles
        .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
        .map(({ id, owner }) => ({ id, owner })),
    },
  };
}

test("Version-one TS migration is pure, defaults experience and preserves progress", () => {
  const old = oldSnapshot();
  old.state.round = 4;
  old.state.gold.player = 23;
  old.state.units[0]!.hp = 5;
  old.state.units[0]!.experience = 7;
  delete old.state.units[1]!.experience;
  const before = structuredClone(old);
  const migrated = migrateV1ToV2(old);
  assert.deepEqual(old, before);
  assert.equal(migrated.version, 2);
  assert.equal("version" in migrated.checkpoint, false);
  assert.equal(migrated.checkpoint.state.round, 4);
  assert.equal(migrated.checkpoint.state.gold.player, 23);
  assert.equal(migrated.checkpoint.state.units[0]!.experience, 7);
  assert.equal(migrated.checkpoint.state.units[1]!.experience, 0);
  assert.deepEqual(migrated.checkpoint.state.campaign, old.state.campaign);
  const game = LiveGame.restore(createTestFactory(), migrateSave(migrated).save.checkpoint);
  assert.equal(game.round, 4);
  assert.equal(game.unit(old.state.units[0]!.id)?.hp, 5);
  game.dispose();
});

test("Migration chain distinguishes the original native envelope and rewrites once", () => {
  const saved = checkpoint();
  saved.seed = 42;
  const old = {
    engine: "rust-wasm",
    pendingEnemy: false,
    saved: { version: 1, ...historicalTiles(saved) },
  };
  const raw = JSON.stringify(old);
  localStorage.setItem(SAVE_KEY, raw);
  const result = migrateSave(old);
  assert(result.migrated);
  assert.deepEqual(result.save.checkpoint, saved);
  assert.equal(localStorage.getItem(SAVE_KEY), raw, "Pure migration must not write storage");
  const restored = loadBattle(createTestFactory());
  assert.equal(restored.notice, undefined);
  const latest = JSON.parse(localStorage.getItem(SAVE_KEY)!);
  assert.equal(latest.version, SAVE_VERSION);
  assert.equal(latest.checkpoint.seed, 42);
  assert.equal(latest.checkpoint.version, undefined);
  assert.equal(migrateSave(latest).migrated, false);
  assert.equal(localStorage.getItem(`${SAVE_KEY}-unrestored`), null);
  restored.game.dispose();
});

test("A native rejection wipes the save without committing migration", () => {
  const old = oldSnapshot();
  const raw = JSON.stringify(old, null, 2);
  localStorage.setItem(SAVE_KEY, raw);
  const factory = createTestFactory();
  const restored = loadBattle({
    ...factory,
    restore: () => ({ ok: false, error: { code: "invalidSave", message: "Rejected" } }),
  });
  assert(restored.notice);
  assert.equal(restored.game.mapId, "mirror");
  assert.equal(localStorage.getItem(SAVE_KEY), null);
  assert.equal(localStorage.getItem(`${SAVE_KEY}-unrestored`), null);
  saveBattle(restored.game);
  assert.equal(JSON.parse(localStorage.getItem(SAVE_KEY)!).version, SAVE_VERSION);
  assert.equal(localStorage.getItem(`${SAVE_KEY}-unrestored`), null);
  restored.game.dispose();
});

for (const raw of [
  "{broken",
  "null",
  JSON.stringify({ version: 999 }),
  JSON.stringify({ version: 0 }),
  JSON.stringify({ version: 1, engine: "unknown" }),
]) {
  test(`Rejected save ${raw} is wiped and leaves a fresh playable battle`, () => {
    localStorage.setItem(SAVE_KEY, raw);
    const restored = loadBattle(createTestFactory());
    assert(restored.notice);
    assert.equal(localStorage.getItem(`${SAVE_KEY}-unrestored`), null);
    assert.equal(localStorage.getItem(SAVE_KEY), null);
    assert.equal(restored.game.round, 1);
    restored.game.move("u3", "hex-0738-f0cce97107");
    restored.game.dispose();
  });
}

test("Pending enemy replay preserves the player checkpoint through both versions", () => {
  const old = oldSnapshot();
  old.pendingEnemy = true;
  localStorage.setItem(SAVE_KEY, JSON.stringify(old));
  const restored = loadBattle(createTestFactory());
  assert.equal(restored.pendingEnemy, true);
  assert.equal(restored.game.turn, "player");
  assert.equal(JSON.parse(localStorage.getItem(SAVE_KEY)!).pendingEnemy, true);
  restored.game.dispose();
  old.state.turn = "enemy";
  assert.throws(() => migrateSave(old));
});

test("Version-four migration discards AI actions without executing the enemy turn", () => {
  const factory = createTestFactory();
  const game = LiveGame.create(factory, { kind: "scenario", map: "mirror" });
  game.endTurn();
  game.enemyStep();
  const checkpoint = nativeValue(game.checkpoint());
  const old = {
    version: 4,
    pendingEnemy: false,
    checkpoint: {
      ...historicalTiles(checkpoint),
      planner: {
        options: { recruitmentLimit: 2 },
        workspace: { recruitmentRemaining: 0, commands: [] },
        pending: { kind: "nonsense" },
      },
    },
  };
  const raw = JSON.stringify(old);
  localStorage.setItem(SAVE_KEY, raw);
  const migrated = migrateSave(old);
  assert.equal("planner" in migrated.save.checkpoint, false);
  assert.deepEqual(migrated.save.checkpoint.state, checkpoint.state);
  assert.equal(localStorage.getItem(SAVE_KEY), raw);
  const loaded = loadBattle(factory);
  assert.equal(loaded.notice, undefined);
  assert.deepEqual(nativeValue(loaded.game.checkpoint()), migrated.save.checkpoint);
  const converted = localStorage.getItem(SAVE_KEY);
  assert.deepEqual(JSON.parse(converted!), migrated.save);
  assert.equal(migrateSave(JSON.parse(converted!)).migrated, false);
  for (let steps = 0; loaded.game.turn === "enemy" && !loaded.game.winner; steps++) {
    assert(steps < 100);
    loaded.game.enemyStep();
    saveBattle(loaded.game);
    if (loaded.game.turn === "enemy" && !loaded.game.winner)
      assert.equal(localStorage.getItem(SAVE_KEY), converted);
  }
  const latest = JSON.parse(localStorage.getItem(SAVE_KEY)!);
  assert.equal(latest.version, SAVE_VERSION);
  assert.equal("planner" in latest.checkpoint, false);
  assert.deepEqual(latest.checkpoint, nativeValue(loaded.game.checkpoint()));
  game.dispose();
  loaded.game.dispose();
});

test("Mid-turn writes retain the pre-turn checkpoint", () => {
  const game = LiveGame.create(createTestFactory(), { kind: "scenario", map: "mirror" });
  saveBattle(game, true);
  const before = localStorage.getItem(SAVE_KEY);
  game.endTurn();
  game.enemyStep();
  saveBattle(game);
  assert.equal(localStorage.getItem(SAVE_KEY), before);
  game.dispose();
});

test("Rejected saves clear obsolete backups and leave saving enabled", () => {
  localStorage.setItem(SAVE_KEY, "{broken");
  localStorage.setItem(`${SAVE_KEY}-unrestored`, "old backup");
  const restored = loadBattle(createTestFactory());
  assert(restored.notice);
  assert.equal(localStorage.getItem(SAVE_KEY), null);
  assert.equal(localStorage.getItem(`${SAVE_KEY}-unrestored`), null);
  restored.game.move("u3", "hex-0738-f0cce97107");
  saveBattle(restored.game);
  assert.equal(JSON.parse(localStorage.getItem(SAVE_KEY)!).version, SAVE_VERSION);
  restored.game.dispose();
});

for (const [id, chapter, type, tile, numericMarks] of [
  ["ashen-oath", 2, "boar", "hex-1019-90a76d4969", ["wave:0", "wave:1"]],
] as const) {
  test(`Legacy ${id} delayed waves retain wounded troops and cannot deploy twice`, () => {
    const game = Game.startCampaign(id, chapter);
    game.advanceDialogue(true);
    game.mapRevision = 1;
    game.round = 8;
    game.addUnit(type, "enemy", tile).hp = 1;
    game.campaign!.marks.push(...numericMarks);
    const { campaignDeploymentFormat: _tag, ...old } = oldSnapshot(game.captureState());
    old.pendingEnemy = true;
    const mission = CAMPAIGNS[id].missions[chapter]!;
    const converted = migrateSave(old).save;
    const expectedMarks = [
      ...numericMarks.map((mark) => importWaveMarker(mark, [...mission.waves].reverse())),
      ...mission.waves
        .filter((wave) => wave.previouslyDeployedBeforeRevision === 2)
        .map((wave) => `wave:${wave.id}`),
    ];
    assert.deepEqual(converted.checkpoint.state.units, game.units);
    assert.deepEqual(converted.checkpoint.state.gold, game.gold);
    assert.equal(converted.pendingEnemy, true);
    for (const mark of expectedMarks)
      assert(converted.checkpoint.state.campaign!.marks.includes(mark));
    localStorage.setItem(SAVE_KEY, JSON.stringify(old));
    const loaded = loadBattle(createTestFactory());
    assert.equal(loaded.notice, undefined);
    assert.equal(loaded.game.mapRevision, 2);
    assert.deepEqual(loaded.game.captureState().units, game.units);
    loaded.game.endTurn();
    assert.deepEqual(
      loaded.game.captureState().units,
      game.units,
      "Previously deployed waves cannot respawn",
    );
    while (loaded.game.turn === "enemy" && !loaded.game.winner) loaded.game.enemyStep();
    saveBattle(loaded.game);
    const again = loadBattle(createTestFactory());
    assert.deepEqual(again.game.captureState(), loaded.game.captureState());
    again.game.dispose();
    loaded.game.dispose();
  });
}

test("Tagged wave IDs survive import regardless of authored list ordering", () => {
  const game = Game.startCampaign("ashen-oath", 2);
  game.campaign!.marks.push("wave:north-binding-rise");
  const old = oldSnapshot(game.captureState());
  const converted = migrateSave(old).save;
  const waves = [...CAMPAIGNS["ashen-oath"].missions[2]!.waves].reverse();
  assert.equal(importWaveMarker("wave:0", waves), "wave:north-binding-rise");
  assert.equal(importWaveMarker("wave:2", waves), "wave:masters-boar");
  assert.deepEqual(converted.checkpoint.state.campaign!.marks, game.campaign!.marks);
  const restored = LiveGame.restore(createTestFactory(), converted.checkpoint);
  assert.deepEqual(restored.campaign!.marks, game.campaign!.marks);
  restored.dispose();
});

test("Legacy pre-rescue hold clocks reset, while earned victories stay completed", () => {
  const game = Game.startCampaign("borrowed-tide", 1);
  const old = oldSnapshot(game.captureState());
  old.state.round = 4;
  old.state.campaign!.holds = historicalTiles({
    "hex-0891-b20a50ef8c": { heldTurns: 2, lastCountedRound: 4 },
  });
  const converted = migrateSave(old).save;
  assert.equal(converted.checkpoint.state.campaign!.holds!["hex-0891-b20a50ef8c"]!.heldTurns, 0);
  assert(!converted.checkpoint.state.campaign!.marks.includes("event:rescue"));
  const restored = LiveGame.restore(createTestFactory(), converted.checkpoint);
  assert.equal(restored.campaign!.holds!["hex-0891-b20a50ef8c"]!.heldTurns, 0);
  restored.dispose();
  old.state.winner = "player";
  old.state.campaign!.outcome = "Mission complete. The company can rest and regroup.";
  const finished = migrateSave(old).save;
  assert.equal(finished.checkpoint.state.winner, "player");
  assert(finished.checkpoint.state.campaign!.marks.includes("event:rescue"));
  const earned = LiveGame.restore(createTestFactory(), finished.checkpoint);
  assert.equal(earned.winner, "player");
  earned.dispose();
});

for (const dead of [false, true]) {
  test(`Version-one truce ${dead ? "keeps Dain dead" : "restores living Dain"}`, () => {
    const game = Game.startCampaign("green-accord", 1);
    const old = oldSnapshot(game.captureState());
    old.state.mapRevision = 1;
    old.state.units = old.state.units.filter(
      (unit) => (unit as unknown as { name?: string }).name !== "Captain Dain",
    );
    old.state.campaign!.choices = dead ? ["green-dain-dead"] : [];
    const converted = migrateSave(old).save;
    assert.equal(converted.checkpoint.state.mapRevision, 2);
    assert.equal(
      converted.checkpoint.state.units.some((unit) => unit.character === "dain"),
      !dead,
    );
    assert(!converted.checkpoint.state.campaign!.marks.includes("dain-fell"));
    const restored = LiveGame.restore(createTestFactory(), converted.checkpoint);
    assert.equal(
      restored.units.some((unit) => unit.character === "dain"),
      !dead,
    );
    restored.dispose();
  });
}
