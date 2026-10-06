import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test, beforeEach, afterEach, vi } from "vitest";
import { initSync, NativeEngineFactory } from "../rust/pkg/hex_command";
import { createNativeEngineFactory } from "../src/game/native-engine";
import { LiveGame, loadLiveBattle } from "../src/game/live-game";
import { BattleController } from "../src/controller/battle";
import { Game } from "../src/game/engine";
import { saveBattle, SAVE_KEY } from "../src/game/save";
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

// Use the real generated WASM with no browser, server, or simulation batch.
initSync({
  module: new Uint8Array(readFileSync(new URL("../rust/pkg/hex_command_bg.wasm", import.meta.url))),
});
let factory: ReturnType<typeof createNativeEngineFactory>;
beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
  factory = createNativeEngineFactory(new NativeEngineFactory());
});
afterEach(() => {
  factory.dispose();
});

test("Live controller uses native movement/undo and keeps previous UI queries immutable", () => {
  const controller = new BattleController(async () => {}, factory);
  let published: BattleView | undefined;
  const unsubscribe = controller.view.subscribe((view) => {
    published = view;
  });
  controller.reset("developer");
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
  const old = new Game("developer");
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
  game.saveNative();
  assert.equal(JSON.parse(localStorage.getItem(SAVE_KEY)!).engine, "rust-wasm");
  const restored = loadLiveBattle(factory).game;
  assert.deepEqual(restored.snapshot(), game.snapshot());
  restored.dispose();
  game.dispose();
});

test("All campaign dialogue and retry controls dispatch through native hooks", () => {
  for (const id of ["lantern-road", "green-accord", "ashen-oath", "borrowed-tide"]) {
    const game = LiveGame.create(factory, { kind: "campaign", id });
    const opening = game.captureState();
    game.advanceDialogue();
    game.previousDialogue();
    assert.equal(game.campaign?.dialogueIndex, 0);
    game.advanceDialogue(true);
    assert.equal(game.campaign?.dialogueIndex, game.campaign?.dialogue.length);
    game.retryCampaignMission();
    assert.deepEqual(game.captureState(), opening);
    game.dispose();
  }
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

test("An invalid native checkpoint preserves the old save and leaves a playable battle", () => {
  const rejected = { engine: "rust-wasm", pendingEnemy: false, saved: { version: 999 } };
  localStorage.setItem(SAVE_KEY, JSON.stringify(rejected));
  const restored = loadLiveBattle(factory);
  assert(restored.notice);
  assert.equal(restored.game.turn, "player");
  assert.deepEqual(JSON.parse(localStorage.getItem(SAVE_KEY + "-unrestored")!), rejected);
  restored.game.dispose();
});
