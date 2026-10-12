import type { Tile, Side, Dialogue } from "../rust/pkg/hex_command";
import { createTestFactory } from "./native-fixture";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game } from "./game-fixture";
import * as Catalog from "@frontend/Catalog";
import * as Native from "@frontend/Native";
import * as Story from "@frontend/Story";
import * as Session from "@frontend/Session";
import { catalog } from "./native-fixture";
import history from "../src/save/migration/legacy/data/stages.json";
import oldRefuge from "../src/save/migration/legacy/data/ashen-refuge-v1.json";
import oldRefugeRoster from "../src/save/migration/legacy/data/ashen-refuge-v1-roster.json";
import { loadBattle, saveBattle, SAVE_KEY } from "./legacy-save";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();

// The gatherers are reachable on the northern bridge approach; old armies keep progress.
test("Gatherer reward moves without losing saved progress", async () => {
  const game = Game.startCampaign("lantern-road", 2);
  const adept = required(game.at("4,3"));
  const reward = required(
    Story.mission(catalog, game.campaign!).source.events.find((event) => event.id === "reeds"),
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
  legacy.tiles = structuredClone(oldRefuge.tiles) as Tile[];
  legacy.units = [];
  legacy.nextId = 1;
  for (const unit of oldRefugeRoster.units)
    Object.assign(legacy.addUnit(unit.type, unit.side as Side, `${unit.col},${unit.row}`), {
      ...(unit.name ? { name: unit.name } : {}),
      ...(unit.role ? { role: unit.role } : {}),
    });
  legacy.campaign = structuredClone(current.campaign!);
  delete legacy.campaign.refuge;
  legacy.campaign.dialogue = required(
    (history as Record<string, { dialogue: Dialogue[] }>)[
      Catalog.campaign(catalog, "ashen-oath").missions[1]!.map.id
    ],
  )
    .dialogue.slice(0, 3)
    .map((line) => ({ ...line }));
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
  game.round = Story.mission(catalog, game.campaign!).source.waves[0]!.round;
  saveBattle(game, true);
  assert.equal(loadBattle().pendingEnemy, true);
  const first = Session.create(async () => {}, createTestFactory());
  const replay = Session.create(async () => {}, createTestFactory());
  await Session.commands(first).end_turn();
  await Session.commands(replay).end_turn();
  assert.deepEqual(Native.state(Session.view(replay).game), Native.state(Session.view(first).game));
  assert.equal(
    Native.state(Session.view(replay).game).campaign?.marks.filter((mark) =>
      mark.startsWith("wave:"),
    ).length,
    3,
  );
  Session.close(first);
  Session.close(replay);
});

// Replacing a session cancels its paused enemy turn; conversations gate player commands.
test("Replacing a paused session cancels it and dialogue gates commands", async () => {
  storage.clear();
  let release: (() => void) | undefined;
  const controller = Session.create(
    () =>
      new Promise<void>((resolve) => {
        release = resolve;
      }),
    createTestFactory(),
  );
  let latest: ReturnType<typeof Session.view> | undefined;
  const unsubscribe = Session.subscribe(controller, (view) => (latest = view));
  const oldTurn = Session.commands(controller).end_turn();
  Session.commands(controller).start_campaign("green-accord");
  required(release)();
  await oldTurn;
  assert.equal(latest && Native.state(latest.game).mapId, "green-border");
  assert.equal(latest && Native.state(latest.game).round, 1);
  await assert.rejects(Session.commands(controller).end_turn());
  Session.commands(controller).dialogue(true);
  assert(!Story.dialogue_pending(latest && Native.state(latest.game).campaign));
  assert(Native.recruitment_options(required(latest).game, "player").length > 0);
  Session.close(controller);
  unsubscribe();
});

beforeEach(() => {
  storage.clear();
  vi.stubGlobal("localStorage", storage);
});
afterEach(() => {
  vi.unstubAllGlobals();
});
