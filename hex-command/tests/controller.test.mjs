import "./storage-fixture.mjs";
import { createHeadlessSession } from "./headless-session.mjs";
import { cellKey } from "./hex-fixture.mjs";
import { isOk, resultError } from "./result-fixture.mjs";
import { assertResultError } from "./result-fixture.mjs";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import * as Session from "@frontend/Session";
import * as HudDetails from "@frontend/Hud_details";
import * as Art from "@frontend/Art";
import * as Presentation from "./presentation-adapter.mjs";
import * as Story from "./story-adapter.mjs";
import { Native, Catalog, factory, catalog, restoreBattle } from "./native-fixture.mjs";
import { Game, TYPES, developerFixture } from "./game-fixture.mjs";
import {
  SAVE_KEY,
  saveBattle as saveLegacyBattle,
  loadBattle as loadLegacyBattle,
} from "./legacy-save.mjs";
import * as Save from "./save-adapter.mjs";
import { saveBattle } from "./persistence-adapter.mjs";
import { required, TestStorage } from "./helpers.mjs";
import * as Domain from "@frontend/Domain";
import * as FixtureJson from "@frontend/Fixture_json";
import * as Tools from "@frontend/Tools";
import * as Setup from "@frontend/Setup";
const motion = vi.hoisted(() => ({ tile: vi.fn(), turn: vi.fn() }));
vi.mock("@frontend/Motion", () => ({ animate_tile: motion.tile, animate_turn: motion.turn }));
const storage = new TestStorage(),
  immediatePause = async (_milliseconds) => {};
const controllers = [];
async function loadBattle() {
  await Promise.all(controllers.map(Session.saved));
  return loadLegacyBattle();
}
async function readSave() {
  await Promise.all(controllers.map(Session.saved));
  return (await import("./persistence-adapter.mjs")).readSave();
}

async function createController(clock = immediatePause) {
  await Promise.all(controllers.map(Session.saved));
  const session = await createHeadlessSession(clock, factory);
  controllers.push(session);
  return session;
}
function state(session) {
  return Native.state(Session.view(session).interaction_game);
}
function snapshot(session) {
  return structuredClone(state(session));
}
function observe(session) {
  let latest;
  const unsubscribe = Session.subscribe(session, (view) => {
    latest = view;
  });
  return {
    get view() {
      return required(latest);
    },
    unsubscribe,
  };
}
function move(session, id, tile) {
  const commands = Session.commands(session),
    view = Session.view(session);
  if (view.map_focus.selected !== id) {
    const actor = required(Native.unit(view.game, id));
    if (view.map_focus.inspected === cellKey(actor.tile) || view.map_focus.target === id)
      commands.clear(true);
    commands.choose(cellKey(actor.tile));
  }
  commands.choose(tile);
}
function panel(view) {
  return HudDetails.panel(view.hud);
}

