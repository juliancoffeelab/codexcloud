import assert from "node:assert/strict";
import { Game, TYPES } from "../src/game/engine";
import { CAMPAIGNS } from "../src/game/campaign-data";
import {
  campaignMission,
  goalTiles,
  goalProgress,
  dialoguePending,
} from "../src/game/campaign-progress";
import { createGameView } from "../src/game/view";
import { createMoveUndo } from "../src/game/undo";
import { loadBattle, saveBattle, SAVE_KEY } from "../src/game/save";
import { describeChanges } from "../src/controller/feedback";
import { attackChoiceViews } from "../src/ui/combat-view";
import { BattleController, type BattleView } from "../src/controller/battle";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();
globalThis.localStorage = storage;

// Delayed deployment changes fresh encounters, not armies already saved with
// those troops present. Existing wave indices and repeated loads stay stable.
for (const [id, chapter, type, tile, newWave, oldMarks] of [
  ["lantern-road", 1, "bonearcher", "8,7", 1, ["wave:0"]],
  ["ashen-oath", 2, "boar", "9,4", 2, ["wave:0", "wave:1"]],
] as const) {
  const old = Game.startCampaign(id, chapter);
  old.advanceDialogue(true);
  old.mapRevision = 1;
  old.round = 8;
  old.campaign!.marks.push(...oldMarks);
  const original = old.addUnit(type, "enemy", tile);
  original.hp = 1;
  const survivors = structuredClone(old.units);
  const gold = { ...old.gold };
  saveBattle(old, true);
  const loaded = loadBattle();
  assert.equal(loaded.pendingEnemy, true);
  assert.equal(loaded.game.mapRevision, 2);
  assert.deepEqual(loaded.game.units, survivors);
  assert.deepEqual(loaded.game.gold, gold);
  for (const mark of [...oldMarks, `wave:${newWave}`])
    assert(loaded.game.campaign!.marks.includes(mark));
  loaded.game.beginTurn("enemy");
  assert.equal(loaded.game.units.length, survivors.length, "No duplicate delayed troop");
  loaded.game.endTurn();
  saveBattle(loaded.game);
  assert.deepEqual(loadBattle().game.snapshot(), loaded.game.snapshot());

  const fresh = Game.startCampaign(id, chapter);
  fresh.advanceDialogue(true);
  const before = fresh.units.filter((unit) => unit.side === "enemy" && unit.type === type).length;
  const wave = campaignMission(fresh.campaign!).waves[newWave]!;
  fresh.round = wave.round;
  saveBattle(fresh);
  const current = loadBattle().game;
  assert(!current.campaign!.marks.includes(`wave:${newWave}`));
  current.beginTurn("enemy");
  assert.equal(
    current.units.filter((unit) => unit.side === "enemy" && unit.type === type).length,
    before + 1,
    "Fresh battles still receive the disclosed wave",
  );
}

// The quarry's bookkeeper guards his office; regular troops still pursue escapees,
// and approaching the office still provokes the leader's ordinary attack.
{
  const game = Game.startCampaign("ashen-oath");
  game.advanceDialogue(true);
  const orin = required(game.units.find((unit) => unit.name === "Orin"));
  const keeper = required(game.units.find((unit) => unit.name === "The Ledgerkeeper"));
  const patrol = required(
    game.units.find((unit) => unit.side === "enemy" && unit.type === "swordsman"),
  );
  orin.tile = "6,4";
  keeper.tile = "9,4";
  game.round = 6;
  game.beginTurn("enemy");
  game.enemyAct(keeper.id);
  assert(game.neighbors("11,5").some((tile) => tile.id === keeper.tile) || keeper.tile === "11,5");
  assert.equal(orin.hp, TYPES.necromancer.hp, "The posted leader cannot chase across the quarry");
  game.enemyAct(patrol.id);
  assert.notEqual(patrol.tile, "9,5", "The regular road patrol still advances");
  game.round = 7;
  game.beginTurn("enemy");
  game.enemyAct(patrol.id);
  assert(orin.hp < TYPES.necromancer.hp, "A post does not protect escapees from mobile troops");

  const intrusion = Game.startCampaign("ashen-oath");
  intrusion.advanceDialogue(true);
  const defender = required(intrusion.units.find((unit) => unit.name === "The Ledgerkeeper"));
  const raider = required(intrusion.unit("u2"));
  raider.tile = "11,4";
  intrusion.round = 6;
  intrusion.beginTurn("enemy");
  intrusion.enemyAct(defender.id);
  assert.equal(raider.hp, TYPES.skeleton.hp - 4, "The office guard still attacks an adjacent raid");
}

