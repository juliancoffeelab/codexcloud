import assert from "node:assert/strict";
import { afterAll, afterEach, beforeAll, beforeEach, test, vi } from "vitest";
import * as Native from "@frontend/Native";
import * as Catalog from "@frontend/Catalog";
import * as Save from "@frontend/Save";
import * as Legacy from "@frontend/Save_legacy";
import type { SavedGame, State, Start, PublicUnit } from "../rust/pkg/hex_command";
import { NativeEngineFactory } from "../rust/pkg/hex_command";
import river from "../src/save/migration/legacy/data/river-v1.json";
import stages from "../src/save/migration/legacy/data/stages.json";
import { required, TestStorage } from "./helpers";

let factory: Awaited<ReturnType<typeof Native.create_factory>>;
const handles = new Set<ReturnType<typeof Native.create>>();
beforeAll(async () => {
  factory = await Native.create_factory();
});
beforeEach(() => vi.stubGlobal("localStorage", new TestStorage()));
afterEach(() => {
  for (const game of handles) Native.dispose(game);
  handles.clear();
});
afterAll(() => Native.dispose_factory(factory));
const catalog = () => Catalog.raw(Native.catalog(factory));
function create(start: Start = { kind: "scenario", map: "mirror" }) {
  const game = Native.create(factory, start, 17);
  handles.add(game);
  return game;
}
function restore(checkpoint: Native.Immutable<SavedGame>) {
  const game = Native.restore(factory, checkpoint);
  handles.add(game);
  return game;
}
function load(restoreGame = restore) {
  return Save.load(catalog(), restoreGame, () => create(), Native.dispose, Native.save);
}
function migrate(input: unknown) {
  const [save, migrated] = Save.migrate(catalog(), input);
  return { save, migrated };
}
function checkpoint() {
  return structuredClone(
    Native.save(create({ kind: "campaign", id: "borrowed-tide" })),
  ) as SavedGame;
}
function campaignState(id: string, chapter: number): State {
  const game = create({ kind: "campaign", id });
  for (let index = 0; index < chapter; index++) {
    const saved = structuredClone(Native.save(game)) as SavedGame;
    saved.state.gold.player = 0;
    saved.state.winner = "player";
    saved.state.campaign!.outcome = "Mission complete. The company can rest and regroup.";
    Native.replace(game, saved);
    Native.dispatch(game, { kind: "recallMission", units: [] });
  }
  return structuredClone(Native.state(Native.view(game))) as State;
}
function oldSnapshot(input = checkpoint().state) {
  const { tiles, cols: _cols, rows: _rows, ...state } = structuredClone(input);
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
function latest() {
  return JSON.parse(localStorage.getItem(Save.key)!);
}

test("Version-two river migration preserves owned tiles and state without changing input", () => {
  const game = create({ kind: "scenario", map: "river" });
  Native.dispatch(game, { kind: "move", unit: 3, tile: "5,3" });
  const checkpoint = structuredClone(Native.save(game)) as SavedGame;
  checkpoint.state.mapRevision = 1;
  checkpoint.state.tiles = structuredClone(river.tiles) as SavedGame["state"]["tiles"];
  const old = { version: 2, checkpoint, pendingEnemy: false };
  const before = structuredClone(old);
  const result = migrate(old);
  assert.equal(result.migrated, true);
  assert.deepEqual(old, before);
  assert.equal(result.save.checkpoint.state.mapRevision, 2);
  assert.deepEqual(result.save.checkpoint.state.tiles, checkpoint.state.tiles);
  assert.deepEqual(result.save.checkpoint.state.units, checkpoint.state.units);
  const restored = restore(result.save.checkpoint);
  assert.equal(Native.tile(Native.view(restored), "7,1")?.terrain, "bridge");
  assert.deepEqual(Native.save(restored), result.save.checkpoint);
});

test("Live native catalog does not contain historical save metadata", () => {
  const current = catalog();
  assert(
    !current.maps.some(
      (map: { id: string; revision: number }) =>
        ["river", "ashen-refuge"].includes(map.id) && map.revision === 1,
    ),
  );
  for (const unit of current.units) assert.equal("terrainModifiers" in unit, false);
  for (const campaign of current.campaigns)
    for (const mission of campaign.missions) {
      assert.equal("legacyDialogue" in mission, false);
      for (const wave of mission.waves) {
        assert.equal("legacyIndex" in wave, false);
        assert.equal("previouslyDeployedBeforeRevision" in wave, false);
      }
    }
});

test("Historical TS snapshots default experience and retain progress without mutation", () => {
  const old = oldSnapshot();
  old.state.round = 4;
  old.state.gold.player = 23;
  old.state.units[0]!.hp = 5;
  old.state.units[0]!.experience = 7;
  delete old.state.units[1]!.experience;
  const before = structuredClone(old);
  const { save } = migrate(old);
  assert.deepEqual(old, before);
  assert.equal("version" in save.checkpoint, false);
  assert.equal(save.checkpoint.state.round, 4);
  assert.equal(save.checkpoint.state.gold.player, 23);
  assert.equal(save.checkpoint.state.units[0]!.experience, 7);
  assert.equal(save.checkpoint.state.units[1]!.experience, 0);
  assert.deepEqual(save.checkpoint.state.campaign, old.state.campaign);
  assert.equal(Native.state(Native.view(restore(save.checkpoint))).round, 4);
});

test("Original native envelope migrates purely and writes only after restoration", () => {
  const saved = checkpoint();
  saved.seed = 42;
  const old = { engine: "rust-wasm", pendingEnemy: false, saved: { version: 1, ...saved } };
  const raw = JSON.stringify(old);
  localStorage.setItem(Save.key, raw);
  assert.deepEqual(migrate(old).save.checkpoint, saved);
  assert.equal(localStorage.getItem(Save.key), raw);
  assert.equal(load().notice, undefined);
  assert.equal(latest().version, Save.version);
  assert.equal(latest().checkpoint.seed, 42);
  assert.equal(latest().checkpoint.version, undefined);
  assert.equal(migrate(latest()).migrated, false);
  assert.equal(localStorage.getItem(Save.rejected_key), null);
});

test("Native rejection never commits migration and retains original bytes", () => {
  const raw = JSON.stringify(oldSnapshot(), null, 2);
  localStorage.setItem(Save.key, raw);
  const loaded = load(() => {
    throw new Error("Native rejected checkpoint");
  });
  assert(loaded.notice);
  assert.equal(Native.state(Native.view(loaded.game)).mapId, "mirror");
  assert.equal(localStorage.getItem(Save.key), raw);
  assert.equal(localStorage.getItem(Save.rejected_key), raw);
  Save.write(loaded.saves, Native.save(loaded.game), false);
  assert.equal(latest().version, Save.version);
  assert.equal(localStorage.getItem(Save.rejected_key), raw);
});

for (const raw of [
  "{broken",
  "null",
  JSON.stringify({ version: 999 }),
  JSON.stringify({ version: 0 }),
  JSON.stringify({ version: 1, engine: "unknown" }),
]) {
  test(`Rejected save ${raw} leaves playable recovery and exact backup`, () => {
    localStorage.setItem(Save.key, raw);
    const loaded = load();
    assert(loaded.notice);
    assert.equal(localStorage.getItem(Save.rejected_key), raw);
    assert.equal(localStorage.getItem(Save.key), raw);
    Native.dispatch(loaded.game, { kind: "move", unit: 3, tile: "3,1" });
  });
}

test("Pending enemy replay retains the player checkpoint and rejects inconsistent turn", () => {
  const old = oldSnapshot();
  old.pendingEnemy = true;
  localStorage.setItem(Save.key, JSON.stringify(old));
  const loaded = load();
  assert.equal(loaded.pending_enemy, true);
  assert.equal(Native.state(Native.view(loaded.game)).turn, "player");
  assert.equal(latest().pendingEnemy, true);
  old.state.turn = "enemy";
  assert.throws(() => migrate(old));
});

test("Current envelopes reject invalid replay, unknown fields and inherited metadata", () => {
  const saved = checkpoint();
  const valid = { version: Save.version, checkpoint: saved, pendingEnemy: false };
  for (const invalid of [
    { ...valid, pendingEnemy: "true" },
    { ...valid, unexpected: true },
    { ...valid, checkpoint: { ...saved, version: 1 } },
    { ...valid, checkpoint: { ...saved, state: null } },
    {
      ...valid,
      pendingEnemy: true,
      checkpoint: { ...saved, state: { ...saved.state, turn: "enemy" } },
    },
    {
      ...valid,
      pendingEnemy: true,
      checkpoint: { ...saved, state: { ...saved.state, winner: "player" } },
    },
    Object.assign(Object.create({ version: Save.version }), {
      checkpoint: saved,
      pendingEnemy: false,
    }),
  ])
    assert.throws(() => migrate(invalid));
  assert.equal(migrate(valid).migrated, false);
});

test("Native migration preserves selected AI command and exact continuation", () => {
  const rawFactory = new NativeEngineFactory();
  const value = <T>(result: { ok: true; value: T } | { ok: false; error: unknown }): T => {
    assert(result.ok);
    return result.value;
  };
  try {
    const game = value(rawFactory.create({ kind: "scenario", map: "mirror" }, 42));
    const enemy = value(rawFactory.dispatch(game.id, game.view.revision, { kind: "endTurn" }));
    const options = { policy: "classic", diplomacy: "restrained", recruitmentLimit: 2 } as const;
    const decision = value(rawFactory.chooseAiCommand(game.id, enemy.view.revision, options));
    const saved = value(rawFactory.save(game.id, enemy.view.revision));
    assert(saved.planner?.pending);
    localStorage.setItem(
      Save.key,
      JSON.stringify({ engine: "rust-wasm", pendingEnemy: false, saved: { version: 1, ...saved } }),
    );
    assert.equal(load().notice, undefined);
    assert.deepEqual(latest().checkpoint.planner, saved.planner);
    const repeated = value(
      rawFactory.restore(structuredClone(migrate(latest()).save.checkpoint) as SavedGame),
    );
    const repeatedDecision = value(
      rawFactory.chooseAiCommand(repeated.id, repeated.view.revision, options),
    );
    assert.deepEqual(repeatedDecision.command, decision.command);
    const expected = value(rawFactory.dispatch(game.id, decision.revision, decision.command));
    const actual = value(
      rawFactory.dispatch(repeated.id, repeatedDecision.revision, repeatedDecision.command),
    );
    assert.deepEqual(actual.view.state, expected.view.state);
  } finally {
    rawFactory.free();
  }
});

test("Failed backup protects original through play; explicit new battle resumes saving", () => {
  class QuotaStorage extends TestStorage {
    override setItem(key: string, value: string) {
      if (key === Save.rejected_key) throw new Error("quota exceeded");
      super.setItem(key, value);
    }
  }
  const storage = new QuotaStorage();
  const raw = "{original malformed save";
  storage.setItem(Save.key, raw);
  vi.stubGlobal("localStorage", storage);
  const loaded = load();
  assert.match(loaded.notice!, /original is retained/);
  Native.dispatch(loaded.game, { kind: "move", unit: 3, tile: "3,1" });
  Save.write(loaded.saves, Native.save(loaded.game), false);
  assert.equal(storage.getItem(Save.key), raw);
  assert.equal(storage.getItem(Save.rejected_key), null);
  const fresh = create({ kind: "scenario", map: "river" });
  Save.resume(loaded.saves);
  Save.write(loaded.saves, Native.save(fresh), false);
  assert.equal(latest().checkpoint.state.mapId, "river");
});

for (const [id, chapter, type, tile, numericMarks] of [
  ["lantern-road", 1, "bonearcher", "8,7", ["wave:0"]],
  ["ashen-oath", 2, "boar", "9,4", ["wave:0", "wave:1"]],
] as const) {
  test(`Legacy ${id} delayed waves preserve wounded troops and cannot deploy twice`, () => {
    const state = campaignState(id, chapter);
    state.mapRevision = 1;
    state.round = 8;
    state.units.push({
      id: `u${state.nextId++}`,
      type,
      side: "enemy",
      tile,
      hp: 1,
      experience: 0,
      moved: false,
      attacked: false,
    });
    state.campaign!.dialogueIndex = state.campaign!.dialogue.length;
    state.campaign!.marks.push(...numericMarks);
    const { campaignDeploymentFormat: _tag, ...old } = oldSnapshot(state);
    old.pendingEnemy = true;
    const mission = required(
      required(catalog().campaigns.find((c: { id: string }) => c.id === id)).missions[chapter],
    );
    const history = stages[mission.map.id as keyof typeof stages].waves;
    const expected = [
      ...numericMarks.map((mark) => Legacy.wave_marker(mark, [...history].reverse())),
      ...history
        .filter(
          (wave) =>
            "previouslyDeployedBeforeRevision" in wave &&
            wave.previouslyDeployedBeforeRevision === 2,
        )
        .map((wave) => `wave:${wave.id}`),
    ];
    const converted = migrate(old).save;
    assert.deepEqual(converted.checkpoint.state.units, state.units);
    assert.deepEqual(converted.checkpoint.state.gold, state.gold);
    for (const mark of expected) assert(converted.checkpoint.state.campaign!.marks.includes(mark));
    localStorage.setItem(Save.key, JSON.stringify(old));
    const loaded = load();
    assert.equal(loaded.notice, undefined);
    assert.equal(Native.state(Native.view(loaded.game)).mapRevision, 2);
    Native.dispatch(loaded.game, { kind: "endTurn" });
    assert.deepEqual(Native.state(Native.view(loaded.game)).units, state.units);
    Save.write(loaded.saves, Native.save(loaded.game), false);
    assert.deepEqual(
      Native.state(Native.view(load().game)),
      Native.state(Native.view(loaded.game)),
    );
  });
}

test("Tagged wave IDs survive import independently of authored ordering", () => {
  const state = campaignState("ashen-oath", 2);
  state.campaign!.marks.push("wave:north-binding-rise");
  const saved = migrate(oldSnapshot(state)).save;
  const waves = [...stages["ashen-dawn"].waves].reverse();
  assert.equal(Legacy.wave_marker("wave:0", waves), "wave:north-binding-rise");
  assert.equal(Legacy.wave_marker("wave:2", waves), "wave:masters-boar");
  assert.deepEqual(saved.checkpoint.state.campaign!.marks, state.campaign!.marks);
  assert.deepEqual(
    Native.state(Native.view(restore(saved.checkpoint))).campaign!.marks,
    state.campaign!.marks,
  );
});

test("Pre-rescue hold clocks reset while earned victories stay completed", () => {
  const old = oldSnapshot(campaignState("borrowed-tide", 1));
  old.state.round = 4;
  old.state.campaign!.holds = { "5,4": { heldTurns: 2, lastCountedRound: 4 } };
  const ongoing = migrate(old).save;
  assert.equal(ongoing.checkpoint.state.campaign!.holds!["5,4"]!.heldTurns, 0);
  assert(!ongoing.checkpoint.state.campaign!.marks.includes("event:rescue"));
  restore(ongoing.checkpoint);
  old.state.winner = "player";
  old.state.campaign!.outcome = "Mission complete. The company can rest and regroup.";
  const won = migrate(old).save;
  assert(won.checkpoint.state.campaign!.marks.includes("event:rescue"));
  assert.equal(Native.state(Native.view(restore(won.checkpoint))).winner, "player");
});

for (const dead of [false, true])
  test(`Historical truce ${dead ? "keeps Dain dead" : "restores living Dain"}`, () => {
    const old = oldSnapshot(campaignState("green-accord", 1));
    old.state.mapRevision = 1;
    old.state.units = old.state.units.filter((unit) => unit.name !== "Captain Dain");
    old.state.campaign!.choices = dead ? ["green-dain-dead"] : [];
    const saved = migrate(old).save;
    assert.equal(saved.checkpoint.state.mapRevision, 2);
    assert.equal(
      saved.checkpoint.state.units.some((unit) => unit.name === "Captain Dain"),
      !dead,
    );
    assert(!saved.checkpoint.state.campaign!.marks.includes("dain-fell"));
    assert.equal(
      Native.state(Native.view(restore(saved.checkpoint))).units.some(
        (unit: PublicUnit) => unit.name === "Captain Dain",
      ),
      !dead,
    );
  });
