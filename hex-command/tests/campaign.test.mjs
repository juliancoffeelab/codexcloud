import "./storage-fixture.mjs";
import { createHeadlessSession } from "./headless-session.mjs";
import { cellKey } from "./hex-fixture.mjs";
import { assertResultError } from "./result-fixture.mjs";
import characterIds from "../packages/save-migrations/src/legacy/data/characters-v25.json";
import tileIds from "../packages/save-migrations/src/legacy/data/tiles-v26.json";
import * as Save from "./save-adapter.mjs";
import * as Store from "./storage-adapter.mjs";
import { historicalState, historicalBattle } from "./historical-fixture.mjs";
import bellConversion from "../packages/save-migrations/src/legacy/data/bell-valley-v10.json";
import { createTestFactory } from "./native-fixture.mjs";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game } from "./game-fixture.mjs";
import * as Catalog from "./catalog-adapter.mjs";
import * as Native from "./native-adapter.mjs";
import * as Story from "./story-adapter.mjs";
import * as Session from "@frontend/Session";
import { catalog } from "./native-fixture.mjs";
import history from "../packages/save-migrations/src/legacy/data/stages.json";
import oldRefuge from "../packages/save-migrations/src/legacy/data/ashen-refuge-v1.json";
import oldRefugeRoster from "../packages/save-migrations/src/legacy/data/ashen-refuge-v1-roster.json";
import { loadBattle, saveBattle, SAVE_KEY } from "./legacy-save.mjs";
import { required, TestStorage } from "./helpers.mjs";
const storage = new TestStorage();
test("Retired gatherer reward preserves gold and wounds through frozen valley migration", async () => {
  for (const rewarded of [false, true]) {
    const old = historicalState("lantern-road", 2);
    old.mapRevision = 1;
    const adept = required(old.units.find((unit) => unit.type === "adept"));
    adept.hp -= 1;
    if (rewarded) old.campaign.marks.push("event:reeds");
    old.gold.player = rewarded ? 26 : 20;
    await saveBattle(historicalBattle(old));
    const restored = (await loadBattle()).game;
    assert.equal(restored.mapId, old.mapId);
    assert.equal(restored.mapRevision, bellConversion.toRevision);
    const companion = required(restored.units.find((unit) => unit.id === adept.id));
    assert.equal(companion.hp, adept.hp);
    assert.equal(cellKey(companion.tile), bellConversion.mapping[cellKey(adept.tile)]);
    assert.equal(companion.character, "oren");
    assert.equal(restored.gold.player, old.gold.player);
  }
});
test("Legacy refuge retains company and ownership", async () => {
  const current = Game.startCampaign("ashen-oath", 1);
  const legacy = new Game("ashen-refuge");
  legacy.tiles = structuredClone(oldRefuge.tiles).map((tile) => ({
    ...tile,
    id: tile.id,
  }));
  legacy.units = [];
  legacy.nextId = 1;
  for (const unit of oldRefugeRoster.units)
    Object.assign(legacy.addUnit(unit.type, unit.side, `${unit.col},${unit.row}`), {
      ...(unit.name ? { character: characterIds.campaigns["ashen-oath"].aliases[unit.name] } : {}),
      ...(unit.role ? { role: unit.role } : {}),
    });
  legacy.campaign = structuredClone(current.campaign);
  delete legacy.campaign.refuge;
  legacy.campaign.dialogue = required(
    history[Catalog.campaign(catalog, "ashen-oath").missions[1].map.id],
  )
    .dialogue.slice(0, 3)
    .map((line) => ({ ...line }));
  legacy.round = 2;
  legacy.gold.player = 32;
  legacy.tile("5,4").owner = "player";
  legacy.tile("5,1").owner = "player";
  await saveBattle(legacy);
  const oldSave = JSON.parse(await Save.flashText());
  oldSave.state.mapRevision = 1;
  await Store.store_save(JSON.stringify(oldSave));
  const restored = (await loadBattle()).game;
  assert.equal(restored.mapId, "ashen-refuge");
  assert.equal(restored.mapRevision, 2);
  assert.equal(restored.round, 2);
  assert.equal(restored.gold.player, 32);
  assert.deepEqual(restored.units, legacy.units);
  assert.equal(restored.tile("3,1")?.owner, "player");
  assert.equal(restored.tile("3,7")?.terrain, "village");
  assert.equal(restored.tile("5,1")?.terrain, "plains");
  await saveBattle(restored);
  assert.deepEqual((await loadBattle()).game.snapshot(), restored.snapshot());
});
test("Interrupted wave checkpoint replays once", async () => {
  const game = Game.startCampaign("ashen-oath", 0);
  game.advanceDialogue(true);
  game.round = 6;
  await Save.write(Save.create(), game.checkpoint(), true);
  assert.equal(JSON.parse(await Save.flashText()).pendingEnemy, true);
  const first = await createHeadlessSession(async () => {}, createTestFactory());
  const replay = await createHeadlessSession(async () => {}, createTestFactory());
  await Session.commands(first).end_turn();
  await Session.commands(replay).end_turn();
  assert.deepEqual(Native.state(Session.view(replay).game), Native.state(Session.view(first).game));
  assert(
    !Native.state(Session.view(replay).game).campaign.scripts.active.includes(
      "southern-capture-party",
    ),
  );
  Session.close(first);
  Session.close(replay);
});
test("Replacing a paused session cancels it and dialogue gates commands", async () => {
  storage.clear();
  let release, markPaused;
  const paused = new Promise((resolve) => {
    markPaused = resolve;
  });
  const controller = await createHeadlessSession(
    () =>
      new Promise((resolve) => {
        release = resolve;
        markPaused();
      }),
    createTestFactory(),
  );
  let latest;
  const unsubscribe = Session.subscribe(controller, (view) => (latest = view));
  const oldTurn = Session.commands(controller).end_turn();
  await paused;
  Session.commands(controller).start_campaign(undefined, "green-accord");
  required(release)();
  await oldTurn;
  assert.equal(latest && Native.state(latest.game).mapId, "green-border");
  assert.equal(latest && Native.state(latest.game).round, 1);
  assertPhaseError(await Session.commands(controller).end_turn());
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

function assertPhaseError(result, pattern) {
  assertResultError(result, Session.error_message, pattern);
}
