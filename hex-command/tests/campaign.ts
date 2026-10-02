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
import { BattleController, type BattleView } from "../src/controller/battle";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();
globalThis.localStorage = storage;

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
    for (const unit of game.units) assert.notEqual(game.tile(unit.tile)?.terrain, "water");
    const leader = required(game.units.find((u) => u.side === "player" && u.role === "leader"));
    const land = game.tiles.filter((tile) => tile.terrain !== "water");
    const reached = new Set([leader.tile]);
    const queue = [leader.tile];
    while (queue.length)
      for (const tile of game.neighbors(required(queue.shift()))) {
        if (tile.terrain !== "water" && !reached.has(tile.id)) {
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
  courier.tile = "10,3";
  game.move(courier.id, "11,3");
  assert.equal(game.winner, "player");
  assert(game.units.some((u) => u.side === "enemy" && u.role === "leader"));
  const survivor = required(game.units.find((u) => u.type === "soldier" && u.side === "player"));
  survivor.hp = 1;
  game.gold.player = 500;
  const next = game.nextCampaignMission([survivor.id]);
  assert.equal(next.campaign?.mission, 1);
  assert.equal(next.campaign?.carryGold, 8);
  assert(
    next.units.some(
      (u) => u.type === "soldier" && u.side === "player" && u.hp === TYPES.soldier.hp,
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
  const sealer = game.addUnit("scout", "player", "9,1");
  game.beginTurn("player");
  assert(game.campaign?.marks.includes("sealed:9,1"));
  game.round = 4;
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

// Hold objectives wait for a player turn boundary; missing dawn is recoverable.
{
  const game = Game.startCampaign("green-accord", 1);
  game.advanceDialogue(true);
  const goal = required(campaignMission(game.campaign!).goals.find((goal) => goal.kind === "hold"));
  game.tile("6,4")!.owner = "enemy";
  game.units = game.units.filter((unit) => unit.tile !== "6,4");
  const wolf = game.addUnit("wolf", "player", "5,4");
  game.round = 6;
  game.move(wolf.id, "6,4");
  assert.equal(game.winner, null, "Capture during a turn does not satisfy dawn");
  assert.equal(goalProgress(game, goal).done, false);
  game.endTurn();
  game.endTurn();
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
  game.round = 4;
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
  assert.equal(game.damage(attacker, target, "bow"), 1);
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