// Visit sites belong on unoccupied map hexes, not under the opening company.
for (const campaign of Object.values(CAMPAIGNS)) {
  for (const mission of campaign.missions) {
    for (const event of mission.events) {
      if (event.trigger.kind !== "visit") continue;
      const target = event.trigger.tile;
      assert(
        !mission.map.units.some((unit) => `${unit.col},${unit.row}` === target),
        `${mission.map.id}: ${event.id} starts occupied`,
      );
    }
  }
}

// The gatherers are reachable on the northern bridge approach; old armies keep progress.
{
  const game = Game.startCampaign("lantern-road", 2);
  const adept = required(game.at("4,3"));
  const reward = required(
    campaignMission(game.campaign!).events.find((event) => event.id === "reeds"),
  );
  assert.equal(reward.trigger.kind, "visit");
  assert.equal("tile" in reward.trigger && reward.trigger.tile, "5,1");
  assert.equal(game.tile("5,1")?.terrain, "grove");
  assert(!game.at("5,1"));
  game.advanceDialogue(true);
  game.move(adept.id, "5,1");
  assert(game.campaign!.marks.includes("event:reeds"));
  assert.equal(game.gold.player, 26);
  for (const rewarded of [false, true]) {
    const old = Game.startCampaign("lantern-road", 2);
    old.mapRevision = 1;
    required(old.tile("4,3")).terrain = "grove";
    required(old.tile("4,3")).owner = "player";
    required(old.tile("5,1")).terrain = "plains";
    if (rewarded) old.campaign!.marks.push("event:reeds");
    old.gold.player = rewarded ? 26 : 20;
    saveBattle(old);
    const restored = loadBattle().game;
    assert.equal(restored.mapId, "lantern-bell");
    assert.equal(restored.mapRevision, 2);
    assert.deepEqual(restored.units, old.units);
    assert.equal(restored.gold.player, old.gold.player);
    assert.equal(restored.campaign!.marks.includes("event:reeds"), rewarded);
    assert.equal(restored.tile("4,3")?.terrain, "plains");
    assert.equal(restored.tile("5,1")?.terrain, "grove");
    assert.equal(restored.tile("5,1")?.owner, "player");
  }
}

// Starting occupancy cannot award a visit during another move, turn, or reload.
{
  const game = Game.startCampaign("lantern-road", 2);
  game.advanceDialogue(true);
  const adept = required(game.at("4,3"));
  adept.tile = "5,1"; // Exercise the movement rule even if a unit is already on the reward.
  assert.throws(() => game.move(adept.id, adept.tile), /different hex/);
  const village = required(
    game.tiles.find((tile) => tile.terrain === "village" && !game.at(tile.id)),
  );
  const approach = required(
    game.neighbors(village.id).find((tile) => tile.terrain !== "water" && !game.at(tile.id)),
  );
  const visitor = game.addUnit("scout", "player", approach.id);
  const gold = game.gold.player;
  game.move(visitor.id, village.id);
  assert.equal(game.gold.player, gold);
  assert(!game.campaign!.marks.includes("event:reeds"));
  game.beginTurn("player");
  saveBattle(game);
  assert(!loadBattle().game.campaign!.marks.includes("event:reeds"));
  const away = required(
    game.neighbors(adept.tile).find((tile) => tile.terrain !== "water" && !game.at(tile.id)),
  );
  game.move(adept.id, away.id);
  game.beginTurn("player");
  const undo = createMoveUndo();
  const before = game.gold.player;
  undo.move(game, adept.id, "5,1");
  assert(game.campaign!.marks.includes("event:reeds"));
  assert.equal(game.gold.player, before + 6);
  assert(undo.undo(game));
  assert(!game.campaign!.marks.includes("event:reeds"));
  assert.equal(game.gold.player, before);
  game.move(adept.id, "5,1");
  game.beginTurn("player");
  assert.equal(game.campaign!.marks.filter((mark) => mark === "event:reeds").length, 1);
  const civilian = required(
    game.campaign!.dialogue.find((line) => line.speaker === "Reed gatherer"),
  );
  civilian.portrait = "scout"; // A previously published conversation.
  saveBattle(game);
  assert.equal(
    loadBattle().game.campaign!.dialogue.find((line) => line.speaker === "Reed gatherer")?.portrait,
    "worker",
  );
}

