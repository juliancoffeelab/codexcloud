import { test } from "vitest";
import assert from "node:assert/strict";
import { CAMPAIGNS } from "../src/game/campaign-data";
import { goalProgress } from "../src/game/campaign-progress";
import { Game } from "./game-fixture";
import { required } from "./helpers";

const campaign = CAMPAIGNS["borrowed-tide"];

const hold = required(campaign.missions[1]!.goals.find((goal) => goal.kind === "hold"));
test("Rescue objective distinguishes reaching, waiting and defending", () => {
  const game = Game.startCampaign("borrowed-tide", 1);
  game.round = 6;
  assert.equal(goalProgress(game, hold).label, "Reach the wreck to start the rescue");
  required(game.tile(hold.tile)).owner = "player";
  assert.equal(goalProgress(game, hold).label, "Wait for high tide on turn 6");
  game.campaign!.marks.push("event:rescue");
  game.campaign!.holds = { [hold.tile]: { heldTurns: hold.turns - 1, lastCountedRound: 6 } };
  assert.equal(goalProgress(game, hold).done, false);
  game.campaign!.holds[hold.tile]!.heldTurns = hold.turns;
  assert.equal(goalProgress(game, hold).done, true);
  required(game.tile(hold.tile)).owner = "enemy";
  assert.equal(goalProgress(game, hold).done, false);
});

test("Merfolk hints describe landmarks instead of map coordinates", () => {
  for (const mission of campaign.missions)
    for (const hint of mission.hints) assert(!/\b\d+,\s*\d+\b/.test(hint), hint);
});
