import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import type * as SessionModule from "@frontend/Session";
import type * as ArtModule from "@frontend/Art";
import type * as PresentationModule from "@frontend/Presentation";
import type * as StoryModule from "@frontend/Story";
import { Native, Catalog, factory, catalog, restoreBattle } from "./native-fixture";
import { Game, TYPES } from "./game-fixture";
import { SAVE_KEY, saveBattle, loadBattle } from "./legacy-save";
import * as Save from "@frontend/Save";
import { required, TestStorage } from "./helpers";
import type { State, AttackDetails } from "../rust/pkg/hex_command";
const motion = vi.hoisted(() => ({ tile: vi.fn(), turn: vi.fn() }));
vi.mock("@frontend/Motion", () => ({ animate_tile: motion.tile, animate_turn: motion.turn }));
let Session: typeof SessionModule,
  Art: typeof ArtModule,
  Presentation: typeof PresentationModule,
  Story: typeof StoryModule;
const storage = new TestStorage(),
  immediatePause = async (_milliseconds: number) => {};
const controllers: SessionModule.t[] = [];
function createController(clock: SessionModule.clock = immediatePause) {
  const session = Session.create(clock, factory);
  controllers.push(session);
  return session;
}
function state(session: SessionModule.t) {
  return Native.state(Session.view(session).game);
}
function snapshot(session: SessionModule.t) {
  return structuredClone(state(session)) as State;
}
function observe(session: SessionModule.t) {
  let latest: SessionModule.model | undefined;
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
/** Exercise UI gestures directly rather than duplicate controller command policy. */
function move(session: SessionModule.t, id: string, tile: string) {
  const commands = Session.commands(session);
  if (Session.view(session).selected !== id)
    commands.choose(required(Native.unit(Session.view(session).game, id)).tile);
  commands.choose(tile);
}
function panel(view: SessionModule.model) {
  return Presentation.unit_panel(
    view.catalog,
    view.game,
    view.selected,
    view.target,
    view.revision,
  );
}
function mission(game: Game) {
  return Story.mission(catalog, required(game.campaign)).source;
}
function pauseTurn(callToPause = 1) {
  let release!: () => void, markReached!: () => void;
  const reached = new Promise<void>((resolve) => {
      markReached = resolve;
    }),
    first = new Promise<void>((resolve) => {
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
function completes(
  session: SessionModule.t,
  predicate: (state: Native.Immutable<State>) => boolean,
) {
  return new Promise<void>((resolve) => {
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
  const session = createController(),
    observed = observe(session),
    commands = Session.commands(session);
  commands.reset("developer");
  const opening = snapshot(session),
    player = required(opening.units.find((unit) => unit.side === "player")),
    enemy = required(opening.units.find((unit) => unit.side === "enemy"));
  commands.choose(player.tile);
  const details = panel(observed.view);
  assert(details !== 0 && details.TAG === 1);
  assert.equal(details._0.action_status, "Ready");
  assert(Object.hasOwn(Native.reachable(observed.view.game, player.id), "0,2"));
  assert(
    Presentation.battlefield(catalog, observed.view.game, player.id, undefined).some((tile) =>
      tile.classes.includes("reachable"),
    ),
  );
  move(session, player.id, "0,2");
  assert(observed.view.can_undo);
  commands.undo();
  assert.deepEqual(snapshot(session), opening);
  commands.choose(enemy.tile);
  const combat = panel(observed.view);
  assert(combat !== 0 && combat.TAG === 2);
  commands.attack(required(Native.best_attack(observed.view.game, player.id, enemy.id)).attack.id);
  assert(required(state(session).units.find((unit) => unit.id === enemy.id)).hp < enemy.hp);
  assert.deepEqual(loadBattle().game.captureState(), snapshot(session));
  await commands.end_turn();
  assert(state(session).round > opening.round || state(session).winner);
  assert.deepEqual(loadBattle().game.captureState(), snapshot(session));
  Session.close(session);
  observed.unsubscribe();
});

test("Dialogue navigation persists its cursor without advancing gameplay", async () => {
  const session = createController(),
    observed = observe(session),
    commands = Session.commands(session);
  commands.start_campaign("green-accord");
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
  await assert.rejects(commands.end_turn(), /Wait for your turn/);
  assert.throws(() => commands.undo(), /Finish the conversation/);
  const restored = createController();
  assert.equal(required(state(restored).campaign).dialogueIndex, 1);
  Session.commands(restored).previous_dialogue();
  const rewound = createController(),
    rewoundCommands = Session.commands(rewound);
  assert.equal(required(state(rewound).campaign).dialogueIndex, 0);
  while (Story.dialogue_pending(state(rewound).campaign)) rewoundCommands.dialogue(false);
  const finished = snapshot(rewound),
    finishedView = observe(rewound),
    publication = finishedView.view,
    save = storage.getItem(SAVE_KEY);
  rewoundCommands.dialogue(false);
  assert.equal(finishedView.view, publication);
  assert.equal(storage.getItem(SAVE_KEY), save);
  rewoundCommands.previous_dialogue();
  assert.deepEqual(snapshot(rewound), finished);
  rewoundCommands.reset("mirror");
  const skirmish = snapshot(rewound);
  rewoundCommands.previous_dialogue();
  assert.deepEqual(snapshot(rewound), skirmish);
  Session.close(rewound);
  assert.throws(() => rewoundCommands.previous_dialogue(), /session has closed/);
  Session.close(session);
  Session.close(restored);
  finishedView.unsubscribe();
  observed.unsubscribe();
});

test("Stray dialogue cannot overwrite a paused turn checkpoint", async () => {
  const paused = pauseTurn(),
    session = createController(paused.pause),
    observed = observe(session),
    commands = Session.commands(session),
    before = snapshot(session);
  const pending = commands.end_turn(),
    checkpoint = storage.getItem(SAVE_KEY),
    enemyView = observed.view;
  commands.dialogue(false);
  commands.dialogue(true);
  commands.previous_dialogue();
  assert.equal(observed.view, enemyView);
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  const loaded = loadBattle();
  assert(loaded.pendingEnemy);
  assert.deepEqual(loaded.game.captureState(), before);
  Session.close(session);
  paused.release();
  await pending;
  observed.unsubscribe();
});

test("Enemy-turn dialogue keeps the recovery save", async () => {
  const fixture = Game.startCampaign("ashen-oath", 1);
  fixture.turn = "enemy";
  fixture.campaign!.dialogue = [...required(mission(fixture).refuge).escalation];
  fixture.campaign!.dialogueIndex = 0;
  const native = restoreBattle({ seed: 17, state: fixture.captureState() });
  Save.write(Save.create(), Native.save(native), false);
  Native.dispose(native);
  const paused = pauseTurn(),
    session = createController(paused.pause),
    observed = observe(session),
    commands = Session.commands(session);
  Session.initialize(session);
  const checkpoint = storage.getItem(SAVE_KEY);
  await paused.reached;
  assert.equal(state(session).turn, "enemy");
  assert(Story.dialogue_pending(state(session).campaign));
  const cursor = required(state(session).campaign).dialogueIndex;
  commands.dialogue(false);
  assert.equal(required(state(session).campaign).dialogueIndex, cursor + 1);
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  commands.previous_dialogue();
  assert.equal(required(state(session).campaign).dialogueIndex, cursor);
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  assert.equal(loadBattle().game.turn, "enemy");
  Session.close(session);
  paused.release();
  await Promise.resolve();
  observed.unsubscribe();
});

test("Completed vigil publishes and persists its outcome", async () => {
  const fixture = Game.startCampaign("green-accord", 1);
  fixture.advanceDialogue(true);
  fixture.units = fixture.units.filter((unit) => unit.side === "player");
  const goal = required(mission(fixture).goals.find((goal) => goal.kind === "vigil"));
  assert.equal(goal.kind, "vigil");
  if (goal.kind !== "vigil") throw new Error("Vigil fixture expected");
  const lark = required(fixture.units.find((unit) => unit.name === "Lark")),
    occupant = fixture.at(goal.tile);
  if (occupant && occupant.id !== lark.id) occupant.tile = "5,4";
  lark.tile = goal.tile;
  required(fixture.tile(goal.tile)).owner = "player";
  fixture.round = 3;
  required(fixture.campaign).vigil = { heldTurns: 2, lastCountedRound: 3 };
  saveBattle(fixture);
  assert.equal(loadBattle().game.mapId, fixture.mapId);
  const session = createController(),
    observed = observe(session);
  await Session.commands(session).end_turn();
  assert.equal(state(session).winner, "player");
  assert.equal(observed.view.status, state(session).campaign?.outcome);
  assert(Story.dialogue_pending(state(session).campaign));
  Session.commands(session).dialogue(false);
  assert.equal(
    required(loadBattle().game.campaign).dialogueIndex,
    required(state(session).campaign).dialogueIndex,
  );
  Session.close(session);
  observed.unsubscribe();
});

test("Finished enemy-turn battles persist outcome dialogue", async () => {
  const fixture = Game.startCampaign("green-accord");
  fixture.advanceDialogue(true);
  fixture.units = fixture.units.filter((unit) => unit.role === "leader" || unit.name === "Lark");
  const leader = required(
      fixture.units.find((unit) => unit.side === "player" && unit.role === "leader"),
    ),
    enemy = fixture.addUnit("soldier", "enemy", "4,4");
  leader.tile = "3,4";
  leader.hp = 1;
  enemy.tile = "4,4";
  saveBattle(fixture);
  const session = createController(),
    commands = Session.commands(session);
  await commands.end_turn();
  assert.equal(state(session).turn, "enemy");
  assert.equal(state(session).winner, "enemy");
  assert(Story.dialogue_pending(state(session).campaign));
  commands.dialogue(false);
  assert.deepEqual(loadBattle().game.captureState(), snapshot(session));
  Session.close(session);
});

test("Rejected replacement preserves undo and pending turns", async () => {
  const session = createController(),
    commands = Session.commands(session),
    observed = observe(session);
  move(session, "u3", "3,1");
  const before = snapshot(session),
    save = storage.getItem(SAVE_KEY);
  assert.throws(() => commands.reset("unknown"), /Unknown battlefield/);
  assert.deepEqual(snapshot(session), before);
  assert.equal(storage.getItem(SAVE_KEY), save);
  assert(observed.view.can_undo);
  commands.undo();
  const turn = commands.end_turn();
  assert.throws(() => commands.reset("unknown"), /Unknown battlefield/);
  await turn;
  assert.equal(state(session).round, 2);
  assert.equal(state(session).turn, "player");
  Session.close(session);
  observed.unsubscribe();
});

test("Publications are immutable and availability follows the revision", async () => {
  const session = createController(),
    commands = Session.commands(session),
    observed = observe(session),
    opening = observed.view;
  assert.equal(Reflect.get(commands, "active"), undefined);
  assert.equal(Reflect.get(opening.game, "move"), undefined);
  assert(Object.isFrozen(commands));
  assert(Object.isFrozen(opening.game));
  assert.equal(Reflect.set(required(Native.unit(opening.game, "u3")), "tile", "3,1"), false);
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
  assert.equal(Native.unit(opening.game, "u3")?.tile, "1,2");
  assert.equal(Native.unit(observed.view.game, "u3")?.tile, "3,1");
  assert(observed.view.can_undo);
  assert.equal(observed.view.selected, "u3");
  assert.equal(observed.view.inspected, "3,1");
  const moved = observed.view.game;
  Session.message(session, "Inspect the field.");
  assert.equal(observed.view.game, moved);
  assert.equal(observed.view.status, "Inspect the field.");
  assert.equal(Reflect.set(observed.view, "selected", "u6"), false);
  commands.undo();
  assert.equal(Native.unit(observed.view.game, "u3")?.tile, "1,2");
  assert.equal(observed.view.can_undo, false);
  commands.choose("1,2");
  commands.choose("0,2");
  const recruitment = Presentation.terrain_panel(
    catalog,
    observed.view.game,
    observed.view.selected,
    observed.view.inspected,
  );
  assert(recruitment?.recruitment);
  assert.equal(observed.view.inspected, "0,2");
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
    enemy = fixture.addUnit("soldier", "enemy", "2,3");
  saveBattle(fixture);
  const paused = pauseTurn(),
    session = createController(paused.pause),
    commands = Session.commands(session);
  const preview = required(Native.actions(Session.view(session).game, "u2")).attacks[0];
  assert(preview);
  const copied = structuredClone(preview) as AttackDetails;
  const damage = TYPES.soldier!.attacks[0]!.damage;
  copied.attack.damage = 999;
  assert.equal(TYPES.soldier!.attacks[0]!.damage, damage);
  assert(Object.isFrozen(preview.attack));
  const before = snapshot(session);
  move(session, enemy.id, "2,2");
  assert.deepEqual(snapshot(session), before);
  assert.match(Session.view(session).status, /cannot act/);
  const pending = commands.end_turn(),
    enemyTurn = snapshot(session);
  assert.equal(state(session).turn, "enemy");
  assert.equal(Session.view(session).can_undo, false);
  move(session, enemy.id, "2,2");
  commands.recruit("soldier", "10,2");
  assert.match(Session.view(session).status, /Wait for your turn/);
  assert.deepEqual(snapshot(session), enemyTurn);
  paused.release();
  await pending;
  assert.equal(state(session).turn, "player");
  Session.close(session);
});

test("Unmounted sessions cannot overwrite replacements", async () => {
  const paused = pauseTurn(),
    previous = createController(paused.pause),
    pending = Session.commands(previous).end_turn();
  Session.close(previous);
  const replacement = createController(),
    commands = Session.commands(replacement);
  commands.reset("river");
  const save = storage.getItem(SAVE_KEY);
  paused.release();
  await pending;
  assert.equal(storage.getItem(SAVE_KEY), save);
  assert.equal(state(replacement).mapId, "river");
  assert.equal(state(replacement).round, 1);
  assert.throws(() => Session.commands(previous).reset("mirror"), /session has closed/);
  assert.throws(() => Session.commands(previous).focus("3,1"), /session has closed/);
  Session.close(replacement);
});

async function settleCallbacks() {
  for (let turn = 0; turn < 24; turn++) await Promise.resolve();
}

test("Replay is consumed once and replacement discards pending initialization", async () => {
  saveBattle(new Game(), true);
  const replacement = createController(),
    observed = observe(replacement),
    commands = Session.commands(replacement);
  commands.reset("river");
  Session.initialize(replacement);
  Session.initialize(replacement);
  assert.equal(state(replacement).mapId, "river");
  assert.equal(state(replacement).turn, "player");
  assert.equal(state(replacement).round, 1);
  Session.close(replacement);
  observed.unsubscribe();
  saveBattle(new Game(), true);
  const paused = pauseTurn(),
    replay = createController(paused.pause);
  Session.initialize(replay);
  Session.initialize(replay);
  assert.equal(state(replay).turn, "enemy");
  const checkpoint = storage.getItem(SAVE_KEY);
  Session.commands(replay).dialogue(false);
  Session.commands(replay).previous_dialogue();
  assert.equal(storage.getItem(SAVE_KEY), checkpoint);
  assert(loadBattle().pendingEnemy);
  const completed = completes(
    replay,
    (current) => current.turn === "player" && current.round === 2,
  );
  paused.release();
  await completed;
  await settleCallbacks();
  assert.equal(state(replay).round, 2);
  assert.equal(JSON.parse(required(storage.getItem(SAVE_KEY))).pendingEnemy, false);
  Session.close(replay);
});

test("Initialization is idempotent and preserves immutable native-URL pixel artwork", async () => {
  const session = createController(),
    observed = observe(session);
  const opening = observed.view,
    artwork = opening.artwork;
  const frame = required(Art.unit_frame(artwork, "player", "commander"));
  assert(Object.isFrozen(artwork));
  assert(Object.isFrozen(frame));
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
  const initialized = observed.view,
    saved = storage.getItem(SAVE_KEY);
  Session.initialize(session);
  await settleCallbacks();
  assert.equal(observed.view, initialized);
  assert.equal(observed.view.artwork, artwork);
  assert.equal(observed.view.game, opening.game);
  assert.equal(Art.unit_frame(artwork, "player", "commander"), frame);
  assert.equal(storage.getItem(SAVE_KEY), saved);
  assert.equal(images.mock.calls.length, 0);
  assert.equal(Reflect.get(Session.commands(session), "set_style"), undefined);
  Session.close(session);
  observed.unsubscribe();
});

test("Hidden scenarios remain command-addressable while visible menus exclude them and campaigns", () => {
  const scenarios = Catalog.scenario_maps(catalog),
    choices = Catalog.battle_maps(catalog);
  const hidden = scenarios.filter((map) => map.hidden === true);
  assert(hidden.length > 0);
  for (const map of hidden) assert(!choices.some((choice) => choice.id === map.id));
  assert(choices.every((map) => map.hidden !== true));
  const session = createController(),
    commands = Session.commands(session);
  commands.reset("developer");
  assert.equal(state(session).mapId, "developer");
  commands.reset("river-valley");
  assert.equal(state(session).mapId, "river-valley");
  const campaign = required(Catalog.campaigns(catalog)[0]),
    chapter = required(campaign.missions[0]);
  assert(!scenarios.some((map) => map.id === chapter.map.id));
  assert.throws(() => commands.reset(chapter.map.id), /Unknown battlefield/);
});

test("Reentrant reset during a command cannot publish obsolete feedback or spend replacement undo", () => {
  const session = createController(),
    commands = Session.commands(session);
  let replaced = false;
  const unsubscribe = Session.subscribe(session, (view) => {
    if (!replaced && Native.unit(view.game, "u3")?.tile === "3,1") {
      replaced = true;
      commands.reset("river");
      Session.rendered(session, Session.view(session).revision);
    }
  });
  move(session, "u3", "3,1");
  assert(replaced);
  assert.equal(state(session).mapId, "river");
  assert.equal(Session.view(session).can_undo, false);
  assert.deepEqual(loadBattle().game.captureState(), snapshot(session));
  assert.equal(
    Session.view(session).status,
    "Select a unit. Capture villages. Defeat the red commander.",
  );
  assert.equal(
    motion.tile.mock.calls.filter(([tile, kind]) => tile === "3,1" && kind === "move").length,
    0,
  );
  unsubscribe();
});

test("Immediate render acknowledgement observes effects once and stale acknowledgements do nothing", () => {
  const session = createController(),
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
  assert.equal(Native.unit(current.game, "u3")?.tile, "3,1");
  assert(acknowledgements > 1);
  assert.equal(
    motion.tile.mock.calls.filter(([tile, kind]) => tile === "3,1" && kind === "move").length,
    1,
  );
  assert.equal(Session.view(session), current);
  listener();
});

beforeEach(async () => {
  vi.resetModules();
  [Session, Art, Presentation, Story] = await Promise.all([
    import("@frontend/Session"),
    import("@frontend/Art"),
    import("@frontend/Presentation"),
    import("@frontend/Story"),
  ]);
  storage.clear();
  motion.tile.mockClear();
  motion.turn.mockClear();
  vi.stubGlobal("localStorage", storage);
});
afterEach(() => {
  for (const session of controllers.splice(0)) Session.close(session);
  vi.unstubAllGlobals();
});
