import {
  inspect_storage as inspectStorageV1ToV2,
  repair_storage as repairStorageV1ToV2,
  history_key as HISTORY_KEY,
  history_payload_key as historyPayloadKey,
} from "@migrations/Migration_api.js";
import { repairImport } from "./persistence-adapter.mjs";
import { readSave, storeSave } from "./persistence-adapter.mjs";
import { historicalCharacters, historicalTiles } from "./legacy-save.mjs";
import { Game } from "./game-fixture.mjs";
import { campaigns as CAMPAIGNS } from "@migrations/Migration_api.js";
import { import_wave_marker as importWaveMarker } from "@migrations/Migration_api.js";
import assert from "node:assert/strict";
import { beforeEach, test, vi } from "vitest";
import { LiveGame, nativeValue } from "./persistence-adapter.mjs";
import { loadBattle } from "./persistence-adapter.mjs";
import { migrate_save as migrateSave } from "@migrations/Migration_api.js";
import historicalLeaderTypes from "../packages/save-migrations/src/legacy/data/type-leaders-v5.json";
import { migrate_v1_to_v2 as migrateV1ToV2 } from "@migrations/Migration_api.js";
import { save_version as SAVE_VERSION } from "@migrations/Migration_api.js";
import { saveBattle } from "./persistence-adapter.mjs";
import { createTestFactory } from "./persistence-adapter.mjs";
import { TestStorage } from "./helpers.mjs";
import { MAPS } from "./persistence-adapter.mjs";
beforeEach(() => vi.stubGlobal("localStorage", new TestStorage()));
for (const [cols, rows] of [
  [6, 20],
  [13, 23],
]) {
  test(`Developer ${cols}\xD7${rows} saves grow without resetting the battle`, async () => {
    const factory = createTestFactory();
    const game = LiveGame.create(factory, { kind: "scenario", map: "developer" });
    const checkpoint2 = structuredClone(nativeValue(game.checkpoint()));
    const state = checkpoint2.state;
    Object.assign(state, { mapRevision: 1, cols, rows, round: 5 });
    state.tiles = state.tiles.filter((tile) => tile.col < cols && tile.row < rows);
    state.gold.player = 23;
    state.units[0].hp -= 1;
    state.units[1].moved = true;
    Object.assign(state.tiles[0], {
      terrain: "road",
      connections: ["e", "w"],
      floor: "green-forest",
    });
    state.tiles.find((tile) => tile.terrain === "village").owner = "player";
    checkpoint2.seed = 42;
    const old = { version: 3, checkpoint: historicalTiles(checkpoint2), pendingEnemy: true };
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
        unit.role === void 0 && historicalLeaderTypes.includes(unit.type)
          ? { ...unit, role: "leader" }
          : unit,
      ),
    );
    assert.deepEqual(expanded.gold, state.gold);
    assert.equal(expanded.round, 5);
    assert.equal(migrated.save.checkpoint.seed, 42);
    assert.equal(migrated.save.pendingEnemy, true);
    assert.equal(migrateSave(migrated.save).migrated, false);
    await storeSave(JSON.stringify(old));
    const restored = await loadBattle(factory);
    assert.equal(restored.notice, void 0);
    assert.equal(restored.game.mapId, "developer");
    assert.deepEqual(nativeValue(restored.game.checkpoint()), migrated.save.checkpoint);
    assert.equal(JSON.parse(await readSave()).version, SAVE_VERSION);
    const incomplete = structuredClone(old);
    incomplete.checkpoint.state.tiles.pop();
    assert.throws(
      () => migrateSave(incomplete),
      (error) =>
        error.MEL_EXN_ID === "Invalid_argument" &&
        /Incomplete historical developer map/.test(error._1),
    );
    const duplicate = structuredClone(old);
    duplicate.checkpoint.state.tiles[1] = duplicate.checkpoint.state.tiles[0];
    assert.throws(
      () => migrateSave(duplicate),
      (error) =>
        error.MEL_EXN_ID === "Invalid_argument" &&
        /Invalid historical developer tile/.test(error._1),
    );
    restored.game.dispose();
    game.dispose();
  });
}
test("Version-three ordinary battles retain their native state through migration", async () => {
  const current = checkpoint({ kind: "scenario", map: "mirror" });
  const old = { version: 3, checkpoint: historicalTiles(current), pendingEnemy: false };
  const migrated = migrateSave(old).save;
  assert.deepEqual(migrated.checkpoint, current);
  assert.equal(migrated.version, SAVE_VERSION);
});
test("Live WASM catalog contains no historical save metadata", async () => {
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
function checkpoint(start = { kind: "campaign", id: "borrowed-tide" }) {
  const factory = createTestFactory();
  const game = LiveGame.create(factory, start);
  const saved = structuredClone(nativeValue(game.checkpoint()));
  game.dispose();
  return saved;
}
function oldSnapshot(input = checkpoint().state) {
  const { tiles, cols: _cols, rows: _rows, ...state } = historicalCharacters(input);
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
test("Version-one TS migration is pure, defaults experience and preserves progress", async () => {
  const old = oldSnapshot();
  old.state.round = 4;
  old.state.gold.player = 23;
  old.state.units[0].hp = 5;
  old.state.units[0].experience = 7;
  delete old.state.units[1].experience;
  const before = structuredClone(old);
  const migrated = migrateV1ToV2(old);
  assert.deepEqual(old, before);
  assert.equal(migrated.version, 2);
  assert.equal("version" in migrated.checkpoint, false);
  assert.equal(migrated.checkpoint.state.round, 4);
  assert.equal(migrated.checkpoint.state.gold.player, 23);
  assert.equal(migrated.checkpoint.state.units[0].experience, 7);
  assert.equal(migrated.checkpoint.state.units[1].experience, 0);
  assert.equal(migrated.checkpoint.state.campaign?.mission, old.state.campaign?.mission);
  const converted = migrateSave(migrated).save;
  const game = LiveGame.restore(createTestFactory(), converted.checkpoint, converted.setup);
  assert.equal(game.round, 4);
  assert.equal(game.unit(old.state.units[0].id)?.hp, 5);
  game.dispose();
});
test("Migration chain distinguishes the original native envelope and rewrites once", async () => {
  const saved = checkpoint({ kind: "scenario", map: "mirror" });
  saved.seed = 42;
  const old = {
    engine: "rust-wasm",
    pendingEnemy: false,
    saved: { version: 1, ...historicalTiles(saved) },
  };
  const raw = JSON.stringify(old);
  await storeSave(raw);
  const result = migrateSave(old);
  assert(result.migrated);
  assert.deepEqual(result.save.checkpoint, saved);
  assert.equal(await readSave(), raw, "Pure migration must not write storage");
  const restored = await loadBattle(createTestFactory());
  assert.equal(restored.notice, void 0);
  const latest = JSON.parse(await readSave());
  assert.equal(latest.version, SAVE_VERSION);
  assert.equal(latest.checkpoint.seed, 42);
  assert.equal(latest.checkpoint.version, void 0);
  assert.equal(migrateSave(latest).migrated, false);
  restored.game.dispose();
});
test("A native rejection retains source data for explicit repair", async () => {
  const old = oldSnapshot();
  const raw = JSON.stringify(old, null, 2);
  await storeSave(raw);
  const factory = createTestFactory();
  const restored = await loadBattle({
    ...factory,
    restore: () => ({ ok: false, error: { code: "invalidSave", message: "Rejected" } }),
  });
  assert(restored.problem);
  assert.equal(restored.game.mapId, "mirror");
  assert.equal(await readSave(), raw);
  await repairImport(restored.problem, { kind: "all" });
  await saveBattle(restored.game);
  assert.equal(JSON.parse(await readSave()).version, SAVE_VERSION);
  restored.game.dispose();
});
for (const raw of [
  "{broken",
  "null",
  JSON.stringify({ version: 999 }),
  JSON.stringify({ version: 0 }),
  JSON.stringify({ version: 1, engine: "unknown" }),
]) {
  test(`Rejected save ${raw} remains available for deletion while fresh play is available`, async () => {
    await storeSave(raw);
    const restored = await loadBattle(createTestFactory());
    assert(restored.problem);
    assert.equal(await readSave(), raw);
    assert.equal(restored.game.round, 1);
    restored.game.move("u3", "3,1");
    restored.game.dispose();
  });
}
test("Pending enemy replay preserves the player checkpoint through both versions", async () => {
  const old = oldSnapshot();
  old.pendingEnemy = true;
  await storeSave(JSON.stringify(old));
  const restored = await loadBattle(createTestFactory());
  assert.equal(restored.pendingEnemy, true);
  assert.equal(restored.game.turn, "player");
  assert.equal(JSON.parse(await readSave()).pendingEnemy, true);
  restored.game.dispose();
  old.state.turn = "enemy";
  assert.throws(() => migrateSave(old));
});
test("Version-four migration discards AI actions without executing the enemy turn", async () => {
  const factory = createTestFactory();
  const game = LiveGame.create(factory, { kind: "scenario", map: "mirror" });
  game.endTurn();
  game.enemyStep();
  const checkpoint2 = nativeValue(game.checkpoint());
  const old = {
    version: 4,
    pendingEnemy: false,
    checkpoint: {
      ...historicalTiles(checkpoint2),
      planner: {
        options: { recruitmentLimit: 2 },
        workspace: { recruitmentRemaining: 0, commands: [] },
        pending: { kind: "nonsense" },
      },
    },
  };
  const raw = JSON.stringify(old);
  await storeSave(raw);
  const migrated = migrateSave(old);
  assert.equal("planner" in migrated.save.checkpoint, false);
  assert.deepEqual(migrated.save.checkpoint.state, checkpoint2.state);
  assert.equal(await readSave(), raw);
  const loaded = await loadBattle(factory);
  assert.equal(loaded.notice, void 0);
  assert.deepEqual(nativeValue(loaded.game.checkpoint()), migrated.save.checkpoint);
  const converted = await readSave();
  assert.deepEqual(JSON.parse(converted), migrated.save);
  assert.equal(migrateSave(JSON.parse(converted)).migrated, false);
  for (let steps = 0; loaded.game.turn === "enemy" && !loaded.game.winner; steps++) {
    assert(steps < 100);
    loaded.game.enemyStep();
    await saveBattle(loaded.game);
    if (loaded.game.turn === "enemy" && !loaded.game.winner)
      assert.equal(await readSave(), converted);
  }
  const latest = JSON.parse(await readSave());
  assert.equal(latest.version, SAVE_VERSION);
  assert.equal("planner" in latest.checkpoint, false);
  assert.deepEqual(latest.checkpoint, nativeValue(loaded.game.checkpoint()));
  game.dispose();
  loaded.game.dispose();
});
test("Mid-turn writes retain the pre-turn checkpoint", async () => {
  const game = LiveGame.create(createTestFactory(), { kind: "scenario", map: "mirror" });
  await saveBattle(game, true);
  const before = await readSave();
  game.endTurn();
  game.enemyStep();
  await saveBattle(game);
  assert.equal(await readSave(), before);
  game.dispose();
});
test("Explicit source reset allows saving after rejection", async () => {
  await storeSave("{broken");
  const restored = await loadBattle(createTestFactory());
  assert(restored.problem);
  assert.equal(await readSave(), "{broken");
  restored.game.move("u3", "3,1");
  await repairImport(restored.problem, { kind: "all" });
  await saveBattle(restored.game);
  assert.equal(JSON.parse(await readSave()).version, SAVE_VERSION);
  restored.game.dispose();
});
for (const [id, chapter, type, tile, numericMarks] of [
  ["ashen-oath", 2, "boar", "9,4", ["wave:0", "wave:1"]],
]) {
  test(`Legacy ${id} delayed waves retain wounded troops and cannot deploy twice`, async () => {
    const game = Game.startCampaign(id, chapter);
    game.advanceDialogue(true);
    game.mapRevision = 1;
    game.round = 8;
    game.addUnit(type, "enemy", tile).hp = 1;
    game.campaign.marks.push(...numericMarks);
    const { campaignDeploymentFormat: _tag, ...old } = oldSnapshot(game.captureState());
    old.pendingEnemy = true;
    const mission = CAMPAIGNS[id].missions[chapter];
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
      assert(!converted.checkpoint.state.campaign.scripts.active.includes(mark.slice(5)));
    await storeSave(JSON.stringify(old));
    const loaded = await loadBattle(createTestFactory());
    assert.equal(loaded.notice, void 0);
    assert.equal(loaded.game.mapRevision, 2);
    assert.deepEqual(loaded.game.captureState().units, game.units);
    loaded.game.endTurn();
    assert.deepEqual(
      loaded.game.captureState().units,
      game.units,
      "Previously deployed waves cannot respawn",
    );
    while (loaded.game.turn === "enemy" && !loaded.game.winner) loaded.game.enemyStep();
    await saveBattle(loaded.game);
    const again = await loadBattle(createTestFactory());
    assert.deepEqual(again.game.captureState(), loaded.game.captureState());
    again.game.dispose();
    loaded.game.dispose();
  });
}
test("Tagged wave IDs survive import regardless of authored list ordering", async () => {
  const game = Game.startCampaign("ashen-oath", 2);
  game.campaign.marks.push("wave:north-binding-rise");
  const old = oldSnapshot(game.captureState());
  const converted = migrateSave(old).save;
  const waves = [...CAMPAIGNS["ashen-oath"].missions[2].waves].reverse();
  assert.equal(importWaveMarker("wave:0", waves), "wave:north-binding-rise");
  assert.equal(importWaveMarker("wave:2", waves), "wave:masters-boar");
  assert(!converted.checkpoint.state.campaign.scripts.active.includes("north-binding-rise"));
  const restored = LiveGame.restore(createTestFactory(), converted.checkpoint, converted.setup);
  assert(!restored.campaign.scripts.active.includes("north-binding-rise"));
  restored.dispose();
});
test("Legacy pre-rescue hold clocks reset, while earned victories stay completed", async () => {
  const game = Game.startCampaign("borrowed-tide", 1);
  const old = oldSnapshot(game.captureState());
  old.state.round = 4;
  old.state.campaign.holds = historicalTiles({
    "5,4": { heldTurns: 2, lastCountedRound: 4 },
  });
  const converted = migrateSave(old).save;
  assert.equal(
    converted.checkpoint.state.campaign.scripts.holds.find((h) => h.event === "goal-1")?.turns ?? 0,
    0,
  );
  assert(!converted.checkpoint.state.campaign.marks.includes("event:rescue"));
  const restored = LiveGame.restore(createTestFactory(), converted.checkpoint, converted.setup);
  assert.equal(restored.campaign.scripts.holds.find((h) => h.event === "goal-1")?.turns ?? 0, 0);
  restored.dispose();
  old.state.winner = "player";
  old.state.campaign.outcome = "Mission complete. The company can rest and regroup.";
  const finished = migrateSave(old).save;
  assert.equal(finished.checkpoint.state.winner, "player");
  assert(finished.checkpoint.state.campaign.flags.rescue?.at(-1)?.state);
  const earned = LiveGame.restore(createTestFactory(), finished.checkpoint, finished.setup);
  assert.equal(earned.winner, "player");
  earned.dispose();
});
for (const dead of [false, true]) {
  test(`Version-one truce ${dead ? "keeps Dain dead" : "restores living Dain"}`, async () => {
    const game = Game.startCampaign("green-accord", 1);
    const old = oldSnapshot(game.captureState());
    old.state.mapRevision = 1;
    old.state.units = old.state.units.filter((unit) => unit.name !== "Captain Dain");
    Object.assign(old.state.campaign, { choices: dead ? ["green-dain-dead"] : [] });
    const converted = migrateSave(old).save;
    assert.equal(converted.checkpoint.state.mapRevision, 2);
    assert.equal(
      converted.checkpoint.state.units.some((unit) => unit.character === "dain"),
      !dead,
    );
    assert(!converted.checkpoint.state.campaign.marks.includes("dain-fell"));
    const restored = LiveGame.restore(createTestFactory(), converted.checkpoint, converted.setup);
    assert.equal(
      restored.units.some((unit) => unit.character === "dain"),
      !dead,
    );
    restored.dispose();
  });
}
test("Repair distinguishes a missing history list from present empty or invalid text", () => {
  for (const text of [null, "", "{broken", JSON.stringify({ version: 4, entries: [] })]) {
    for (const hasSnapshot of [false, true]) {
      const source = { [HISTORY_KEY]: text };
      if (hasSnapshot) source[historyPayloadKey("unidentified")] = "{broken";
      const before = structuredClone(source);
      const records = inspectStorageV1ToV2(source);
      const indexes = records.filter((record) => record.key === HISTORY_KEY);
      assert.equal(indexes.length, text === null && !hasSnapshot ? 0 : 1);
      assert.equal(new Set(records.map((record) => record.key)).size, records.length);
      if (text === "") assert(indexes[0]?.error);
      assert.deepEqual(source, before);
    }
  }
});
test("Repair preserves disputed snapshot ownership regardless of index row order", () => {
  const entry = {
    id: "point",
    checkpointId: "point",
    branchId: "branch",
    runId: "campaign-a",
    ordinal: 1,
    savedAt: 1,
    run: "mirror",
    mission: "mirror",
    round: 1,
    side: "player",
    kind: "turn",
    missionStart: true,
    manual: false,
  };
  const other = { ...entry, runId: "campaign-b", run: "green-accord" };
  const key = historyPayloadKey(entry.id);
  for (const rows of [
    [entry, other],
    [other, entry],
  ]) {
    const source = {
      [HISTORY_KEY]: JSON.stringify({ version: 4, entries: rows }),
      [key]: "{unreadable snapshot",
    };
    const before = structuredClone(source);
    const record = inspectStorageV1ToV2(source).find((record2) => record2.key === key);
    assert.equal(record.ownership, "conflicting");
    assert.equal(record.runId, void 0);
    assert.equal(record.issue, "ownership_conflict");
    for (const runId of [entry.runId, other.runId]) {
      const plan = repairStorageV1ToV2(source, { kind: "campaign", runId });
      assert(!plan.deleted.includes(key));
      assert.equal(plan.writes[HISTORY_KEY], void 0);
    }
    assert.throws(() => repairStorageV1ToV2(source, { kind: "index" }));
    const deletion = repairStorageV1ToV2(source, { kind: "record", key });
    assert(deletion.deleted.includes(key));
    assert.equal(JSON.parse(deletion.writes[HISTORY_KEY]).entries.length, 0);
    assert.deepEqual(source, before);
  }
});

test("Repair cannot authorize campaign deletion from an index hint after payload damage", () => {
  const indexed = {
    id: "point",
    checkpointId: "point",
    branchId: "branch-a",
    runId: "campaign-a",
    ordinal: 1,
    savedAt: 1,
    run: "mirror",
    mission: "mirror",
    round: 1,
    side: "player",
    kind: "turn",
    missionStart: false,
    manual: false,
  };
  const key = historyPayloadKey(indexed.id);
  for (const version of [34, 36, 37, 38, SAVE_VERSION]) {
    for (const identifiable of [true, false]) {
      // The game body is deliberately unreadable; ownership inspection is independent.
      const payload = {
        version,
        checkpoint: { state: {} },
        ...(identifiable ? { progress: { entry: { ...indexed, runId: "campaign-b" } } } : {}),
      };
      const source = {
        [HISTORY_KEY]: JSON.stringify({ version: 4, entries: [indexed] }),
        [key]: JSON.stringify(payload),
      };
      const before = structuredClone(source);
      const record = inspectStorageV1ToV2(source).find((row) => row.key === key);
      assert.equal(record.ownership, identifiable ? "conflicting" : "unknown");
      assert.equal(record.save, undefined, "A rejected save is not reconstructed from metadata");
      for (const runId of ["campaign-a", "campaign-b"])
        assert(!repairStorageV1ToV2(source, { kind: "campaign", runId }).deleted.includes(key));
      assert.throws(() => repairStorageV1ToV2(source, { kind: "index" }));
      assert.deepEqual(source, before);
      assert(repairStorageV1ToV2(source, { kind: "record", key }).deleted.includes(key));
    }
  }
});

test("Version-36 flash saves retire parent links without changing the saved position", () => {
  const saved = checkpoint({ kind: "scenario", map: "mirror" });
  const old = {
    version: 36,
    checkpoint: historicalTiles(saved),
    pendingEnemy: false,
    undo: [],
    setup: { first: "player", enemy: { policy: "classic", diplomacy: "diplomatic" } },
    progress: {
      entry: {
        id: "branch",
        branchId: "branch",
        checkpointId: "point",
        runId: "run",
        ordinal: 1,
        savedAt: 1,
        run: "mirror",
        mission: "mirror",
        round: 1,
        side: "player",
        kind: "recovery",
        missionStart: false,
      },
    },
  };
  const expected = migrateSave(old).save;
  for (const parentId of [undefined, "historical-parent"]) {
    const input = structuredClone(old);
    input.progress.entry.parentId = parentId;
    const before = structuredClone(input);
    const converted = migrateSave(input).save;
    assert.deepEqual(converted, expected);
    assert(!Object.hasOwn(converted.progress.entry, "parentId"));
    assert.deepEqual(input, before);
    const restored = LiveGame.restore(createTestFactory(), converted.checkpoint, converted.setup);
    assert.deepEqual(nativeValue(restored.checkpoint()), saved);
    restored.dispose();
  }
});
