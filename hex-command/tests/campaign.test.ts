import tileHistory from "../src/save/migrations/legacy/data/tiles-v26.json";
import characterHistory from "../src/save/migrations/legacy/data/characters-v25.json";
import { createTestFactory } from "./native-fixture";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game } from "./game-fixture";
import { campaignMission, dialoguePending } from "../src/game/campaign-progress";
import { loadBattle, saveBattle, SAVE_KEY } from "./legacy-save";
import { CAMPAIGNS as MIGRATION_CAMPAIGNS } from "../src/save/migrations/legacy/catalog";
import { LEGACY_ASHEN_REFUGE } from "../src/save/migrations/legacy/layouts";
import { legacyBoard } from "../src/save/migrations/legacy/board";
import { BattleController, type BattleView } from "../src/controller/battle";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();

// The previously published refuge save keeps its units, gold and village ownership.
test("Legacy refuge retains company and ownership", async () => {
  const current = Game.startCampaign("ashen-oath", 1);
  const legacy = new Game("ashen-refuge");
  const board = legacyBoard("ashen-refuge", 1);
  legacy.tiles = board.tiles.map((tile) => ({
    ...tile,
    id: tileHistory["ashen-refuge"][tile.id as keyof (typeof tileHistory)["ashen-refuge"]],
  }));
  legacy.units = [];
  legacy.nextId = 1;
  for (const unit of LEGACY_ASHEN_REFUGE.units)
    Object.assign(
      legacy.addUnit(
        unit.type,
        unit.side,
        legacy.tiles.find((t) => t.col === unit.col && t.row === unit.row)!.id,
      ),
      {
        ...(unit.name
          ? {
              character:
                characterHistory.campaigns["ashen-oath"].aliases[
                  unit.name as keyof (typeof characterHistory.campaigns)["ashen-oath"]["aliases"]
                ],
            }
          : {}),
        ...(unit.role ? { role: unit.role } : {}),
      },
    );
  legacy.campaign = structuredClone(current.campaign!);
  delete legacy.campaign.refuge;
  legacy.campaign.dialogue = MIGRATION_CAMPAIGNS["ashen-oath"].missions[1]!.legacyDialogue.slice(
    0,
    3,
  ).map((line) => ({ ...line }));
  legacy.round = 2;
  legacy.gold.player = 32;
  legacy.tile("hex-0891-b20a50ef8c")!.owner = "player";
  legacy.tile("hex-0866-c54da0213d")!.owner = "player";
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
  assert.equal(restored.tile("hex-0738-f0cce97107")?.owner, "player");
  assert.equal(restored.tile("hex-0766-4907292e1d")?.terrain, "village");
  assert.equal(restored.tile("hex-0866-c54da0213d")?.terrain, "plains");
  saveBattle(restored);
  assert.deepEqual(loadBattle().game.snapshot(), restored.snapshot());
});
// Interrupted enemy checkpoints replay waves and campaign ownership deterministically.
test("Interrupted wave checkpoint replays once", async () => {
  const game = Game.startCampaign("ashen-oath", 0);
  game.advanceDialogue(true);
  game.round = 6;
  saveBattle(game, true);
  assert.equal(loadBattle().pendingEnemy, true);
  const first = new BattleController(async () => {}, createTestFactory());
  const replay = new BattleController(async () => {}, createTestFactory());
  await first.endTurn();
  await replay.endTurn();
  assert.deepEqual(replay.snapshot(), first.snapshot());
  assert(!replay.snapshot().campaign?.scripts?.active.includes("southern-capture-party"));
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
  await controller.commands.startCampaign("green-accord");
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