function mission(game) {
  return Story.mission(catalog, required(game.campaign)).source;
}
function pauseTurn(callToPause = 1) {
  let release, markReached;
  const reached = new Promise((resolve) => {
      markReached = resolve;
    }),
    first = new Promise((resolve) => {
      release = resolve;
    });
  let calls = 0;
  return {
    reached,
    pause: () => {
      if (++calls !== callToPause) return Promise.resolve();
      markReached();
      return first;
    },
    release: () => release(),
  };
}
function completes(session, predicate) {
  return new Promise((resolve) => {
    let unsubscribe = () => {};
    unsubscribe = Session.subscribe(session, (view) => {
      if (predicate(Native.state(view.game))) {
        unsubscribe();
        resolve();
      }
    });
  });
}
test("Developer map supports normal actions and persistence", async () => {
  await saveBattle(developerFixture());
  const session = await createController(),
    observed = observe(session),
    commands = Session.commands(session);
  const opening = snapshot(session),
    player = required(opening.units.find((unit) => unit.side === "player")),
    enemy = required(opening.units.find((unit) => unit.side === "ai"));
  commands.choose(cellKey(player.tile));
  const details = panel(observed.view);
  assert(details !== 0 && details.TAG === 1);
  assert.equal(details._0.action_status, "Ready");
  assert(Object.hasOwn(Native.reachable(observed.view.game, player.id), "0,2"));
  assert(
    Presentation.battlefield(catalog, observed.view.game, player.id, void 0).some((tile) =>
      tile.classes.includes("reachable"),
    ),
  );
  move(session, player.id, "0,2");
  assert(observed.view.can_undo);
  commands.undo();
  assert.deepEqual(snapshot(session), opening);
  commands.choose(cellKey(enemy.tile));
  const combat = panel(observed.view);
  assert(combat !== 0 && combat.TAG === 2);
  commands.attack(required(Native.best_attack(observed.view.game, player.id, enemy.id)).attack.id);
  assert(required(state(session).units.find((unit) => unit.id === enemy.id)).hp < enemy.hp);
  assert.deepEqual((await loadBattle()).game.captureState(), snapshot(session));
  await commands.end_turn();
  assert(state(session).round > opening.round || state(session).winner);
  assert.deepEqual((await loadBattle()).game.captureState(), snapshot(session));
  Session.close(session);
  observed.unsubscribe();
});
test("Inspection switches tiles and closes when the inspected unit or terrain is tapped again", async () => {
  const session = await createController(),
    observed = observe(session),
    commands = Session.commands(session);
  const opening = snapshot(session);
  const player = required(opening.units.find((unit) => unit.side === "player"));
  const enemy = required(opening.units.find((unit) => unit.side === "ai"));
  const empty = required(
    opening.tiles.find((tile) => !opening.units.some((unit) => cellKey(unit.tile) === tile.id)),
  );
  const other = required(
    opening.tiles.find(
      (tile) =>
        tile.id !== empty.id && !opening.units.some((unit) => cellKey(unit.tile) === tile.id),
    ),
  );
  const saved = await readSave(),
    game = observed.view.game;
  commands.choose(cellKey(enemy.tile));
  assert.equal(observed.view.map_focus.selected, enemy.id);
  commands.choose(cellKey(enemy.tile));
  assert.equal(observed.view.map_focus.selected, void 0);
  assert.equal(observed.view.map_focus.inspected, void 0);
  commands.choose(empty.id);
  assert.equal(observed.view.map_focus.inspected, empty.id);
  commands.choose(other.id);
  assert.equal(observed.view.map_focus.inspected, other.id);
  commands.choose(other.id);
  assert.equal(observed.view.map_focus.inspected, void 0);
  commands.choose(cellKey(player.tile));
  commands.choose(cellKey(player.tile));
  assert.equal(observed.view.map_focus.selected, void 0);
  assert.equal(observed.view.map_focus.inspected, void 0);
  commands.choose(cellKey(enemy.tile));
  commands.choose(empty.id);
  assert.equal(observed.view.map_focus.selected, void 0);
  assert.equal(observed.view.map_focus.inspected, empty.id);
  commands.choose(empty.id);
  assert.equal(observed.view.map_focus.inspected, void 0);
  assert.equal(observed.view.map_focus.target, void 0);
  assert.equal(observed.view.game, game);
  assert.deepEqual(snapshot(session), opening);
  assert.equal(await readSave(), saved);
  observed.unsubscribe();
});
test("Click-away dismissal preserves movement, attack targeting and undo", async () => {
  await saveBattle(developerFixture());
  const session = await createController(),
    observed = observe(session),
    commands = Session.commands(session);
  const opening = snapshot(session),
    player = required(opening.units.find((unit) => unit.side === "player"));
  const destination = required(Object.keys(Native.reachable(observed.view.game, player.id))[0]);
  commands.choose(cellKey(player.tile));
  commands.choose(destination);
  assert.equal(cellKey(Native.unit(observed.view.game, player.id)?.tile), destination);
  assert.equal(observed.view.map_focus.inspected, destination);
  assert(observed.view.can_undo);
  commands.clear(true);
  assert.equal(observed.view.map_focus.inspected, void 0);
  assert(observed.view.can_undo, "dismissing the panel must not spend undo history");
  commands.undo();
  assert.deepEqual(snapshot(session), opening);
  const target = required(Native.targets(observed.view.game, player.id)[0]);
  commands.choose(cellKey(target.tile));
  const combat = panel(observed.view);
  assert(combat !== 0 && combat.TAG === 2);
  commands.choose(cellKey(target.tile));
  assert.equal(observed.view.map_focus.selected, void 0);
  assert.equal(observed.view.map_focus.target, void 0);
  assert.equal(observed.view.map_focus.inspected, void 0);
  assert.deepEqual(snapshot(session), opening, "targeting or dismissal must not attack");
  observed.unsubscribe();
});
test("Dialogue navigation persists its cursor without advancing gameplay", async () => {
  const session = await createController(),
    observed = observe(session),
    commands = Session.commands(session);
  commands.start_campaign(undefined, "green-accord");
  const opening = observed.view,
    campaign = required(Native.state(opening.game).campaign);
  assert(campaign.dialogue.length > 1);
  commands.previous_dialogue();
  assert.equal(observed.view, opening);
  commands.dialogue(false);
  assert.equal(required(state(session).campaign).dialogueIndex, 1);
  assert.equal(campaign.dialogueIndex, 0);
  assert.equal(observed.view.revision, opening.revision);
  const second = snapshot(session);
  move(session, required(second.units[0]).id, "3,2");
  assert.deepEqual(snapshot(session), second);
  assertPhaseError(await commands.end_turn(), /Wait for your turn/);
  assert.throws(() => commands.undo(), sessionException(/Finish the conversation/));
  const restored = await createController();
  assert.equal(required(state(restored).campaign).dialogueIndex, 1);
  Session.commands(restored).previous_dialogue();
  const rewound = await createController(),
    rewoundCommands = Session.commands(rewound);
  assert.equal(required(state(rewound).campaign).dialogueIndex, 0);
  while (Story.dialogue_pending(state(rewound).campaign)) rewoundCommands.dialogue(false);
  const finished = snapshot(rewound),
    finishedView = observe(rewound),
    save = await readSave(),
    publication = finishedView.view;
  rewoundCommands.dialogue(false);
  assert.equal(finishedView.view, publication);
  assert.equal(await readSave(), save);
  rewoundCommands.previous_dialogue();
  assert.deepEqual(snapshot(rewound), finished);
  rewoundCommands.reset(undefined, undefined, "mirror");
  const skirmish = snapshot(rewound);
  rewoundCommands.previous_dialogue();
  assert.deepEqual(snapshot(rewound), skirmish);
  Session.close(rewound);
  assert.throws(() => rewoundCommands.previous_dialogue(), sessionException(/session has closed/));
  Session.close(session);
  Session.close(restored);
  finishedView.unsubscribe();
  observed.unsubscribe();
});
test("Stray dialogue cannot overwrite a paused turn checkpoint", async () => {
  const paused = pauseTurn(),
    session = await createController(paused.pause),
    observed = observe(session),
    commands = Session.commands(session),
    before = snapshot(session);
  const pending = commands.end_turn(),
    checkpoint = await readSave(),
    enemyView = observed.view;
  commands.dialogue(false);
  commands.dialogue(true);
  commands.previous_dialogue();
  assert.equal(observed.view, enemyView);
  assert.equal(await readSave(), checkpoint);
  const loaded = await loadBattle();
  assert(loaded.pendingEnemy);
  assert.deepEqual(loaded.game.captureState(), before);
  Session.close(session);
  paused.release();
  await pending;
  observed.unsubscribe();
});
test("Completed vigil publishes and persists its outcome", async () => {
  const fixture = Game.startCampaign("green-accord", 1);
  fixture.advanceDialogue(true);
  fixture.units = fixture.units.filter((unit) => unit.side === "player");
  const goal = { tile: { col: 6, row: 4 } };
  const lark = required(fixture.units.find((unit) => unit.character === "lark")),
    occupant = fixture.at(cellKey(goal.tile));
  if (occupant && occupant.id !== lark.id) occupant.tile = { col: 5, row: 4 };
  lark.tile = goal.tile;
  required(fixture.tile(cellKey(goal.tile))).owner = "player";
  fixture.round = 3;
  required(fixture.campaign).vigil = { heldTurns: 2, lastCountedRound: 3 };
  await saveLegacyBattle(fixture);
  assert.equal((await loadBattle()).game.mapId, fixture.mapId);
  const session = await createController(),
    observed = observe(session);
  await Session.commands(session).end_turn();
  assert.equal(state(session).winner, "playerVictory");
  assert.equal(observed.view.status, state(session).campaign?.outcome);
  assert(Story.dialogue_pending(state(session).campaign));
  Session.commands(session).dialogue(false);
  assert.equal(
    required((await loadBattle()).game.campaign).dialogueIndex,
    required(state(session).campaign).dialogueIndex,
  );
  Session.close(session);
  observed.unsubscribe();
});
test("Finished enemy-turn battles persist outcome dialogue", async () => {
  const fixture = Game.startCampaign("green-accord");
  fixture.advanceDialogue(true);
  fixture.units = fixture.units.filter(
    (unit) => unit.role === "leader" || unit.character === "lark",
  );
  const leader = required(
      fixture.units.find((unit) => unit.side === "player" && unit.role === "leader"),
    ),
    enemy = fixture.addUnit("soldier", "ai", "4,4");
  leader.tile = { col: 3, row: 4 };
  leader.hp = 1;
  enemy.tile = { col: 4, row: 4 };
  await saveBattle(fixture);
  const session = await createController(),
    commands = Session.commands(session);
  await commands.end_turn();
  assert.equal(state(session).turn, "ai");
  assert.equal(state(session).winner, "playerDefeat");
  assert(Story.dialogue_pending(state(session).campaign));
  commands.dialogue(false);
  assert.deepEqual((await loadBattle()).game.captureState(), snapshot(session));
  Session.close(session);
});
test("Rejected replacement preserves undo and pending turns", async () => {
  const session = await createController(),
    commands = Session.commands(session),
    observed = observe(session);
  move(session, "u3", "3,1");
  const before = snapshot(session),
    save = await readSave();
  assert.throws(
    () => commands.reset(undefined, undefined, "unknown"),
    sessionException(/Unknown battlefield/),
  );
  assert.deepEqual(snapshot(session), before);
  assert.equal(await readSave(), save);
  assert(observed.view.can_undo);
  commands.undo();
  const turn = commands.end_turn();
  assert.throws(
    () => commands.reset(undefined, undefined, "unknown"),
    sessionException(/Unknown battlefield/),
  );
  await turn;
  assert.equal(state(session).round, 2);
  assert.equal(state(session).turn, "player");
  Session.close(session);
  observed.unsubscribe();
});
test("Publications are detached and availability follows the revision", async () => {
  const session = await createController(),
    commands = Session.commands(session),
    observed = observe(session),
    opening = observed.view;
  assert.equal(Reflect.get(commands, "active"), void 0);
  assert.equal(Reflect.get(opening.game, "move"), void 0);

  commands.choose("1,2");
  assert.equal(observed.view.game, opening.game);
  commands.focus("2,2");
  assert.equal(observed.view.game, opening.game);
  Session.message(session, "Inspect the field.");
  assert.equal(observed.view.game, opening.game);
  await commands.set_sound(false);
  assert.equal(observed.view.game, opening.game);
  commands.clear(false);
  assert.equal(observed.view.game, opening.game);
  move(session, "u3", "3,1");
  assert.notEqual(observed.view.game, opening.game);
  assert.equal(cellKey(Native.unit(opening.game, "u3")?.tile), "1,2");
  assert.equal(cellKey(Native.unit(observed.view.game, "u3")?.tile), "3,1");
  assert(observed.view.can_undo);
  assert.equal(observed.view.map_focus.selected, "u3");
  assert.equal(observed.view.map_focus.inspected, "3,1");
  const moved = observed.view.game;
  Session.message(session, "Inspect the field.");
  assert.equal(observed.view.game, moved);
  assert.equal(observed.view.status, "Inspect the field.");
  commands.undo();
  assert.equal(cellKey(Native.unit(observed.view.game, "u3")?.tile), "1,2");
  assert.equal(observed.view.can_undo, false);
  commands.choose("1,2");
  commands.choose("0,2");
  const recruitment = Presentation.terrain_panel(
    catalog,
    observed.view.game,
    observed.view.map_focus.selected,
    observed.view.map_focus.inspected,
  );
  assert(recruitment?.recruitment);
  assert.equal(observed.view.map_focus.inspected, "0,2");
  const inspected = observed.view;
  await commands.end_turn();
  assert.equal(Native.state(inspected.game).turn, "player");
  assert.equal(Native.state(inspected.game).round, 1);
  assert.equal(state(session).turn, "player");
  assert.equal(state(session).round, 2);
  Session.close(session);
  observed.unsubscribe();
});
test("Browser commands are player-only and previews cannot mutate rules", async () => {
  const fixture = new Game(),
    enemy = fixture.addUnit("soldier", "ai", "2,3");
  await saveBattle(fixture);
  const paused = pauseTurn(),
    session = await createController(paused.pause),
    commands = Session.commands(session);
  const preview = required(Native.actions(Session.view(session).interaction_game, "u2")).attacks[0];
  assert(preview);
  const copied = structuredClone(preview);
  const damage = TYPES.soldier.attacks[0].damage;
  copied.attack.damage = 999;
  assert.equal(TYPES.soldier.attacks[0].damage, damage);

  const before = snapshot(session);
  move(session, enemy.id, "2,2");
  assert.deepEqual(snapshot(session), before);
  assert.equal(Session.view(session).map_focus.selected, void 0);
  assert.equal(Session.view(session).map_focus.inspected, "2,2");
  const pending = commands.end_turn(),
    enemyTurn = snapshot(session);
  assert.equal(state(session).turn, "ai");
  assert.equal(Session.view(session).can_undo, false);
  move(session, enemy.id, "2,2");
  const recovery = await readSave();
  commands.choose("2,2");
  assert.equal(Session.view(session).map_focus.inspected, void 0);
  assert.equal(Session.view(session).map_focus.selected, void 0);
  assert.equal(await readSave(), recovery);
  assert.deepEqual(snapshot(session), enemyTurn);
  commands.recruit("soldier", "10,2");
  assert.equal(Session.view(session).status, "This unit cannot act now.");
  assert.deepEqual(snapshot(session), enemyTurn);
  paused.release();
  await pending;
  assert.equal(state(session).turn, "player");
  Session.close(session);
});
test("An accepted command with failed inspection retains the coherent view and saved flash checkpoint", async () => {
  const session = await createController(),
    commands = Session.commands(session),
    observed = observe(session);
  commands.reset(undefined, undefined, "mirror");
  const opening = observed.view.game,
    before = snapshot(session),
    saved = await readSave();
  assert(saved);
  const actor = required(Native.unit(opening, "u3"));
  assert(Object.hasOwn(Native.reachable(opening, actor.id), "3,1"));
  commands.choose(cellKey(actor.tile));
  const originalDispatch = Domain.NativeEngineFactory.dispatch_update;
  const dispatch = vi
    .spyOn(Domain.NativeEngineFactory, "dispatch_update")
    .mockImplementation((...args) => {
      const result = originalDispatch(...args);
      return { ...result, _0: { ...result._0, inspection: undefined } };
    });
  const release = vi.spyOn(Domain.NativeEngineFactory, "dispose_battle");
  const inspection = vi.spyOn(Domain.NativeEngineFactory, "inspect").mockImplementationOnce(() => {
    throw new Error("Native inspection unavailable after command acceptance");
  });
  commands.choose("3,1");
  const failed = observed.view;
  assert.match(required(failed.failure), /Native inspection unavailable after command acceptance/);
  assert.equal(dispatch.mock.calls.length, 1);
  assert.equal(FixtureJson.to_command(dispatch.mock.calls[0]?.at(-2)).kind, "move");
  assert.equal(inspection.mock.calls.length, 1);
  assert.equal(release.mock.calls.length, 1);
  assert.equal(failed.game, opening);

  assert.deepEqual(Native.state(failed.game), before);
  assert.equal(Native.unit(opening, actor.id)?.tile, actor.tile);
  assert.equal(await readSave(), saved);
  assert.equal(failed.can_undo, false);
  commands.choose("3,1");
  assert.throws(() => commands.reset(undefined, undefined, "river-valley"));
  assertPhaseError(await commands.end_turn());
  assert.equal(Session.view(session).interaction_game, opening);
  assert.equal(Session.view(session).failure, failed.failure);
  assert.equal(await readSave(), saved);
  assert.equal(dispatch.mock.calls.length, 1);
  Session.close(session);
  Session.close(session);
  assert.equal(release.mock.calls.length, 1);
  observed.unsubscribe();
});
test("Unmounted sessions cannot overwrite replacements", async () => {
  const paused = pauseTurn(),
    previous = await createController(paused.pause),
    pending = Session.commands(previous).end_turn();
  Session.close(previous);
  const replacement = await createController(),
    commands = Session.commands(replacement);
  commands.reset(undefined, undefined, "river-valley");
  const save = await readSave();
  paused.release();
  await pending;
  assert.equal(await readSave(), save);
  assert.equal(state(replacement).mapId, "river-valley");
  assert.equal(state(replacement).round, 1);
  assert.throws(
    () => Session.commands(previous).reset(undefined, undefined, "mirror"),
    sessionException(/session has closed/),
  );
  assert.throws(
    () => Session.commands(previous).focus("3,1"),
    sessionException(/session has closed/),
  );
  Session.close(replacement);
});
async function settleCallbacks() {
  await Promise.all(controllers.map(Session.saved));
  for (let turn = 0; turn < 24; turn++) await Promise.resolve();
}
test("Replay is consumed once and replacement discards pending initialization", async () => {
  await saveBattle(new Game(), true);
  const replacement = await createController(),
    observed = observe(replacement),
    commands = Session.commands(replacement);
  commands.reset(undefined, undefined, "river-valley");
  Session.initialize(replacement);
  Session.initialize(replacement);
  assert.equal(state(replacement).mapId, "river-valley");
  assert.equal(state(replacement).turn, "player");
  assert.equal(state(replacement).round, 1);
  Session.close(replacement);
  observed.unsubscribe();
  await saveBattle(new Game(), true);
  const paused = pauseTurn(),
    replay = await createController(paused.pause);
  Session.initialize(replay);
  Session.initialize(replay);
  assert.equal(state(replay).turn, "ai");
  const checkpoint = await readSave();
  Session.commands(replay).dialogue(false);
  Session.commands(replay).previous_dialogue();
  assert.equal(await readSave(), checkpoint);
  assert((await loadBattle()).pendingEnemy);
  const completed = completes(
    replay,
    (current) => current.turn === "player" && current.round === 2,
  );
  paused.release();
  await completed;
  await settleCallbacks();
  assert.equal(state(replay).round, 2);
  assert.equal(JSON.parse(required(await readSave())).pendingEnemy, false);
  Session.close(replay);
});
test("Initialization is idempotent and preserves native-URL pixel artwork", async () => {
  const session = await createController(),
    observed = observe(session);
  const opening = observed.view,
    artwork = opening.artwork;
  const frame = required(Art.unit_frame(artwork, "player", "commander"));

  assert.equal(frame.width, 64);
  assert.equal(frame.height, 64);
  assert(!frame.url.startsWith("data:"));
  const images = vi.fn();
  vi.stubGlobal(
    "Image",
    class {
      constructor() {
        images();
      }
    },
  );
  Session.initialize(session);
  const saved = await readSave(),
    initialized = observed.view;
  Session.initialize(session);
  await settleCallbacks();
  assert.equal(observed.view, initialized);
  assert.equal(observed.view.artwork, artwork);
  assert.equal(observed.view.game, opening.game);
  assert.equal(Art.unit_frame(artwork, "player", "commander"), frame);
  assert.equal(await readSave(), saved);
  assert.equal(images.mock.calls.length, 0);
  assert.equal(Reflect.get(Session.commands(session), "set_style"), void 0);
  Session.close(session);
  observed.unsubscribe();
});
test("Non-human scenarios remain command-addressable while campaign maps use their own launcher", async () => {
  const scenarios = Catalog.scenario_maps(catalog);
  const hidden = scenarios.filter((map) => map.human === false);
  const hiddenId = required(hidden[0]).id;
  const session = await createController(),
    commands = Session.commands(session);
  commands.reset(undefined, undefined, hiddenId);
  assert.equal(state(session).mapId, hiddenId);
  const visibleId = required(scenarios.find((map) => map.human)).id;
  commands.reset(undefined, undefined, visibleId);
  assert.equal(state(session).mapId, visibleId);
  const campaign = required(Catalog.campaigns(catalog)[0]),
    chapter = required(campaign.missions[0]);
  assert(!scenarios.some((map) => map.id === chapter.map.id));
  assert.throws(
    () => commands.reset(undefined, undefined, chapter.map.id),
    sessionException(/Unknown battlefield/),
  );
});
test("Reentrant reset during a command cannot publish obsolete feedback or spend replacement undo", async () => {
  const session = await createController(),
    commands = Session.commands(session);
  let replaced = false;
  const unsubscribe = Session.subscribe(session, (view) => {
    if (!replaced && cellKey(Native.unit(view.game, "u3")?.tile) === "3,1") {
      replaced = true;
      commands.reset(undefined, undefined, "river-valley");
      Session.rendered(session, Session.view(session).revision);
    }
  });
  move(session, "u3", "3,1");
  assert(replaced);
  assert.equal(state(session).mapId, "river-valley");
  assert.equal(Session.view(session).can_undo, false);
  assert.deepEqual((await loadBattle()).game.captureState(), snapshot(session));
  assert.equal(
    Session.view(session).status,
    "Select a unit. Capture villages. Defeat the red commander.",
  );
  assert.equal(
    motion.tile.mock.calls.filter(([_sound, tile, kind]) => tile === "3,1" && kind === "move")
      .length,
    0,
  );
  unsubscribe();
});
test("Immediate render acknowledgement observes effects once and stale acknowledgements do nothing", async () => {
  const session = await createController(),
    commands = Session.commands(session);
  let acknowledgements = 0;
  const listener = Session.subscribe(session, (view) => {
    acknowledgements++;
    Session.rendered(session, view.revision);
  });
  move(session, "u3", "3,1");
  const current = Session.view(session);
  Session.rendered(session, current.revision - 1);
  Session.rendered(session, current.revision);
  Session.rendered(session, current.revision);
  assert.equal(cellKey(Native.unit(current.game, "u3")?.tile), "3,1");
  assert(acknowledgements > 1);
  console.log("MOTION CALLS", JSON.stringify(motion.tile.mock.calls));
  assert.equal(
    motion.tile.mock.calls.filter(([_sound, tile, kind]) => tile === "3,1" && kind === "move")
      .length,
    1,
  );
  assert.equal(Session.view(session), current);
  listener();
});
test("Optional browser tools share player commands and return detached JSON", async () => {
  const fixture = new Game(),
    enemy = fixture.addUnit("soldier", "ai", "2,3");
  fixture.unit("u3").items = [
    {
      id: "unicode-token",
      name: "Nera’s λ🦀",
      description: "A test token",
      attacks: [],
      armor: { blade: -1, pierce: 0, impact: 0 },
    },
  ];
  await saveBattle(fixture);
  const session = await createController(),
    tools = new Map(),
    removed = [];
  vi.stubGlobal("document", {
    modelContext: {
      registerTool: (tool) => tools.set(tool.name, tool),
      unregisterTool: (name) => removed.push(name),
    },
  });
  const close = Tools.register(session),
    execute = (name, input) => required(tools.get(name)).execute(input);
  assert.deepEqual([...tools.keys()].sort(), [
    "advance_dialogue",
    "advance_unit",
    "attack_unit",
    "end_turn",
    "get_battle_state",
    "get_unit_actions",
    "move_unit",
    "recruit_unit",
    "start_campaign",
    "start_new_battle",
    "undo_move",
  ]);
  assert.equal(tools.get("get_battle_state").annotations.readOnlyHint, true);
  assert.equal(tools.get("get_unit_actions").inputSchema.additionalProperties, false);
  assert.match(tools.get("recruit_unit").description, /mission’s/);
  const opening = await execute("get_battle_state"),
    actions = await execute("get_unit_actions", { unitId: "u3" });
  assert.equal(opening.units.find((unit) => unit.id === "u3").items[0].name, "Nera’s λ🦀");
  assert(Array.isArray(actions.moves));
  assert(
    actions.moves.every(
      (move) =>
        Number.isInteger(move.tile.col) &&
        Number.isInteger(move.tile.row) &&
        typeof move.cost === "number",
    ),
  );
  assert(actions.moves.some(({ tile }) => cellKey(tile) === "3,1"));
  opening.units[0].hp = 999;
  assert.notEqual(state(session).units[0].hp, 999);
  const attacks = await execute("get_unit_actions", { unitId: "u2" }),
    target = required(attacks.attacks.find(({ id }) => id === enemy.id)),
    weapon = required(target.weapons.find(({ id }) => id === target.recommendedAttackId));
  assert.equal(weapon.damage, target.damage);
  assert.equal(weapon.retaliation, target.retaliation);
  assert.equal(weapon.attack.id, weapon.id);
  assert.equal(weapon.damageDetail.total, weapon.damage);
  weapon.attack.damage = 999;
  const repeated = await execute("get_unit_actions", { unitId: "u2" });
  assert.notEqual(repeated.attacks.find(({ id }) => id === enemy.id).weapons[0].attack.damage, 999);
  await assert.rejects(execute("move_unit", null), /Expected tool inputs/);
  await assert.rejects(execute("move_unit", { unitId: 3, tileId: "3,1" }), /Expected unitId/);
  await assert.rejects(execute("get_unit_actions", { unitId: "missing" }), /Unknown unit/);
  await assert.rejects(
    execute("advance_unit", { unitId: "u3", type: "missing" }),
    /Unknown advancement/,
  );
  await assert.rejects(execute("start_campaign", { id: "missing" }), /Unknown campaign/);
  await assert.rejects(
    execute("recruit_unit", { commanderId: "u1", type: "missing", tileId: "0,2" }),
    /Unknown recruit/,
  );
  await assert.rejects(
    execute("recruit_unit", {
      commanderId: "u1",
      type: "commander",
      tileId: "0,2",
    }),
    sessionException(/IllegalRecruit/),
  );
  await assert.rejects(
    execute("move_unit", { unitId: "u4", tileId: "3,1" }),
    sessionException(/cannot act/),
  );
  const moved = await execute("move_unit", { unitId: "u3", tileId: "3,1" });
  assert.equal(cellKey(moved.units.find((unit) => unit.id === "u3").tile), "3,1");
  assert.equal(Session.view(session).can_undo, true);
  await execute("undo_move");
  assert.equal(cellKey(state(session).units.find((unit) => unit.id === "u3").tile), "1,2");
  const attacked = await execute("attack_unit", {
    unitId: "u2",
    targetId: enemy.id,
    attackId: target.recommendedAttackId,
  });
  assert.equal(attacked.units.find((unit) => unit.id === "u2").attackSpent, true);
  assert.equal(
    (await execute("start_new_battle", { mapId: "river-valley" })).mapId,
    "river-valley",
  );
  assert.equal((await execute("start_new_battle")).mapId, "mirror");
  assert.equal(await execute("advance_dialogue"), null);
  close();
  close();
  await settleCallbacks();
  assert.deepEqual(removed.sort(), [...tools.keys()].sort());
  await assert.rejects(execute("get_battle_state"), /session has closed/);
});
test("Optional tool failures and late registrations cannot outlive their session", async () => {
  const paused = pauseTurn(),
    session = await createController(paused.pause);
  vi.stubGlobal("document", {});
  assert.doesNotThrow(Tools.register(session));
  const tools = new Map(),
    removed = [],
    pending = [];
  vi.stubGlobal("document", {
    modelContext: {
      registerTool(tool) {
        tools.set(tool.name, tool);
        return new Promise((resolve) => pending.push(resolve));
      },
      unregisterTool(name) {
        removed.push(name);
        return Promise.reject(new Error("optional cleanup"));
      },
    },
  });
  const close = Tools.register(session);
  const turn = tools.get("end_turn").execute();
  await paused.reached;
  close();
  Session.close(session);
  pending.forEach((resolve) => resolve());
  paused.release();
  await assert.rejects(turn, /session has closed/);
  await settleCallbacks();
  for (const name of tools.keys())
    assert.equal(removed.filter((value) => value === name).length, 1);
  vi.stubGlobal("document", {
    modelContext: {
      registerTool() {
        throw new Error("optional registration");
      },
    },
  });
  assert.doesNotThrow(Tools.register(await createController()));
});
for (const boundary of ["register-resolve", "register-reject", "unregister-reject"]) {
  test(`Optional tool replacement waits for prior ${boundary} without losing the live owner`, async () => {
    const installed = new Map(),
      pending = [],
      firstTools = new Map();
    let initial = true;
    vi.stubGlobal("document", {
      modelContext: {
        registerTool(tool) {
          if (initial) firstTools.set(tool.name, tool);
          if (initial && boundary.startsWith("register"))
            return new Promise((resolve, reject) =>
              pending.push(() => {
                if (boundary === "register-reject") reject(new Error("optional registration"));
                else {
                  installed.set(tool.name, tool);
                  resolve();
                }
              }),
            );
          installed.set(tool.name, tool);
        },
        unregisterTool(name) {
          if (initial && boundary === "unregister-reject")
            return new Promise((_resolve, reject) =>
              pending.push(() => {
                installed.delete(name);
                reject(new Error("optional cleanup"));
              }),
            );
          installed.delete(name);
        },
      },
    });
    const previous = await createController(),
      closePrevious = Tools.register(previous);
    if (boundary === "unregister-reject") await settleCallbacks();
    closePrevious();
    Session.close(previous);
    const replacement = await createController();
    Session.commands(replacement).reset(undefined, undefined, "river-valley");
    initial = false;
    const closeReplacement = Tools.register(replacement);
    assert.equal(pending.length, 11);
    pending.forEach((settle) => settle());
    await settleCallbacks();
    assert.equal(installed.size, 11);
    assert.notEqual(installed.get("get_battle_state"), firstTools.get("get_battle_state"));
    assert.equal((await installed.get("get_battle_state").execute()).mapId, "river-valley");
    await assert.rejects(firstTools.get("get_battle_state").execute(), /session has closed/);
    closePrevious();
    await settleCallbacks();
    assert.equal(installed.size, 11);
    closeReplacement();
    await settleCallbacks();
    assert.equal(installed.size, 0);
  });
}
test("Closing a superseded tool owner does not unregister the live owner", async () => {
  const installed = new Map();
  vi.stubGlobal("document", {
    modelContext: {
      registerTool: (tool) => installed.set(tool.name, tool),
      unregisterTool: (name) => installed.delete(name),
    },
  });
  const previous = await createController(),
    closePrevious = Tools.register(previous),
    replacement = await createController(),
    closeReplacement = Tools.register(replacement);
  closePrevious();
  await settleCallbacks();
  assert.equal(installed.size, 11);
  await installed.get("move_unit").execute({ unitId: "u3", tileId: "3,1" });
  assert.equal(cellKey(state(previous).units.find((unit) => unit.id === "u3").tile), "1,2");
  assert.equal(cellKey(state(replacement).units.find((unit) => unit.id === "u3").tile), "3,1");
  closeReplacement();
  await settleCallbacks();
  assert.equal(installed.size, 0);
});
test("Synchronous browser registry reentry keeps the newest complete tool set", async () => {
  const previous = await createController(),
    replacement = await createController(),
    installed = new Map();
  Session.commands(replacement).reset(undefined, undefined, "river-valley");
  let closeReplacement, firstTool;
  vi.stubGlobal("document", {
    modelContext: {
      registerTool(tool) {
        installed.set(tool.name, tool);
        if (!firstTool) {
          firstTool = tool;
          closeReplacement = Tools.register(replacement);
        }
      },
      unregisterTool: (name) => installed.delete(name),
    },
  });
  const closePrevious = Tools.register(previous);
  await settleCallbacks();
  assert.equal(installed.size, 11);
  assert.notEqual(installed.get(firstTool.name), firstTool);
  assert.equal((await installed.get("get_battle_state").execute()).mapId, "river-valley");
  closePrevious();
  await settleCallbacks();
  assert.equal(installed.size, 11);
  await assert.rejects(firstTool.execute(), /session has closed/);
  closeReplacement();
  await settleCallbacks();
  assert.equal(installed.size, 0);
});
beforeEach(() => {
  storage.clear();
  motion.tile.mockClear();
  motion.turn.mockClear();
  vi.stubGlobal("localStorage", storage);
});
afterEach(() => {
  for (const session of controllers.splice(0)) Session.close(session);
  vi.unstubAllGlobals();
});

