import { test } from "vitest";
import assert from "node:assert/strict";
import { Game, TYPES, distance } from "../src/game/engine";
import { playPolicyTurn, POLICY_IDS } from "../src/game/ai/policies";
import { arenaMatchups, campaignMatchups } from "../src/game/ai/matchups";
import { createArena, ARMY_IDS, LANDSCAPES } from "../src/game/ai/simulation";
import { DIPLOMACY_IDS, diplomacyAllowsAttack, withdrawalLanes } from "../src/game/ai/diplomacy";
import { updateLeaderRebuilding } from "../src/game/leader-restraint";
import { reservedTiles, objectivesFor } from "../src/game/ai/objectives";

const limits = { seed: 4, maxRounds: 30, recruitLimit: 1, policies: POLICY_IDS };
const schedule = arenaMatchups({ ...limits, armies: ARMY_IDS, landscapes: LANDSCAPES });
test("Arena schedule covers every race, policy, terrain, seat and initiative", async () => {
  assert.equal(schedule.length, 1152);
  assert.equal(new Set(schedule.map((config) => JSON.stringify(config))).size, 1152);
  for (const config of schedule) {
    assert.ok(
      schedule.some(
        (other) =>
          other.landscape === config.landscape &&
          other.armies.player === config.armies.enemy &&
          other.armies.enemy === config.armies.player &&
          other.player === config.enemy &&
          other.enemy === config.player &&
          other.first !== config.first,
      ),
    );
  }
});
test("Campaign schedule includes both diplomacy settings", async () => {
  const campaigns = campaignMatchups({
    ...limits,
    campaigns: ["ashen-oath"],
    diplomacy: DIPLOMACY_IDS,
  });
  assert.equal(campaigns.length, 18);
  assert.equal(campaigns.filter((config) => config.diplomacy === "restrained").length, 9);
});

test("Cross-race arenas keep separate recruit pools", async () => {
  const arena = createArena(
    schedule.find(
      (config) => config.armies.player === "human" && config.armies.enemy === "coastal",
    )!,
  );
  assert.ok(arena.recruitmentOptions("player").includes("swordsman"));
  assert.ok(!arena.recruitmentOptions("player").includes("reefguard"));
  assert.ok(arena.recruitmentOptions("enemy").includes("reefguard"));
  assert.ok(!arena.recruitmentOptions("enemy").includes("swordsman"));
});

// Per-command recording must include troops absent from the beginning-of-phase roster.
test("Casualty and combat recording includes freshly recruited troops", async () => {
  const arena = createArena(
    schedule.find(
      (config) => config.armies.player === "human" && config.armies.enemy === "coastal",
    )!,
  );
  const ownLeader = arena.units.find((unit) => unit.side === "player" && TYPES[unit.type].leader)!;
  const spawn = arena.recruitTiles(ownLeader.id, "swordsman")[0]!;
  const fresh = arena.recruit(ownLeader.id, "swordsman", spawn.id);
  fresh.hp = 1;
  const attackerTile = arena.neighbors(fresh.tile).find((tile) => !arena.at(tile.id))!;
  const recordingAttacker = arena.addUnit("bonearcher", "enemy", attackerTile.id);
  arena.beginTurn("enemy");
  arena.attack(recordingAttacker.id, fresh.id);
  assert.equal(arena.casualties.at(-1)!.id, fresh.id);
  assert.equal(arena.combats.at(-1)!.target, fresh.id);
  assert.ok(arena.combats.at(-1)!.damage >= 1);
});

function emptyGame() {
  const game = new Game("mirror");
  const state = game.captureState();
  state.units = [];
  state.gold = { player: 0, enemy: 0 };
  state.tiles = state.tiles.map((tile) => ({ ...tile, terrain: "plains", owner: null }));
  game.restoreState(state);
  return game;
}