// Dain's Ashen finale ending requires his survival, without changing the peaceful mission variant.
for (const survives of [true, false]) {
  const game = Game.startCampaign("ashen-oath", 2, [], 0, ["refuge-peace"]);
  const mission = campaignMission(game.campaign!);
  game.advanceDialogue(true);
  const dain = required(game.units.find((unit) => unit.name === "Captain Dain"));
  if (!survives) {
    dain.hp = 1;
    const approach = required(
      game.neighbors(dain.tile).find((tile) => !game.at(tile.id) && tile.terrain !== "water"),
    );
    const attacker = game.addUnit("skeleton", "enemy", approach.id);
    game.beginTurn("enemy");
    game.attack(attacker.id, dain.id, "boneclub");
    assert(!game.unit(dain.id));
    game.endTurn();
    saveBattle(game);
    const restored = loadBattle().game;
    assert.deepEqual(restored.snapshot(), game.snapshot());
    assert.equal(campaignMission(restored.campaign!), mission);
    assert(!restored.campaign!.marks.includes("dain-fell"));
  }
  const dialogueStart = game.campaign!.dialogue.length;
  game.campaign!.marks.push("sealed:7,2", "sealed:7,6");
  game.units = game.units.filter((unit) => unit.side === "player");
  game.beginTurn("player");
  assert.equal(game.winner, "player");
  assert.equal(
    game.campaign!.dialogue.slice(dialogueStart).some((line) => line.speaker === "Dain"),
    survives,
  );
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
}

// The previously published refuge save keeps its units, gold and village ownership.
{
  const current = Game.startCampaign("ashen-oath", 1);
  const legacy = new Game("ashen-refuge", 1);
  legacy.campaign = structuredClone(current.campaign!);
  delete legacy.campaign.refuge;
  legacy.campaign.dialogue = CAMPAIGNS["ashen-oath"].missions[1]!.legacyDialogue!.slice(0, 3).map(
    (line) => ({ ...line }),
  );
  legacy.round = 2;
  legacy.gold.player = 32;
  legacy.tile("5,4")!.owner = "player";
  legacy.tile("5,1")!.owner = "player";
  saveBattle(legacy);
  const restored = loadBattle().game;
  assert.equal(restored.mapId, "ashen-refuge");
  assert.equal(restored.mapRevision, 2);
  assert.equal(restored.round, 2);
  assert.equal(restored.gold.player, 32);
  assert.deepEqual(restored.units, legacy.units);
  assert.equal(restored.tile("3,1")?.owner, "player");
  assert.equal(restored.tile("3,7")?.terrain, "village");
  assert.equal(restored.tile("5,1")?.terrain, "plains");
  saveBattle(restored);
  assert.deepEqual(loadBattle().game.snapshot(), restored.snapshot());
}

