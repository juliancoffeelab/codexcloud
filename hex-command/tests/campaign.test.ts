import { createTestFactory } from "./native-fixture";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game, TYPES } from "./game-fixture";
import { CAMPAIGNS } from "../src/game/campaign-data";
import {
  campaignMission,
  goalTiles,
  goalProgress,
  dialoguePending,
} from "../src/game/campaign-progress";
import { createGameView } from "./fixture-view";
import { createMoveUndo } from "../src/game/undo";
import { loadBattle, saveBattle, SAVE_KEY, importWaveMarker } from "./legacy-save";
import { describeChanges } from "../src/controller/feedback";
import { attackChoiceViews } from "../src/ui/combat-view";
import { BattleController, type BattleView } from "../src/controller/battle";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();

// Visit sites belong on unoccupied map hexes, not under the opening company.
for (const campaign of Object.values(CAMPAIGNS)) {
  test("Visit rewards start on unoccupied hexes — " + campaign.id, async () => {
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
  });
}

// The gatherers are reachable on the northern bridge approach; old armies keep progress.
test("Gatherer reward moves without losing saved progress", async () => {
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
});

// The previously published refuge save keeps its units, gold and village ownership.
test("Legacy refuge retains company and ownership", async () => {
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
});
// Killing troops who are still fighting does not close the peaceful route.
test("Fighting troop deaths preserve mediation", async () => {
  const game = Game.startCampaign("ashen-oath", 1);
  const defender = required(game.at("5,4"));
  const target = required(
    game.units.find((unit) => unit.side === "enemy" && unit.type === "soldier"),
  );
  target.tile = "6,4";
  target.hp = 1;
  game.attack(defender.id, target.id, "boneclub");
  assert.equal(game.campaign?.refuge?.escalated, false);
});

// Each mission has reachable objectives, lawful starting armies and a usable recruiting keep.
for (const definition of Object.values(CAMPAIGNS)) {
  test("Campaign missions have legal armies and reachable goals — " + definition.id, async () => {
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
  });
}

// Interrupted enemy checkpoints replay waves and campaign ownership deterministically.
test("Interrupted wave checkpoint replays once", async () => {
  const game = Game.startCampaign("lantern-road", 2);
  game.advanceDialogue(true);
  game.round = campaignMission(game.campaign!).waves[0]!.round;
  saveBattle(game, true);
  assert.equal(loadBattle().pendingEnemy, true);
  const first = new BattleController(async () => {}, createTestFactory());
  const replay = new BattleController(async () => {}, createTestFactory());
  await first.endTurn();
  await replay.endTurn();
  assert.deepEqual(replay.snapshot(), first.snapshot());
  assert.equal(
    replay.snapshot().campaign?.marks.filter((mark) => mark.startsWith("wave:")).length,
    3,
  );
  first.dispose();
  replay.dispose();
});

// Replacing a session cancels its paused enemy turn; conversations gate player commands.
test("Replacing a paused session cancels it and dialogue gates commands", async () => {
  storage.clear();
  let release: (() => void) | undefined;
  const controller = new BattleController(
    () =>
      new Promise<void>((resolve) => {
        release = resolve;
      }),
    createTestFactory(),
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
});

beforeEach(() => {
  storage.clear();
  vi.stubGlobal("localStorage", storage);
});
afterEach(() => {
  vi.unstubAllGlobals();
});
