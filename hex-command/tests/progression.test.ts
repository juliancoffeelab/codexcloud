import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { describeChanges } from "../src/controller/feedback";
import { Game, RACES, TERRAIN, TYPES } from "../src/game/engine";
import { CAMPAIGNS } from "../src/game/campaign-data";
import { goalProgress } from "../src/game/campaign-progress";
import { UNIT_TYPES, type UnitType } from "../src/game/types";
import { movementCost } from "../src/game/movement";
import { createGameView } from "../src/game/view";
import { saveBattle, loadBattle, SAVE_KEY } from "../src/game/save";
import { required, TestStorage } from "./helpers";

test("Unit enum and definitions cover the same roster", async () => {
  assert.deepEqual(Object.keys(TYPES).sort(), [...UNIT_TYPES].sort());
});
for (const type of UNIT_TYPES) {
  test(
    "Every unit belongs to an acyclic race-preserving progression ladder — " +
      JSON.stringify([type]),
    async () => {
      const definition = TYPES[type];
      assert.equal(
        definition.damage,
        definition.attacks[0].damage,
        `${type} primary damage cannot drift from its weapon`,
      );
      assert.equal(
        definition.range,
        Math.max(...definition.attacks.map((attack) => attack.range)),
        `${type} range summarizes all available weapons`,
      );
      assert(definition.family && TYPES[definition.family]);
      const family = TYPES[definition.family];
      assert((family.advancesTo ?? []).length, `${type} belongs to a progression ladder`);
      const seen = new Set<UnitType>();
      const visit = (next: UnitType) => {
        assert(!seen.has(next), `No cyclic advancement at ${next}`);
        seen.add(next);
        for (const advanced of TYPES[next].advancesTo ?? []) {
          assert(TYPES[advanced].level! > TYPES[next].level!);
          assert.equal(TYPES[advanced].race, definition.race);
          visit(advanced);
        }
      };
      visit(definition.family);
    },
  );
}
// Access precedence and additive costs are independent of numeric Infinity sentinels.
test("Terrain, race and unit access compose with additive costs", async () => {
  assert.equal(TERRAIN.water.cost, 1);
  assert.equal(TERRAIN.water.passable, false);
  assert.equal(movementCost(TERRAIN.water, "water", RACES.human, TYPES.archer), Infinity);
  assert.equal(movementCost(TERRAIN.water, "water", RACES.merfolk, TYPES.harpooner), 1);
  assert.equal(
    movementCost(TERRAIN.water, "water", RACES.merfolk, {
      terrainModifiers: { water: { passable: false } },
    }),
    Infinity,
  );
  assert.equal(
    movementCost(TERRAIN.water, "water", RACES.human, {
      terrainModifiers: { water: { passable: true, cost: 1 } },
    }),
    2,
  );
  assert.equal(
    movementCost(TERRAIN.forest, "forest", RACES.merfolk, {
      terrainModifiers: { forest: { cost: -1 } },
    }),
    2,
  );
});

test("Combat XP and promotion preserve wounds, identity and immutable publications", async () => {
  const game = new Game();
  game.units = [];
  const attacker = game.addUnit("swordsman", "player", "3,3");
  const defender = game.addUnit("archer", "enemy", "4,3");
  attacker.hp = 6;
  defender.hp = 3;
  game.attack(attacker.id, defender.id, "sword");
  assert.equal(attacker.experience, 6);
  assert.deepEqual(game.promotionOptions(attacker.id), []);
  attacker.experience = 10;
  attacker.moved = attacker.attacked = false;
  const before = createGameView(game);
  const beforePromotion = game.snapshot();
  assert.deepEqual(before.promotionOptions(attacker.id), ["soldier"]);
  game.advanceUnit(attacker.id, "soldier");
  const feedback = describeChanges(beforePromotion, game.snapshot());
  assert(feedback.advanced);
  assert.equal(feedback.attacked, false, "Promotion must not replay the prior combat cue");
  assert.equal(attacker.hp, 8, "Promotion preserves all four wounds");
  assert.equal(attacker.id, before.units[0]!.id);
  assert.equal(attacker.experience, 0);
  assert(attacker.moved && attacker.attacked);
  assert.equal(
    before.unit(attacker.id)?.type,
    "swordsman",
    "Published progression remains immutable",
  );
  assert(TYPES.soldier.attacks.some((attack) => attack.damageType === "impact"));
  assert.equal(TYPES.swordsman.attacks.length, 1);
  assert.throws(() => game.advanceUnit(attacker.id, "soldier"), /not available/);
});

