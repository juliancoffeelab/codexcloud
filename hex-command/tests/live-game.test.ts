import { IDBFactory } from "fake-indexeddb";
import { TurnCheckpoint, serializeBattle } from "../src/save/write";
import { describeFlashSave, newRun, progressId } from "../src/save/metadata";
import { readSave, storeSave } from "../src/save/storage";
import { loadBattle as loadLiveBattle } from "../src/save/load";
import { saveBattle as writeBattle } from "../src/save/write";
import assert from "node:assert/strict";
import { test, beforeEach, afterEach, vi } from "vitest";
import { NativeEngineFactory } from "../rust/pkg/hex_command";
import { createNativeEngineFactory } from "../src/game/native-engine";
import { LiveGame, nativeValue } from "../src/game/live-game";
import { BattleController } from "../src/controller/battle";
import { Game, developerFixture } from "./game-fixture";
import { saveBattle, SAVE_KEY } from "./legacy-save";
import { SAVE_VERSION } from "../src/save/schema";
import { TestStorage, required } from "./helpers";
import type { BattleView } from "../src/controller/view";

vi.mock("../src/presentation/sound", () => ({
  soundEnabled: () => false,
  cancelSound: () => {},
  playSound: () => {},
  playInteraction: () => {},
  setSoundEnabled: () => {},
  unlockSound: async () => {},
}));

let factory: ReturnType<typeof createNativeEngineFactory>;
beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
  factory = createNativeEngineFactory(new NativeEngineFactory());
});
afterEach(() => {
  factory.dispose();
});

test("Live controller uses native movement/undo and keeps previous UI queries immutable", async () => {
  await saveBattle(developerFixture());
  const controller = await BattleController.create(async () => {}, factory);
  let published: BattleView | undefined;
  const unsubscribe = controller.view.subscribe((view) => {
    published = view;
  });
  const opening = controller.snapshot();
  const previous = required(published).game;
  const actor = required(opening.units.find((unit) => unit.side === "player"));
  const moves = previous.reachable(actor.id);
  const tile = required([...moves.keys()][0]);
  controller.move(actor.id, tile);
  assert(required(published).canUndo);
  assert.equal(previous.unit(actor.id)?.tile, actor.tile);
  assert.deepEqual(previous.reachable(actor.id), moves);
  assert(Object.isFrozen(previous.neighbors(actor.tile)));
  assert.throws(() => Object.assign(previous.gold, { player: 99 }), TypeError);
  controller.undoMove();
  assert.deepEqual(controller.snapshot(), opening);
  await controller.saved();
  const restored = await loadLiveBattle(factory);
  assert.deepEqual(restored.game.snapshot(), opening);
  restored.game.dispose();
  controller.dispose();
  unsubscribe();
});

test("A native enemy turn cancelled by map selection never acts on the new battle", async () => {
  let resume!: () => void;
  const pause = new Promise<void>((resolve) => {
    resume = resolve;
  });
  const controller = await BattleController.create(() => pause, factory);
  const opening = controller.snapshot();
  const pending = controller.endTurn();
  await controller.saved();
  const checkpoint = await loadLiveBattle(factory);
  assert(checkpoint.pendingEnemy);
  assert.deepEqual(checkpoint.game.snapshot(), opening);
  checkpoint.game.dispose();
  await controller.reset("river-valley");
  const replacement = controller.snapshot();
  resume();
  await pending;
  assert.deepEqual(controller.snapshot(), replacement);
  controller.dispose();
});

test("Native combat inspection feeds UI damage breakdowns and old saves migrate once", async () => {
  const old = developerFixture();
  await saveBattle(old);
  const game = (await loadLiveBattle(factory)).game;
  assert.deepEqual(game.snapshot(), old.snapshot());
  const actor = required(game.units.find((unit) => unit.side === "player"));
  const target = required(game.targets(actor.id)[0]);
  const forecast = game.previewDetails(actor, target);
  const previous = game.publishedView();
  game.attack(actor.id, target.id, forecast.attack.id);
  assert.equal(previous.unit(target.id)?.hp, target.hp);
  const after = game.unit(target.id);
  const remaining = after?.hp ?? 0;
  assert(remaining >= Math.max(0, target.hp - required(forecast.damageDetail.maximum)));
  assert(remaining <= Math.max(0, target.hp - required(forecast.damageDetail.minimum)));
  await writeBattle(game);
  assert.equal(JSON.parse((await readSave())!).version, SAVE_VERSION);
  const restored = (await loadLiveBattle(factory)).game;
  assert.deepEqual(restored.snapshot(), game.snapshot());
  restored.dispose();
  game.dispose();
});

