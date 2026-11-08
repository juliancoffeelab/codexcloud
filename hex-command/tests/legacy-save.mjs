import { Game } from "./game-fixture.mjs";
import { Catalog, catalog } from "./native-fixture.mjs";
import * as Save from "./save-adapter.mjs";
import * as Legacy from "./legacy-adapter.mjs";
import * as Store from "./storage-adapter.mjs";
const SAVE_KEY = Save.key;
const SAVE_VERSION = 1;
const importWaveMarker = Legacy.wave_marker;
async function saveBattle(game, pendingEnemy = false) {
  const {
    tiles,
    cols: _cols,
    rows: _rows,
    lastCombat: _combat,
    ...state
  } = historicalCharacters(game.captureState());
  const villages = tiles
    .filter((tile) => tile.terrain === "village")
    .map(({ id, owner }) => ({ id, owner }));
  const sites = tiles
    .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
    .map(({ id, owner }) => ({ id, owner }));
  await Store.store_save(
    JSON.stringify({
      version: 1,
      campaignDeploymentFormat: "named-waves-v1",
      pendingEnemy,
      state: { ...state, villages, ...(state.campaign || sites.length ? { sites } : {}) },
    }),
  );
}
async function loadBattle() {
  try {
    const [save] = Save.migrate(
      Catalog.raw(catalog),
      JSON.parse((await Store.read_session())[0] ?? "null"),
    );
    const game = Game.fromCheckpoint(save.checkpoint);
    return { game, pendingEnemy: save.pending_enemy };
  } catch {
    return { game: new Game(), pendingEnemy: false };
  }
}
export { SAVE_KEY, SAVE_VERSION, importWaveMarker, loadBattle, saveBattle };

import { campaigns as CAMPAIGNS } from "@migrations/Migration_api.js";
import tileHistory from "../packages/save-migrations/src/legacy/data/tiles-v26.json";
import historical from "../packages/save-migrations/src/legacy/data/characters-v25.json";
import locationHistory from "../packages/save-migrations/src/legacy/data/locations-v37.json";
function historicalTiles(input) {
  const reverse = new Map(
    Object.values(tileHistory).flatMap((board) =>
      Object.entries(board).map(([position, id]) => [id, position]),
    ),
  );
  const state = input.state ?? input;
  const locations = locationHistory[state.mapId] ?? {};
  const names = new Map(
    (state.tiles ?? []).filter((t) => t.location).map((t) => [t.location, `${t.col},${t.row}`]),
  );
  const oldNames = new Map(
    Object.entries(tileHistory[state.mapId] ?? {}).flatMap(([old, id]) => {
      const position = names.get(locations[id]);
      return position ? [[position, old]] : [];
    }),
  );
  const address = (hex) => oldNames.get(`${hex.col},${hex.row}`) ?? `${hex.col},${hex.row}`;
  const convert = (v) => {
    if (typeof v === "string")
      return v.replace(/hex-[0-9]{4}-[a-f0-9]{10}/g, (id) => reverse.get(id) ?? id);
    if (Array.isArray(v)) return v.map(convert);
    if (v && typeof v === "object") {
      const keys = Object.keys(v);
      if (keys.length === 2 && keys.includes("col") && keys.includes("row")) return address(v);
      if (keys.length === 1 && v.hex) return address(v.hex);
      if (keys.length === 1 && v.location) {
        const point = names.get(v.location);
        return oldNames.get(point) ?? point ?? v.location;
      }
      if (keys.length === 1 && v.binding) return v.binding;
      return Object.fromEntries(
        Object.entries(v)
          .filter(([k]) => k !== "location")
          .map(([k, value]) => [
            String(convert(k)),
            k === "id" && typeof value === "string"
              ? (oldNames.get(value) ?? convert(value))
              : convert(value),
          ]),
      );
    }
    return v;
  };
  const converted = convert(input);
  const old = converted.state ?? converted;
  // These callers construct pre-v16 fixtures. Recreate their declared castle shape.
  for (const tile of old.tiles ?? [])
    if (["castle-wall", "castle-bell"].includes(tile.terrain) && tile.walls) {
      tile.connections = ["e", "se", "sw", "w", "nw", "ne"].filter(
        (edge) => !tile.walls.includes(edge),
      );
      tile.wallCorners = [0, 0, 0, 0, 0, 0];
      delete tile.walls;
    }
  return converted;
}
function historicalCharacters(input) {
  const state = historicalTiles(input);
  const campaign = state.campaign;
  const scope = campaign?.id ?? historical.maps[state.mapId];
  const registry = historical.campaigns[scope];
  const display = (id) => registry?.characters.find((c) => c.id === id)?.name ?? id;
  const unit = (u) => {
    const { character, ...rest } = u;
    return character ? { ...rest, name: u.name ?? display(character) } : rest;
  };
  const condition = (c) => {
    const { character, ...rest } = c;
    return {
      ...rest,
      ...(character ? { name: display(character) } : {}),
      ...(Array.isArray(c.conditions) ? { conditions: c.conditions.map(condition) } : {}),
    };
  };
  state.units = state.units.map(unit);
  if (campaign) {
    if (campaign.flags) {
      campaign.choices = Object.entries(campaign.flags)
        .filter(([, history]) => history.at(-1)?.state)
        .map(([id]) => id);
      delete campaign.flags;
    }
    const definition = CAMPAIGNS[campaign.id];
    const mission = definition?.missions[campaign.mission];
    const lines = [
      ...(mission?.intro ?? []),
      ...(mission?.victory ?? []),
      ...(mission?.events.flatMap((e) => [...e.dialogue, ...(e.warDialogue ?? [])]) ?? []),
      ...(mission?.legacyDialogue ?? []),
      ...definition.ending,
    ];
    campaign.dialogue = campaign.dialogue.map((line) => {
      const { character, ...rest } = line;
      const old = lines.find(
        (known) => known.text === line.text && known.portrait === line.portrait,
      );
      return old
        ? { ...rest, speaker: old.speaker }
        : character
          ? { ...rest, speaker: display(character) }
          : rest;
    });
    if (campaign.veterans) campaign.veterans = campaign.veterans.map(unit);
    if (campaign.storyVeterans) campaign.storyVeterans = campaign.storyVeterans.map(unit);
    if (campaign.evacuated)
      campaign.evacuated = campaign.evacuated.map((r) => ({ ...r, unit: unit(r.unit) }));
    if (campaign.fallenCompanions)
      campaign.fallenCompanions = campaign.fallenCompanions.map(display);
    campaign.marks = campaign.marks.map((mark) =>
      mark.startsWith("fallen:") ? `fallen:${display(mark.slice(7))}` : mark,
    );
  }
  for (const faction of state.factions ?? [])
    if (faction.transfer)
      faction.transfer.when.condition = condition(faction.transfer.when.condition);
  return state;
}
export { historicalCharacters, historicalTiles };
