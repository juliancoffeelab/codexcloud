import { openHarness as openBrowser, runCampaigns } from "./campaign.mjs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export { runCampaigns };

// Browser-only instrumentation of the generated Session module. The compiler's
// fixture codecs handle variants, lists, strings and records; no fabricated tags.
export async function installOcamlAdapter({ page, config, errors }) {
  await page.route("**/frontend/output/frontend/game/Session.js*", async (route) => {
    try {
      const response = await route.fetch();
      const source = await response.text();
      const marker = "bind_failure(t, loaded.game);";
      if (source.split(marker).length !== 2)
        throw new Error("Generated Session adapter marker changed; inspect Session.ml/Session.js");
      const bridge = `
import * as LiveJson from "../Fixture_json.js";
function installLivePlay(t) {
  const N = Hex_engine__Native, U = Hex_platform__Utf8, J = LiveJson;
  const actions = () => commands(t);
  const current = () => view(t).interaction_game;
  const text = value => U.of_js(value);
  const outcome = (result, message) => {
    if (J.is_ok(result)) return J.value(result);
    throw new Error(U.to_js(message(J.result_error(result))));
  };
  t.clock = ms => new Promise(resolve => setTimeout(resolve, ms * ${config.enemyPauseScale}));
  const snapshot = () => {
    const v = current(), raw = J.to_state(N.state(v));
    const {mapId, mapRevision, round, turn, gold, winner} = raw;
    const sites = roles => raw.tiles.filter(tile => roles.includes(J.to_terrain_role(
      Hex_engine__Catalog.terrain_role(t.catalog, text(tile.terrain))
    ))).map(({id, owner}) => ({id, owner}));
    const result = {mapId, mapRevision, round, turn, gold, winner, villages: sites(["Village"])};
    const otherSites = sites(["Grove", "Fort", "Ruins"]);
    if (raw.campaign || otherSites.length) result.sites = otherSites;
    result.units = N.state(v).units.filter(unit => N.visible(v, unit)).map(unit => {
      const value = J.to_public_unit(unit), name = N.unit_name(v, unit.id);
      if (name !== undefined) value.name = U.to_js(name);
      return value;
    });
    if (raw.campaign) result.campaign = raw.campaign;
    return result;
  };
  const controller = {
    get initialized() { return t.initialized; },
    get commands() { return actions(); },
    snapshot,
    move: (unit, tile) => move_unit(t, text(unit), text(tile)),
    attack: (unit, target, weapon) => attack_unit(t, text(unit), text(target), weapon === undefined ? undefined : text(weapon)),
    recruit: (unit, kind, tile) => recruit_unit(t, text(unit), text(kind), text(tile)),
    advance: (unit, kind) => advance_unit(t, text(unit), text(kind)),
    dropItem: (unit, item) => actions().drop(text(unit), text(item)),
    endTurn: async () => outcome(await actions().end_turn(), error_message),
    undo: () => actions().undo(),
    choose: tile => actions().choose(text(tile)),
    startCampaign: async (id, setup) => {actions().start_campaign(setup === undefined ? undefined : Hex_engine__Setup.of_json(setup), text(id));await saved(t);},
    reset: async (map = "mirror", options) => {
      const armies = options?.armies ? J.get_tuple_2(J.get_string, J.get_string, options.armies) : undefined;
      actions().reset(options?.setup ? Hex_engine__Setup.of_json(options.setup) : undefined, armies, text(map));await saved(t);
    },
    nextMission: async units => {actions().next_mission(J.list_of_array(units.map(text)));await saved(t);},
    retryMission: async () => {actions().retry();await saved(t);},
    advanceDialogue: (skip = false) => actions().dialogue(skip),
    previousDialogue: () => actions().previous_dialogue(),
    saved: () => saved(t),
    savedProgress: () => saved_progress(t),
    saveGame: () => save_game(t),
    savedFlashSave: () => saved_flash(t),
    restoreProgress: id => restore_progress(t, text(id)),
    deleteSavedProgress: id => delete_saved_progress(t, text(id)),
    deleteSavedRun: id => delete_saved_run(t, text(id)),
    // Session.dispatch already performs commit, publication and persistence.
    commitAction: () => {},
  };
  const game = () => ({
    checkpoint: () => ({ok: true, value: JSON.parse(outcome(N.save_json(t.battle), N.message))}),
    reachable: id => new Map(J.bindings_string_map(N.reachable(current(), text(id))).map(([key, value]) => [U.to_js(key), value])),
    at: id => {const unit = N.at(current(), text(id)); return unit === undefined ? undefined : J.to_public_unit(unit);},
    dispatch: (command, revision) => {
      if (revision !== N.revision(current())) throw new Error("Stale live-play decision");
      dispatch(undefined, undefined, t, J.of_command(command));
    },
    engine: {
      view: () => ({ok: true, value: {revision: N.revision(current())}}),
      chooseAiCommand: options => ({ok: true, value: J.to_ai_decision(outcome(N.choose_ai_command(t.battle, J.of_ai_options(options)), N.message))}),
    },
  });
  globalThis.__engineProbe = {controller, game};
}
`;
      await route.fulfill({
        response,
        body:
          source.replace(marker, "bind_failure(t, loaded.game);\n    installLivePlay(t);") + bridge,
      });
    } catch (error) {
      errors.push(String(error));
      await route.abort();
    }
  });
}

export function openHarness(options = {}) {
  return openBrowser({ ...options, installAdapter: installOcamlAdapter });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const harness = await openHarness();
  try {
    const results = await runCampaigns(harness, {
      campaigns: (process.env.CAMPAIGNS || "lantern-road").split(","),
    });
    console.log(
      JSON.stringify(
        results.map(({ campaign, terminal, retries }) => ({ campaign, terminal, retries })),
        null,
        2,
      ),
    );
    if (
      harness.errors.length ||
      results.some((result) => result.terminal !== "campaign-complete-next-game-ready")
    )
      process.exitCode = 1;
  } finally {
    await harness.close();
  }
}
