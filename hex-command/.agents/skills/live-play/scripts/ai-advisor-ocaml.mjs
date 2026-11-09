import { suggestAction } from "./ai-advisor.mjs";
export { observe, recommend } from "./ai-advisor.mjs";

/** Observe the OCaml session; plan only on a separately restored Rust battle. */
export async function installAiAdvisor(page) {
  await page.route("**/frontend/output/frontend/game/Session.js*", async (route) => {
    const response = await route.fetch();
    const source = await response.text();
    const marker = "bind_failure(t, loaded.game);";
    if (source.split(marker).length !== 2)
      throw new Error("OCaml AI advisor: Session observation seam changed");
    const bridge = `
import * as AdvisorJson from "../Fixture_json.js";
function installUiAdvisor(t) {
  const N = Hex_engine__Native, U = Hex_platform__Utf8, J = AdvisorJson;
  const get = (result, message) => {
    if (J.is_ok(result)) return J.value(result);
    throw new Error(message(J.result_error(result)));
  };
  const native = result => get(result, error => U.to_js(N.message(error)));
  const ok = value => ({ok: true, value});
  // This facade never wraps t.battle: suggestAction owns and disposes its copy.
  const factory = {restore: checkpoint => {
    const copy = native(N.restore(t.factory, J.of_saved_game(checkpoint)));
    return ok({
      chooseAiCommand: options => ok(J.to_ai_decision(native(N.choose_ai_command(copy, J.of_ai_options(options))))),
      dispatch: (command, revision) => {
        if (revision !== N.revision(N.view(copy))) throw new Error("Stale advisor copy decision");
        native(N.dispatch(copy, J.of_command(command)));
        return ok(undefined);
      },
      dispose: () => {N.dispose(copy); return ok(undefined);},
    });
  }};
  const checkpoint = () => JSON.parse(native(N.save_json(t.battle)));
  const suggestOnCopy = (${suggestAction.toString()});
  globalThis.__livePlayAdvisor = Object.freeze({
    observe: () => {
      const v = view(t), game = v.interaction_game;
      return structuredClone({
        revision: N.revision(game), state: J.to_state(N.state(game)), checkpoint: checkpoint(),
        selected: v.map_focus.selected === undefined ? null : U.to_js(v.map_focus.selected),
        target: v.map_focus.target === undefined ? null : U.to_js(v.map_focus.target),
        canAct: t.initialized && v.failure === undefined && active(t),
      });
    },
    suggest: options => {
      if (!t.initialized || view(t).failure !== undefined || !active(t))
        throw new Error("AI advisor: finish the visible turn or dialog before planning");
      return {revision: N.revision(view(t).interaction_game),
        command: suggestOnCopy(factory, checkpoint(), options)};
    },
  });
}
`;
    await route.fulfill({
      response,
      body: source.replace(marker, `${marker}\n installUiAdvisor(t);`) + bridge,
    });
  });
}
