/** Check authored document references; browser metadata is supplied by WASM. */
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { posix } from "node:path";
import type { TerrainOverride } from "../src/game/types";
import { terrainArtNames, validTerrainConnections } from "../src/presentation/terrain-connections";

const root = new URL("../rust/data/", import.meta.url);
const floors = JSON.parse(
  readFileSync(new URL("../pixel/art/terrain/floors.json", import.meta.url), "utf8"),
) as { palettes: Record<string, unknown> };
const paths = new Set<string>();
function read<T>(path: string): T {
  if (!path || path.startsWith("/") || path.split("/").includes(".."))
    throw new Error(`Invalid content path: ${path}`);
  paths.add(path);
  return JSON.parse(readFileSync(new URL(path, root), "utf8")) as T;
}
function components(path: string, fields: string[]): void {
  const descriptor = read<Record<string, string>>(path);
  for (const field of fields) {
    const component = descriptor[field];
    if (!component) throw new Error(`${path}: missing ${field}`);
    const componentPath = posix.join(posix.dirname(path), component);
    const document = read<{ tiles?: TerrainOverride[] }>(componentPath);
    if (field === "map") {
      for (const tile of document.tiles ?? []) {
        if (!validTerrainConnections(tile))
          throw new Error(
            `${componentPath} at ${tile.col},${tile.row}: unsupported terrain connections`,
          );
        const frame = terrainArtNames(tile)[0]!;
        const stem = frame === tile.terrain ? "tile" : frame.slice(tile.terrain.length + 1);
        if (
          !existsSync(new URL(`../pixel/art/terrain/${tile.terrain}/${stem}.txt`, import.meta.url))
        )
          throw new Error(
            `${componentPath} at ${tile.col},${tile.row}: missing terrain source ${frame}`,
          );
        if (tile.floor !== undefined && !Object.hasOwn(floors.palettes, tile.floor))
          throw new Error(`${componentPath}: unknown floor ${tile.floor}`);
      }
    }
  }
}
const manifest = read<{
  units: string[];
  terrain: string;
  recruitment?: string;
  races: string[];
  scenarios: string[];
  arenaScenarios?: string[];
  campaigns: string[];
  simulation: { rosters: string; arena: string; campaign: string };
}>("manifest.json");
for (const path of [
  ...manifest.units,
  manifest.terrain,
  ...(manifest.recruitment ? [manifest.recruitment] : []),
  ...manifest.races,
  ...Object.values(manifest.simulation),
])
  read(path);
// Enabled factions must implement their entire authored tree. Standalone
// scenario wildlife does not enable its entire artwork faction.
type ContentUnit = { id: string; level: number; family: string; advancesTo: string[] };
type LineupUnit = { id: string; level: number; gameplayId?: string };
const units = new Map(
  manifest.units.map((path) => {
    const unit = read<ContentUnit>(path);
    return [unit.id, unit] as const;
  }),
);
const enabledLeaders = new Set(
  read<{ leader: string }[]>(manifest.simulation.rosters).map((roster) => roster.leader),
);
const lineupRoot = new URL("../pixel/art/units/lineups/", import.meta.url);
for (const file of readdirSync(lineupRoot).filter((file) => file.endsWith(".json"))) {
  const lineup = JSON.parse(readFileSync(new URL(file, lineupRoot), "utf8")) as {
    trees: { base: LineupUnit; branches: { units: LineupUnit[] }[] }[];
    commander?: { units: LineupUnit[] };
  };
  const trees = [...lineup.trees];
  if (lineup.commander)
    trees.push({
      base: lineup.commander.units[0]!,
      branches: [{ units: lineup.commander.units.slice(1) }],
    });
  const id = (unit: LineupUnit) => unit.gameplayId ?? unit.id;
  const entries = trees.flatMap((tree) => [
    tree.base,
    ...tree.branches.flatMap((branch) => branch.units),
  ]);
  if (!entries.some((entry) => enabledLeaders.has(id(entry)))) continue;
  for (const tree of trees) {
    for (const entry of [tree.base, ...tree.branches.flatMap((branch) => branch.units)]) {
      const unit = units.get(id(entry));
      const next =
        entry === tree.base
          ? tree.branches.map((branch) => id(branch.units[0]!))
          : tree.branches.flatMap((branch) =>
              entry === branch.units[0] ? [id(branch.units[1]!)] : [],
            );
      if (
        !unit ||
        unit.level !== entry.level ||
        unit.family !== id(tree.base) ||
        [...unit.advancesTo].sort().join(",") !== next.sort().join(",")
      )
        throw new Error(
          `${file}: gameplay unit ${id(entry)} must match its lineup tier, family and promotions`,
        );
    }
  }
}
for (const path of manifest.scenarios) components(path, ["map", "roster"]);
for (const path of manifest.arenaScenarios ?? []) components(path, ["map", "roster"]);
for (const path of manifest.campaigns) {
  const campaign = read<{ stages: string[] }>(path);
  for (const stage of campaign.stages)
    components(posix.join(posix.dirname(path), stage), [
      "map",
      "roster",
      "objectives",
      "story",
      "scripts",
    ]);
}
console.log(`Read ${paths.size} referenced JSON documents; no browser index is generated.`);
