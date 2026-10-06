import { test } from "vitest";
import assert from "node:assert/strict";
import { Game, TYPES, distance } from "../src/game/engine";
import { enemyActionIds } from "../src/game/enemy-turn";
import { playPolicyTurn } from "../src/game/ai/policies";
import { canPursue, objectivesFor } from "../src/game/ai/objectives";
import { advanceArenaTerritoryHold, createArena, LANDSCAPES } from "../src/game/ai/simulation";
import { enemyTileOrder } from "../src/game/enemy-order";
import { leaderMayAdvance } from "../src/game/leader-restraint";
import type { AiAction } from "../src/game/ai/policies";

// Generic-side entry points preserve the actual live classic enemy decisions.
test("Generic classic policy preserves live enemy decisions", async () => {
  const live = new Game("mirror");
  live.units = live.units.filter((unit) => !TYPES[unit.type].leader);
  live.endTurn();
  const testing = new Game("mirror");
  testing.restoreState(live.captureState());
  live.enemyRecruit();
  for (const id of enemyActionIds(live.units)) live.enemyAct(id);
  playPolicyTurn(testing, "classic", { seed: 11 });
  assert.deepEqual(testing.captureState(), live.captureState());
});

const config = {
  seed: 17,
  armies: { player: "coastal" as const, enemy: "coastal" as const },
  landscape: "crossings" as const,
  player: "rush" as const,
  enemy: "mixed" as const,
  first: "player" as const,
  maxRounds: 1,
  recruitLimit: 1,
};
for (const landscape of LANDSCAPES)
  for (const owner of ["player", "enemy"] as const) {
    test(`Territory hold counts opposing turns and resets on loss — ${landscape}/${owner}`, () => {
      const arena = createArena({ ...config, landscape });
      const held = { player: 0, enemy: 0 };
      const opponent = owner === "player" ? "enemy" : "player";
      const objectives = ["4,4", "3,2", "5,6"];
      for (const tile of objectives) arena.tile(tile)!.owner = owner;

      // Capturing and then taking another own turn cannot advance the clock.
      assert.equal(advanceArenaTerritoryHold(arena, held, owner, landscape), null);
      assert.equal(held[owner], 0);
      for (let turn = 0; turn < 2; turn++)
        assert.equal(advanceArenaTerritoryHold(arena, held, opponent, landscape), null);
      assert.equal(held[owner], 2);

      // Losing just one village breaks continuity, even during the owner's turn.
      arena.tile("3,2")!.owner = opponent;
      assert.equal(advanceArenaTerritoryHold(arena, held, owner, landscape), null);
      assert.equal(held[owner], 0);
      arena.tile("3,2")!.owner = owner;
      const required = landscape === "islands" ? 6 : 4;
      for (let turn = 1; turn <= required; turn++) {
        assert.equal(advanceArenaTerritoryHold(arena, held, owner, landscape), null);
        assert.equal(held[owner], turn - 1);
        assert.equal(
          advanceArenaTerritoryHold(arena, held, opponent, landscape),
          turn === required ? owner : null,
        );
        assert.equal(held[owner], turn);
        assert.equal(held[opponent], 0);
      }
    });
  }
