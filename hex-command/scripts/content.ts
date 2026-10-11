/** Check authored document references; browser metadata is supplied by WASM. */
import { existsSync, readFileSync } from "node:fs";
import { posix } from "node:path";
import type { Tile } from "../rust/pkg/hex_command";
type TerrainOverride = Pick<Tile, "col" | "row" | "terrain" | "floor" | "connections" | "deadEnds">;

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
        // Rust validates directional topology when loading the catalog. This
        // document checker owns file references, not a second rules implementation.
        if (
          !existsSync(new URL(`../pixel/art/terrain/${tile.terrain}/palette.json`, import.meta.url))
        )
          throw new Error(`${componentPath}: missing terrain source ${tile.terrain}`);
        if (tile.floor !== undefined && !Object.hasOwn(floors.palettes, tile.floor))
          throw new Error(`${componentPath}: unknown floor ${tile.floor}`);
      }
    }
  }
}
const manifest = read<{
  units: string[];
  terrain: string;
  races: string[];
  scenarios: string[];
  campaigns: string[];
  simulation: Record<string, string>;
}>("manifest.json");
for (const path of [
  ...manifest.units,
  manifest.terrain,
  ...manifest.races,
  ...Object.values(manifest.simulation),
])
  read(path);
for (const path of manifest.scenarios) components(path, ["map", "roster"]);
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
