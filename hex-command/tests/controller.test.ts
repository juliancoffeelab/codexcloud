import { cellKey } from "../src/game/hex";
import { readSave as readStoredSave, storeSave } from "../src/save/storage";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { currentArtwork } from "../src/presentation/art";
import {
  BattleController,
  type BattleController as Controller,
  type BattleView,
} from "../src/controller/battle";
import { createTestFactory } from "./native-fixture";
import { Game, TYPES, developerFixture } from "./game-fixture";
import { campaignMission, dialoguePending } from "../src/game/campaign-progress";
import { SAVE_KEY, saveBattle, loadBattle as loadStoredBattle } from "./legacy-save";
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
  await saveBattle(developerFixture());
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  const opening = controller.snapshot();
  const player = required(opening.units.find((unit) => unit.side === "player"));
  const enemy = required(opening.units.find((unit) => unit.side === "enemy"));
  controller.commands.choose(cellKey(player.tile));
  const panel = unitPanelView(observed.view);
  assert.equal(panel.kind, "details");
  if (panel.kind === "details") assert.equal(panel.details.actionStatus, "Ready");
  assert(controller.actions(player.id).moves.some((move) => move.tile === "0,2"));
  assert(
    battlefieldTiles(observed.view.game, player.id, currentArtwork(), null).some((tile) =>
      tile.classes.includes("reachable"),
    ),
  );
  controller.move(player.id, "0,2");
  assert(observed.view.canUndo);
  controller.undoMove();
  assert.deepEqual(controller.snapshot(), opening);
  controller.commands.choose(cellKey(enemy.tile));
  assert.equal(unitPanelView(observed.view).kind, "combat");
  controller.attack(player.id, enemy.id);
  assert(required(controller.snapshot().units.find((unit) => unit.id === enemy.id)).hp < enemy.hp);
  assert.deepEqual((await loadBattle()).game.snapshot(), controller.snapshot());
  await controller.commands.endTurn();
  assert(controller.snapshot().round > opening.round || controller.snapshot().winner);
  assert.deepEqual((await loadBattle()).game.snapshot(), controller.snapshot());
  controller.dispose();
  observed.unsubscribe();
  storage.clear();
});

test("Inspection switches tiles and closes when the inspected unit or terrain is tapped again", async () => {
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  const opening = controller.snapshot();
  const player = required(opening.units.find((unit) => unit.side === "player"));
  const enemy = required(opening.units.find((unit) => unit.side === "enemy"));
  const empty = required(
    observed.view.game.tiles.find(
      (tile) => !opening.units.some((u) => cellKey(u.tile) === tile.id),
    ),
  );
  const other = required(
    observed.view.game.tiles.find(
      (tile) => tile.id !== empty.id && !opening.units.some((u) => cellKey(u.tile) === tile.id),
    ),
  );
  const saved = await readSave();
  const game = observed.view.game;
  controller.commands.choose(cellKey(enemy.tile));
  assert.equal(observed.view.selected, enemy.id);
  controller.commands.choose(cellKey(enemy.tile));
  assert.equal(observed.view.selected, null);
  assert.equal(observed.view.inspectedTile, null);
  controller.commands.choose(empty.id);
  assert.equal(observed.view.inspectedTile, empty.id);
  controller.commands.choose(other.id);
  assert.equal(observed.view.inspectedTile, other.id);
  controller.commands.choose(other.id);
  assert.equal(observed.view.inspectedTile, null);
  controller.commands.choose(cellKey(player.tile));
  controller.commands.choose(cellKey(player.tile));
  assert.equal(observed.view.selected, null);
  assert.equal(observed.view.inspectedTile, null);
  controller.commands.choose(cellKey(enemy.tile));
  controller.commands.choose(empty.id);
  assert.equal(observed.view.selected, null);
  assert.equal(observed.view.inspectedTile, empty.id);
  controller.commands.choose(empty.id);
  assert.equal(observed.view.inspectedTile, null);
  assert.equal(observed.view.target, null);
  assert.equal(observed.view.game, game);
  assert.deepEqual(controller.snapshot(), opening);
  assert.equal(await readSave(), saved);
  observed.unsubscribe();
});