test("Arena terrain, resources and recruit pools are rotationally symmetric", async () => {
  const arena = createArena(config);
  assert.equal(arena.mapId, "river", "Crossing arenas must retain classic's live river routing");
  for (const tile of arena.tiles) {
    const partner = arena.tile(`${8 - tile.col - (tile.row % 2)},${8 - tile.row}`)!;
    assert.ok(partner, "Every arena hex must have a rotational partner");
    assert.equal(tile.terrain, partner.terrain);
  }
  assert.equal(arena.gold.player, arena.gold.enemy);
  assert.equal(arena.recruitmentOptions("player").join(), arena.recruitmentOptions("enemy").join());
});
test.each([1, 17, 41])(
  "Island objectives are separate, symmetric and reachable (seed %i)",
  (seed) => {
    const game = createArena({ ...config, seed, landscape: "islands" });
    assert.equal(game.mapId, "river", "Classic must use routes around the channels");
    assert.deepEqual(
      game.captureState(),
      createArena({ ...config, seed, landscape: "islands" }).captureState(),
    );
    for (const tile of game.tiles) {
      const partner = game.tile(rotate(tile.id))!;
      assert.equal(tile.terrain, partner.terrain);
      assert.equal(tile.owner, partner.owner ? opposite(partner.owner) : null);
    }

    // Remove the crossing hexes: the three objectives must be on genuinely
    // separate islands, rather than on a continuous strip through the water.
    const islands = new Map<string, number>();
    for (const tile of game.tiles) {
      if (["water", "bridge", "ford"].includes(tile.terrain) || islands.has(tile.id)) continue;
      const island = islands.size;
      const queue = [tile];
      islands.set(tile.id, island);
      for (const current of queue) {
        for (const next of game.neighbors(current.id)) {
          if (["water", "bridge", "ford"].includes(next.terrain) || islands.has(next.id)) continue;
          islands.set(next.id, island);
          queue.push(next);
        }
      }
    }
    assert.equal(new Set(islands.values()).size, 5, "Two home islands and three objective islands");
    const goals = ["4,4", "3,2", "5,6"].map((id) => game.tile(id)!);
    const objectiveIslands = goals.map((goal) => islands.get(goal.id));
    assert(objectiveIslands.every((island) => island !== undefined));
    assert.equal(new Set(objectiveIslands).size, 3);
    assert(game.tiles.filter((tile) => tile.terrain === "water").length >= 20);
    assert.equal(game.tiles.filter((tile) => tile.terrain === "bridge").length, 2);
    assert.equal(game.tiles.filter((tile) => tile.terrain === "ford").length, 12);
    for (const side of ["player", "enemy"] as const) {
      game.beginTurn(side);
      const leader = game.units.find((unit) => unit.side === side && TYPES[unit.type].leader)!;
      const homeIsland = islands.get(leader.tile);
      assert(homeIsland !== undefined);
      assert(!objectiveIslands.includes(homeIsland), "Objectives are offshore from both keeps");
      for (const type of ["swordsman", "boar", "skeleton", "reefguard"] as const) {
        for (const goal of goals) {
          assert(
            game.routeDistances({ ...leader, type }, [goal]).has(leader.tile),
            `${side} ${type} can reach ${goal.id}`,
          );
        }
      }
      const nearVillage = goals[side === "player" ? 1 : 2]!;
      assert(
        game.routeDistances({ ...leader, type: "reefguard" }, [nearVillage]).get(leader.tile)! <
          game.routeDistances({ ...leader, type: "swordsman" }, [nearVillage]).get(leader.tile)!,
        "Merfolk can bypass ground-army crossing routes",
      );
      assert(game.recruitTiles(leader.id, "reefguard").some((tile) => tile.terrain === "water"));
    }
  },
);
test("Rush purchases legal melee troops", async () => {
  const arena = createArena(config);
  const actions: AiAction[] = [];
  playPolicyTurn(arena, "rush", { seed: 17, onAction: (action) => actions.push(action) });
  const purchased = actions.filter((action) => action.kind === "recruit");
  assert.equal(purchased.length, 1);
  assert.ok(
    purchased.every((action) => TYPES[action.type].range === 1 && !TYPES[action.type].healing),
  );
});

// Test initiative clock without playing a match.
test("Reversed initiative advances rounds once per pair of turns", async () => {
  const reverseInitiative = createArena({ ...config, first: "enemy" });
  const rounds: number[] = [];
  for (let phase = 0; phase < 4; phase++) {
    rounds.push(reverseInitiative.round);
    reverseInitiative.endTurn();
  }
  assert.deepEqual(rounds, [1, 1, 2, 2]);
});

const rotate = (id: string) => {
  const [col, row] = id.split(",").map(Number) as [number, number];
  return `${8 - col - (row % 2)},${8 - row}`;
};
const opposite = (side: "player" | "enemy") => (side === "player" ? "enemy" : "player");
function rotateState(state: ReturnType<Game["captureState"]>) {
  const rotated = structuredClone(state);
  rotated.turn = opposite(state.turn);
  rotated.gold = { player: state.gold.enemy, enemy: state.gold.player };
  rotated.tiles = state.tiles.map((tile) => {
    const partner = state.tiles.find((other) => other.id === rotate(tile.id))!;
    return {
      ...tile,
      terrain: partner.terrain,
      owner: partner.owner ? opposite(partner.owner) : null,
    };
  });
  rotated.units = state.units.map((unit) => ({
    ...unit,
    tile: rotate(unit.tile),
    side: opposite(unit.side),
  }));
  return rotated;
}
function assertRotatedCommands(state: ReturnType<Game["captureState"]>) {
  const original = createArena(config);
  original.restoreState(state);
  const rotated = createArena(config);
  rotated.restoreState(rotateState(state));
  const objectives = [
    { id: "territory", kind: "capture" as const, tiles: ["4,4", "3,2", "5,6"], weight: 24 },
  ];
  playPolicyTurn(original, "classic", { seed: 1, objectives });
  playPolicyTurn(rotated, "classic", {
    seed: 1,
    objectives: objectives.map((goal) => ({ ...goal, tiles: goal.tiles.map(rotate) })),
  });
  assert.deepEqual(
    rotated.commands,
    original.commands.map((command) =>
      "tile" in command ? { ...command, tile: rotate(command.tile) } : command,
    ),
    "Classic decisions must rotate with the battlefield, including equal movement/attack scores",
  );
}
for (const army of ["human", "woodland", "undead", "coastal"] as const)
  for (const landscape of LANDSCAPES) {
    test(
      "Classic commands rotate across every race and terrain — " +
        JSON.stringify([army, landscape]),
      async () => {
        const sample = createArena({
          ...config,
          armies: { player: army, enemy: army },
          landscape,
        }).captureState();
        sample.round = 5;
        assertRotatedCommands(sample);
      },
    );
  }