test("Enemy-first setup persists its untouched opening and survives reload until a cancellable replacement", async () => {
  const input = {
    player: null,
    first: "ai",
    enemy: {
      policy: "predictor",
      diplomacy: "diplomatic",
      blocks: { campaign: "mission", planner: "queued", recruiter: "predictor" },
    },
  };
  const paused = pauseTurn();
  const session = await createController(paused.pause);
  Session.initialize(session);
  Session.commands(session).reset(Setup.of_json(input), ["woodland", "coastal"], "mirror");
  await paused.reached;
  const opening = snapshot(session);
  assert.equal(opening.turn, "ai");
  assert.deepEqual(opening.armies, ["woodland", "coastal"]);
  assert.equal(Session.view(session).completed_turns, 0);
  const saved = JSON.parse(await readSave());
  assert.deepEqual(saved.checkpoint.state, opening);
  assert.deepEqual(saved.setup, input);
  Session.close(session);
  const replay = pauseTurn();
  const restored = await createController(replay.pause);
  Session.initialize(restored);
  await replay.reached;
  assert.deepEqual(state(restored), opening);
  assert.deepEqual(Setup.to_json(Session.view(restored).setup), input);
  const restoredBattle = restoreBattle(saved.checkpoint);
  for (const side of ["player", "ai"])
    assert.deepEqual(
      Native.recruitment_options(Native.view(restoredBattle), side),
      Native.recruitment_options(Session.view(restored).game, side),
    );
  Session.commands(restored).reset(undefined, undefined, "mirror");
  const replacement = await readSave();
  paused.release();
  replay.release();
  await settleCallbacks();
  assert.equal(await readSave(), replacement);
  assert.equal(state(restored).turn, "player");
  assert.deepEqual(Setup.to_json(Session.view(restored).setup), Setup.to_json(Setup.default));
});

