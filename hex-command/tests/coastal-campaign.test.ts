import { test } from "vitest";
import assert from "node:assert/strict";
import { CAMPAIGNS } from "../src/game/campaign-data";
import { goalProgress, goalTiles } from "../src/game/campaign-progress";
import { Game } from "./game-fixture";
import { required } from "./helpers";

const campaign = CAMPAIGNS["borrowed-tide"];

const hold = required(campaign.missions[1]!.goals.find((goal) => goal.kind === "hold"));
const rescueText = required(campaign.missions[1]!.events.find((event) => event.id === "rescue"))
  .dialogue[0]!.text;
const waitingText = required(
  campaign.missions[1]!.events.find((event) => event.id === "await-tide"),
).dialogue[0]!.text;
const reachText = required(
  campaign.missions[1]!.events.find((event) => event.id === "tide-arrived"),
).dialogue[0]!.text;
function advanceTo(game: Game, round: number) {
  while (game.round < round) {
    game.advanceDialogue(true);
    game.endTurn();
    game.endTurn();
  }
}
function reachWreck(game: Game) {
  game.advanceDialogue(true);
  const edda = required(game.units.find((unit) => unit.name === "Edda"));
  edda.tile = "4,4";
  game.move(edda.id, "5,4");
}

test("Early arrival waits for turn six, then starts three defended enemy turns", () => {
  const game = Game.startCampaign("borrowed-tide", 1);
  reachWreck(game);
  assert(game.campaign!.dialogue.some((line) => line.text === waitingText));
  assert(!game.campaign!.dialogue.some((line) => line.text === rescueText));
  advanceTo(game, 5);
  const waiting = goalProgress(game, hold);
  assert.equal(waiting.current, 0);
  assert.equal(waiting.done, false);
  assert.equal(waiting.label, "Wait for high tide on turn 6");
  assert.equal(game.winner, null);
  advanceTo(game, 6);
  assert.equal(
    goalProgress(game, hold).current,
    0,
    "Turn six starts rescue, it does not complete a defended turn",
  );
  assert(game.campaign!.dialogue.some((line) => line.text === rescueText));
  assert(!game.campaign!.dialogue.some((line) => line.text === reachText));
  advanceTo(game, 8);
  assert.equal(goalProgress(game, hold).current, 2);
  assert.equal(game.winner, null);
  advanceTo(game, 9);
  assert.equal(goalProgress(game, hold).done, true);
  assert.equal(game.winner, "player");
});

test.each([6, 8])(
  "Arrival on turn %i starts rescue immediately, after the reach-wreck prompt",
  (arrival) => {
    const game = Game.startCampaign("borrowed-tide", 1);
    advanceTo(game, arrival);
    assert(game.campaign!.dialogue.some((line) => line.text === reachText));
    assert.equal(goalProgress(game, hold).label, "Reach the wreck to start the rescue");
    assert(!game.campaign!.marks.includes("event:rescue"));
    reachWreck(game);
    assert(game.campaign!.marks.includes("event:rescue"));
    assert(game.campaign!.dialogue.some((line) => line.text === rescueText));
    assert(!game.campaign!.dialogue.some((line) => line.text === waitingText));
    assert.equal(goalProgress(game, hold).current, 0);
    advanceTo(game, arrival + 2);
    assert.equal(goalProgress(game, hold).current, 2);
    assert.equal(game.winner, null);
    advanceTo(game, arrival + 3);
    assert.equal(game.winner, "player");
    assert.equal(game.campaign!.dialogue.filter((line) => line.text === rescueText).length, 1);
  },
);

test("Losing the wreck before high tide postpones rescue until recapture", () => {
  const game = Game.startCampaign("borrowed-tide", 1);
  reachWreck(game);
  game.advanceDialogue(true);
  const edda = required(game.units.find((unit) => unit.name === "Edda"));
  // Move out to screen the landing, then allow an actual enemy recapture.
  advanceTo(game, 4);
  game.move(edda.id, "5,3");
  const raider = game.addUnit("swordsman", "enemy", "6,4");
  game.endTurn();
  game.move(raider.id, "5,4");
  game.endTurn();
  advanceTo(game, 6);
  assert(!game.campaign!.marks.includes("event:rescue"));
  assert(game.campaign!.dialogue.some((line) => line.text === reachText));
  assert.equal(goalProgress(game, hold).label, "Reach the wreck to start the rescue");
  raider.tile = "6,4";
  game.advanceDialogue(true);
  game.move(edda.id, "5,4");
  assert(game.campaign!.marks.includes("event:rescue"));
  assert.equal(goalProgress(game, hold).current, 0);
});

test("Merfolk hints describe landmarks instead of map coordinates", () => {
  for (const mission of campaign.missions)
    for (const hint of mission.hints) assert(!/\b\d+,\s*\d+\b/.test(hint), hint);
});