// A wounded troop's retreat follows the same rule when its home changes sides.
test("Wounded troop retreat rotates with its home side", async () => {
  const retreatState = createArena(config).captureState();
  retreatState.round = 5;
  retreatState.gold = { player: 0, enemy: 0 };
  retreatState.tiles = retreatState.tiles.map((tile) => ({
    ...tile,
    terrain: tile.terrain === "keep" ? "keep" : "plains",
  }));
  retreatState.units = [
    {
      id: "u1",
      type: "swordsman",
      side: "player",
      tile: "4,4",
      hp: 1,
      moved: false,
      attacked: false,
    },
    {
      id: "u2",
      type: "swordsman",
      side: "enemy",
      tile: "5,4",
      hp: 10,
      moved: false,
      attacked: false,
    },
  ];
  assertRotatedCommands(retreatState);
});

test("Equal attack scores rotate with the battlefield", async () => {
  const retreatState = createArena(config).captureState();
  retreatState.round = 5;
  retreatState.gold = { player: 0, enemy: 0 };
  retreatState.tiles = retreatState.tiles.map((tile) => ({
    ...tile,
    terrain: tile.terrain === "keep" ? "keep" : "plains",
  }));
  retreatState.units = [
    {
      id: "u1",
      type: "swordsman",
      side: "player",
      tile: "4,4",
      hp: 1,
      moved: false,
      attacked: false,
    },
    {
      id: "u2",
      type: "swordsman",
      side: "enemy",
      tile: "5,4",
      hp: 10,
      moved: false,
      attacked: false,
    },
  ];
  const attackState = structuredClone(retreatState);
  attackState.units[0]!.hp = 10;
  attackState.units.push({
    id: "u3",
    type: "swordsman",
    side: "enemy",
    tile: "3,4",
    hp: 10,
    moved: false,
    attacked: false,
  });
  assertRotatedCommands(attackState);
});

test("Equal village choices favor the home keep", async () => {
  const homeArena = createArena(config);
  const homeLeader = homeArena.units.find(
    (unit) => unit.side === "player" && TYPES[unit.type].leader,
  )!;
  const villageOrder = enemyTileOrder(homeArena.tiles, homeArena.units, homeLeader, true);
  assert.ok(
    villageOrder(homeArena.tile("1,3")!, homeArena.tile("3,2")!) < 0,
    "Equal village choices favor the home keep",
  );
});

// Leader restraint depends on surviving, nearby troops, not elapsed turns.
function leaderFixture() {
  const leaderState = createArena({
    ...config,
    armies: { player: "human", enemy: "human" },
    landscape: "open",
  }).captureState();
  leaderState.units = leaderState.units.filter((unit) => TYPES[unit.type].leader);
  leaderState.gold = { player: 0, enemy: 0 };
  const ownLeader = leaderState.units.find((unit) => unit.side === "player")!;
  const opposingLeader = leaderState.units.find((unit) => unit.side === "enemy")!;
  return { leaderState, ownLeader, opposingLeader };
}
for (const policy of ["classic", "rush", "mixed"] as const)
  for (const round of [1, 99]) {
    test(
      "Unescorted leaders stay restrained regardless of elapsed turns — " +
        JSON.stringify([policy, round]),
      async () => {
        const { leaderState, ownLeader, opposingLeader } = leaderFixture();
        const game = createArena(config);
        game.restoreState({ ...leaderState, round });
        playPolicyTurn(game, policy, {
          seed: 1,
          recruitLimit: 0,
          objectives: [{ id: "fort", kind: "capture", tiles: ["4,4"], weight: 24 }],
        });
        assert.ok(
          distance(game.tile(game.unit(ownLeader.id)!.tile)!, game.tile(opposingLeader.tile)!) >=
            distance(game.tile(ownLeader.tile)!, game.tile(opposingLeader.tile)!),
          `${policy} must not send an unescorted leader forward on turn ${round}`,
        );
      },
    );
  }
