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
// Test-only down-conversion: old fixture formats predate named campaign opponents.
const namedOpponents = new Set([
  "masters-host",
  "quarry-guards",
  "charter-patrol",
  "bell-host",
  "factors-guards",
  "sluice-guards",
  "wreck-guards",
  "crypt-hunters",
  "crypt-defenders",
  "charter-guards",
  "rootbinders-host",
  "road-skeletons",
  "toll-skeletons",
  "night-host",
  "forest-hunters",
  "forest-defenders",
]);
function historicalFactions(value, parent) {
  if (Array.isArray(value)) {
    for (const entry of value) historicalFactions(entry, parent);
  } else if (value && typeof value === "object") {
    for (const key of Object.keys(value)) {
      if (
        (key === "id" && parent === "factions") ||
        ["faction", "owner", "to"].includes(key) ||
        (key === "side" && value.kind === "owner")
      ) {
        if (namedOpponents.has(value[key])) value[key] = "enemy";
      }
      if (["gold", "relations", "recruitment"].includes(key)) {
        for (const id of Object.keys(value[key] ?? {})) {
          if (namedOpponents.has(id)) {
            value[key].enemy = value[key][id];
            delete value[key][id];
          }
        }
      }
      historicalFactions(value[key], key);
    }
  }
}
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
  // Explicit test-only writer for pre-perception histories. Do not stamp an
  // old version onto the current phase, outcome or action representation.
  const historicalInput = structuredClone(input);
  const current = historicalInput.state ?? historicalInput;
  // Explicit pre-v16 test writer: those serializers omitted absent optionals.
  // Current snapshots emit them; old readers must still receive their frozen shape.
  const oldOptionals = new Set([
    "armies",
    "campaign",
    "lastCombat",
    "location",
    "connections",
    "deadEnds",
    "walls",
    "floor",
    "light",
    "policy",
    "name",
    "recruits",
    "transfer",
    "cost",
    "character",
    "aiGroup",
    "retreating",
    "rebuilding",
    "role",
    "armor",
    "healing",
    "perception",
    "sound",
    "refuge",
    "vigil",
    "holds",
    "when",
    "color",
    "fixture",
    "dx",
    "dy",
  ]);
  const omit = (value) => {
    if (Array.isArray(value)) {
      for (const entry of value) omit(entry);
    } else if (value && typeof value === "object") {
      for (const key of Object.keys(value)) {
        if (value[key] === null && oldOptionals.has(key)) delete value[key];
        else omit(value[key]);
      }
    }
  };
  omit(current);
  if (current.campaign?.scripts)
    current.campaign.scripts.active = current.campaign.scripts.active.filter(
      (id) => id !== "company-survival",
    );
  historicalFactions(current);
  if (current.visionGroups || current.units?.some((unit) => Object.hasOwn(unit, "attackSpent"))) {
    const phase = (value) => (value === "ai" ? "enemy" : value);
    current.turn = phase(current.turn);
    current.winner =
      current.winner === "playerVictory"
        ? "player"
        : current.winner === "playerDefeat"
          ? "enemy"
          : current.winner;
    delete current.visionGroups;
    for (const faction of current.factions ?? []) {
      faction.phase = phase(faction.phase);
      delete faction.visionGroup;
    }
    for (const unit of current.units ?? []) {
      unit.side = phase(unit.side ?? current.factions.find((f) => f.id === unit.faction).phase);
      unit.attacked = unit.attackSpent;
      delete unit.attackSpent;
      delete unit.revealedTo;
      delete unit.perceptionEffects;
    }
    for (const tile of current.tiles ?? []) {
      const fog = tile.perception?.concealment.find((rule) => rule.kind === "fog");
      if (fog)
        tile.fog = {
          condition: { kind: "proximity", side: "observer", radius: fog.revealWithin },
          remember: tile.perception.remember,
        };
      delete tile.perception;
      if (tile.light) delete tile.light.emission;
    }
    for (const mark of current.campaign?.scripts?.marks ?? []) {
      delete mark.sight;
      if (mark.light) delete mark.light.emission;
    }
  }
  const converted = convert(historicalInput);
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

// Older snapshots stored their ambient level on every tile. Migration preserves
// those facts even when today's authored map uses a neutral local contribution.
export function explicitAmbient(state) {
  const result = structuredClone(state);
  for (const tile of result.tiles)
    tile.perception.illumination ??= result.lighting.time === "day" ? "lit" : "unlit";
  return result;
}