// An advanced enemy scout retains its capture role, rather than abandoning villages.
for (const type of ["scout", "ranger"] as const) {
  test(
    "Promoted scouts retain their village capture role — " + JSON.stringify([type]),
    async () => {
      const patrol = new Game();
      patrol.units = [];
      for (const tile of patrol.tiles) {
        tile.terrain = "plains";
        tile.owner = null;
      }
      const village = required(patrol.tile("10,6"));
      village.terrain = "village";
      patrol.addUnit("swordsman", "player", "0,0");
      const runner = patrol.addUnit(type, "enemy", "6,5");
      patrol.beginTurn("enemy");
      patrol.enemyAct(runner.id);
      assert.equal(runner.tile, village.id);
      assert.equal(village.owner, "enemy");
    },
  );
}

test("Zero damage attacks cannot farm XP", async () => {
  const zero = new Game();
  zero.units = [];
  const scout = zero.addUnit("scout", "player", "3,3");
  const heavy = zero.addUnit("soldier", "enemy", "4,3");
  required(zero.tile(heavy.tile)).terrain = "hill";
  zero.attack(scout.id, heavy.id, "knife");
  assert.equal(scout.experience, 0, "Zero damage never farms XP");
});

// Race armor remains separate from inherited environmental damage.
test("Race weaknesses and movement remain separate from weapon armor", async () => {
  const terrainGame = new Game();
  terrainGame.units = [];
  const bow = terrainGame.addUnit("archer", "player", "3,3");
  const human = terrainGame.addUnit("swordsman", "enemy", "4,3");
  const swimmer = terrainGame.addUnit("harpooner", "enemy", "4,4");
  for (const terrain of ["forest", "grove", "hill"] as const) {
    required(terrainGame.tile(swimmer.tile)).terrain = terrain;
    assert.equal(terrainGame.damage(bow, swimmer, "bow"), 5);
    assert.equal(terrainGame.cost(swimmer, { terrain }), 3);
  }
  for (const terrain of ["ford", "marsh"] as const) {
    required(terrainGame.tile(human.tile)).terrain = terrain;
    required(terrainGame.tile(swimmer.tile)).terrain = terrain;
    assert.equal(terrainGame.damage(bow, human, "bow"), 5);
    assert.equal(terrainGame.damage(bow, swimmer, "bow"), terrain === "marsh" ? 3 : 4);
  }
  assert.equal(terrainGame.cost(swimmer, { terrain: "plains" }), 1);
});

test("Veteran and named companion progression carries forward and retry restores incoming XP", async () => {
  const campaign = Game.startCampaign("borrowed-tide");
  campaign.advanceDialogue(true);
  const leader = required(
    campaign.units.find((unit) => unit.role === "leader" && unit.side === "player"),
  );
  assert(campaign.recruitTiles(leader.id, "reefguard").some((tile) => tile.terrain === "water"));
  const recruit = required(
    campaign.units.find((unit) => unit.type === "reefguard" && unit.side === "player"),
  );
  recruit.experience = 10;
  campaign.advanceUnit(recruit.id, "tridentguard");
  leader.experience = 18;
  campaign.advanceUnit(leader.id, "tideseer");
  const pilot = required(campaign.units.find((unit) => unit.name === "Edda"));
  pilot.experience = 7;
  campaign.winner = "player";
  const next = campaign.nextCampaignMission([recruit.id]);
  assert.equal(
    next.units.find((unit) => unit.role === "leader" && unit.side === "player")?.type,
    "tideseer",
  );
  assert.equal(next.units.filter((unit) => unit.name === "Edda").length, 1);
  assert.equal(next.units.find((unit) => unit.name === "Edda")?.experience, 7);
  assert.equal(
    next.units.find((unit) => unit.type === "tridentguard" && unit.side === "player")?.experience,
    0,
  );
  required(next.units.find((unit) => unit.name === "Edda")).experience = 10;
  assert.equal(
    next.retryCampaignMission().units.find((unit) => unit.name === "Edda")?.experience,
    7,
    "Retry restores incoming XP",
  );
});