test("Click-away dismissal preserves movement, attack targeting and undo", async () => {
  await saveBattle(developerFixture());
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  const opening = controller.snapshot();
  const player = required(opening.units.find((unit) => unit.side === "player"));
  const move = required(controller.actions(player.id).moves[0]);
  controller.commands.choose(cellKey(player.tile));
  controller.commands.choose(cellKey(move.tile));
  assert.equal(cellKey(observed.view.game.unit(player.id)?.tile), move.tile);
  assert.equal(observed.view.inspectedTile, move.tile);
  assert(observed.view.canUndo);
  controller.commands.clearSelection(true);
  assert.equal(observed.view.inspectedTile, null);
  assert(observed.view.canUndo, "dismissing the panel must not spend undo history");
  controller.commands.undoMove();
  assert.deepEqual(controller.snapshot(), opening);
  const target = required(controller.actions(player.id).attacks[0]);
  const targetTile = required(observed.view.game.unit(target.id)).tile;
  controller.commands.choose(cellKey(targetTile));
  assert.equal(unitPanelView(observed.view).kind, "combat");
  controller.commands.choose(cellKey(targetTile));
  assert.equal(observed.view.selected, null);
  assert.equal(observed.view.target, null);
  assert.equal(observed.view.inspectedTile, null);
  assert.deepEqual(controller.snapshot(), opening, "targeting or dismissal must not attack");
  observed.unsubscribe();
});

// Dialogue navigation persists its cursor without advancing gameplay or reopening finished lines.
test("Dialogue navigation persists its cursor without advancing gameplay", async () => {
  storage.clear();
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  await controller.commands.startCampaign("green-accord");
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

  const restored = await createController(immediatePause);
  assert.equal(required(restored.snapshot().campaign).dialogueIndex, 1);
  restored.commands.previousDialogue();
  assert.equal(required(restored.snapshot().campaign).dialogueIndex, 0);
  const rewound = await createController(immediatePause);
  assert.equal(required(rewound.snapshot().campaign).dialogueIndex, 0);
  while (dialoguePending(rewound.snapshot().campaign)) rewound.commands.advanceDialogue();
  const finished = rewound.snapshot();
  const finishedView = observe(rewound);
  await rewound.saved();
  const finishedPublication = finishedView.view;
  const finishedSave = await readSave();
  rewound.commands.advanceDialogue();
  assert.equal(finishedView.view, finishedPublication, "finished dialogue is a true no-op");
  assert.equal(await readSave(), finishedSave);
  rewound.commands.previousDialogue();
  assert.deepEqual(rewound.snapshot(), finished, "back cannot reopen a completed conversation");
  await rewound.reset();
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
  const controller = await createController(paused.pause);
  const observed = observe(controller);
  const beforeTurn = controller.snapshot();
  const pending = controller.endTurn();
  const checkpoint = await readSave();
  const enemyView = observed.view;
  controller.commands.advanceDialogue();
  controller.commands.advanceDialogue(true);
  controller.commands.previousDialogue();
  assert.equal(observed.view, enemyView, "absent dialogue cannot publish an unrelated revision");
  assert.equal(await readSave(), checkpoint);
  const loaded = await loadBattle();
  assert(loaded.pendingEnemy);
  assert.deepEqual(loaded.game.snapshot(), beforeTurn);
  controller.dispose();
  paused.release();
  await pending;
  observed.unsubscribe();
});