test("Campaign dialogue and retry controls dispatch through native hooks", async () => {
  const game = LiveGame.create(factory, { kind: "campaign", id: "lantern-road" });
  const opening = game.captureState();
  game.advanceDialogue();
  game.previousDialogue();
  assert.equal(game.campaign?.dialogueIndex, 0);
  game.advanceDialogue(true);
  assert.equal(game.campaign?.dialogueIndex, game.campaign?.dialogue.length);
  game.retryCampaignMission();
  assert.deepEqual(game.captureState(), opening);
  game.dispose();
});

test("Supported legacy woodland revisions migrate without losing campaign progress", async () => {
  const old = Game.startCampaign("green-accord");
  old.mapRevision = 1;
  old.round = 3;
  old.gold.player = 19;
  await saveBattle(old);
  const restored = await loadLiveBattle(factory);
  assert.equal(restored.notice, undefined);
  assert.equal(restored.game.mapRevision, 2);
  assert.equal(restored.game.round, 3);
  assert.equal(restored.game.gold.player, 19);
  assert.deepEqual(restored.game.captureState().units, old.units);
  await writeBattle(restored.game);
  const reloaded = await loadLiveBattle(factory);
  assert.deepEqual(reloaded.game.snapshot(), restored.game.snapshot());
  reloaded.game.dispose();
  restored.game.dispose();
});

test("A rejected historical save is retained without preventing fresh play", async () => {
  await saveBattle(new Game("mirror"));
  const original = await readSave();
  const restored = await loadLiveBattle({
    ...factory,
    restore: () => ({ ok: false, error: { code: "invalidRequest", message: "Rejected" } }),
  });
  assert(restored.problem);
  assert.equal(restored.game.turn, "player");
  assert.equal(await readSave(), original);
  restored.game.dispose();
});

test("Rejected native campaign data leaves the WASM factory usable", async () => {
  const game = LiveGame.create(factory, { kind: "campaign", id: "lantern-road" });
  await writeBattle(game);
  game.dispose();
  const checkpoint = JSON.parse((await readSave())!);
  const corruptions = [
    () => {
      checkpoint.checkpoint.state.campaign.fallenCompanions = ["Captain Nera"];
    },
    () => {
      checkpoint.checkpoint.state.campaign.fallenCompanions = [];
      checkpoint.checkpoint.state.units[0].experience = 0xffffffff;
    },
  ];
  for (const corrupt of corruptions) {
    corrupt();
    await storeSave(JSON.stringify(checkpoint));
    const restored = await loadLiveBattle(factory);
    assert(restored.problem);
    restored.game.dispose();
  }
  const next = LiveGame.create(factory, { kind: "campaign", id: "lantern-road" });
  next.retryCampaignMission();
  assert.equal(next.units.find((unit) => unit.role === "leader")?.name, "Captain Nera");
  next.dispose();
});

test("Live enemy planning finishes a phase and saves the resulting player turn", async () => {
  const controller = await BattleController.create(async () => {}, factory);
  await controller.endTurn();
  const after = controller.snapshot();
  assert.equal(after.turn, "player");
  assert.equal(after.round, 2);
  const restored = await loadLiveBattle(factory);
  assert.equal(restored.pendingEnemy, false);
  assert.deepEqual(restored.game.snapshot(), after);
  restored.game.dispose();
  controller.dispose();
});

test("An invalid native checkpoint is retained for repair alongside a fresh battle", async () => {
  const rejected = { version: 999, checkpoint: {}, pendingEnemy: false };
  await storeSave(JSON.stringify(rejected));
  const restored = await loadLiveBattle(factory);
  assert(restored.problem);
  assert.equal(restored.game.turn, "player");
  assert.equal(await readSave(), JSON.stringify(rejected));
  restored.game.dispose();
});

