import assert from "node:assert/strict";
import { BattleController, type BattleView } from "../src/controller/battle";
import { Game, TYPES } from "../src/game/engine";
import { SAVE_KEY, saveBattle } from "../src/game/save";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();
globalThis.localStorage = storage;
const immediatePause = async () => {};

function observe(controller: BattleController) {
  let latest: BattleView | undefined;
  const unsubscribe = controller.view.subscribe((view) => {
    latest = view;
  });
  return {
    get view() {
      return required(latest);
    },
    unsubscribe,
  };
}

function pauseFirstAction() {
  let release: (() => void) | undefined;
  const first = new Promise<void>((resolve) => {
    release = resolve;
  });
  let paused = false;
  return {
    pause: () => {
      if (paused) return Promise.resolve();
      paused = true;
      return first;
    },
    release: () => required(release)(),
  };
}

// Rejected replacement must not spend undo history or cancel an enemy turn.
{
  const controller = new BattleController(immediatePause);
  const observed = observe(controller);
  controller.move("u3", "3,1");
  const before = controller.snapshot();
  const saved = storage.getItem(SAVE_KEY);
  assert.throws(() => controller.reset("unknown"), /Unknown battlefield/);
  assert.deepEqual(controller.snapshot(), before);
  assert.equal(storage.getItem(SAVE_KEY), saved);
  assert(observed.view.canUndo);
  controller.undoMove();
  const turn = controller.endTurn();
  assert.throws(() => controller.reset("unknown"), /Unknown battlefield/);
  await turn;
  assert.equal(observed.view.game.round, 2);
  assert.equal(observed.view.game.turn, "player");
  controller.dispose();
  observed.unsubscribe();
}

// Revisions are immutable, and all availability belongs to the published revision.
{
  storage.clear();
  const controller = new BattleController(immediatePause);
  const observed = observe(controller);
  const opening = observed.view;
  assert.equal(Reflect.get(controller, "game"), undefined);
  assert.equal(Reflect.get(controller.commands, "active"), undefined);
  assert.equal(Reflect.get(opening.game, "move"), undefined);
  assert(Object.isFrozen(controller.commands));
  assert.equal(Reflect.set(required(opening.game.unit("u3")), "tile", "3,1"), false);

  controller.move("u3", "3,1");
  assert.equal(opening.game.unit("u3")?.tile, "1,2");
  assert.equal(observed.view.game.unit("u3")?.tile, "3,1");
  assert(observed.view.canUndo);
  assert.equal(observed.view.selected, "u3");
  assert.equal(observed.view.inspectedTile, "3,1");
  const movedGame = observed.view.game;
  controller.message("Inspect the field.");
  assert.equal(observed.view.game, movedGame, "metadata safely reuses an immutable revision");
  assert.equal(observed.view.status, "Inspect the field.");
  assert.equal(Reflect.set(observed.view, "selected", "u6"), false);

  controller.undoMove();
  assert.equal(observed.view.game.unit("u3")?.tile, "1,2");
  assert.equal(observed.view.canUndo, false);
  controller.choose("1,2"); // Clear the restored scout selection.
  controller.choose("0,2");
  assert(observed.view.recruitment?.stationed);
  assert.equal(observed.view.recruitment?.commander?.id, "u1");
  assert.equal(observed.view.inspectedTile, "0,2");
  const inspected = observed.view;
  await controller.endTurn();
  assert.equal(inspected.game.turn, "player");
  assert.equal(inspected.game.round, 1);
  assert.equal(observed.view.game.turn, "player");
  assert.equal(observed.view.game.round, 2);
  assert(observed.view.recruitment?.stationed);
  controller.dispose();
  observed.unsubscribe();
}

// Browser tools may only act for the player; preview results cannot mutate rules.
{
  storage.clear();
  const fixture = new Game();
  const enemy = fixture.addUnit("soldier", "enemy", "2,3");
  saveBattle(fixture);
  const paused = pauseFirstAction();
  const controller = new BattleController(paused.pause);
  const observed = observe(controller);
  const preview = required(controller.actions("u2").attacks[0]);
  const weapon = required(preview.weapons[0]);
  const damage = TYPES.soldier.attacks[0].damage;
  weapon.attack.damage = 999;
  assert.equal(TYPES.soldier.attacks[0].damage, damage);

  assert.throws(() => controller.move(enemy.id, "2,2"), /cannot act/);
  const pending = controller.endTurn();
  const enemyTurn = controller.snapshot();
  assert.equal(observed.view.game.turn, "enemy");
  assert.equal(observed.view.canUndo, false);
  assert.throws(() => controller.move(enemy.id, "2,2"), /Wait for your turn/);
  assert.throws(() => controller.attack(enemy.id, "u2"), /Wait for your turn/);
  assert.throws(() => controller.recruit("u4", "soldier", "10,2"), /Wait for your turn/);
  assert.deepEqual(controller.snapshot(), enemyTurn);
  paused.release();
  await pending;
  assert.equal(observed.view.game.turn, "player");
  controller.dispose();
  observed.unsubscribe();
}