// Refuge defense counts completed enemy turns, not repeated publications or captures.
{
  const game = Game.startCampaign("ashen-oath", 1);
  game.advanceDialogue(true);
  assert.equal(game.tile("3,1")?.terrain, "village");
  assert.equal(game.tile("3,7")?.terrain, "village");
  assert.equal(game.tile("5,1")?.terrain, "plains");
  const goal = campaignMission(game.campaign!).goals[0]!;
  assert.equal(goalProgress(game, goal).current, 0);
  for (let round = 2; round <= 6; round++) {
    game.round = round;
    game.beginTurn("player");
    game.beginTurn("player");
    assert.equal(game.campaign?.refuge?.heldTurns, round - 1);
    assert.equal(game.winner, null);
  }
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
  game.round = 7;
  game.beginTurn("player");
  assert.equal(game.winner, "player");
  assert.deepEqual(game.campaign?.choices, ["refuge-peace"]);
  const next = game.nextCampaignMission([]);
  assert(next.units.some((unit) => unit.name === "Captain Dain" && unit.side === "player"));
  assert(next.recruitmentOptions("player").includes("adept"));
}
{
  const game = Game.startCampaign("ashen-oath", 1);
  game.advanceDialogue(true);
  game.round = 3;
  game.beginTurn("player");
  assert.equal(game.campaign?.refuge?.heldTurns, 1);
  // A lost refuge resets the count even if recaptured before the next dawn.
  game.tile("5,4")!.owner = "enemy";
  game.beginTurn("enemy");
  assert.equal(game.campaign?.refuge?.heldTurns, 0);
  game.tile("5,4")!.owner = "player";
  game.round = 4;
  game.beginTurn("player");
  assert.equal(game.campaign?.refuge?.heldTurns, 1);
  const collector = game.addUnit("skeleton", "player", "3,1");
  const gold = game.gold.player;
  game.move(collector.id, "3,0");
  assert.equal(game.gold.player, gold + 6);
  const goal = campaignMission(game.campaign!).goals[0]!;
  assert.equal(goalProgress(game, goal).total, 5);
  for (let round = 5; round <= 7; round++) {
    game.round = round;
    game.beginTurn("player");
    assert.equal(game.winner, null);
  }
  game.round = 8;
  game.beginTurn("player");
  assert.equal(game.winner, "player");
}
// Pursuing a withdrawing soldier closes mediation, changes objectives and removes the ally.
{
  const game = Game.startCampaign("ashen-oath", 1);
  game.advanceDialogue(true);
  const defender = required(game.at("5,4"));
  const fleeing = required(
    game.units.find((unit) => unit.side === "enemy" && unit.type === "soldier"),
  );
  fleeing.tile = "6,4";
  fleeing.hp = 1;
  fleeing.retreating = true;
  const choice = required(
    attackChoiceViews(createGameView(game), defender, fleeing).find(
      (choice) => choice.id === "boneclub",
    ),
  );
  assert(choice.available);
  assert(choice.enemyForecast.impactLabel.endsWith("(dies)"));
  game.attack(defender.id, fleeing.id, "boneclub");
  assert.equal(game.campaign?.refuge?.escalated, true);
  const goal = campaignMission(game.campaign!).goals[0]!;
  assert.equal(goalProgress(game, goal).label, "Defeat the remaining patrol");
  assert.equal(goalProgress(game, goal).done, false);
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
  game.round = 40;
  game.beginTurn("player");
  assert.equal(game.winner, null, "waiting cannot end the hostile route");
  const captain = required(
    game.units.find((unit) => unit.side === "enemy" && unit.role === "leader"),
  );
  // Killing Dain alone is insufficient while another patrol member survives.
  captain.tile = "6,4";
  captain.hp = 1;
  game.attack(defender.id, captain.id, "boneclub");
  assert.equal(game.winner, null);
  game.units = game.units.filter((unit) => unit.side === "player");
  game.beginTurn("player");
  assert.equal(game.winner, "player");
  assert.deepEqual(game.campaign?.choices, ["refuge-war"]);
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
  const next = game.nextCampaignMission([]);
  assert(!next.units.some((unit) => unit.name === "Captain Dain"));
  assert.deepEqual(next.recruitmentOptions("player"), ["skeleton", "bonearcher"]);
  assert(!campaignMission(next.campaign!).intro.some((line) => line.speaker === "Dain"));
  assert.deepEqual(next.retryCampaignMission().snapshot(), next.snapshot());
  saveBattle(next);
  assert.deepEqual(loadBattle().game.snapshot(), next.snapshot());
}
// Killing troops who are still fighting does not close the peaceful route.
{
  const game = Game.startCampaign("ashen-oath", 1);
  const defender = required(game.at("5,4"));
  const target = required(
    game.units.find((unit) => unit.side === "enemy" && unit.type === "soldier"),
  );
  target.tile = "6,4";
  target.hp = 1;
  game.attack(defender.id, target.id, "boneclub");
  assert.equal(game.campaign?.refuge?.escalated, false);
}

