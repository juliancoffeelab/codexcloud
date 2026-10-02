import { required } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game, TYPES } from "../src/game/engine.ts";

// Pathfinder affects forests and hills, while every water crossing keeps its cost.
for (const type of ["scout", "soldier", "archer", "commander"] as const) {
  const game = new Game();
  const unit =
    game.units.find((unit) => unit.type === type) ||
    game.addUnit(type, "player", "2,3");
  for (const terrain of ["forest", "hill"] as const) {
    assert.equal(game.cost(unit, { terrain }), type === "scout" ? 1 : 2);
  }
  assert.equal(game.cost(unit, { terrain: "water" }), Infinity);
  assert.equal(game.cost(unit, { terrain: "bridge" }), 1);
  assert.equal(
    game.cost(unit, { terrain: "ford" }),
    ["soldier", "commander"].includes(type) ? 3 : 2,
  );
}

// Actual route planning and movement must use the scout's hill discount.
{
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "water";
  required(game.tile("0,0")).terrain = "plains";
  for (let col = 1; col <= 5; col++)
    required(game.tile(`${col},0`)).terrain = "hill";
  const scout = game.addUnit("scout", "player", "0,0");
  assert.equal(game.reachable(scout.id).get("5,0"), 5);
  game.move(scout.id, "5,0");
  assert.equal(scout.tile, "5,0");
  assert.equal(scout.moved, true);
}

// Prices govern both affordability and the gold spent on a successful recruit.
for (const [type, price] of [
  ["scout", 5],
  ["archer", 8],
  ["soldier", 8],
] as const) {
  const game = new Game();
  const commander = required(
    game.units.find(
      (unit) => unit.side === "player" && unit.type === "commander",
    ),
  );
  assert.equal(TYPES[type].cost, price);
  game.gold.player = price;
  const destination = required(game.recruitTiles(commander.id, type)[0]).id;
  game.gold.player = price - 1;
  assert.deepEqual(game.recruitTiles(commander.id, type), []);
  assert.throws(
    () => game.recruit(commander.id, type, destination),
    /Cannot recruit/,
  );
  game.gold.player = price;
  const recruit = game.recruit(commander.id, type, destination);
  assert.equal(game.gold.player, 0);
  assert.equal(recruit.type, type);
  assert.equal(recruit.moved, true);
  assert.equal(recruit.attacked, true);
}
console.log(
  "PASS: scout hill/forest movement, water costs, route planning, and recruitment prices",
);