// A living force at intermediate sizes must preserve its previous release/rebuild decision.
test("Leader release and rebuild use hysteresis", async () => {
  const leader = emptyGame().addUnit("commander", "player", "1,3");
  assert.equal(updateLeaderRebuilding(leader, 3), true);
  assert.equal(updateLeaderRebuilding(leader, 5), false);
  assert.equal(updateLeaderRebuilding(leader, 3), false);
  assert.equal(updateLeaderRebuilding(leader, 2), true);
  assert.equal(updateLeaderRebuilding(leader, 4), true);
  assert.equal(updateLeaderRebuilding(leader, 5), false);
});

for (const policy of POLICY_IDS) {
  test(
    "Every policy returns depleted leaders to their keep — " + JSON.stringify([policy]),
    async () => {
      const game = emptyGame();
      game.tile("1,3")!.terrain = "keep";
      game.gold.player = 100;
      game.tile("1,3")!.owner = "player";
      const commander = game.addUnit("commander", "player", "4,3");
      commander.rebuilding = false;
      game.addUnit("swordsman", "player", "5,4");
      game.addUnit("swordsman", "player", "5,2");
      playPolicyTurn(game, policy, {
        seed: 1,
        recruitLimit: 0,
        objectives: [{ id: "forward", tiles: ["7,3"], kind: "capture", weight: 24 }],
      });
      assert.equal(commander.rebuilding, true, `${policy} must notice losses`);
      assert.equal(commander.tile, "1,3", `${policy} must return to recruit`);
      game.beginTurn("player");
      playPolicyTurn(game, policy, {
        seed: 1,
        recruitLimit: 0,
        objectives: [{ id: "forward", tiles: ["7,3"], kind: "capture", weight: 24 }],
      });
      assert.equal(commander.tile, "1,3", `${policy} must remain while rebuilding`);
      const clone = new Game("mirror");
      clone.restoreState(game.captureState());
      assert.equal(clone.unit(commander.id)!.rebuilding, true);
    },
  );
}

// Reserved actor and family endpoints are cleared, even without enemies or other goals.
for (const policy of POLICY_IDS) {
  test(
    "Reserved actor and family goals clear unrelated blockers — " + JSON.stringify([policy]),
    async () => {
      for (const family of [false, true]) {
        const game = emptyGame();
        const actor = game.addUnit(family ? "adept" : "scout", "player", "4,3");
        if (!family) actor.role = "courier";
        const blocker = game.addUnit("boar", "player", "5,3");
        const goal = {
          id: "locked",
          kind: family ? ("seal" as const) : ("escort" as const),
          tiles: ["5,3"],
          weight: 24,
          ...(family ? { family: "adept" } : { actor: actor.id }),
        };
        assert.ok(reservedTiles(blocker, [goal]).has("5,3"));
        playPolicyTurn(game, policy, { seed: 2, recruitLimit: 0, objectives: [goal] });
        assert.notEqual(blocker.tile, "5,3", `${policy} must clear the endpoint`);
        assert.equal(actor.tile, "5,3", `${policy} must let the eligible actor arrive`);
      }
    },
  );
}