// Every withdrawing troop or captain death closes mediation, including a lethal reply.
for (const scenario of ["retreating", "captain", "fighting", "survives"] as const) {
  const game = Game.startCampaign("ashen-oath", 1);
  game.advanceDialogue(true);
  const defender = required(game.at("5,4"));
  const attacker = required(
    game.units.find((unit) =>
      scenario === "captain"
        ? unit.side === "enemy" && unit.role === "leader"
        : unit.side === "enemy" && unit.type === "soldier",
    ),
  );
  attacker.tile = "6,4";
  attacker.hp = scenario === "survives" ? TYPES[attacker.type].hp : 1;
  if (scenario === "retreating" || scenario === "survives") attacker.retreating = true;
  game.turn = "enemy";
  const weapon = TYPES[attacker.type].attacks[0]!;
  assert(game.previewDetails(attacker, defender, weapon.id).retaliation > 0);
  game.attack(attacker.id, defender.id, weapon.id);
  const escalated = scenario === "retreating" || scenario === "captain";
  assert.equal(game.campaign?.refuge?.escalated, escalated, scenario);
  assert.equal(Boolean(game.unit(attacker.id)), scenario === "survives", scenario);
  if (escalated) {
    game.beginTurn("player");
    saveBattle(game);
    const restored = loadBattle().game;
    assert.equal(restored.campaign?.refuge?.escalated, true);
    restored.advanceDialogue(true);
    restored.round = 40;
    restored.beginTurn("player");
    assert.equal(restored.winner, null, "a lethal reply permanently closes mediation");
    assert.equal(
      goalProgress(restored, campaignMission(restored.campaign!).goals[0]!).label,
      "Defeat the remaining patrol",
    );
  }
}

// Each mission has reachable objectives, lawful starting armies and a usable recruiting keep.
for (const definition of Object.values(CAMPAIGNS)) {
  assert.equal(definition.missions.length, 3);
  for (const [index, mission] of definition.missions.entries()) {
    const game = Game.startCampaign(definition.id, index);
    assert.equal(
      new Set(mission.map.terrain.map((t) => `${t.col},${t.row}`)).size,
      mission.map.terrain.length,
    );
    assert.equal(new Set(game.units.map((u) => u.tile)).size, game.units.length);
    for (const side of ["player", "enemy"] as const) {
      const leader = required(game.units.find((u) => u.side === side && u.role === "leader"));
      assert.equal(game.tile(leader.tile)?.terrain, "keep");
      assert.equal(game.tile(leader.tile)?.owner, side);
      assert(TYPES[leader.type].leader);
    }
    for (const unit of game.units)
      assert(Number.isFinite(game.cost(unit, required(game.tile(unit.tile)))));
    const leader = required(game.units.find((u) => u.side === "player" && u.role === "leader"));
    const land = game.tiles.filter((tile) => Number.isFinite(game.cost(leader, tile)));
    const reached = new Set([leader.tile]);
    const queue = [leader.tile];
    while (queue.length)
      for (const tile of game.neighbors(required(queue.shift()))) {
        if (Number.isFinite(game.cost(leader, tile)) && !reached.has(tile.id)) {
          reached.add(tile.id);
          queue.push(tile.id);
        }
      }
    assert.equal(reached.size, land.length);
    for (const id of mission.goals.flatMap(goalTiles)) assert(reached.has(id));
    for (const wave of mission.waves)
      for (const unit of wave.units) assert(game.tile(`${unit.col},${unit.row}`));
    assert(dialoguePending(game.campaign));
    game.advanceDialogue(true);
    saveBattle(game);
    assert.deepEqual(loadBattle().game.snapshot(), game.snapshot(), "Campaign save round trip");
  }
}