test("Leader release accounts for escorts, health, required movement and rebuilding", async () => {
  const { leaderState, ownLeader } = leaderFixture();
  const homeArena = createArena(config);
  const leadContext = {
    units: leaderState.units,
    definitions: TYPES,
    distance: (first: string, second: string) =>
      distance(homeArena.tile(first)!, homeArena.tile(second)!),
    recruiting: false,
    requiredMovement: false,
  };
  assert.equal(leaderMayAdvance(ownLeader, "4,4", leadContext), false);
  assert.equal(leaderMayAdvance(ownLeader, "0,4", leadContext), true);
  assert.equal(
    leaderMayAdvance(ownLeader, "4,4", { ...leadContext, requiredMovement: true }),
    true,
  );
  const escorts: typeof leaderState.units = [
    {
      id: "guard1",
      type: "swordsman",
      side: "player",
      tile: "5,4",
      hp: 10,
      moved: false,
      attacked: false,
    },
    {
      id: "guard2",
      type: "swordsman",
      side: "player",
      tile: "4,5",
      hp: 10,
      moved: false,
      attacked: false,
    },
  ];
  assert.equal(
    leaderMayAdvance(ownLeader, "4,4", {
      ...leadContext,
      units: [...leadContext.units, ...escorts],
    }),
    true,
  );
  assert.equal(
    leaderMayAdvance(ownLeader, "4,4", {
      ...leadContext,
      units: [...leadContext.units, ...escorts.map((unit) => ({ ...unit, hp: 1 }))],
    }),
    false,
  );
  assert.equal(
    leaderMayAdvance({ ...ownLeader, rebuilding: true }, "4,4", {
      ...leadContext,
      units: [...leadContext.units, ...escorts],
      recruiting: true,
    }),
    false,
  );
});

function escapeFixture() {
  const escaping = Game.startCampaign("ashen-oath");
  escaping.advanceDialogue(true);
  const escapingState = escaping.captureState();
  const escapeLeader = escapingState.units.find(
    (unit) => unit.role === "leader" && unit.side === "player",
  )!;
  escapeLeader.tile = "10,1";
  escapingState.units = [escapeLeader];
  escapingState.gold.player = 0;
  // Isolate the escape command from classic's unrelated combat/capture priorities.
  escapingState.tiles = escapingState.tiles.map((tile) => ({ ...tile, terrain: "plains" }));
  return { escapingState };
}
for (const policy of ["classic", "rush", "mixed"] as const) {
  test("Every policy allows a required leader escape — " + JSON.stringify([policy]), async () => {
    const { escapingState } = escapeFixture();
    const game = Game.startCampaign("ashen-oath");
    game.restoreState(escapingState);
    playPolicyTurn(game, policy, { seed: 1, recruitLimit: 0 });
    assert.equal(game.winner, "player", `${policy} must permit a required leader escape`);
  });
}

test("Escort objectives require their assigned actor", async () => {
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
});

// A healer follows a wounded, stationary front line without standing ahead of it.
test("Mixed healer follows and heals a wounded front line", async () => {
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
});

// Clear an allied blocker, then recompute the courier's formerly unavailable move.
test("Mixed army clears blockers then recomputes courier routes", async () => {
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
});

// An escort wins immediately; danger on a nonexistent next phase mustn't prevent it.
test("Terminal escort wins ignore nonexistent future danger", async () => {
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
});

// Killing the opposing leader also wins immediately, with no next phase to fear.
test("Terminal leader kills ignore nonexistent future danger", async () => {
  const blocked = new Game("mirror");
  const empty = blocked.captureState();
  empty.units = [];
  empty.gold = { player: 0, enemy: 0 };
  empty.tiles = empty.tiles.map((tile) => ({ ...tile, terrain: "plains", owner: null }));
  blocked.restoreState(empty);
  const decisive = new Game("mirror");
  decisive.restoreState(empty);
  const fragile = decisive.addUnit("commander", "player", "4,3");
  fragile.hp = 1;
  decisive.addUnit("commander", "enemy", "5,3").hp = 1;
  decisive.addUnit("bonearcher", "enemy", "7,3");
  playPolicyTurn(decisive, "mixed", { seed: 4, recruitLimit: 0 });
  assert.equal(decisive.winner, "player");
  assert.ok(decisive.unit(fragile.id));
});
