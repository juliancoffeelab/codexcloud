import assert from "node:assert/strict";
import { Game } from "../src/game/engine";
import { campaignMission, goalProgress, returnsWithStory } from "../src/game/campaign-progress";
import { createGameView } from "../src/game/view";
import { createMoveUndo } from "../src/game/undo";
import { enemyActionIds } from "../src/game/enemy-turn";
import { loadBattle, saveBattle, SAVE_KEY } from "../src/game/save";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();
globalThis.localStorage = storage;
const companion = (game: Game) =>
  required(game.units.find((unit) => unit.side === "player" && unit.name === "Lark"));

// Occupying guards hold the cutting camps while the company assembles, and retake a lost post.
{
  const game = Game.startCampaign("green-accord");
  game.advanceDialogue(true);
  const company = game.units.filter((unit) => unit.side === "player").map((unit) => ({ ...unit }));
  const villages = game.tiles.filter((tile) => tile.owner === "player").map((tile) => tile.id);
  for (let turn = 0; turn < 5; turn++) {
    game.endTurn();
    game.enemyRecruit();
    for (const id of enemyActionIds(game.units)) game.enemyAct(id);
    game.endTurn();
    game.advanceDialogue(true);
  }
  assert.equal(game.winner, null);
  for (const initial of company) {
    assert.equal(required(game.unit(initial.id)).hp, initial.hp);
  }
  for (const village of villages) assert.equal(required(game.tile(village)).owner, "player");
  const raid = Game.startCampaign("green-accord");
  raid.advanceDialogue(true);
  const guard = required(raid.at("7,3"));
  guard.tile = "9,3";
  required(raid.tile("7,3")).owner = "player";
  raid.beginTurn("enemy");
  raid.enemyAct(guard.id);
  assert.equal(guard.tile, "7,3");
  assert.equal(required(raid.tile("7,3")).owner, "enemy");
}

// Camps stop the cutting, but the missing workers' ledger is also required.
{
  const game = Game.startCampaign("green-accord");
  game.advanceDialogue(true);
  for (const goal of campaignMission(game.campaign!).goals) {
    if (goal.kind === "control") {
      assert.deepEqual(goal.tiles, ["7,3", "7,5"]);
      for (const id of goal.tiles) required(game.tile(id)).owner = "player";
    }
  }
  game.beginTurn("player");
  assert.equal(game.winner, null);
  const scout = game.addUnit("wolf", "player", "6,0");
  game.move(scout.id, "6,1");
  assert(game.campaign!.marks.includes("event:lost-tools"));
  assert.equal(game.winner, "player");
  const vigil = game.nextCampaignMission([]);
  const dain = required(vigil.units.find((unit) => unit.name === "Captain Dain"));
  assert.equal(dain.side, "player");
  assert(vigil.units.some((unit) => unit.side === "enemy" && unit.name === "The Overseer"));
  saveBattle(vigil);
  assert.deepEqual(loadBattle().game.snapshot(), vigil.snapshot());
}

// Earlier vigil saves gain the living truce participant, but not a dead Dain.
{
  const vigil = Game.startCampaign("green-accord", 1);
  const dain = required(vigil.units.find((unit) => unit.name === "Captain Dain"));
  assert(returnsWithStory(vigil.campaign!, dain));
  vigil.winner = "player";
  assert.throws(() => vigil.nextCampaignMission([dain.id]), /Only surviving recruits/);
  const ordinary = vigil.addUnit("warden", "player", "1,1");
  assert(!returnsWithStory(vigil.campaign!, ordinary));
  const next = vigil.nextCampaignMission([ordinary.id]);
  assert.equal(next.units.filter((unit) => unit.name === "Captain Dain").length, 1);
}

// Earlier vigil saves gain the living truce participant, but not a dead Dain.
for (const dead of [false, true]) {
  const game = Game.startCampaign("green-accord", 1, [], 0, dead ? ["green-dain-dead"] : []);
  game.mapRevision = 1;
  game.units = game.units.filter((unit) => unit.name !== "Captain Dain");
  saveBattle(game);
  const loaded = loadBattle().game;
  assert.equal(loaded.mapId, "green-vigil");
  assert.equal(loaded.mapRevision, 2);
  assert.equal(
    loaded.units.some((unit) => unit.name === "Captain Dain"),
    !dead,
  );
  assert.equal(loaded.campaign!.marks.includes("dain-fell"), false);
}

// Lark must survive both early chapters, including dying while attacking.
for (const mission of [0, 1]) {
  for (const retaliation of [false, true]) {
    const game = Game.startCampaign("green-accord", mission);
    game.advanceDialogue(true);
    const lark = companion(game);
    lark.tile = "3,3";
    lark.hp = 1;
    game.units = game.units.filter((unit) => unit.tile !== "4,3");
    const enemy = game.addUnit("warden", "enemy", "4,3");
    if (retaliation) game.attack(lark.id, enemy.id, "rootstaff");
    else {
      game.beginTurn("enemy");
      game.attack(enemy.id, lark.id, "spear");
    }
    assert.equal(game.winner, "enemy");
    assert.equal(game.campaign?.outcome, "Lark fell. The company cannot continue without them.");
    saveBattle(game);
    assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
    assert.equal(companion(game.retryCampaignMission()).name, "Lark");
  }
}

