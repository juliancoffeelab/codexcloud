import { test, beforeEach, afterEach } from "vitest";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Game } from "../src/game/engine";
import { enemyActionIds } from "../src/game/enemy-turn";

let directory: string;
let path: string;
beforeEach(() => {
  directory = mkdtempSync(join(tmpdir(), "hex-headless-"));
  path = join(directory, "trial.json");
});
afterEach(() => {
  rmSync(directory, { recursive: true, force: true });
});
function invoke(request: unknown) {
  return spawnSync(
    process.execPath,
    ["--import", "tsx", "scripts/play-campaign.ts", path, JSON.stringify(request)],
    { encoding: "utf8", timeout: 3000 },
  );
}
function call(request: unknown) {
  const result = invoke(request);
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as { error?: string; result: unknown };
}
const saved = () => JSON.parse(readFileSync(path, "utf8"));
function start() {
  call({ op: "start", campaign: "lantern-road" });
  const game = Game.startCampaign("lantern-road");
  game.advanceDialogue(true);
  return game;
}
test("Starting a trial publishes rules and dialogue and persists its state", () => {
  const started = call({ op: "start", campaign: "lantern-road" }).result as {
    conversation: unknown;
    rules: { terrain: { water: { cost: unknown; passable: boolean }; forest: { cost: unknown } } };
  };
  assert.equal(started.rules.terrain.water.cost, 1);
  assert.equal(started.rules.terrain.water.passable, false);
  assert.equal(started.rules.terrain.forest.cost, 2);
  assert.match(saved().rulesFingerprint, /^[a-f0-9]{64}$/);
  const game = Game.startCampaign("lantern-road");
  assert.deepEqual(started.conversation, game.campaign!.dialogue);
  game.advanceDialogue(true);
  assert.deepEqual(saved().state, game.captureState());
});
test("Inspections and duplicate start cannot modify an existing trial", () => {
  const game = start();
  const leader = game.units.find((unit) => unit.role === "leader" && unit.side === "player")!;
  const before = saved();
  call({ op: "options", unit: leader.id });
  call({ op: "threats", unit: leader.id, tiles: [leader.tile, "1,2"] });
  assert.deepEqual(saved(), before, "Mechanical inspections must not change the trial");
  const duplicate = invoke({ op: "start", campaign: "lantern-road" });
  assert.notEqual(duplicate.status, 0);
  assert.match(duplicate.stderr, /Trial already exists/);
  assert.deepEqual(saved(), before, "Starting again must not erase attempts or progress");
});
test("Successful actions persist before a later invalid action and recruitment clears undo", () => {
  const game = start();
  const leader = game.units.find((unit) => unit.role === "leader" && unit.side === "player")!;
  const destination = game.recruitTiles(leader.id, "swordsman")[0]!.id;
  const response = call({
    op: "act",
    actions: [
      { op: "recruit", unit: leader.id, type: "swordsman", tile: destination },
      { op: "move", unit: leader.id, tile: "99,99" },
    ],
  });
  assert(response.error, "Invalid actions must return an error and current observation");
  game.recruit(leader.id, "swordsman", destination);
  assert.deepEqual(saved().state, game.captureState(), "Prior successful actions persist");
  assert.equal(saved().moves.length, 0, "Recruitment clears move undo");
});
test("Enemy forecasts are pure and execute the same response as the live engine", () => {
  const game = start();
  const leader = game.units.find((unit) => unit.role === "leader" && unit.side === "player")!;
  const destination = game.recruitTiles(leader.id, "swordsman")[0]!.id;
  call({
    op: "act",
    actions: [{ op: "recruit", unit: leader.id, type: "swordsman", tile: destination }],
  });
  game.recruit(leader.id, "swordsman", destination);
  const before = saved();
  const forecast = call({ op: "forecast" }).result as { observation: unknown };
  assert.deepEqual(saved(), before, "Enemy forecasts must not advance the trial");
  const actual = call({ op: "act", actions: [], endTurn: true });
  assert.deepEqual(forecast.observation, actual.result, "Forecast executes the actual enemy turn");
  game.endTurn();
  game.enemyRecruit();
  for (const id of enemyActionIds(game.units)) {
    game.enemyAct(id);
    if (game.winner) break;
  }
  if (!game.winner) game.endTurn();
  game.advanceDialogue(true);
  assert.deepEqual(
    saved().state,
    game.captureState(),
    "Headless enemy response matches live rules",
  );
});
test("A changed rules fingerprint rejects continuation without modifying the trial", () => {
  start();
  const mismatched = saved();
  mismatched.rulesFingerprint = "changed";
  writeFileSync(path, JSON.stringify(mismatched));
  const rejected = invoke({ op: "observe" });
  assert.notEqual(rejected.status, 0);
  assert.match(rejected.stderr, /Campaign rules changed/);
  assert.deepEqual(saved(), mismatched, "A changed ruleset must not continue or modify a trial");
});