test.each([
  ["replacement", "close"],
  ["replacement", "replace"],
  ["replacement", "end-turn"],
  ["initialize", "close"],
  ["initialize", "replace"],
  ["initialize", "end-turn"],
  ["pending-replay", "end-turn"],
])(
  "Reentrant %s publication can %s without obsolete or duplicate enemy work",
  async (phase, action) => {
    const pauses = [];
    if (phase === "pending-replay") await Save.write(Save.create(), new Game().checkpoint(), true);
    const session = await createController(() => new Promise((resolve) => pauses.push(resolve)));
    if (phase === "replacement") Session.initialize(session);
    const commands = Session.commands(session);
    const setup = Setup.of_json({
      player: null,
      first: action === "end-turn" ? "player" : "ai",
      enemy: { policy: "classic", diplomacy: "diplomatic" },
    });
    let pending, pendingError;
    let armed = false,
      handled = false;
    const unsubscribe = Session.subscribe(session, (view) => {
      if (
        armed &&
        !handled &&
        (phase === "initialize" || action === "end-turn" || Native.state(view.game).turn === "ai")
      ) {
        handled = true;
        if (action === "close") Session.close(session);
        else if (action === "end-turn")
          pending = commands.end_turn().then((result) => {
            if (!isOk(result)) pendingError = new Error(Session.error_message(resultError(result)));
          });
        else commands.reset(setup, undefined, "river-valley");
      }
    });
    armed = true;
    assert.doesNotThrow(() =>
      phase === "replacement"
        ? commands.reset(setup, undefined, "mirror")
        : Session.initialize(session),
    );
    assert(handled);
    await settleCallbacks();
    assert.doesNotMatch(Session.view(session).status, /Wait for your turn/);
    assert.equal(pauses.length, action === "close" ? 0 : 1);
    if (action === "replace") assert.equal(state(session).mapId, "river-valley");
    Session.close(session);
    for (const resume of pauses) resume();
    await pending;
    if (phase === "pending-replay") assert.match(pendingError.message, /Wait for your turn/);
    else assert.equal(pendingError, undefined);
    await settleCallbacks();
    assert.equal(pauses.length, action === "close" ? 0 : 1);
    unsubscribe();
  },
);

