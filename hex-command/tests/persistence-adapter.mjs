// Drive the OCaml persistence boundary with the same fixtures as upstream.
import * as Save from "@frontend/Save";
import * as Store from "./storage-adapter.mjs";
import * as Persistence from "./save-adapter.mjs";
import * as Native from "./native-adapter.mjs";
import * as Catalog from "./catalog-adapter.mjs";
import { catalog } from "./native-fixture.mjs";
import { Game } from "./game-fixture.mjs";
export const nativeValue = (value) => {
  if (value?.ok === false) throw new Error(value.error.message);
  return value?.ok === true ? value.value : value;
};
export const createTestFactory = () => ({
  catalog: () => ({ ok: true, value: Catalog.raw(catalog) }),
});
export const LiveGame = {
  create: (_factory, start) =>
    start.kind === "campaign" ? Game.startCampaign(start.id) : new Game(start.map, start.revision),
  restore: (owner, checkpoint) => {
    if (owner.restore) nativeValue(owner.restore(checkpoint));
    return Game.fromCheckpoint(checkpoint);
  },
};
export const loadBattle = async (owner) => {
  const loaded = await Persistence.load(
    undefined,
    (checkpoint) => LiveGame.restore(owner, checkpoint),
    () => new Game(),
    (game) => game.dispose(),
  );
  const problem = Save.problem(loaded.saves);
  if (problem) problems.set(problem, loaded.saves);
  return { ...loaded, problem, pendingEnemy: loaded.pending_enemy };
};
export const saveBattle = async (game, pending = false) => {
  if (game.turn === "player" || game.winner) {
    const owner = Save.create();
    Save.write_json(undefined, undefined, owner, Native.save_json(game.nativeBattle()), pending);
    await Save.saved(owner);
  }
};

export const SAVE_KEY = Save.key;
export const MAPS = Object.fromEntries(Catalog.maps(catalog).map((map) => [map.id, map]));

export const readSave = async () => (await Store.read_session())[0] ?? null;
export const storeSave = (text) => Store.store_save(text);
const problems = new WeakMap();
export const repairImport = (problem, target) => Save.repair(problems.get(problem), target);
