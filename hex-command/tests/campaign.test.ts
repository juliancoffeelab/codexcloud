import { createTestFactory } from "./native-fixture";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game } from "./game-fixture";
import { CAMPAIGNS } from "../src/game/campaign-data";
import { campaignMission, dialoguePending } from "../src/game/campaign-progress";
import { loadBattle, saveBattle, SAVE_KEY } from "./legacy-save";
import { CAMPAIGNS as MIGRATION_CAMPAIGNS } from "../src/save/migrations/legacy/catalog";
import { LEGACY_ASHEN_REFUGE } from "../src/save/migrations/legacy/layouts";
import { legacyBoard } from "../src/save/migrations/legacy/board";
import { BattleController, type BattleView } from "../src/controller/battle";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();

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
  const legacy = new Game("ashen-refuge");
  const board = legacyBoard("ashen-refuge", 1);
  legacy.tiles = board.tiles;
  legacy.units = [];
  legacy.nextId = 1;
  for (const unit of LEGACY_ASHEN_REFUGE.units)
    Object.assign(legacy.addUnit(unit.type, unit.side, `${unit.col},${unit.row}`), {
      ...(unit.name ? { name: unit.name } : {}),
      ...(unit.role ? { role: unit.role } : {}),
    });
  legacy.campaign = structuredClone(current.campaign!);
  delete legacy.campaign.refuge;
  legacy.campaign.dialogue = MIGRATION_CAMPAIGNS["ashen-oath"].missions[1]!.legacyDialogue.slice(
    0,
    3,
  ).map((line) => ({ ...line }));
  legacy.round = 2;
  legacy.gold.player = 32;
  legacy.tile("5,4")!.owner = "player";
  legacy.tile("5,1")!.owner = "player";
  saveBattle(legacy);
  const oldSave = JSON.parse(localStorage.getItem(SAVE_KEY)!);
  oldSave.state.mapRevision = 1;
  localStorage.setItem(SAVE_KEY, JSON.stringify(oldSave));
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
  assert(required(latest).game.recruitmentOptions("player").length > 0);
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
