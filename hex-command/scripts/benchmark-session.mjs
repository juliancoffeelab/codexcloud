import * as Json from "@frontend/Fixture_json";
import * as Native from "@frontend/Native";
import * as Session from "@frontend/Session";
import * as Action from "@frontend/Action_presentation";
import * as Store from "@frontend/Save_storage";
import * as Catalog from "@frontend/Catalog";
import * as Domain from "@frontend/Domain";
import * as Enum from "@frontend/Enum";
import * as Array_ from "melange/array.js";
const stringMap = Reflect.get(Domain, "String_map");
const keys = stringMap
  ? (value) => Array_.of_list(stringMap.bindings(value)).map(([key]) => key)
  : Object.keys;
const cell = (tile) => `${tile.col},${tile.row}`;
let factory,
  scenario = "mirror",
  lookupSession;
const state = (session) => Native.state(Session.view(session).interaction_game);
export async function initialize(map = "mirror") {
  scenario = map;
  localStorage.clear();
  localStorage.setItem("hex-command-sound-enabled", "false");
  const outcome = await Native.create_factory();
  if (!Json.is_ok(outcome)) throw new Error(Native.message(Json.result_error(outcome)));
  factory = Json.value(outcome);
}
export async function prepare(checkpoint) {
  await Store.Raw.transaction(["state", "entries", "turns"], "readwrite", (tx) => {
    for (const name of ["state", "entries", "turns"]) tx.objectStore(name).clear();
    const state = tx.objectStore("state");
    state.put(2, "initialized");
    if (checkpoint) state.put(checkpoint, "current");
    return Promise.resolve();
  });
}
async function create() {
  await prepare();
  const session = await Session.create_async(() => Promise.resolve(), factory);
  // This operation benchmark excludes rendering. Complete its explicit silent
  // presentation consumer; the renderer page measures real frame/audio completion.
  Session.subscribe(session, (view) =>
    queueMicrotask(() => {
      view.combat_playback?.skip();
      if (view.action) Action.complete(Action.claim(view.action));
    }),
  );
  if (scenario !== "mirror") Session.commands(session).reset(undefined, undefined, scenario);
  Session.initialize(session);
  await Session.saved(session);
  return session;
}
export async function checkpoint() {
  const session = await create();
  try {
    return await readCheckpoint();
  } finally {
    Session.close(session);
  }
}
export async function readCheckpoint() {
  const outcome = await Store.read_session();
  if (!Json.is_ok(outcome)) throw new Error(Json.result_error(outcome).detail);
  return Json.value(outcome)[0];
}
export async function cases() {
  lookupSession = await create();
  const view = Session.view(lookupSession).game,
    catalog = Native.catalog(factory);
  const kinds = Array_.of_list(Catalog.units(catalog)).map((u) => u.id);
  const terrains = Array_.of_list(Catalog.terrain(catalog)).map((t) =>
    Enum.string_of_terrain(t.id),
  );
  const initial = state(lookupSession),
    tiles = initial.tiles.map((t) => t.id),
    side = initial.units[0].side,
    id = initial.units[0].id;
  let index = 0;
  return [
    {
      name: "movement cost",
      call: () => {
        const n = index++;
        return Native.movement_cost(view, kinds[n % kinds.length], terrains[n % terrains.length]);
      },
    },
    { name: "neighbors", call: () => Native.neighbors(view, tiles[index++ % tiles.length]) },
    { name: "recruitment", call: () => Native.recruitment_options(view, side) },
    {
      name: "placements",
      call: () => Native.recruit_tiles(view, id, kinds[index++ % kinds.length]),
    },
    {
      name: "session commands lifecycle",
      defer: true,
      call: async () => {
        const session = await create();
        try {
          const actions = Session.commands(session),
            initial = state(session);
          const actor =
            initial.units.find((u) => u.id === "u3") ??
            initial.units.find((u) => Enum.string_of_side(u.side) === "player");
          const occupied = new Set(initial.units.map((u) => cell(u.tile))),
            reachable = new Set(
              keys(Native.reachable(Session.view(session).interaction_game, actor.id)),
            );
          const destination = initial.tiles
            .map((t) => t.id)
            .find((id) => reachable.has(id) && id !== cell(actor.tile) && !occupied.has(id));
          actions.choose(cell(actor.tile));
          actions.choose(destination);
          actions.undo();
          const outcome = await actions.end_turn();
          if (!Json.is_ok(outcome))
            throw new Error(Session.error_message(Json.result_error(outcome)));
          await Session.saved(session);
        } finally {
          Session.close(session);
        }
      },
    },
  ];
}
export function close() {
  if (lookupSession) Session.close(lookupSession);
  Native.dispose_factory(factory);
}