// Potential attackers cannot hide behind each other in the required-actor forecast.
function pressureFixture() {
  const pressure = emptyGame();
  const courier = pressure.addUnit("scout", "player", "4,3");
  courier.role = "courier";
  pressure.addUnit("swordsman", "enemy", "6,3");
  pressure.addUnit("archer", "enemy", "7,3");
  pressure.addUnit("archer", "enemy", "7,2");
  return { pressure, courier };
}
test("Danger forecasts include all potential attackers without mutating state", async () => {
  const { pressure, courier } = pressureFixture();
  assert.equal(
    pressure.forecastDanger(courier, courier.tile),
    11,
    "Three attackers contribute 3 + 4 + 4 damage",
  );
  const snapshot = pressure.captureState();
  pressure.forecastDanger(courier, "3,3");
  assert.deepEqual(pressure.captureState(), snapshot, "Forecasts must not mutate gameplay");
});
test("Danger forecasts account for attackers vacating a blocked corridor", async () => {
  const corridor = emptyGame();
  for (const tile of corridor.tiles)
    tile.terrain = ["4,3", "5,3", "6,3", "7,3", "6,2"].includes(tile.id) ? "plains" : "water";
  const exposed = corridor.addUnit("scout", "player", "4,3");
  exposed.role = "courier";
  corridor.addUnit("swordsman", "enemy", "6,3");
  const bow = corridor.addUnit("archer", "enemy", "7,3");
  const source = new Game("mirror");
  source.restoreState(corridor.captureState());
  source.endTurn();
  assert.ok(
    ![bow.tile, ...source.reachable(bow.id).keys()].some((tile) =>
      source.canAttackFrom(source.unit(bow.id), tile, source.unit(exposed.id)),
    ),
  );
  assert.equal(
    corridor.forecastDanger(exposed, exposed.tile),
    7,
    "The sword can vacate the corridor, enabling the archer's later attack",
  );
});
for (const policy of POLICY_IDS) {
  test(
    "Required actors withdraw from danger under every policy — " + JSON.stringify([policy]),
    async () => {
      const { pressure, courier } = pressureFixture();
      const snapshot = pressure.captureState();
      const game = emptyGame();
      game.restoreState(snapshot);
      const actor = game.unit(courier.id)!;
      actor.hp = 4;
      const initialRisk = game.forecastDanger(actor, actor.tile);
      playPolicyTurn(game, policy, {
        seed: 1,
        recruitLimit: 0,
        objectives: [{ id: "escort", kind: "escort", actor: actor.id, tiles: ["7,4"], weight: 24 }],
      });
      assert.ok(
        game.forecastDanger(actor, actor.tile) < initialRisk,
        `${policy} must withdraw from danger`,
      );
    },
  );
}

// Mediation rules are a selectable behavior setting, lifted by the actual story transition.
test("Selectable mediation forbids lethal or retreating attacks until escalation", async () => {
  const refuge = Game.startCampaign("ashen-oath");
  refuge.campaign!.mission = 1;
  const attacker = refuge.units.find((unit) => unit.side === "player")!;
  const defender = refuge.units.find((unit) => unit.side === "enemy")!;
  attacker.tile = "6,3";
  defender.tile = "7,3";
  defender.role = "leader";
  defender.hp = 1;
  const weapon = TYPES[attacker.type].attacks[0]!.id;
  assert.equal(
    diplomacyAllowsAttack(refuge, attacker, defender, weapon, attacker.tile, "pragmatic"),
    true,
  );
  assert.equal(
    diplomacyAllowsAttack(refuge, attacker, defender, weapon, attacker.tile, "restrained"),
    false,
  );
  defender.retreating = true;
  defender.hp = TYPES[defender.type].hp;
  assert.equal(
    diplomacyAllowsAttack(refuge, attacker, defender, weapon, attacker.tile, "restrained"),
    false,
  );
  assert.ok(withdrawalLanes(refuge, "player", "restrained").size > 0);
  refuge.campaign!.refuge = { heldTurns: 0, lastCountedRound: 1, escalated: true };
  assert.equal(
    diplomacyAllowsAttack(refuge, attacker, defender, weapon, attacker.tile, "restrained"),
    true,
  );
  assert.equal(withdrawalLanes(refuge, "player", "restrained").size, 0);
});

// Preserve authored enemy posts with every planner.
for (const policy of POLICY_IDS) {
  test("Every policy respects authored enemy posts — " + JSON.stringify([policy]), async () => {
    const game = Game.startCampaign("ashen-oath");
    game.advanceDialogue(true);
    game.beginTurn("enemy");
    const commander = game.units.find((unit) => unit.side === "enemy" && TYPES[unit.type].leader)!;
    const post = objectivesFor(game, "enemy").find((goal) => goal.id === "authored-post")!
      .tiles[0]!;
    playPolicyTurn(game, policy, { seed: 1, recruitLimit: 0 });
    assert.ok(
      distance(game.tile(commander.tile)!, game.tile(post)!) <= 1,
      `${policy} must respect the authored post`,
    );
  });
}