test("Reset before initialize reuses its phase and never saves a partially executed enemy opening", async () => {
  const waits = [];
  const decisions = vi.spyOn(Domain.NativeEngineFactory, "choose_ai_command");
  const session = await createController(
    (milliseconds) => new Promise((resolve) => waits.push({ milliseconds, resolve })),
  );
  const commands = Session.commands(session);
  const setup = Setup.of_json({
    player: null,
    first: "ai",
    enemy: { policy: "classic", diplomacy: "diplomatic" },
  });
  commands.reset(setup, undefined, "mirror");
  const opening = await readSave();
  await settleCallbacks();
  assert.equal(waits.length, 1);
  assert.equal(waits[0].milliseconds, 0);
  assert.equal(
    decisions.mock.calls.length,
    1,
    "The initial decision is prepared inside the thinking deadline",
  );
  assert.equal(Native.revision(Session.view(session).interaction_game), 0);
  assertPhaseError(await commands.end_turn(), /Wait for your turn/);
  waits[0].resolve();
  await settleCallbacks();
  assert.equal(waits.length, 2);
  assert.equal(waits[1].milliseconds, 80);
  assert.equal(
    decisions.mock.calls.length,
    3,
    "Applying the prepared decision prepares the next one during its action deadline",
  );
  assert(Native.revision(Session.view(session).interaction_game) > 0);
  Session.initialize(session);
  Session.initialize(session);
  await settleCallbacks();
  assert.equal(waits.length, 2);
  assert.equal(await readSave(), opening);
  Session.close(session);
  waits[1].resolve();
  await settleCallbacks();
  assert.equal(await readSave(), opening);
});

