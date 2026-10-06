import { test } from "vitest";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Game, distance } from "../src/game/engine";
import { playPolicyTurn } from "../src/game/ai/policies";
import type { AiAction } from "../src/game/ai/policies";
import type { GameState } from "../src/game/types";

// Extracted start of player turn 30, seed 1 versus classic, from the full run.
// Only the state is retained; this regression does not run a campaign batch.
const recorded = JSON.parse(
  readFileSync(new URL("fixtures/woodland-stall.json", import.meta.url), "utf8"),
) as { seed: number; states: Record<"rush" | "mixed", GameState> };

function emptyGame() {
  const game = new Game("mirror");
  const state = game.captureState();
  state.units = [];
  state.gold = { player: 0, enemy: 0 };
  state.tiles = state.tiles.map((tile) => ({ ...tile, terrain: "plains", owner: null }));
  game.restoreState(state);
  return game;
}

for (const policy of ["rush", "mixed"] as const) {
  test(`Recorded woodland deadlock advances toward the camp defenders — ${policy}`, () => {
    const state = recorded.states[policy];
    const game = new Game(state.mapId, state.mapRevision);
    game.restoreState(state);
    const actions: AiAction[] = [];
    const camps = [game.tile("7,3")!, game.tile("7,5")!];
    const nearestCamp = (tile: string) =>
      Math.min(...camps.map((camp) => distance(game.tile(tile)!, camp)));
    const startingDistances = new Map(game.units.map((unit) => [unit.id, nearestCamp(unit.tile)]));
    playPolicyTurn(game, policy, {
      seed: recorded.seed,
      recruitLimit: 0,
      onAction: (action) => actions.push(action),
    });
    assert(
      actions.some((action) => action.kind === "move"),
      "Break the passive rear-line cycle",
    );
    const wolves = game.units.filter((unit) => unit.side === "player" && unit.type === "wolf");
    assert(wolves.length >= 2);
    for (const wolf of wolves) {
      // Cover and balance changes can alter the safe approach and attack timing.
      // Require substantial real progress, rather than an exact staging hex.
      assert(nearestCamp(wolf.tile) <= 2);
      assert(startingDistances.get(wolf.id)! - nearestCamp(wolf.tile) >= 2);
    }
    assert(game.units.some((unit) => unit.name === "Lark"));
    assert(game.units.some((unit) => unit.name === "Alder"));
  });

  test(`Healthy troops wait when no unfinished duty or tactical improvement exists — ${policy}`, () => {
    const game = emptyGame();
    const wolf = game.addUnit("wolf", "player", "4,3");
    game.tile("7,3")!.terrain = "fort";
    game.tile("7,3")!.owner = "player";
    const actions: AiAction[] = [];
    playPolicyTurn(game, policy, {
      seed: 1,
      recruitLimit: 0,
      objectives: [{ id: "camp", kind: "capture", tiles: ["7,3"], weight: 24 }],
      onAction: (action) => actions.push(action),
    });
    assert.deepEqual(actions, []);
    assert.equal(wolf.tile, "4,3");
  });

  test(`Captured sites do not distract troops from the remaining objective — ${policy}`, () => {
    const game = emptyGame();
    const wolf = game.addUnit("wolf", "player", "4,3");
    game.tile("5,3")!.terrain = "fort";
    game.tile("5,3")!.owner = "player";
    game.tile("1,3")!.terrain = "fort";
    game.tile("1,3")!.owner = "enemy";
    playPolicyTurn(game, policy, {
      seed: 1,
      recruitLimit: 0,
      objectives: [{ id: "camps", kind: "capture", tiles: ["5,3", "1,3"], weight: 24 }],
    });
    assert.equal(wolf.tile, "1,3");
    assert.equal(game.tile("1,3")!.owner, "player");
  });

  test(`One garrison holds the owned site while other troops advance — ${policy}`, () => {
    const game = emptyGame();
    game.tile("4,3")!.terrain = "fort";
    game.tile("4,3")!.owner = "player";
    game.tile("7,3")!.terrain = "fort";
    game.tile("7,3")!.owner = "enemy";
    const guard = game.addUnit("wolf", "player", "4,3");
    const attacker = game.addUnit("boar", "player", "3,3");
    playPolicyTurn(game, policy, {
      seed: 1,
      recruitLimit: 0,
      objectives: [
        { id: "hold", kind: "capture", tiles: ["4,3"], weight: 24, hold: true },
        { id: "advance", kind: "capture", tiles: ["7,3"], weight: 24 },
      ],
    });
    assert.equal(guard.tile, "4,3");
    assert(distance(game.tile(attacker.tile)!, game.tile("7,3")!) < 4);
  });

  test(`Assigned couriers pursue their endpoint rather than a closer shared capture — ${policy}`, () => {
    const game = emptyGame();
    const courier = game.addUnit("scout", "player", "4,3");
    courier.role = "courier";
    game.tile("5,3")!.terrain = "village";
    playPolicyTurn(game, policy, {
      seed: 1,
      recruitLimit: 0,
      objectives: [
        { id: "village", kind: "capture", tiles: ["5,3"], weight: 24 },
        { id: "escort", kind: "escort", actor: courier.id, tiles: ["7,3"], weight: 24 },
      ],
    });
    assert.equal(courier.tile, "7,3");
  });
}