test("Reloading during an enemy turn replays the whole phase from its boundary", async () => {
  // Node has no image decoder; artwork failure must not affect turn persistence.
  vi.stubGlobal(
    "Image",
    class {
      onerror: (() => void) | null = null;
      set src(_source: string) {
        queueMicrotask(() => this.onerror?.());
      }
    },
  );
  const game = LiveGame.create(factory, { kind: "scenario", map: "mirror" });
  await writeBattle(game, true);
  game.endTurn();
  const phaseSnapshot = game.snapshot();
  game.enemyStep();
  await writeBattle(game);
  for (let steps = 0; game.turn === "enemy"; steps++) {
    assert(steps < 100, "planner must finish this small phase");
    game.enemyStep();
  }
  const expected = game.snapshot();
  game.dispose();
  let release!: () => void;
  const paused = new Promise<void>((resolve) => {
    release = resolve;
  });
  const controller = await BattleController.create(() => paused, factory);
  let completed!: () => void;
  const completion = new Promise<void>((resolve) => {
    completed = resolve;
  });
  const unsubscribe = controller.view.subscribe((view) => {
    if (view.game.turn === "player" && view.game.round === 2) completed();
  });
  await controller.initialize();
  await controller.initialize();
  assert.deepEqual(controller.snapshot(), phaseSnapshot);
  release();
  await completion;
  assert.equal(controller.snapshot().round, 2);
  assert.deepEqual(
    controller.snapshot(),
    expected,
    "replay must reproduce the complete enemy phase",
  );
  await Promise.resolve();
  const restored = await loadLiveBattle(factory);
  assert.equal(restored.pendingEnemy, false);
  assert.deepEqual(restored.game.snapshot(), controller.snapshot());
  restored.game.dispose();
  controller.dispose();
  unsubscribe();
});

test("Import repair removes a corrupt historical payload and preserves playable flash data", async () => {
  vi.stubGlobal("indexedDB", new IDBFactory());
  const game = LiveGame.create(factory, { kind: "scenario", map: "mirror" });
  const identity = newRun();
  const good = required(
    TurnCheckpoint.capture(
      game,
      { ...identity, checkpointId: progressId(), ordinal: 1 },
      true,
      identity.checkpointId,
    ),
  );
  const bad = required(
    TurnCheckpoint.capture(
      game,
      { ...identity, checkpointId: progressId(), ordinal: 2 },
      false,
      good.entry.id,
    ),
  );
  const flash = JSON.parse(
    required(
      serializeBattle(game, false, {
        undo: [],
        progress: { entry: describeFlashSave(game, identity) },
      }),
    ),
  );
  const expected = nativeValue(game.checkpoint());
  flash.progress.entry.kind = "recovery";
  const raw = JSON.stringify(flash);
  localStorage.setItem("hex-command-battle", raw);
  localStorage.setItem(
    "hex-command-progress-index",
    JSON.stringify({ version: 4, entries: [good.entry, bad.entry] }),
  );
  localStorage.setItem(`hex-command-progress:${good.entry.id}`, TurnCheckpoint.text(good));
  const key = `hex-command-progress:${bad.entry.id}`;
  localStorage.setItem(key, "{broken payload");
  const controller = await BattleController.create(async () => {}, factory);
  try {
    await assert.rejects(controller.savedProgress(), /Repair saved data/);
    assert(
      (await controller.repairInventory()).records.find((record) => record.key === key)?.error,
    );
    await controller.repairSavedData({ kind: "record", key });
    assert.equal(localStorage.getItem("hex-command-battle"), raw);
    assert.equal(await controller.retrySavedProgress(), true);
    assert.deepEqual(controller.snapshot(), game.snapshot());
    assert.deepEqual(JSON.parse(required(await readSave())).checkpoint, expected);
    assert.deepEqual(
      (await controller.savedProgress()).map((entry) => entry.id),
      [good.entry.id],
    );
  } finally {
    controller.dispose();
    game.dispose();
  }
});

test("Unavailable storage with no source saves allows a new campaign and player actions", async () => {
  vi.stubGlobal("indexedDB", {
    open: () => {
      throw new DOMException("Storage disabled", "SecurityError");
    },
  });
  const controller = await BattleController.create(async () => {}, factory);
  try {
    await controller.initialize();
    await controller.startCampaign("lantern-road");
    controller.advanceDialogue(true);
    const actor = required(
      controller
        .snapshot()
        .units.find((unit) => unit.side === "player" && controller.actions(unit.id).moves.length),
    );
    const destination = required(controller.actions(actor.id).moves[0]).tile;
    controller.move(actor.id, destination);
    await controller.saved();
    assert.equal(controller.snapshot().campaign?.id, "lantern-road");
    assert.equal(
      required(controller.snapshot().units.find((unit) => unit.id === actor.id)).tile,
      destination,
    );
    assert.equal(localStorage.length, 0);
    let failure: BattleView["loadProblem"];
    const unsubscribe = controller.view.subscribe((view) => (failure = view.loadProblem));
    assert.equal(failure?.kind, "blocked");
    unsubscribe();
  } finally {
    controller.dispose();
  }
});
