import { cellKey } from "./hex-fixture.mjs";
import * as Native from "@frontend/Native";
import { fromJSON as from, toJSON as to, command, toArray, mapBindings } from "./json-fixture.mjs";
export * from "@frontend/Native";
import { requireResult as requireOutcome } from "./result-fixture.mjs";
export const requireResult = (result) => requireOutcome(result, Native.message);
export const create_factory = (...args) => Native.create_factory(...args).then(requireResult);
export const save_json = (...args) => requireResult(Native.save_json(...args));
export const prepare_enemy_step = (...args) => requireResult(Native.prepare_enemy_step(...args));
export function assertionNative(raw) {
  const wrapped = { ...raw };
  for (const [key, action] of Object.entries(raw)) {
    if (typeof action !== "function" || key === "message") continue;
    wrapped[key] = (...args) => {
      try {
        return action(...args);
      } catch (error) {
        if (error.MEL_EXN_ID === raw.Rejected)
          throw new Error(raw.message(error._1), { cause: error });
        throw error;
      }
    };
  }
  for (const key of [
    "create",
    "restore",
    "dispatch",
    "replace",
    "prepare_enemy_step",
    "enemy_step",
    "enemy_step_recorded",
    "save",
    "save_json",
    "validate_undo",
  ]) {
    const action = wrapped[key];
    wrapped[key] = (...args) => requireOutcome(action(...args), raw.message);
  }
  wrapped.create_factory = (...args) =>
    raw.create_factory(...args).then((result) => requireOutcome(result, raw.message));
  wrapped.on_failure = (battle, callback) =>
    raw.on_failure(battle, (error) => callback(raw.message(error)));
  return wrapped;
}
export const create = (f, start, seed) =>
  requireResult(Native.create(f, from("Start", start), seed));
export const restore = (f, saved) => requireResult(Native.restore(f, from("SavedGame", saved)));
export const replace = (b, saved) => requireResult(Native.replace(b, from("SavedGame", saved)));
export const state = (v) => to("State", Native.state(v));
export const combat = (v) => to("CombatAnimation", Native.combat(v));
export const inspection = (v) => to("BattleInspection", Native.inspection(v));
export const save = (b) => to("SavedGame", requireResult(Native.save(b)));
export const dispatch = (b, c) => requireResult(Native.dispatch(b, command(c)));
export const enemy_step = (b, o) => requireResult(Native.enemy_step(b, from("AiOptions", o)));
export const tile = (v, id) => to("Tile", Native.tile(v, cellKey(id)));
export const unit = (v, id) => to("PublicUnit", Native.unit(v, id));
export const at = (v, id) => to("PublicUnit", Native.at(v, cellKey(id)));
export const actions = (v, id) => to("UnitActions", Native.actions(v, id));
const array = (type, values) => toArray(values).map((v) => to(type, v));
export const reachable = (v, id) =>
  Object.fromEntries(mapBindings("String", Native.reachable(v, id)));
export const neighbors = (v, id) => array("Tile", Native.neighbors(v, cellKey(id)));
export const targets = (v, id) => array("PublicUnit", Native.targets(v, id));
export const recruit_tiles = (v, id, kind) => array("Tile", Native.recruit_tiles(v, id, kind));
export const attack_options = (v, id, target) =>
  array("AttackDetails", Native.attack_options(v, id, target));
export const best_attack = (v, id, target) =>
  to("AttackDetails", Native.best_attack(v, id, target));
export const attack_details = (v, id, target, weapon) =>
  to("AttackDetails", Native.attack_details(v, id, target, weapon));
export const recruitment_options = (v, side) => toArray(Native.recruitment_options(v, side));
export const promotion_options = (v, id) => toArray(Native.promotion_options(v, id));
export const income = (v, side) => Native.income(v, side);

export const campaign = (view) => to("CampaignState", Native.campaign(view));
