/** Browser metadata from the same authored JSON embedded in the Rust engine.
 * Rust validates the documents when building the catalog. This module projects
 * their shapes for the UI and data-only save migrations.
 */
import manifest from "../../rust/data/manifest.json";
import { CONTENT_DOCUMENTS } from "./content-documents";
import type {
  CampaignDefinition,
  CampaignId,
  MapDefinition,
  MissionDefinition,
  Race,
  RaceDefinition,
  StartingUnit,
  TerrainDefinition,
  TerrainType,
  Tile,
  UnitDefinition,
  UnitType,
} from "./types";

function document<T>(path: string): T {
  const value = CONTENT_DOCUMENTS[path];
  if (!value) throw new Error(`Missing authored document: ${path}`);
  return value as T;
}
function sibling(path: string, name: string): string {
  return path.slice(0, path.lastIndexOf("/") + 1) + name;
}
function definitions<T>(paths: string[]): Record<string, T> {
  return Object.fromEntries(
    paths.map((path) => {
      const { id, ...definition } = document<T & { id: string }>(path);
      return [id, definition as T];
    }),
  );
}
function named<T>(path: string): Record<string, T> {
  return Object.fromEntries(
    document<(T & { id: string })[]>(path).map(({ id, ...definition }) => [id, definition as T]),
  );
}

export const UNIT_SPECIFICATIONS = definitions<UnitDefinition>(manifest.units) as Record<
  UnitType,
  UnitDefinition
>;
export const AUTHORED_RACES = definitions<RaceDefinition>(manifest.races) as Record<
  Race,
  RaceDefinition
>;
export const AUTHORED_TERRAIN = named<TerrainDefinition>(manifest.terrain) as Record<
  TerrainType,
  TerrainDefinition
>;

interface MapDocument {
  mapId: MapDefinition["id"];
  mapRevision: number;
  name: string;
  description: string;
  cols: number;
  rows: number;
  tiles: Tile[];
}
function map(path: string, units: StartingUnit[]): MapDefinition {
  const data = document<MapDocument>(path);
  return {
    id: data.mapId,
    revision: data.mapRevision,
    name: data.name,
    description: data.description,
    cols: data.cols,
    rows: data.rows,
    terrain: data.tiles
      .filter((tile) => tile.terrain !== "plains" || tile.owner !== null)
      .map(({ col, row, terrain, owner }) => ({ col, row, terrain, owner })),
    units,
  };
}
function mission(path: string): MissionDefinition {
  const descriptor = document<Record<string, string>>(path);
  const roster = document<
    Pick<MissionDefinition, "recruits" | "enemyRecruits" | "gold" | "enemyGold"> & {
      units: StartingUnit[];
      initial: unknown;
    }
  >(sibling(path, descriptor.roster!));
  const { units, initial: _initial, ...chapter } = roster;
  // initial.units/initial.gold construct Rust's battle; units/treasury describe
  // the chapter and its restart/carryover rules. Do not substitute one for the other.
  return {
    ...chapter,
    ...document<object>(sibling(path, descriptor.objectives!)),
    ...document<object>(sibling(path, descriptor.story!)),
    ...document<object>(sibling(path, descriptor.scripts!)),
    map: map(sibling(path, descriptor.map!), units),
  } as MissionDefinition;
}
export const AUTHORED_CAMPAIGNS = Object.fromEntries(
  manifest.campaigns.map((path) => {
    const { stages, ...definition } = document<
      Omit<CampaignDefinition, "missions"> & { stages: string[] }
    >(path);
    return [
      definition.id,
      { ...definition, missions: stages.map((stage) => mission(sibling(path, stage))) },
    ];
  }),
) as Record<CampaignId, CampaignDefinition>;

export const AUTHORED_SCENARIOS = manifest.scenarios.map((path) => {
  const descriptor = document<{ map: string; roster: string }>(path);
  const roster = document<{ units: StartingUnit[] }>(sibling(path, descriptor.roster));
  return map(sibling(path, descriptor.map), roster.units);
});
