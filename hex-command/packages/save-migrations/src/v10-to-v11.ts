import type { Save } from "./formats/history";
import { readSave, readFragment } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import historical from "./legacy/data/bell-valley-v10.json";

/** Move the published placeholder battle onto the valley without respawning its army. */
export function migrateV10ToV11(input: unknown): Save<11> {
  const value = readSave(input, 10);
  if (value.version !== 10 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-ten save");
  const old = value.checkpoint.state;
  if (!Array.isArray(old.tiles) || !old.tiles.every(isRecord))
    throw new Error("Invalid historical map tiles");
  const patches = historical.patches[old.mapId as keyof typeof historical.patches] ?? [];
  let state: import("./formats/history").State<11> = {
    ...old,
    tiles: old.tiles.map((tile) => {
      const patch = patches.find(
        (patch) => patch.tile.id === tile.id && patch.oldTerrain === tile.terrain,
      );
      return patch
        ? {
            ...readFragment([patch.tile], "tiles11")[0]!,
            ...(patch.tile.terrain.startsWith("castle-") ? { owner: tile.owner } : {}),
          }
        : { ...tile };
    }),
  };
  if (
    old.mapId === historical.mapId &&
    historical.fromRevisions.includes(Number(old.mapRevision))
  ) {
    if (
      old.cols !== historical.fromCols ||
      old.rows !== historical.fromRows ||
      old.tiles.length !== historical.fromCols * historical.fromRows ||
      new Set(old.tiles.map((tile) => tile.id)).size !== old.tiles.length ||
      !Array.isArray(old.units) ||
      !old.units.every(isRecord) ||
      !Array.isArray(old.groundItems) ||
      !old.groundItems.every(isRecord) ||
      !isRecord(old.campaign) ||
      !Array.isArray(old.campaign.marks)
    )
      throw new Error("Invalid historical bell battle");
    const mapping: Record<string, string> = historical.mapping;
    const destination = (tile: unknown): string => {
      if (typeof tile !== "string" || !Object.hasOwn(mapping, tile))
        throw new Error("Unknown historical bell tile");
      return mapping[tile]!;
    };
    for (const tile of old.tiles) {
      if (tile.id !== `${tile.col},${tile.row}`)
        throw new Error("Invalid historical bell coordinate");
      destination(tile.id);
    }
    const rename = <T extends { name?: string }>(unit: T) => ({
      ...unit,
      ...(unit.name === "Meri" ? { name: "Oren" } : {}),
    });
    const roster = (units: import("./formats/history").Veteran<10>[]) => {
      if (!Array.isArray(units) || !units.every(isRecord))
        throw new Error("Invalid historical companions");
      return units.map(rename);
    };
    const campaign = { ...old.campaign };
    for (const key of ["storyVeterans", "veterans"] as const)
      if (campaign[key] !== undefined) campaign[key] = roster(campaign[key]);
    if (Array.isArray(campaign.fallenCompanions))
      campaign.fallenCompanions = campaign.fallenCompanions.map((name) =>
        name === "Meri" ? "Oren" : name,
      );
    campaign.marks = old.campaign.marks.map((mark) =>
      mark === "fallen:Meri" ? "fallen:Oren" : mark,
    );
    campaign.holds = { "5,5": { heldTurns: 0, lastCountedRound: old.round } };
    const ownership = new Map(old.tiles.map((tile) => [destination(tile.id), tile.owner]));
    const spent: Record<string, string> = historical.spent;
    const spentTiles = new Set(
      old.campaign.marks
        .filter((mark): mark is string => typeof mark === "string")
        .map((mark) => spent[mark])
        .filter(Boolean),
    );
    state = {
      ...state,
      mapRevision: historical.toRevision,
      cols: historical.cols,
      rows: historical.rows,
      factions: readFragment(historical.factions, "factions11"),
      conditionMemory: [],
      campaign,
      tiles: readFragment(historical.tiles, "tiles11").map((tile) => ({
        ...structuredClone(tile),
        fog: readFragment(historical.fog, "rule11"),
        ...(ownership.has(tile.id) ? { owner: ownership.get(tile.id)! } : {}),
        ...(spentTiles.has(tile.id) ? { terrain: "ritual-spent" } : {}),
      })),
      units: old.units.map((unit) => ({ ...rename(unit), tile: destination(unit.tile) })),
      groundItems: old.groundItems.map((item) => ({ ...item, tile: destination(item.tile) })),
      ...(old.lastCombat !== undefined ? { lastCombat: old.lastCombat } : {}),
    };
  }
  return { ...value, version: 11, checkpoint: { ...value.checkpoint, state } };
}
