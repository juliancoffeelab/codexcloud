import { loadBattle as loadLiveBattle } from "../src/save/load";
import { saveBattle as writeBattle } from "../src/save/write";
import assert from "node:assert/strict";
import { test, beforeEach, afterEach, vi } from "vitest";
import { NativeEngineFactory } from "../rust/pkg/hex_command";
import { createNativeEngineFactory } from "../src/game/native-engine";
import { LiveGame } from "../src/game/live-game";
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

test("Live controller uses native movement/undo and keeps previous UI queries immutable", () => {
  saveBattle(developerFixture());
  const controller = new BattleController(async () => {}, factory);
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
  const restored = loadLiveBattle(factory);
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
  const controller = new BattleController(() => pause, factory);
  const opening = controller.snapshot();
  const pending = controller.endTurn();
  const checkpoint = loadLiveBattle(factory);
  assert(checkpoint.pendingEnemy);
  assert.deepEqual(checkpoint.game.snapshot(), opening);
  checkpoint.game.dispose();
  controller.reset("river");
  const replacement = controller.snapshot();
  resume();
  await pending;
  assert.deepEqual(controller.snapshot(), replacement);
  controller.dispose();
});

test("Native combat inspection feeds UI damage breakdowns and old saves migrate once", () => {
  const old = developerFixture();
  saveBattle(old);
  const game = loadLiveBattle(factory).game;
  assert.deepEqual(game.snapshot(), old.snapshot());
  const actor = required(game.units.find((unit) => unit.side === "player"));
  const target = required(game.targets(actor.id)[0]);
  const forecast = game.previewDetails(actor, target);
  const previous = game.publishedView();
  game.attack(actor.id, target.id, forecast.attack.id);
  assert.equal(previous.unit(target.id)?.hp, target.hp);
  const after = game.unit(target.id);
  assert.equal(after?.hp ?? 0, Math.max(0, target.hp - forecast.damage));
  writeBattle(game);
  assert.equal(JSON.parse(localStorage.getItem(SAVE_KEY)!).version, SAVE_VERSION);
  const restored = loadLiveBattle(factory).game;
  assert.deepEqual(restored.snapshot(), game.snapshot());
  restored.dispose();
  game.dispose();
});

test("Campaign dialogue and retry controls dispatch through native hooks", () => {
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

test("Supported legacy woodland revisions migrate without losing campaign progress", () => {
  const old = Game.startCampaign("green-accord");
  old.mapRevision = 1;
  old.round = 3;
  old.gold.player = 19;
  saveBattle(old);
  const restored = loadLiveBattle(factory);
  assert.equal(restored.notice, undefined);
  assert.equal(restored.game.mapRevision, 2);
  assert.equal(restored.game.round, 3);
  assert.equal(restored.game.gold.player, 19);
  assert.deepEqual(restored.game.units, old.units);
  writeBattle(restored.game);
  const reloaded = loadLiveBattle(factory);
  assert.deepEqual(reloaded.game.snapshot(), restored.game.snapshot());
  reloaded.game.dispose();
  restored.game.dispose();
});

test("A failed legacy import is wiped and cannot prevent startup", () => {
  saveBattle(new Game("mirror"));
  const restored = loadLiveBattle({
    ...factory,
    restore: () => ({ ok: false, error: { code: "invalidRequest", message: "Rejected" } }),
  });
  assert(restored.notice);
  assert.equal(restored.game.turn, "player");
  assert.equal(localStorage.getItem(SAVE_KEY), null);
  restored.game.dispose();
});

test("Rejected native campaign data leaves the WASM factory usable", () => {
  const game = LiveGame.create(factory, { kind: "campaign", id: "lantern-road" });
  writeBattle(game);
  game.dispose();
  const checkpoint = JSON.parse(localStorage.getItem(SAVE_KEY)!);
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
    localStorage.setItem(SAVE_KEY, JSON.stringify(checkpoint));
    const restored = loadLiveBattle(factory);
    assert(restored.notice);
    restored.game.dispose();
  }
  const next = LiveGame.create(factory, { kind: "campaign", id: "lantern-road" });
  next.retryCampaignMission();
  assert.equal(next.units.find((unit) => unit.role === "leader")?.name, "Captain Nera");
  next.dispose();
});

test("Live enemy planning finishes a phase and saves the resulting player turn", async () => {
  const controller = new BattleController(async () => {}, factory);
  await controller.endTurn();
  const after = controller.snapshot();
  assert.equal(after.turn, "player");
  assert.equal(after.round, 2);
  const restored = loadLiveBattle(factory);
  assert.equal(restored.pendingEnemy, false);
  assert.deepEqual(restored.game.snapshot(), after);
  restored.game.dispose();
  controller.dispose();
});

test("An invalid native checkpoint is wiped and leaves a playable battle", () => {
  const rejected = { version: 999, checkpoint: {}, pendingEnemy: false };
  localStorage.setItem(SAVE_KEY, JSON.stringify(rejected));
  const restored = loadLiveBattle(factory);
  assert(restored.notice);
  assert.equal(restored.game.turn, "player");
  assert.equal(localStorage.getItem(SAVE_KEY), null);
  restored.game.dispose();
});

test("Reloading during an enemy turn replays the whole phase from its boundary", async () => {
  // Node has no image decoder; artwork failure must not affect turn recovery.
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
  writeBattle(game, true);
  game.endTurn();
  const phaseSnapshot = game.snapshot();
  game.enemyStep();
  writeBattle(game);
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
  const controller = new BattleController(() => paused, factory);
  let completed!: () => void;
  const completion = new Promise<void>((resolve) => {
    completed = resolve;
  });
  const unsubscribe = controller.view.subscribe((view) => {
    if (view.game.turn === "player" && view.game.round === 2) completed();
  });
  controller.initialize();
  controller.initialize();
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
  const restored = loadLiveBattle(factory);
  assert.equal(restored.pendingEnemy, false);
  assert.deepEqual(restored.game.snapshot(), controller.snapshot());
  restored.game.dispose();
  controller.dispose();
  unsubscribe();
});
