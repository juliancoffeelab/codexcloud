/** Fixtures for published version-one saves; never used by the browser. */
import { Game } from "./game-fixture";
import { nativeValue, type LiveGame } from "../src/game/live-game";
import { loadBattle as loadCurrent } from "../src/save/load";
import { saveBattle as writeCurrent } from "../src/save/write";
import { createTestFactory } from "./native-fixture";
import type { SavedGame } from "../rust/pkg/hex_command";
export { importWaveMarker } from "../src/save/migrations/legacy/snapshot";
export { SAVE_KEY } from "../src/save/migrations/storage-v1-to-v2";
import { storeSave } from "../src/save/storage";
import { CAMPAIGNS } from "../src/save/migrations/legacy/catalog";
import tileHistory from "../src/save/migrations/legacy/data/tiles-v26.json";
import historical from "../src/save/migrations/legacy/data/characters-v25.json";

/** Encode a current geometric fixture in its published coordinate-key format. */
export function historicalTiles<T>(input: T): T {
  const reverse = new Map(
    Object.values(tileHistory).flatMap((board) =>
      Object.entries(board).map(([position, id]) => [id, position]),
    ),
  );
  const convert = (v: unknown): unknown => {
    if (typeof v === "string")
      return v.replace(/hex-[0-9]{4}-[a-f0-9]{10}/g, (id) => reverse.get(id) ?? id);
    if (Array.isArray(v)) return v.map(convert);
    if (v && typeof v === "object") {
      const item = v as Record<string, unknown>;
      if (
        Object.keys(item).length === 2 &&
        typeof item.col === "number" &&
        typeof item.row === "number"
      )
        return `${item.col},${item.row}`;
      if (Object.keys(item).length === 1 && "hex" in item) return convert(item.hex);
    }
    if (v && typeof v === "object")
      return Object.fromEntries(
        Object.entries(v)
          .filter(([key]) => key !== "location")
          .map(([k, value]) => [String(convert(k)), convert(value)]),
      );
    return v;
  };
  return convert(input) as T;
}

/** Encode current identity-bearing fixtures in the actual published name-based format. */
export function historicalCharacters<T>(input: T): T {
  const state = historicalTiles(input) as Record<string, any>;
  const campaign = state.campaign;
  const scope = campaign?.id ?? historical.maps[state.mapId as keyof typeof historical.maps];
  const registry = historical.campaigns[scope as keyof typeof historical.campaigns];
  const display = (id: string) => registry?.characters.find((c) => c.id === id)?.name ?? id;
  const unit = (u: Record<string, any>) => {
    const { character, ...rest } = u;
    return character ? { ...rest, name: u.name ?? display(character) } : rest;
  };
  const condition = (c: Record<string, any>): Record<string, any> => {
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
        .filter(([, history]) => (history as { state: boolean }[]).at(-1)?.state)
        .map(([id]) => id);
      delete campaign.flags;
    }
    const definition = CAMPAIGNS[campaign.id as keyof typeof CAMPAIGNS];
    const mission = definition?.missions[campaign.mission];
    const lines = [
      ...(mission?.intro ?? []),
      ...(mission?.victory ?? []),
      ...(mission?.events.flatMap((e) => [...e.dialogue, ...(e.warDialogue ?? [])]) ?? []),
      ...(mission?.legacyDialogue ?? []),
      ...definition.ending,
    ];
    campaign.dialogue = campaign.dialogue.map((line: Record<string, any>) => {
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
      campaign.evacuated = campaign.evacuated.map((r: any) => ({ ...r, unit: unit(r.unit) }));
    if (campaign.fallenCompanions)
      campaign.fallenCompanions = campaign.fallenCompanions.map(display);
    campaign.marks = campaign.marks.map((mark: string) =>
      mark.startsWith("fallen:") ? `fallen:${display(mark.slice(7))}` : mark,
    );
  }
  for (const faction of state.factions ?? [])
    if (faction.transfer)
      faction.transfer.when.condition = condition(faction.transfer.when.condition);
  return state as T;
}
export const SAVE_VERSION = 1;
export async function saveBattle(game: Game | LiveGame, pendingEnemy = false): Promise<void> {
  if (!(game instanceof Game)) {
    await writeCurrent(game, pendingEnemy);
    return;
  }
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
  await storeSave(
    JSON.stringify({
      version: 1,
      campaignDeploymentFormat: "named-waves-v1",
      pendingEnemy,
      state: { ...state, villages, ...(state.campaign || sites.length ? { sites } : {}) },
    }),
  );
}
export async function loadBattle() {
  const loaded = await loadCurrent(createTestFactory());
  try {
    return {
      game: Game.fromCheckpoint(
        structuredClone(nativeValue(loaded.game.checkpoint())) as SavedGame,
      ),
      pendingEnemy: loaded.pendingEnemy,
    };
  } finally {
    loaded.game.dispose();
  }
}