// An old unmounted session must never save over its replacement after an await.
{
  storage.clear();
  const paused = pauseFirstAction();
  const previous = new BattleController(paused.pause);
  const pending = previous.endTurn();
  previous.dispose();
  const replacement = new BattleController(immediatePause);
  replacement.reset("river");
  const saved = storage.getItem(SAVE_KEY);
  paused.release();
  await pending;
  assert.equal(storage.getItem(SAVE_KEY), saved);
  assert.equal(replacement.snapshot().mapId, "river");
  assert.equal(replacement.snapshot().round, 1);
  assert.throws(() => previous.reset(), /session has closed/);
  assert.throws(() => previous.move("u3", "3,1"), /session has closed/);
  replacement.dispose();
}

// Complete image requests explicitly; no network, browser, or timer is involved.
class ControlledImage {
  static requests: ControlledImage[] = [];
  src = "";
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  completed = false;
  constructor() {
    ControlledImage.requests.push(this);
  }
  static finish(style: string, success: boolean) {
    const requests = this.requests.filter((image) => !image.completed && image.src.includes(style));
    assert.equal(requests.length, 3, `Expected one pending ${style} artwork family`);
    for (const image of requests) {
      image.completed = true;
      if (success) image.onload?.();
      else image.onerror?.();
    }
  }
}
const nativeImage = Reflect.get(globalThis, "Image") as typeof Image | undefined;
globalThis.Image = ControlledImage as unknown as typeof Image;
async function settleCallbacks() {
  for (let turn = 0; turn < 6; turn++) await Promise.resolve();
}

// A replay is consumed once, and replacement before initialization discards it.
{
  storage.clear();
  saveBattle(new Game(), true);
  const replacement = new BattleController(immediatePause);
  const observed = observe(replacement);
  replacement.reset("river");
  replacement.initialize();
  replacement.initialize();
  assert.equal(observed.view.game.mapId, "river");
  assert.equal(observed.view.game.turn, "player");
  assert.equal(observed.view.game.round, 1);
  replacement.dispose();
  observed.unsubscribe();

  saveBattle(new Game(), true);
  const paused = pauseFirstAction();
  const replay = new BattleController(paused.pause);
  replay.initialize();
  replay.initialize();
  assert.equal(replay.snapshot().turn, "enemy");
  paused.release();
  await settleCallbacks();
  assert.equal(replay.snapshot().turn, "player");
  assert.equal(replay.snapshot().round, 2);
  const saved = JSON.parse(required(storage.getItem(SAVE_KEY))) as { pendingEnemy: boolean };
  assert.equal(saved.pendingEnemy, false);
  replay.dispose();
}

// Manual preferences supersede fallback work and older manual requests.
{
  storage.clear();
  const controller = new BattleController(immediatePause);
  const observed = observe(controller);
  controller.initialize();
  ControlledImage.finish("normal", false);
  await settleCallbacks(); // The initial request now waits on low-poly fallback.
  const latest = controller.setArtStyle("pixel");
  ControlledImage.finish("pixel", true);
  await latest;
  const chosen = observed.view;
  ControlledImage.finish("low-poly", false);
  await settleCallbacks();
  assert.equal(observed.view, chosen, "stale fallback failure must not publish or report an error");
  assert.equal(observed.view.artStyle, "pixel");
  assert.equal(observed.view.artLoading, false);
  assert.equal(storage.getItem("hex-command-art-style"), "pixel");

  const older = controller.setArtStyle("low-poly");
  const newer = controller.setArtStyle("pixel"); // Pixel is already loaded.
  await newer;
  const current = observed.view;
  ControlledImage.finish("low-poly", true);
  await older;
  assert.equal(observed.view, current);
  assert.equal(observed.view.artStyle, "pixel");
  assert.equal(observed.view.artLoading, false);
  assert.equal(storage.getItem("hex-command-art-style"), "pixel");
  controller.dispose();
  observed.unsubscribe();
}

// Initialization cannot cancel a manual preference, and unmount cancels its writes.
{
  storage.clear();
  const controller = new BattleController(immediatePause);
  const observed = observe(controller);
  const pending = controller.setArtStyle("normal");
  controller.initialize();
  ControlledImage.finish("normal", false);
  await pending;
  assert.equal(observed.view.artLoading, false);
  assert.match(observed.view.status, /Could not load/);

  const afterUnmount = controller.setArtStyle("normal");
  const beforeUnmount = observed.view;
  controller.dispose();
  ControlledImage.finish("normal", true);
  await afterUnmount;
  assert.equal(observed.view, beforeUnmount);
  assert.equal(storage.getItem("hex-command-art-style"), null);
  observed.unsubscribe();
}
if (nativeImage) globalThis.Image = nativeImage;
else Reflect.deleteProperty(globalThis, "Image");

console.log(
  "PASS: immutable publications, player-only tools, isolated previews, cancellation, replay and latest artwork requests",
);
