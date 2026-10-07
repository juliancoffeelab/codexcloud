import { LiveGame } from "../src/game/live-game";
import { saveBattle as writeBattle } from "../src/save/write";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { artworkForStyle } from "../src/presentation/art";
import type { BattleController as Controller, BattleView } from "../src/controller/battle";
let BattleController: typeof Controller;
import { createTestFactory } from "./native-fixture";
import { Game, TYPES } from "./game-fixture";
import { campaignMission, dialoguePending } from "../src/game/campaign-progress";
import { SAVE_KEY, saveBattle, loadBattle } from "./legacy-save";
import { required, TestStorage, mockSpriteCanvas } from "./helpers";
import { battlefieldTiles } from "../src/ui/battlefield-view";
import { unitPanelView } from "../src/ui/unit-panel-view";

const storage = new TestStorage();

const immediatePause = async () => {};

function observe(controller: Controller) {
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

function pauseTurn(callToPause = 1) {
  let release: (() => void) | undefined;
  let markReached: (() => void) | undefined;
  const reached = new Promise<void>((resolve) => {
    markReached = resolve;
  });
  const first = new Promise<void>((resolve) => {
    release = resolve;
  });
  let calls = 0;
  return {
    reached,
    pause: () => {
      if (++calls !== callToPause) return Promise.resolve();
      required(markReached)();
      return first;
    },
    release: () => required(release)(),
  };
}

// The developer layout uses ordinary selection, movement, combat, turns and persistence.
test("Developer map supports normal actions and persistence", async () => {
  storage.clear();
  const controller = createController(immediatePause);
  const observed = observe(controller);
  controller.commands.reset("developer");
  const opening = controller.snapshot();
  const player = required(opening.units.find((unit) => unit.side === "player"));
  const enemy = required(opening.units.find((unit) => unit.side === "enemy"));
  controller.commands.choose(player.tile);
  const panel = unitPanelView(observed.view);
  assert.equal(panel.kind, "details");
  if (panel.kind === "details") assert.equal(panel.details.actionStatus, "Ready");
  assert(controller.actions(player.id).moves.some((move) => move.tile === "0,2"));
  assert(
    battlefieldTiles(observed.view.game, player.id, artworkForStyle("pixel"), null).some((tile) =>
      tile.classes.includes("reachable"),
    ),
  );
  controller.move(player.id, "0,2");
  assert(observed.view.canUndo);
  controller.undoMove();
  assert.deepEqual(controller.snapshot(), opening);
  controller.commands.choose(enemy.tile);
  assert.equal(unitPanelView(observed.view).kind, "combat");
  controller.attack(player.id, enemy.id);
  assert(required(controller.snapshot().units.find((unit) => unit.id === enemy.id)).hp < enemy.hp);
  assert.deepEqual(loadBattle().game.snapshot(), controller.snapshot());
  await controller.commands.endTurn();
  assert(controller.snapshot().round > opening.round || controller.snapshot().winner);
  assert.deepEqual(loadBattle().game.snapshot(), controller.snapshot());
  controller.dispose();
  observed.unsubscribe();
  storage.clear();
});

// Dialogue navigation persists its cursor without advancing gameplay or reopening finished lines.
test("Dialogue navigation persists its cursor without advancing gameplay", async () => {
  storage.clear();
  const controller = createController(immediatePause);
  const observed = observe(controller);
  controller.commands.startCampaign("green-accord");
  const opening = observed.view;
  const campaign = required(opening.game.campaign);
  assert(campaign.dialogue.length > 1);
  controller.commands.previousDialogue();
  assert.equal(observed.view, opening, "the first line has no previous line");
  controller.commands.advanceDialogue();
  assert.equal(required(observed.view.game.campaign).dialogueIndex, 1);
  assert.equal(campaign.dialogueIndex, 0, "previous publications keep their original cursor");
  assert.equal(observed.view.renderRevision, opening.renderRevision);
  const onSecondLine = controller.snapshot();
  assert.throws(
    () => controller.move(required(onSecondLine.units[0]).id, "3,2"),
    /Wait for your turn/,
  );
  await assert.rejects(controller.endTurn(), /Wait for your turn/);
  assert.throws(() => controller.undoMove(), /Finish the conversation/);
  assert.deepEqual(controller.snapshot(), onSecondLine);

  const restored = createController(immediatePause);
  assert.equal(required(restored.snapshot().campaign).dialogueIndex, 1);
  restored.commands.previousDialogue();
  assert.equal(required(restored.snapshot().campaign).dialogueIndex, 0);
  const rewound = createController(immediatePause);
  assert.equal(required(rewound.snapshot().campaign).dialogueIndex, 0);
  while (dialoguePending(rewound.snapshot().campaign)) rewound.commands.advanceDialogue();
  const finished = rewound.snapshot();
  const finishedView = observe(rewound);
  const finishedPublication = finishedView.view;
  const finishedSave = storage.getItem(SAVE_KEY);
  rewound.commands.advanceDialogue();
  assert.equal(finishedView.view, finishedPublication, "finished dialogue is a true no-op");
  assert.equal(storage.getItem(SAVE_KEY), finishedSave);
  rewound.commands.previousDialogue();
  assert.deepEqual(rewound.snapshot(), finished, "back cannot reopen a completed conversation");
  rewound.reset();
  const skirmish = rewound.snapshot();
  rewound.commands.previousDialogue();
  assert.deepEqual(rewound.snapshot(), skirmish, "back has no stale campaign to change");
  rewound.dispose();
  assert.throws(() => rewound.commands.previousDialogue(), /session has closed/);
  controller.dispose();
  restored.dispose();
  finishedView.unsubscribe();
  observed.unsubscribe();
  storage.clear();
});

// Stray dialogue commands cannot overwrite the checkpoint of a paused enemy turn.
test("Stray dialogue cannot overwrite a paused turn checkpoint", async () => {
  storage.clear();
  const paused = pauseTurn();
  const controller = createController(paused.pause);
  const observed = observe(controller);
  const beforeTurn = controller.snapshot();
  const pending = controller.endTurn();
  const checkpoint = storage.getItem(SAVE_KEY);
  const enemyView = observed.view;
  controller.commands.advanceDialogue();
  controller.commands.advanceDialogue(true);
  controller.commands.previousDialogue();
  assert.equal(observed.view, enemyView, "absent dialogue cannot publish an unrelated revision");
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  const loaded = loadBattle();
  assert(loaded.pendingEnemy);
  assert.deepEqual(loaded.game.snapshot(), beforeTurn);
  controller.dispose();
  paused.release();
  await pending;
  observed.unsubscribe();
});

// Navigating native enemy-phase dialogue cannot overwrite its recovery checkpoint.
test("Enemy-turn dialogue keeps the recovery save", async () => {
  const fixture = Game.startCampaign("ashen-oath", 1);
  fixture.turn = "enemy";
  fixture.campaign!.dialogue = [...campaignMission(fixture.campaign!).refuge!.escalation];
  fixture.campaign!.dialogueIndex = 0;
  const native = LiveGame.restore(createTestFactory(), {
    seed: 17,
    state:
      fixture.captureState() as unknown as import("../rust/pkg/hex_command").SavedGame["state"],
  });
  writeBattle(native);
  native.dispose();
  const paused = pauseTurn();
  const controller = createController(paused.pause);
  const observed = observe(controller);
  controller.initialize();
  const checkpoint = storage.getItem(SAVE_KEY);
  await paused.reached;
  assert.equal(observed.view.game.turn, "enemy");
  assert(dialoguePending(observed.view.game.campaign));
  const originalCursor = required(observed.view.game.campaign).dialogueIndex;
  controller.commands.advanceDialogue();
  assert.equal(required(observed.view.game.campaign).dialogueIndex, originalCursor + 1);
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  controller.commands.previousDialogue();
  assert.equal(required(observed.view.game.campaign).dialogueIndex, originalCursor);
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  assert.equal(loadBattle().game.turn, "enemy");
  controller.dispose();
  paused.release();
  await Promise.resolve();
  observed.unsubscribe();
});

// Completing a vigil at the next player turn keeps the published mission outcome status.
test("Completed vigil publishes and persists its outcome", async () => {
  storage.clear();
  const fixture = Game.startCampaign("green-accord", 1);
  fixture.advanceDialogue(true);
  fixture.units = fixture.units.filter((unit) => unit.side === "player");
  const goal = required(
    campaignMission(required(fixture.campaign)).goals.find((g) => g.kind === "vigil"),
  );
  assert.equal(goal.kind, "vigil");
  const lark = required(fixture.units.find((unit) => unit.name === "Lark"));
  const occupant = fixture.at(goal.tile);
  if (occupant && occupant.id !== lark.id) occupant.tile = "5,4";
  lark.tile = goal.tile;
  required(fixture.tile(goal.tile)).owner = "player";
  fixture.round = 3;
  required(fixture.campaign).vigil = { heldTurns: 2, lastCountedRound: 3 };
  saveBattle(fixture);
  assert.equal(loadBattle().game.mapId, fixture.mapId);
  const controller = createController(immediatePause);
  const observed = observe(controller);
  await controller.endTurn();
  assert.equal(observed.view.game.winner, "player");
  assert.equal(observed.view.status, observed.view.game.campaign?.outcome);
  assert(dialoguePending(observed.view.game.campaign));
  controller.commands.advanceDialogue();
  assert.equal(
    required(loadBattle().game.campaign).dialogueIndex,
    required(observed.view.game.campaign).dialogueIndex,
  );
  controller.dispose();
  observed.unsubscribe();
  storage.clear();
});

// Finished enemy-turn battles persist their outcome conversation normally.
test("Finished enemy-turn battles persist outcome dialogue", async () => {
  const fixture = Game.startCampaign("green-accord");
  fixture.advanceDialogue(true);
  fixture.units = fixture.units.filter((unit) => unit.role === "leader" || unit.name === "Lark");
  const leader = required(
    fixture.units.find((unit) => unit.side === "player" && unit.role === "leader"),
  );
  // Use a frontline attacker so leader recruitment restraint cannot alter this defeat fixture.
  const enemy = fixture.addUnit("soldier", "enemy", "4,4");
  leader.tile = "3,4";
  leader.hp = 1;
  enemy.tile = "4,4";
  saveBattle(fixture);
  const controller = createController(immediatePause);
  await controller.endTurn();
  assert.equal(controller.snapshot().turn, "enemy");
  assert.equal(controller.snapshot().winner, "enemy");
  assert(dialoguePending(controller.snapshot().campaign));
  controller.commands.advanceDialogue();
  assert.deepEqual(loadBattle().game.snapshot(), controller.snapshot());
  controller.dispose();
  storage.clear();
});

// Rejected replacement must not spend undo history or cancel an enemy turn.
test("Rejected replacement preserves undo and pending turns", async () => {
  const controller = createController(immediatePause);
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
});

// Revisions are immutable, and all availability belongs to the published revision.
test("Publications are immutable and availability follows the revision", async () => {
  storage.clear();
  const controller = createController(immediatePause);
  const observed = observe(controller);
  const opening = observed.view;
  assert.equal(Reflect.get(controller, "game"), undefined);
  assert.equal(Reflect.get(controller.commands, "active"), undefined);
  assert.equal(Reflect.get(opening.game, "move"), undefined);
  assert(Object.isFrozen(controller.commands));
  assert.equal(Reflect.set(required(opening.game.unit("u3")), "tile", "3,1"), false);

  controller.choose("1,2");
  assert.equal(observed.view.game, opening.game, "selection must not rebuild the game view");
  controller.focusTile("2,2");
  assert.equal(observed.view.game, opening.game, "keyboard focus reuses the published game");
  controller.message("Inspect the field.");
  assert.equal(observed.view.game, opening.game, "status reuses the published game");
  await controller.setSound(false);
  assert.equal(observed.view.game, opening.game, "sound preferences reuse the published game");
  controller.clearSelection();
  assert.equal(observed.view.game, opening.game, "deselection must not rebuild the game view");

  controller.move("u3", "3,1");
  assert.notEqual(observed.view.game, opening.game, "engine actions publish a fresh game");
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
});

// Browser tools may only act for the player; preview results cannot mutate rules.
test("Browser commands are player-only and previews cannot mutate rules", async () => {
  storage.clear();
  const fixture = new Game();
  const enemy = fixture.addUnit("soldier", "enemy", "2,3");
  saveBattle(fixture);
  const paused = pauseTurn();
  const controller = createController(paused.pause);
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
});

// An old unmounted session must never save over its replacement after an await.
test("Unmounted sessions cannot overwrite replacements", async () => {
  storage.clear();
  const paused = pauseTurn();
  const previous = createController(paused.pause);
  const pending = previous.endTurn();
  previous.dispose();
  const replacement = createController(immediatePause);
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
});

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
  static finish(style: string, success: boolean, terrainSuccess = success) {
    const requests = this.requests.filter((image) => !image.completed && image.src.includes(style));
    assert.equal(requests.length, 2, `${style} artwork loads one unit atlas and one terrain atlas`);
    assert(requests.some((image) => image.src.includes(`/units-${style}.png`)));
    assert(requests.some((image) => image.src.includes(`/terrain-${style}.png`)));
    assert.equal(new Set(requests.map((image) => image.src)).size, requests.length);
    for (const image of requests) {
      image.completed = true;
      if (image.src.includes(`/terrain-${style}.png`) ? terrainSuccess : success) image.onload?.();
      else image.onerror?.();
    }
  }
}