// Completing a vigil at the next player turn keeps the published mission outcome status.
test("Completed vigil publishes and persists its outcome", async () => {
  storage.clear();
  const fixture = Game.startCampaign("green-accord", 1);
  fixture.advanceDialogue(true);
  fixture.units = fixture.units.filter((unit) => unit.side === "player");
  const goal = { tile: { col: 6, row: 4 } };
  const lark = required(fixture.units.find((unit) => unit.character === "lark"));
  const occupant = fixture.at(goal.tile);
  if (occupant && occupant.id !== lark.id) occupant.tile = { col: 5, row: 4 };
  lark.tile = goal.tile;
  required(fixture.tile(goal.tile)).owner = "player";
  fixture.round = 3;
  required(fixture.campaign).vigil = { heldTurns: 2, lastCountedRound: 3 };
  await saveBattle(fixture);
  assert.equal((await loadBattle()).game.mapId, fixture.mapId);
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  await controller.endTurn();
  assert.equal(observed.view.game.winner, "player");
  assert.equal(observed.view.status, observed.view.game.campaign?.outcome);
  assert(dialoguePending(observed.view.game.campaign));
  controller.commands.advanceDialogue();
  assert.equal(
    required((await loadBattle()).game.campaign).dialogueIndex,
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
  fixture.units = fixture.units.filter(
    (unit) => unit.role === "leader" || unit.character === "lark",
  );
  const leader = required(
    fixture.units.find((unit) => unit.side === "player" && unit.role === "leader"),
  );
  // Use a frontline attacker so leader recruitment restraint cannot alter this defeat fixture.
  const enemy = fixture.addUnit("soldier", "enemy", "4,4");
  leader.tile = { col: 3, row: 4 };
  leader.hp = 1;
  enemy.tile = { col: 4, row: 4 };
  await saveBattle(fixture);
  const controller = await createController(immediatePause);
  await controller.endTurn();
  assert.equal(controller.snapshot().turn, "enemy");
  assert.equal(controller.snapshot().winner, "enemy");
  assert(dialoguePending(controller.snapshot().campaign));
  controller.commands.advanceDialogue();
  assert.deepEqual((await loadBattle()).game.snapshot(), controller.snapshot());
  controller.dispose();
  storage.clear();
});

// Rejected replacement must not spend undo history or cancel an enemy turn.
test("Rejected replacement preserves undo and pending turns", async () => {
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  controller.move("u3", "3,1");
  const before = controller.snapshot();
  const saved = await readSave();
  assert.throws(() => controller.reset("unknown"), /Unknown battlefield/);
  assert.deepEqual(controller.snapshot(), before);
  assert.equal(await readSave(), saved);
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
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  const opening = observed.view;
  assert.equal(Reflect.get(controller, "game"), undefined);
  assert.equal(Reflect.get(controller.commands, "active"), undefined);
  assert.equal(Reflect.get(opening.game, "move"), undefined);
  assert(Object.isFrozen(controller.commands));
  assert.equal(Reflect.set(required(opening.game.unit("u3")), "tile", "3,1"), false);

  controller.choose("1,2");
  assert.equal(observed.view.game, opening.game, "selection must not rebuild the game view");
  controller.message("Inspect the field.");
  assert.equal(observed.view.game, opening.game, "status reuses the published game");
  await controller.setSound(false);
  assert.equal(observed.view.game, opening.game, "sound preferences reuse the published game");
  controller.clearSelection();
  assert.equal(observed.view.game, opening.game, "deselection must not rebuild the game view");

  controller.move("u3", "3,1");
  assert.notEqual(observed.view.game, opening.game, "engine actions publish a fresh game");
  assert.deepEqual(opening.game.unit("u3")?.tile, { col: 1, row: 2 });
  assert.deepEqual(observed.view.game.unit("u3")?.tile, { col: 3, row: 1 });
  assert(observed.view.canUndo);
  assert.equal(observed.view.selected, "u3");
  assert.equal(observed.view.inspectedTile, "3,1");
  const movedGame = observed.view.game;
  controller.message("Inspect the field.");
  assert.equal(observed.view.game, movedGame, "metadata safely reuses an immutable revision");
  assert.equal(observed.view.status, "Inspect the field.");
  assert.equal(Reflect.set(observed.view, "selected", "u6"), false);

  controller.undoMove();
  assert.deepEqual(observed.view.game.unit("u3")?.tile, { col: 1, row: 2 });
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
  await saveBattle(fixture);
  const paused = pauseTurn();
  const controller = await createController(paused.pause);
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
  const previous = await createController(paused.pause);
  const pending = previous.endTurn();
  previous.dispose();
  const replacement = await createController(immediatePause);
  await replacement.reset("river-valley");
  const saved = await readSave();
  paused.release();
  await pending;
  assert.equal(await readSave(), saved);
  assert.equal(replacement.snapshot().mapId, "river-valley");
  assert.equal(replacement.snapshot().round, 1);
  assert.throws(() => previous.reset(), /session has closed/);
  assert.throws(() => previous.move("u3", "3,1"), /session has closed/);
  replacement.dispose();
});

// A replay is consumed once, and replacement before initialization discards it.
test("Replay is consumed once and replacement discards pending initialization", async () => {
  storage.clear();
  await saveBattle(new Game(), true);
  const replacement = await createController(immediatePause);
  const observed = observe(replacement);
  await replacement.reset("river-valley");
  await replacement.initialize();
  await replacement.initialize();
  assert.equal(observed.view.game.mapId, "river-valley");
  assert.equal(observed.view.game.turn, "player");
  assert.equal(observed.view.game.round, 1);
  replacement.dispose();
  observed.unsubscribe();

  await saveBattle(new Game(), true);
  const paused = pauseTurn();
  const replay = await createController(paused.pause);
  await replay.initialize();
  await replay.initialize();
  assert.equal(replay.snapshot().turn, "enemy");
  const checkpoint = await readSave();
  replay.commands.advanceDialogue();
  replay.commands.previousDialogue();
  assert.equal(await readSave(), checkpoint);
  assert((await loadBattle()).pendingEnemy);
  let unsubscribe = () => {};
  const finished = new Promise<void>((resolve) => {
    unsubscribe = replay.view.subscribe((view) => {
      if (view.game.turn === "player") {
        unsubscribe();
        resolve();
      }
    });
  });
  paused.release();
  await finished;
  assert.equal(replay.snapshot().turn, "player");
  assert.equal(replay.snapshot().round, 2);
  const saved = JSON.parse(required(await readSave())) as { pendingEnemy: boolean };
  assert.equal(saved.pendingEnemy, false);
  replay.dispose();
});

test("Artwork URLs are published immediately and ignore retired style preferences", async () => {
  storage.setItem("hex-command-art-style", "low-poly");
  const controller = await createController(immediatePause);
  const observed = observe(controller);
  assert(Object.isFrozen(observed.view.artwork.frames));
  assert(observed.view.artwork.frames.player_thorncaller?.url.includes("assets/sprites/"));
  const opening = observed.view.artwork;
  await controller.initialize();
  assert.equal(observed.view.artwork, opening);
  controller.dispose();
  observed.unsubscribe();
});

async function settleWrites() {
  await Promise.all(controllers.map((controller) => controller.saved()));
}
async function readSave() {
  await settleWrites();
  return readStoredSave();
}
async function loadBattle() {
  await settleWrites();
  return loadStoredBattle();
}

const controllers: Controller[] = [];
async function createController(pause?: Parameters<typeof Controller.create>[0]) {
  await settleWrites();
  const controller = await BattleController.create(pause, createTestFactory());
  controllers.push(controller);
  return controller;
}
let restoreCanvas: () => void;
beforeEach(() => {
  storage.clear();
  vi.stubGlobal("localStorage", storage);
  restoreCanvas = mockSpriteCanvas();
});
afterEach(() => {
  for (const controller of controllers.splice(0)) controller.dispose();
  restoreCanvas();
  vi.unstubAllGlobals();
});