// Courier objectives win independently of enemy leaders; losing the courier defeats the mission.
{
  const game = Game.startCampaign("lantern-road");
  const courier = required(game.units.find((u) => u.role === "courier"));
  courier.tile = "10,1";
  game.move(courier.id, "11,1");
  assert.equal(game.winner, "player");
  assert(game.units.some((u) => u.side === "enemy" && u.role === "leader"));
  const survivor = required(game.units.find((u) => u.type === "swordsman" && u.side === "player"));
  survivor.hp = 1;
  game.gold.player = 500;
  const next = game.nextCampaignMission([survivor.id]);
  assert.equal(next.campaign?.mission, 1);
  assert.equal(next.campaign?.carryGold, 8);
  assert(
    next.units.some(
      (u) => u.type === "swordsman" && u.side === "player" && u.hp === TYPES.swordsman.hp,
    ),
  );
  assert.throws(() => game.nextCampaignMission([courier.id]));
  assert.deepEqual(next.retryCampaignMission().snapshot(), next.snapshot());
  const loss = Game.startCampaign("lantern-road");
  loss.units = loss.units.filter((u) => u.role !== "courier");
  loss.beginTurn("player");
  assert.equal(loss.winner, "enemy");
  saveBattle(loss);
  assert.equal(loadBattle().game.winner, "enemy");
}

// Seals suppress their own waves permanently; killing the commander alone is insufficient.
{
  const game = Game.startCampaign("lantern-road", 2);
  game.advanceDialogue(true);
  game.units = game.units.filter((u) => !(u.side === "enemy" && u.role === "leader"));
  game.beginTurn("player");
  assert.equal(game.winner, null);
  const scout = game.addUnit("scout", "player", "9,1");
  game.beginTurn("player");
  assert(!game.campaign?.marks.includes("sealed:9,1"), "A scout cannot seal a Lantern grave");
  game.units = game.units.filter((unit) => unit.id !== scout.id);
  const sealer = game.addUnit("adept", "player", "9,1");
  game.beginTurn("player");
  assert(game.campaign?.marks.includes("sealed:9,1"));
  game.round = campaignMission(game.campaign!).waves[0]!.round;
  const before = game.units.length;
  const beforeWave = game.snapshot();
  game.beginTurn("enemy");
  assert.equal(
    describeChanges(beforeWave, game.snapshot()).recruited,
    false,
    "Grave waves do not play coin sounds",
  );
  assert.equal(game.units.length, before + 2);
  game.beginTurn("enemy");
  assert.equal(game.units.length, before + 2, "Waves arrive only once");
  game.units = game.units.filter((unit) => unit.tile !== "11,4" && unit.tile !== "9,7");
  sealer.tile = "11,4";
  game.beginTurn("player");
  sealer.tile = "9,7";
  game.beginTurn("player");
  assert.equal(game.winner, "player");
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
}

