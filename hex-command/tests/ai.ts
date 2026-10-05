import assert from "node:assert/strict";
import { Game, TYPES, distance } from "../src/game/engine";
import { enemyActionIds } from "../src/game/enemy-turn";
import { playPolicyTurn } from "../src/game/ai/policies";
import { canPursue, objectivesFor } from "../src/game/ai/objectives";
import { createArena, runArena } from "../src/game/ai/simulation";
import type { AiAction } from "../src/game/ai/policies";

// Generic-side entry points preserve the actual live classic enemy decisions.
const live = new Game("mirror");
live.endTurn();
const testing = new Game("mirror");
testing.restoreState(live.captureState());
live.enemyRecruit();
for (const id of enemyActionIds(live.units)) live.enemyAct(id);
playPolicyTurn(testing, "classic", { seed: 11 });
assert.deepEqual(testing.captureState(), live.captureState());

const config = {
  seed: 17,
  army: "coastal" as const,
  landscape: "crossings" as const,
  player: "rush" as const,
  enemy: "mixed" as const,
  first: "player" as const,
  maxRounds: 1,
  recruitLimit: 1,
};
const arena = createArena(config);
assert.equal(arena.mapId, "river", "Crossing arenas must retain classic's live river routing");
for (const tile of arena.tiles) {
  const partner = arena.tile(`${8 - tile.col - (tile.row % 2)},${8 - tile.row}`)!;
  assert.ok(partner, "Every arena hex must have a rotational partner");
  assert.equal(tile.terrain, partner.terrain);
}
assert.equal(arena.gold.player, arena.gold.enemy);
assert.equal(arena.recruitmentOptions("player").join(), arena.recruitmentOptions("enemy").join());
const actions: AiAction[] = [];
playPolicyTurn(arena, "rush", { seed: 17, onAction: (action) => actions.push(action) });
const purchased = actions.filter((action) => action.kind === "recruit");
assert.equal(purchased.length, 1);
assert.ok(
  purchased.every((action) => TYPES[action.type].range === 1 && !TYPES[action.type].healing),
);

const battle = runArena(config);
assert.deepEqual(runArena(config), battle, "Seeded results and commands must reproduce exactly");
assert.equal(battle.phases.length, 2);
assert.equal(battle.reason, "cap");
assert.equal(battle.winner, null, "Unfinished games aren't fabricated draws or losses");

// Same planner, independently selected recruitment: no shared override leaks between seats.
const crossed = runArena({
  ...config,
  player: "classic",
  enemy: "classic",
  recruitmentBySide: { player: "rush", enemy: "mixed" },
});
const buys = crossed.phases.map((phase) =>
  phase.actions.find((action) => action.kind === "recruit"),
);
assert.equal(buys[0]?.type, "reefguard");
assert.equal(buys[1]?.type, "harpooner");
assert.deepEqual(runArena(crossed.config), crossed);

const campaign = Game.startCampaign("lantern-road");
campaign.advanceDialogue(true);
const escort = objectivesFor(campaign, "player").find((goal) => goal.kind === "escort")!;
assert.ok(escort.actor);
assert.equal(canPursue(campaign.unit(escort.actor)!, escort), true);
assert.equal(
  canPursue(
    campaign.units.find((unit) => unit.side === "player" && unit.id !== escort.actor)!,
    escort,
  ),
  false,
);

// A healer follows a wounded, stationary front line without standing ahead of it.
const formation = new Game("mirror");
const state = formation.captureState();
state.units = [];
state.gold = { player: 0, enemy: 0 };
state.tiles = state.tiles.map((tile) => ({ ...tile, terrain: "plains", owner: null }));
formation.restoreState(state);
const tank = formation.addUnit("reefguard", "player", "5,3");
tank.hp = 3;
tank.moved = tank.attacked = true;
const healer = formation.addUnit("wavemender", "player", "2,3");
const foe = formation.addUnit("swordsman", "enemy", "7,3");
playPolicyTurn(formation, "mixed", { seed: 3, recruitLimit: 0 });
assert.equal(distance(formation.tile(healer.tile)!, formation.tile(tank.tile)!), 1);
assert.ok(
  distance(formation.tile(healer.tile)!, formation.tile(foe.tile)!) >
    distance(formation.tile(tank.tile)!, formation.tile(foe.tile)!),
  "Healer belongs behind the tank",
);
formation.beginTurn("player");
assert.equal(tank.hp, 4, "The chosen formation must actually provide live-engine healing");

// Clear an allied blocker, then recompute the courier's formerly unavailable move.
const blocked = new Game("mirror");
const empty = blocked.captureState();
empty.units = [];
empty.gold = { player: 0, enemy: 0 };
empty.tiles = empty.tiles.map((tile) => ({ ...tile, terrain: "plains", owner: null }));
blocked.restoreState(empty);
const courier = blocked.addUnit("scout", "player", "4,3");
courier.role = "courier";
const blocker = blocked.addUnit("boar", "player", "5,3");
const sequence: AiAction[] = [];
playPolicyTurn(blocked, "mixed", {
  seed: 2,
  recruitLimit: 0,
  objectives: [{ id: "escort", tiles: ["5,3"], kind: "escort", actor: courier.id, weight: 24 }],
  onAction: (action) => sequence.push(action),
});
assert.equal(sequence[0]?.unit, blocker.id);
assert.equal(courier.tile, "5,3", "Courier must use the route after its blocker has moved");

// An escort wins immediately; danger on a nonexistent next phase mustn't prevent it.
const finish = Game.startCampaign("lantern-road");
finish.advanceDialogue(true);
const destination = objectivesFor(finish, "player").find((goal) => goal.kind === "escort")!;
const incoming = finish.captureState();
const actor = incoming.units.find((unit) => unit.id === destination.actor)!;
const leader = incoming.units.find((unit) => unit.side === "player" && unit.role === "leader")!;
incoming.units = [leader, actor];
incoming.gold = { player: 0, enemy: 0 };
incoming.tiles = incoming.tiles.map((tile) => ({ ...tile, terrain: "plains", owner: null }));
actor.tile = finish.neighbors(destination.tiles[0]!)[0]!.id;
actor.hp = 1;
finish.restoreState(incoming);
finish.addUnit("bonearcher", "enemy", finish.neighbors(destination.tiles[0]!)[1]!.id);
playPolicyTurn(finish, "mixed", { seed: 9, recruitLimit: 0 });
assert.equal(finish.winner, "player");
assert.ok(finish.unit(actor.id));

// Killing the opposing leader also wins immediately, with no next phase to fear.
const decisive = new Game("mirror");
decisive.restoreState(empty);
const fragile = decisive.addUnit("commander", "player", "4,3");
fragile.hp = 1;
decisive.addUnit("commander", "enemy", "5,3").hp = 1;
decisive.addUnit("bonearcher", "enemy", "7,3");
playPolicyTurn(decisive, "mixed", { seed: 4, recruitLimit: 0 });
assert.equal(decisive.winner, "player");
assert.ok(decisive.unit(fragile.id));

console.log(
  "PASS: classic compatibility, symmetric resources, legal melee recruitment, deterministic caps, courier objectives and healer formation",
);