// Required companions continue as story characters, never generic recall duplicates.
{
  const game = Game.startCampaign("green-accord", 0);
  for (const goal of campaignMission(game.campaign!).goals) {
    if (goal.kind === "event") game.campaign!.marks.push(`event:${goal.event}`);
    if (goal.kind === "control") {
      for (const tile of goal.tiles) required(game.tile(tile)).owner = "player";
    }
  }
  game.beginTurn("player");
  assert.equal(game.winner, "player");
  assert.throws(() => game.nextCampaignMission([companion(game).id]), /surviving recruits/);
  const next = game.nextCampaignMission([]);
  assert.equal(next.units.filter((unit) => unit.name === "Lark").length, 1);
}

function atVigil(): Game {
  const game = Game.startCampaign("green-accord", 1);
  game.advanceDialogue(true);
  const lark = companion(game);
  const goal = campaignMission(game.campaign!).goals.find((goal) => goal.kind === "vigil");
  assert(goal?.kind === "vigil");
  const occupant = game.at(goal.tile);
  if (occupant && occupant.id !== lark.id) occupant.tile = "5,4";
  lark.tile = goal.tile;
  required(game.tile(goal.tile)).owner = "player";
  return game;
}

// A late capture or an empty grove cannot complete the vigil by reaching a round number.
{
  const empty = Game.startCampaign("green-accord", 1);
  empty.round = 20;
  empty.beginTurn("player");
  assert.equal(empty.winner, null);
  assert.equal(empty.campaign?.vigil?.heldTurns, 0);
  const game = atVigil();
  const goal = required(
    campaignMission(game.campaign!).goals.find((goal) => goal.kind === "vigil"),
  );
  for (let round = 2; round <= 4; round++) {
    game.round = round;
    game.beginTurn("player");
    game.beginTurn("player");
    createGameView(game);
    assert.equal(game.campaign?.vigil?.heldTurns, round - 1);
    assert.equal(goalProgress(game, goal).current, round - 1);
    assert.equal(game.winner, round === 4 ? "player" : null);
    saveBattle(game);
    assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
  }
}

// Leaving or losing the grove resets progress immediately; undo restores that same progress.
{
  const game = atVigil();
  game.round = 2;
  game.beginTurn("player");
  assert.equal(game.campaign?.vigil?.heldTurns, 1);
  const lark = companion(game);
  const undo = createMoveUndo();
  undo.move(game, lark.id, "5,3");
  assert.equal(game.campaign?.vigil?.heldTurns, 0);
  assert(undo.undo(game));
  assert.equal(game.campaign?.vigil?.heldTurns, 1);
  required(game.tile(companion(game).tile)).owner = "enemy";
  game.beginTurn("enemy");
  assert.equal(game.campaign?.vigil?.heldTurns, 0);
  required(game.tile(companion(game).tile)).owner = "player";
  game.round = 3;
  game.beginTurn("player");
  assert.equal(game.campaign?.vigil?.heldTurns, 1);
}

// Legacy saves start the new vigil at zero without resetting the saved company or old victory.
{
  const game = atVigil();
  game.round = 8;
  game.gold.player = 27;
  delete game.campaign!.vigil;
  game.campaign!.marks.push("held:6,4");
  saveBattle(game);
  const restored = loadBattle().game;
  assert.equal(restored.mapId, "green-vigil");
  assert.equal(restored.round, 8);
  assert.equal(restored.gold.player, 27);
  assert.deepEqual(restored.units, game.units);
  assert.deepEqual(restored.campaign?.vigil, { heldTurns: 0, lastCountedRound: 8 });
  restored.beginTurn("player");
  assert.equal(restored.winner, null);
  assert.equal(restored.campaign?.vigil?.heldTurns, 0);
  saveBattle(restored);
  const saved = JSON.parse(required(storage.getItem(SAVE_KEY)));
  saved.state.campaign.vigil.heldTurns = 4;
  storage.setItem(SAVE_KEY, JSON.stringify(saved));
  assert.equal(loadBattle().game.mapId, "mirror", "Out-of-range counters are rejected");
  game.winner = "player";
  game.campaign!.outcome = "Mission complete. The company can rest and regroup.";
  saveBattle(game);
  assert.equal(loadBattle().game.winner, "player", "An already won legacy stage stays won");
}

// Dain's death changes this debrief and future missions, but retry rewinds current-stage deaths.
for (const mission of [0, 1, 2]) {
  const game = Game.startCampaign("green-accord", mission);
  game.units = game.units.filter((unit) => unit.name !== "Captain Dain");
  game.round = 4;
  game.gold.player = 23;
  saveBattle(game);
  const restored = loadBattle().game;
  assert.equal(restored.mapId, game.mapId);
  assert.equal(restored.round, 4);
  assert.equal(restored.gold.player, 23);
  assert.deepEqual(restored.units, game.units);
  assert(restored.campaign?.marks.includes("dain-fell"));
  assert(
    !campaignMission(restored.campaign!).victory.some((line) => line.speaker.includes("Dain")),
  );
  assert(restored.retryCampaignMission().units.some((unit) => unit.name === "Captain Dain"));
  saveBattle(restored);
  assert.deepEqual(loadBattle().game.snapshot(), restored.snapshot());
}

