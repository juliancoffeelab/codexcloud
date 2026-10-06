import { test } from "vitest";
import assert from "node:assert/strict";
import { TYPES } from "../src/game/engine";
import { playPolicyTurn, type AiAction } from "../src/game/ai/policies";
import { createArena, type ArmyId } from "../src/game/ai/simulation";
import type { UnitType } from "../src/game/types";

function recruitmentScene(army: ArmyId, troops: UnitType[], gold = 20, income = true) {
  const game = createArena({
    seed: 1,
    armies: { player: army, enemy: "human" },
    landscape: "open",
    first: "player",
  });
  game.units = game.units.filter((unit) => TYPES[unit.type].leader);
  for (const tile of game.tiles) {
    if (tile.terrain !== "keep") tile.terrain = "plains";
    if (tile.terrain !== "keep") tile.owner = null;
  }
  if (income) {
    game.tile("2,2")!.terrain = "village";
    game.tile("2,2")!.owner = "player";
  }
  for (const [index, type] of troops.entries()) game.addUnit(type, "player", `${index + 2},3`);
  for (const unit of game.units) unit.moved = unit.attacked = true;
  game.gold.player = gold;
  return game;
}

function purchases(game: ReturnType<typeof recruitmentScene>) {
  const actions: AiAction[] = [];
  playPolicyTurn(game, "mixed", {
    seed: 1,
    objectives: [],
    onAction: (action) => actions.push(action),
  });
  return actions.filter((action) => action.kind === "recruit").map((action) => action.type);
}

test("Mixed replaces lost tanks even when several melee flankers survive", () => {
  const game = recruitmentScene("woodland", ["wolf", "wolf", "wolf", "thorncaller", "thorncaller"]);
  assert.deepEqual(purchases(game), ["boar"]);
});

test("Mixed preserves its income for a replacement tank, then buys it", () => {
  const game = recruitmentScene("woodland", ["wolf", "wolf", "thorncaller"], 5);
  assert.deepEqual(purchases(game), []);
  assert.equal(game.gold.player, 5);
  game.beginTurn("player");
  assert.equal(game.gold.player, 7);
  for (const unit of game.units) unit.moved = unit.attacked = true;
  assert.deepEqual(purchases(game), []);
  game.beginTurn("player");
  assert.equal(game.gold.player, 9);
  for (const unit of game.units) unit.moved = unit.attacked = true;
  assert.deepEqual(purchases(game), ["boar"]);
  assert.equal(game.gold.player, 0);
});

test("Mixed does not wait for an unaffordable tank when there is no income", () => {
  const game = recruitmentScene("woodland", ["wolf", "thorncaller"], 5, false);
  assert.deepEqual(purchases(game), ["wolf"]);
});

test("Human mixed recruitment includes flankers and ranged troops alongside infantry", () => {
  assert.deepEqual(
    purchases(recruitmentScene("human", ["swordsman", "spearman", "archer", "archer"])),
    ["scout"],
  );
  assert.deepEqual(purchases(recruitmentScene("human", ["swordsman", "spearman", "scout"])), [
    "archer",
  ]);
});

test("Mixed buys healing support when the race offers it", () => {
  const game = recruitmentScene("coastal", ["reefguard", "reefguard", "harpooner", "harpooner"]);
  assert.deepEqual(purchases(game), ["wavemender"]);
});

test("Mixed pools without healers or flankers still build ranged support", () => {
  const game = recruitmentScene("undead", ["skeleton", "skeleton"]);
  assert.deepEqual(purchases(game), ["bonearcher"]);
});
