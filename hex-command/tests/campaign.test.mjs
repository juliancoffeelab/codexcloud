import "./storage-fixture.mjs";
import { createHeadlessSession } from "./headless-session.mjs";
import { cellKey } from "./hex-fixture.mjs";
import { assertResultError } from "./result-fixture.mjs";
import { historicalState, historicalBattle } from "./historical-fixture.mjs";
import bellConversion from "../packages/save-migrations/src/legacy/data/bell-valley-v10.json";
import { createTestFactory } from "./native-fixture.mjs";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import * as Native from "./native-adapter.mjs";
import * as Story from "./story-adapter.mjs";
import * as Session from "@frontend/Session";
import { loadBattle, saveBattle } from "./legacy-save.mjs";
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
  Session.commands(controller).start_campaign(undefined, "lantern-road");
  required(release)();
  await oldTurn;
  assert.equal(latest && Native.state(latest.game).mapId, "lantern-ferry");
  assert.equal(latest && Native.state(latest.game).round, 1);
  assertPhaseError(await Session.commands(controller).end_turn());
  Session.commands(controller).dialogue(true);
  assert(!Story.dialogue_pending(latest && Native.state(latest.game).campaign));
  assert.equal(Native.state(required(latest).game).turn, "player");
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