test("A completed enemy phase releases ownership before a subscriber starts the next phase", async () => {
  const waits = [];
  const session = await createController(
    (milliseconds) => new Promise((resolve) => waits.push({ milliseconds, resolve })),
  );
  Session.initialize(session);
  const commands = Session.commands(session);
  let second;
  let armed = false;
  const unsubscribe = Session.subscribe(session, (view) => {
    if (
      armed &&
      !second &&
      !view.settling &&
      Native.state(view.interaction_game).turn === "player"
    ) {
      armed = false;
      second = commands.end_turn();
    }
  });
  const first = commands.end_turn();
  armed = true;
  for (let step = 0; !second && step < 64; step++) {
    await settleCallbacks();
    if (second) break;
    const wait = waits.shift();
    if (wait) wait.resolve();
    else await new Promise((resolve) => setTimeout(resolve, 0)); // durable save completion has no pacing timer
  }
  await first;
  assert(second, "The first phase must reach its player publication");
  assert.notEqual(first, second);
  await settleCallbacks();
  assert.equal(state(session).turn, "ai");
  assertPhaseError(await commands.end_turn(), /Wait for your turn/);
  assert.equal(required(waits[0]).milliseconds, 0);
  const revision = Native.revision(Session.view(session).interaction_game);
  waits.shift().resolve();
  await settleCallbacks();
  assert(
    Native.revision(Session.view(session).interaction_game) > revision,
    "The old completion cannot clear its successor",
  );
  Session.close(session);
  await second;
  for (const wait of waits) wait.resolve();
  await settleCallbacks();
  unsubscribe();
});

