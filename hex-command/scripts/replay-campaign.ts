/** Verify an agent's recorded decisions against the current game, without choosing actions. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Game } from "../src/game/engine";
import { enemyActionIds } from "../src/game/enemy-turn";
import type { CampaignId, GameState, RecruitType, UnitType } from "../src/game/types";

const path = process.argv[2];
if (!path) throw new Error("Usage: replay-campaign.ts TRIAL.json");
// Trials are local artifacts written by play-campaign.ts, rather than imported battle saves.
const trial = JSON.parse(readFileSync(path, "utf8")) as {
  state: GameState;
  transcript: Record<string, unknown>[];
};
function string(value: unknown): string {
  assert.equal(typeof value, "string");
  return value as string;
}
function record(value: unknown): Record<string, unknown> {
  assert(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
let game = Game.startCampaign(trial.state.campaign!.id as CampaignId);
game.advanceDialogue(true);
let moves: GameState[] = [];
for (const entry of trial.transcript) {
  if (entry.action) {
    assert.equal(game.winner, null);
    assert.equal(game.turn, "player");
    assert.equal(game.round, entry.round);
    assert.equal(game.mapId, entry.mission);
    const action = record(entry.action);
    const id = string(action.unit);
    assert.equal(game.unit(id)?.side, "player");
    if (action.op === "move") {
      const before = game.captureState();
      game.move(id, string(action.tile));
      if (!game.winner) moves.push(before);
    } else if (action.op === "attack") {
      game.attack(id, string(action.target), action.weapon as string | undefined);
      moves = [];
    } else if (action.op === "advance") {
      game.advanceUnit(id, string(action.type) as UnitType);
      moves = [];
    } else if (action.op === "recruit") {
      game.recruit(id, string(action.type) as RecruitType, string(action.tile));
      moves = [];
    } else throw new Error("Unknown recorded action.");
  } else if (entry.enemyTurnAfter !== undefined) {
    assert.equal(game.winner, null);
    moves = [];
    game.endTurn();
    game.enemyRecruit();
    for (const id of enemyActionIds(game.units)) {
      game.enemyAct(id);
      if (game.winner) break;
    }
    if (!game.winner) game.endTurn();
    game.advanceDialogue(true);
    assert.deepEqual(game.snapshot(), entry.state, "Recorded enemy response must replay exactly");
  } else if (entry.undo) {
    assert.equal(game.winner, null);
    assert(moves.length);
    game.restoreState(moves.pop()!);
  } else if (entry.retry) {
    assert.equal(game.winner, "enemy");
    game = game.retryCampaignMission();
    moves = [];
  } else if (entry.next) {
    assert.equal(game.winner, "player");
    assert(Array.isArray(entry.recall));
    game = game.nextCampaignMission(entry.recall.map(string));
    moves = [];
  } else throw new Error("Unknown transcript entry.");
  game.advanceDialogue(true);
}
assert.deepEqual(
  game.captureState(),
  trial.state,
  "Final trial state must match recorded decisions",
);
console.log(
  JSON.stringify({
    verified: true,
    campaign: game.campaign!.id,
    completed: game.campaign!.completed,
    decisions: trial.transcript.length,
  }),
);
