import * as Save from "@frontend/Save";
import * as Native from "./native-adapter.mjs";
import * as Store from "./storage-adapter.mjs";
import { factory } from "./native-fixture.mjs";
import { fromJSON as from, toJSON as to } from "./json-fixture.mjs";
export * from "@frontend/Save";
export const migrate = (catalog, value) => {
  const [save, changed] = Save.migrate(value);
  return [{ ...save, checkpoint: to("SavedGame", save.checkpoint) }, changed];
};
export const write = async (owner, checkpoint, pending, setup) => {
  await Store.initialize({ turns: [], chooseNewGame: false });
  Save.write(setup, owner, from("SavedGame", checkpoint), pending);
  return Save.saved(owner);
};
export const write_json = async (owner, checkpoint, pending, setup) => {
  await Store.initialize({ turns: [], chooseNewGame: false });
  Save.write_json(setup, undefined, owner, checkpoint, pending);
  return Save.saved(owner);
};
export const load = (_catalog, restore, fresh, dispose) =>
  Save.load(
    (v) => restore(to("SavedGame", v)),
    (request) => Native.validate_undo(factory, request),
    fresh,
    dispose,
  ).then((loaded) => ({ ...loaded, problem: Save.problem(loaded.saves) }));

export const flashText = async () => (await Store.read_session())[0];