async function settleCallbacks() {
  for (let turn = 0; turn < 6; turn++) await Promise.resolve();
}

// A replay is consumed once, and replacement before initialization discards it.
test("Replay is consumed once and replacement discards pending initialization", async () => {
  storage.clear();
  saveBattle(new Game(), true);
  const replacement = createController(immediatePause);
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
  const paused = pauseTurn();
  const replay = createController(paused.pause);
  replay.initialize();
  replay.initialize();
  assert.equal(replay.snapshot().turn, "enemy");
  const checkpoint = storage.getItem(SAVE_KEY);
  replay.commands.advanceDialogue();
  replay.commands.previousDialogue();
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  assert(loadBattle().pendingEnemy);
  paused.release();
  await settleCallbacks();
  assert.equal(replay.snapshot().turn, "player");
  assert.equal(replay.snapshot().round, 2);
  const saved = JSON.parse(required(storage.getItem(SAVE_KEY))) as { pendingEnemy: boolean };
  assert.equal(saved.pendingEnemy, false);
  replay.dispose();
});

// Manual preferences supersede fallback work and older manual requests.
test("Manual art preferences supersede fallback and older requests", async () => {
  storage.clear();
  const controller = createController(immediatePause);
  const observed = observe(controller);
  controller.initialize();
  const openingGame = observed.view.game;
  const openingArtwork = observed.view.artwork;
  ControlledImage.finish("normal", true, false);
  await settleCallbacks(); // The initial request now waits on low-poly fallback.
  const latest = controller.setArtStyle("pixel");
  ControlledImage.finish("pixel", true);
  await latest;
  const chosen = observed.view;
  assert.equal(
    Object.keys(openingArtwork.frames).length,
    0,
    "Loading must not mutate an already-published artwork snapshot",
  );
  assert(Object.isFrozen(chosen.artwork));
  assert(Object.isFrozen(chosen.artwork.frames));
  assert(chosen.artwork.frames.player_thorncaller?.url.startsWith("data:image/png;base64,"));
  ControlledImage.finish("low-poly", false);
  await settleCallbacks();
  assert.equal(observed.view, chosen, "stale fallback failure must not publish or report an error");
  assert.equal(observed.view.artStyle, "pixel");
  assert.equal(observed.view.game, openingGame, "artwork changes reuse the published game");
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
});

// Initialization cannot cancel a manual preference, and unmount cancels its writes.
test("Initialization preserves manual preference and unmount cancels writes", async () => {
  storage.clear();
  const controller = createController(immediatePause);
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
});

const controllers: Controller[] = [];
function createController(pause?: ConstructorParameters<typeof Controller>[0]) {
  const controller = new BattleController(pause, createTestFactory());
  controllers.push(controller);
  return controller;
}
let restoreCanvas: () => void;
beforeEach(async () => {
  vi.resetModules();
  ({ BattleController } = await import("../src/controller/battle"));
  storage.clear();
  ControlledImage.requests = [];
  vi.stubGlobal("localStorage", storage);
  vi.stubGlobal("Image", ControlledImage);
  restoreCanvas = mockSpriteCanvas();
});
afterEach(() => {
  for (const controller of controllers.splice(0)) controller.dispose();
  restoreCanvas();
  vi.unstubAllGlobals();
});