// A legacy victory cannot resurrect Lark; surviving Lark keeps the already earned victory.
for (const mission of [0, 1, 2]) {
  const game = Game.startCampaign("green-accord", mission);
  game.winner = "player";
  game.campaign!.outcome = "Mission complete. The company can rest and regroup.";
  game.campaign!.completed = mission === 2;
  game.round = 9;
  game.gold.player = 31;
  saveBattle(game);
  assert.equal(loadBattle().game.winner, "player");
  game.units = game.units.filter((unit) => unit.name !== "Lark");
  saveBattle(game);
  const restored = loadBattle().game;
  assert.equal(restored.mapId, game.mapId);
  assert.equal(restored.winner, "enemy");
  assert.equal(restored.campaign?.completed, false);
  assert.equal(restored.campaign?.outcome, "Lark fell. The company cannot continue without them.");
  assert.equal(restored.round, 9);
  assert.equal(restored.gold.player, 31);
  assert.deepEqual(restored.units, game.units);
  assert.equal(
    restored.campaign?.dialogue[restored.campaign.dialogueIndex]?.text,
    restored.campaign?.outcome,
  );
  assert.throws(() => restored.nextCampaignMission([]), /Win this mission/);
  assert.equal(companion(restored.retryCampaignMission()).name, "Lark");
  saveBattle(restored);
  assert.deepEqual(loadBattle().game.snapshot(), restored.snapshot());
}

for (const retaliation of [false, true]) {
  const game = Game.startCampaign("green-accord", 0);
  game.advanceDialogue(true);
  const dain = required(game.units.find((unit) => unit.name === "Captain Dain"));
  dain.tile = "4,4";
  dain.hp = 1;
  game.units = game.units.filter((unit) => unit.tile !== "3,4");
  const wolf = game.addUnit("wolf", "player", "3,4");
  if (retaliation) {
    game.beginTurn("enemy");
    game.attack(dain.id, wolf.id, "sword");
  } else game.attack(wolf.id, dain.id, "fangs");
  assert(game.campaign?.marks.includes("dain-fell"));
  assert(!game.campaign?.choices?.includes("green-dain-dead"));
  assert(!campaignMission(game.campaign!).victory.some((line) => line.speaker.includes("Dain")));
  assert(game.retryCampaignMission().units.some((unit) => unit.name === "Captain Dain"));
  for (const goal of campaignMission(game.campaign!).goals) {
    if (goal.kind === "event") game.campaign!.marks.push(`event:${goal.event}`);
    if (goal.kind === "control") {
      for (const tile of goal.tiles) required(game.tile(tile)).owner = "player";
    }
  }
  game.beginTurn("player");
  assert.equal(game.winner, "player");
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
  const next = game.nextCampaignMission([]);
  assert(next.campaign?.choices?.includes("green-dain-dead"));
  assert(!campaignMission(next.campaign!).victory.some((line) => line.speaker.includes("Dain")));
  const finale = Game.startCampaign("green-accord", 2, [], 0, next.campaign!.choices);
  assert(!finale.units.some((unit) => unit.name === "Captain Dain"));
  assert(finale.units.some((unit) => unit.name === "Adept Tamsin"));
  assert(!finale.retryCampaignMission().units.some((unit) => unit.name === "Captain Dain"));
  saveBattle(finale);
  assert.deepEqual(loadBattle().game.snapshot(), finale.snapshot());
}

// An allied Dain who falls in the finale cannot speak its victory or ending.
{
  const game = Game.startCampaign("green-accord", 2);
  const dain = required(game.units.find((unit) => unit.name === "Captain Dain"));
  dain.hp = 1;
  game.units = game.units.filter((unit) => unit.tile !== "4,4");
  const attacker = game.addUnit("wolf", "enemy", "4,4");
  game.beginTurn("enemy");
  game.attack(attacker.id, dain.id, "fangs");
  assert(game.campaign?.marks.includes("dain-fell"));
  const lark = companion(game);
  for (const goal of campaignMission(game.campaign!).goals) {
    if (goal.kind === "escort") lark.tile = goal.tile;
    if (goal.kind === "control") {
      for (const tile of goal.tiles) required(game.tile(tile)).owner = "player";
    }
  }
  const dialogueStart = game.campaign!.dialogue.length;
  game.beginTurn("player");
  assert.equal(game.winner, "player");
  assert(
    !game.campaign!.dialogue.slice(dialogueStart).some((line) => line.speaker.includes("Dain")),
  );
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
}

console.log("PASS: Lark survival, vigil defense, legacy saves, Dain continuity, and retry");