test("An enemy task reports an ordinary planning failure once and retains the flash checkpoint", async () => {
  const session = await createController();
  Session.initialize(session);
  const commands = Session.commands(session);
  let reports = 0;
  const unsubscribe = Session.subscribe(session, (view) => {
    if (view.status === "planner failed") reports++;
  });
  vi.spyOn(Domain.NativeEngineFactory, "choose_ai_command").mockImplementationOnce(() => {
    throw new Error("planner failed");
  });
  assertPhaseError(await commands.end_turn(), /planner failed/);
  assert.equal(reports, 1);
  assert.equal(Session.view(session).failure, undefined);
  const saved = JSON.parse(await readSave());
  assert(saved.pendingEnemy);
  assert.equal(saved.checkpoint.state.turn, "player");
  commands.reset(undefined, undefined, "mirror");
  assert.equal(state(session).turn, "player");
  assert.equal(JSON.parse(await readSave()).pendingEnemy, false);
  unsubscribe();
});

test("EndTurn that resolves the campaign persists the accepted result without starting AI", async () => {
  const fixture = Game.startCampaign("green-accord");
  fixture.advanceDialogue(true);
  const capture = required(
    mission(fixture).scriptedEvents.find((event) => event.objective?.intent.capture),
  ).objective.intent.capture;
  for (const tile of capture)
    required(fixture.tile(tile.hex ?? tile.location ?? tile)).owner = "player";
  fixture.campaign.flags["lost-tools"] = [{ state: true, stage: 0, turn: 1 }];
  await Save.write(Save.create(), fixture.checkpoint(), false);
  const clock = vi.fn(() => Promise.resolve());
  const session = await createController(clock);
  assert.equal(state(session).winner, null);
  await Session.commands(session).end_turn();
  assert.equal(state(session).winner, "playerVictory");
  assert.equal(clock.mock.calls.length, 0);
  const saved = JSON.parse(await readSave());
  assert.equal(saved.pendingEnemy, false);
  assert.deepEqual(saved.checkpoint.state, state(session));
});