// Late arrival cannot borrow elapsed rounds as evacuation defense. Ownership,
// rather than an occupying unit, sustains progress; reload and repeated views do not add turns.
test("Evacuation defense counts owned completed turns and migrates legacy clocks", async () => {
  const evacuation = Game.startCampaign("borrowed-tide", 1);
  evacuation.advanceDialogue(true);
  evacuation.round = 20;
  const edda = required(evacuation.units.find((unit) => unit.name === "Edda"));
  edda.tile = "4,4";
  const hold = required(
    CAMPAIGNS["borrowed-tide"].missions[1]!.goals.find((goal) => goal.kind === "hold"),
  );
  evacuation.move(edda.id, "5,4");
  assert.equal(goalProgress(evacuation, hold).current, 0);
  evacuation.endTurn();
  evacuation.endTurn();
  assert.equal(evacuation.round, 21);
  assert.equal(evacuation.winner, null);
  assert.equal(goalProgress(evacuation, hold).current, 1);
  evacuation.move(edda.id, "5,3");
  evacuation.endTurn();
  evacuation.endTurn();
  assert.equal(goalProgress(evacuation, hold).current, 2);
  saveBattle(evacuation);
  const restored = loadBattle().game;
  assert.equal(restored.mapId, "tide-wreck");
  assert.deepEqual(restored.campaign!.holds, evacuation.campaign!.holds);
  restored.beginTurn("player");
  assert.equal(goalProgress(restored, hold).current, 2);
  restored.endTurn();
  restored.endTurn();
  assert.equal(restored.round, 23);
  assert.equal(restored.winner, "player");

  // An ongoing legacy save has no evidence of defended duration: retain its
  // battle and ledger, but start the newly explicit defense clock at zero.
  const legacy = JSON.parse(required(localStorage.getItem(SAVE_KEY)));
  delete legacy.state.campaign.holds;
  localStorage.setItem(SAVE_KEY, JSON.stringify(legacy));
  const migrated = loadBattle().game;
  assert.equal(migrated.mapId, "tide-wreck");
  assert.equal(migrated.campaign!.holds!["5,4"]!.heldTurns, 0);
  assert(migrated.campaign!.marks.includes("event:ledger"));
  legacy.state.campaign.holds = { "5,4": { heldTurns: 4, lastCountedRound: 22 } };
  localStorage.setItem(SAVE_KEY, JSON.stringify(legacy));
  assert.equal(loadBattle().game.mapId, "mirror");
});
test("Enemy capture immediately resets evacuation defense", async () => {
  const contested = Game.startCampaign("borrowed-tide", 1);
  contested.advanceDialogue(true);
  contested.round = 6;
  const edda = required(contested.units.find((unit) => unit.name === "Edda"));
  edda.tile = "4,4";
  contested.move(edda.id, "5,4");
  contested.endTurn();
  contested.endTurn();
  assert.equal(contested.campaign!.holds!["5,4"]!.heldTurns, 1);
  contested.move(edda.id, "5,3");
  const raider = contested.addUnit("swordsman", "enemy", "6,4");
  contested.endTurn();
  contested.move(raider.id, "5,4");
  assert.equal(contested.campaign!.holds!["5,4"]!.heldTurns, 0);
  contested.endTurn();
  assert.equal(contested.winner, null);
  saveBattle(contested);
  assert.equal(loadBattle().game.campaign!.holds!["5,4"]!.heldTurns, 0);
});
test("Legacy pre-tide defense does not count toward the new rescue sequence", () => {
  const game = Game.startCampaign("borrowed-tide", 1);
  game.advanceDialogue(true);
  const edda = required(game.units.find((unit) => unit.name === "Edda"));
  edda.tile = "4,4";
  game.move(edda.id, "5,4");
  saveBattle(game);
  const legacy = JSON.parse(required(localStorage.getItem(SAVE_KEY)));
  legacy.state.campaign.holds = { "5,4": { heldTurns: 2, lastCountedRound: 4 } };
  legacy.state.round = 4;
  localStorage.setItem(SAVE_KEY, JSON.stringify(legacy));
  const restored = loadBattle().game;
  assert.equal(restored.mapId, "tide-wreck");
  assert.equal(restored.campaign!.holds!["5,4"]!.heldTurns, 0);
  assert(!restored.campaign!.marks.includes("event:rescue"));
  restored.round = 6;
  restored.beginTurn("player");
  assert(restored.campaign!.marks.includes("event:rescue"));
  assert.equal(restored.campaign!.holds!["5,4"]!.heldTurns, 0);
  saveBattle(restored);
  const resumed = loadBattle().game;
  assert.equal(resumed.mapId, "tide-wreck");
  resumed.advanceDialogue(true);
  resumed.endTurn();
  resumed.endTurn();
  assert.equal(resumed.campaign!.holds!["5,4"]!.heldTurns, 1);
  assert.equal(
    resumed.campaign!.dialogue.filter(
      (line) =>
        line.text ===
        "The boat is here. Hold the wreck for three turns while we bring the workers out.",
    ).length,
    1,
  );
});
test("Progressed campaign units persist and invalid XP is rejected", async () => {
  const campaign = Game.startCampaign("borrowed-tide");
  campaign.advanceDialogue(true);
  const leader = required(
    campaign.units.find((unit) => unit.role === "leader" && unit.side === "player"),
  );

  const recruit = required(
    campaign.units.find((unit) => unit.type === "reefguard" && unit.side === "player"),
  );
  recruit.experience = 10;
  campaign.advanceUnit(recruit.id, "tridentguard");
  leader.experience = 18;
  campaign.advanceUnit(leader.id, "tideseer");
  const pilot = required(campaign.units.find((unit) => unit.name === "Edda"));
  pilot.experience = 7;
  campaign.winner = "player";
  const next = campaign.nextCampaignMission([recruit.id]);
  saveBattle(next);
  assert.equal(loadBattle().game.campaign?.id, "borrowed-tide");
  assert.deepEqual(loadBattle().game.units, next.units);
  const saved = JSON.parse(required(localStorage.getItem(SAVE_KEY)));
  saved.state.units[0].experience = -1;
  localStorage.setItem(SAVE_KEY, JSON.stringify(saved));
  assert.equal(loadBattle().game.mapId, "mirror");
});
test("Promoted skirmish leaders survive reload", async () => {
  const classic = new Game();
  const commander = required(classic.unit("u1"));
  commander.experience = 18;
  classic.advanceUnit(commander.id, "marshal");
  saveBattle(classic);
  assert.equal(
    loadBattle().game.unit("u1")?.type,
    "marshal",
    "Promoted leaders survive reload outside campaigns",
  );
});
// A promoted bow specialist still counts as the classic enemy's ranged family.
test("Promoted archers retain their recruitment family", async () => {
  const recruitment = new Game();
  recruitment.addUnit("marksman", "enemy", "8,2");
  recruitment.beginTurn("enemy");
  recruitment.gold.enemy = 100;
  const oldIds = new Set(recruitment.units.map((unit) => unit.id));
  recruitment.enemyRecruit();
  assert.equal(recruitment.units.find((unit) => !oldIds.has(unit.id))?.type, "swordsman");
});