// Side quests and their dialogue are rolled back with the move, then awarded only once.
{
  const game = Game.startCampaign("lantern-road");
  game.advanceDialogue(true);
  const courier = required(game.units.find((u) => u.role === "courier"));
  courier.tile = "8,1";
  const gold = game.gold.player;
  const history = createMoveUndo();
  history.move(game, courier.id, "9,1");
  assert.equal(game.gold.player, gold + 6);
  assert(dialoguePending(game.campaign));
  history.undo(game);
  assert.equal(game.gold.player, gold);
  assert(!dialoguePending(game.campaign));
  game.move(courier.id, "9,1");
  game.beginTurn("player");
  assert.equal(game.gold.player, gold + 6);
  const view = createGameView(game);
  assert(Object.isFrozen(view.campaign?.marks));
  assert(Object.isFrozen(view.campaign?.dialogue));
  saveBattle(game);
  const saved = required(storage.getItem(SAVE_KEY));
  for (const mutate of [
    (state: {
      campaign: { mission: number; dialogueIndex: number; marks: string[] };
      sites: unknown[];
    }) => (state.campaign.mission = 99),
    (state: {
      campaign: { mission: number; dialogueIndex: number; marks: string[] };
      sites: unknown[];
    }) => (state.campaign.dialogueIndex = 999),
    (state: {
      campaign: { mission: number; dialogueIndex: number; marks: string[] };
      sites: unknown[];
    }) => state.campaign.marks.push("event:fake"),
    (state: {
      campaign: { mission: number; dialogueIndex: number; marks: string[] };
      sites: unknown[];
    }) => (state.sites = []),
  ]) {
    const corrupt = JSON.parse(saved);
    mutate(corrupt.state);
    storage.setItem(SAVE_KEY, JSON.stringify(corrupt));
    assert.equal(loadBattle().game.mapId, "mirror");
  }
}

// Interrupted enemy checkpoints replay waves and campaign ownership deterministically.
{
  const game = Game.startCampaign("lantern-road", 2);
  game.advanceDialogue(true);
  game.round = campaignMission(game.campaign!).waves[0]!.round;
  saveBattle(game, true);
  assert.equal(loadBattle().pendingEnemy, true);
  const first = new BattleController(async () => {});
  const replay = new BattleController(async () => {});
  await first.endTurn();
  await replay.endTurn();
  assert.deepEqual(replay.snapshot(), first.snapshot());
  assert.equal(
    replay.snapshot().campaign?.marks.filter((mark) => mark.startsWith("wave:")).length,
    3,
  );
  first.dispose();
  replay.dispose();
}

// New races have real counterplay under flat modifiers and matching retaliation categories.
{
  const game = new Game();
  game.units = [];
  game.tile("2,3")!.terrain = "plains";
  game.tile("3,3")!.terrain = "plains";
  const attacker = game.addUnit("soldier", "player", "2,3");
  const target = game.addUnit("skeleton", "enemy", "3,3");
  assert.equal(game.damage(attacker, target, "sword"), 4);
  assert.equal(game.damage(attacker, target, "mace"), 4);
  attacker.type = "archer";
  assert.equal(game.damage(attacker, target, "bow"), 2);
  attacker.type = "adept";
  assert.equal(game.previewDetails(attacker, target, "spark").retaliation, 0);
  assert.equal(game.damage(attacker, target, "spark"), 4);
  target.type = "wolf";
  attacker.type = "warden";
  assert.equal(game.damage(attacker, target, "spear"), 5);
  target.type = "soldier";
  assert.equal(game.damage(attacker, target, "spear"), 3);
  assert.equal(game.damage(attacker, target, "hammer"), 3);
}

// Replacing a session cancels its paused enemy turn; conversations gate player commands.
{
  storage.clear();
  let release: (() => void) | undefined;
  const controller = new BattleController(
    () =>
      new Promise<void>((resolve) => {
        release = resolve;
      }),
  );
  let latest: BattleView | undefined;
  const unsubscribe = controller.view.subscribe((view) => (latest = view));
  const oldTurn = controller.endTurn();
  controller.commands.startCampaign("green-accord");
  required(release)();
  await oldTurn;
  assert.equal(latest?.game.mapId, "green-border");
  assert.equal(latest?.game.round, 1);
  await assert.rejects(controller.endTurn());
  controller.commands.advanceDialogue(true);
  assert(!dialoguePending(latest?.game.campaign));
  assert.equal(required(latest).game.recruitmentOptions("player")[0], "wolf");
  controller.dispose();
  unsubscribe();
}
console.log(
  "PASS: nine connected missions, campaign saves, goals, waves, rewards, recall, counters and cancellation",
);
