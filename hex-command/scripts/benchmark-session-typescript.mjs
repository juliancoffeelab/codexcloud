import init, { NativeEngineFactory } from "@wasm/hex_command.js";
import { createNativeEngineFactory } from "@upstream/game/native-engine.ts";
import { startEngine } from "@upstream/game/engine-startup.ts";
let BattleController, TYPES, TERRAIN, get, transaction, readSave, claimAction;
const cell = (tile) => `${tile.col},${tile.row}`;
let factory,
  scenario = "mirror",
  lookupSession;
const state = (session) => get(session.view).interactionGame;
export async function initialize(map = "mirror") {
  scenario = map;
  localStorage.clear();
  localStorage.setItem("hex-command-sound-enabled", "false");
  await init({ module_or_path: new URL("hex_command_bg.wasm", import.meta.url) });
  factory = createNativeEngineFactory(new NativeEngineFactory(), {
    responseOwnership: "transferred",
  });
  startEngine(() => Promise.resolve(factory));
  [{ BattleController }, { TYPES, TERRAIN }, { get }, { transaction, readSave }, { claimAction }] =
    await Promise.all([
      import("@upstream/controller/battle.ts"),
      import("@upstream/game/catalog.ts"),
      import("svelte/store"),
      import("@upstream/save/storage.ts"),
      import("@upstream/controller/action-presentation.ts"),
    ]);
}
export async function prepare(checkpoint) {
  await transaction(["state", "entries", "turns"], "readwrite", (tx) => {
    for (const name of ["state", "entries", "turns"]) tx.objectStore(name).clear();
    const state = tx.objectStore("state");
    state.put(2, "initialized");
    if (checkpoint) state.put(checkpoint, "current");
  });
}
async function create() {
  await prepare();
  const session = await BattleController.create(() => Promise.resolve(), factory);
  session.view.subscribe((view) =>
    queueMicrotask(() => {
      view.combatPlayback?.skip();
      if (view.actionPresentation) claimAction(view.actionPresentation).complete();
    }),
  );
  if (scenario !== "mirror") await session.commands.reset(scenario);
  session.initialize();
  await session.saved();
  return session;
}
export async function checkpoint() {
  const session = await create();
  try {
    return await readCheckpoint();
  } finally {
    session.dispose();
  }
}
export async function readCheckpoint() {
  return readSave();
}
export async function cases() {
  lookupSession = await create();
  const view = state(lookupSession),
    kinds = Object.keys(TYPES),
    terrains = Object.keys(TERRAIN),
    tiles = view.tiles.map((t) => t.id),
    side = view.units[0].side,
    id = view.units[0].id;
  const units = kinds.map((type) => ({ type })),
    costTiles = terrains.map((terrain) => ({ terrain }));
  let index = 0;
  return [
    {
      name: "movement cost",
      call: () => {
        const n = index++;
        return view.cost(units[n % units.length], costTiles[n % costTiles.length]);
      },
    },
    { name: "neighbors", call: () => view.neighbors(tiles[index++ % tiles.length]) },
    { name: "recruitment", call: () => view.recruitmentOptions(side) },
    { name: "placements", call: () => view.recruitTiles(id, kinds[index++ % kinds.length]) },
    {
      name: "session commands lifecycle",
      defer: true,
      call: async () => {
        const session = await create();
        try {
          const actions = session.commands,
            initial = state(session),
            actor =
              initial.units.find((u) => u.id === "u3") ??
              initial.units.find((u) => u.side === "player");
          const occupied = new Set(initial.units.map((u) => cell(u.tile))),
            reachable = initial.reachable(actor.id);
          const destination = initial.tiles
            .map((t) => t.id)
            .find((id) => reachable.has(id) && id !== cell(actor.tile) && !occupied.has(id));
          actions.choose(cell(actor.tile));
          actions.choose(destination);
          actions.undoMove();
          await actions.endTurn();
          await session.saved();
        } finally {
          session.dispose();
        }
      },
    },
  ];
}
export function close() {
  if (lookupSession) lookupSession.dispose();
  factory.dispose();
}