// A dead optional named ally must stay dead across victory, reload and chapter retry.
test("Dead optional companions remain dead across chapters and reload", async () => {
  const rescue = Game.startCampaign("borrowed-tide", 1);
  rescue.advanceDialogue(true);
  const tomas = required(rescue.units.find((unit) => unit.name === "Tomas"));
  const shotTile = required(rescue.neighbors(tomas.tile).find((tile) => !rescue.at(tile.id)));
  const guard = rescue.addUnit("archer", "enemy", shotTile.id);
  tomas.hp = 1;
  rescue.beginTurn("enemy");
  rescue.attack(guard.id, tomas.id, "bow");
  rescue.endTurn();
  assert(!rescue.unit(tomas.id));
  assert(
    rescue.retryCampaignMission().units.some((unit) => unit.name === "Tomas"),
    "A failed-attempt casualty must not change the incoming checkpoint",
  );
  rescue.winner = "player";
  const gates = rescue.nextCampaignMission([]);
  assert(!gates.units.some((unit) => unit.name === "Tomas"));
  assert(!gates.campaign!.dialogue.some((line) => line.speaker === "Tomas"));
  saveBattle(gates);
  const loadedGates = loadBattle().game;
  assert.equal(loadedGates.campaign?.id, "borrowed-tide");
  assert.deepEqual(loadedGates.campaign?.fallenCompanions, ["Tomas"]);
  assert(!loadedGates.retryCampaignMission().units.some((unit) => unit.name === "Tomas"));
});
for (const [chapter, oldName, officerName] of [
  [0, "Captain Vale", "Postmaster Harl"],
  [1, "Factor Senn", "Foreman Cale"],
] as const) {
  test(
    "Legacy coastal officers migrate names without resetting progress — " +
      JSON.stringify([chapter, oldName, officerName]),
    async () => {
      const oldCoast = Game.startCampaign("borrowed-tide", chapter);
      const officer = required(
        oldCoast.units.find((unit) => unit.side === "enemy" && unit.role === "leader"),
      );
      officer.name = oldName;
      officer.hp = 5;
      officer.experience = 7;
      officer.moved = officer.attacked = true;
      oldCoast.campaign!.dialogue = [
        required(CAMPAIGNS["borrowed-tide"].missions[chapter]!.legacyDialogue?.[0]),
      ];
      saveBattle(oldCoast);
      const migrated = loadBattle().game;
      assert.equal(migrated.campaign?.id, "borrowed-tide");
      assert.deepEqual(migrated.unit(officer.id), { ...officer, name: officerName });
      assert.equal(migrated.campaign!.dialogue[0]?.speaker, officerName);
    },
  );
}

beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
});

afterEach(() => {
  vi.unstubAllGlobals();
});
