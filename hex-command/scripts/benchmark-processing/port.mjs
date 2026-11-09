import * as Json from "@frontend/Fixture_json";
import * as Native from "@frontend/Native";
import * as Enum from "@frontend/Enum";
import { initialize as load } from "@pkg/bindings.runtime.mjs";
let factory;
export async function initialize(wasm) {
  await load(wasm);
  const outcome = await Native.create_factory();
  if (!Json.is_ok(outcome)) throw new Error(Native.message(Json.result_error(outcome)));
  factory = Json.value(outcome);
}
export function phase(input) {
  const value = (result) => {
    if (Json.is_ok(result)) return Json.value(result);
    throw new Error(Native.message(Json.result_error(result)));
  };
  const battle = value(Native.restore(factory, Json.of_saved_game(input.checkpoint)));
  const options = Json.of_ai_options({ policy: input.policy, diplomacy: "diplomatic" });
  try {
    let state = Native.state(Native.view(battle));
    while (Enum.string_of_side(state.turn) === "enemy" && state.winner === undefined) {
      value(Native.enemy_step_recorded(battle, options));
      state = Native.state(Native.view(battle));
    }
    return value(Native.save_json(battle));
  } finally {
    Native.dispose(battle);
  }
}
export function close() {
  Native.dispose_factory(factory);
}