test("An active enemy phase retains player flash save regardless of first-side preferences", async () => {
  const fixture = new Game();
  const setup = {
    player: null,
    first: "ai",
    enemy: { policy: "classic", diplomacy: "diplomatic" },
  };
  await Save.write(Save.create(), fixture.checkpoint(), false, Setup.of_json(setup));
  const paused = pauseTurn();
  const session = await createController(paused.pause);
  const opening = snapshot(session);
  const pending = Session.commands(session).end_turn();
  await paused.reached;
  const saved = JSON.parse(await readSave());
  assert.equal(saved.pendingEnemy, true);
  assert.deepEqual(saved.checkpoint.state, opening);
  assert.deepEqual(saved.setup, setup);
  Session.close(session);
  paused.release();
  await pending;
});

test("Dropping carried equipment preserves the unit turn and atomically persists the ground pile", async () => {
  const game = new Game();
  const actor = required(
    game.units.find((unit) => unit.side === "player" && unit.role !== "leader"),
  );
  actor.items = [
    {
      id: "fixture-token",
      name: "Token",
      description: "A transferable token.",
      attacks: [],
      armor: { blade: -1, pierce: 0, impact: 0 },
    },
  ];
  const checkpoint = game.checkpoint();
  await Save.write(Save.create(), checkpoint, false);
  const session = await createController(),
    view = Session.view(session),
    before = required(Native.unit(view.game, actor.id));
  Session.commands(session).drop(actor.id, "fixture-token");
  const after = required(Native.unit(Session.view(session).interaction_game, actor.id));
  assert.deepEqual(after.items ?? [], []);
  assert.equal(after.moved, before.moved);
  assert.equal(after.attackSpent, before.attackSpent);
  assert.deepEqual(
    state(session).groundItems.map(({ tile, item }) => [tile, item.id]),
    [[actor.tile, "fixture-token"]],
  );
  assert.match(Session.view(session).status, /Item dropped/);
  const persisted = JSON.parse(required(await readSave()));
  assert.equal(persisted.pendingEnemy, false);
  assert.deepEqual(persisted.checkpoint.state.groundItems, state(session).groundItems);
});

function assertPhaseError(result, pattern) {
  assertResultError(result, Session.error_message, pattern);
}

function sessionException(pattern) {
  return (error) => {
    assert.equal(error.MEL_EXN_ID, Session.Failed);
    assert.match(Session.error_message(error._1), pattern);
    return true;
  };
}
