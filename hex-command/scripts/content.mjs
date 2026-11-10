// tmp/pr43-reference/hex-command/scripts/content.ts
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { posix } from "node:path";

// tmp/pr43-reference/hex-command/pixel/art/terrain/castle-bell/variants.json
var variants_default = [
  {
    id: "63",
    walls: ["e", "se", "sw", "w", "nw", "ne"],
  },
];

// tmp/pr43-reference/hex-command/pixel/art/terrain/castle-wall/variants.json
var variants_default2 = [
  {
    id: "0",
    walls: [],
  },
  {
    id: "1",
    walls: ["e"],
  },
  {
    id: "10",
    walls: ["se", "w"],
  },
  {
    id: "11",
    walls: ["e", "se", "w"],
  },
  {
    id: "12",
    walls: ["sw", "w"],
  },
  {
    id: "13",
    walls: ["e", "sw", "w"],
  },
  {
    id: "14",
    walls: ["se", "sw", "w"],
  },
  {
    id: "15",
    walls: ["e", "se", "sw", "w"],
  },
  {
    id: "16",
    walls: ["nw"],
  },
  {
    id: "17",
    walls: ["e", "nw"],
  },
  {
    id: "18",
    walls: ["se", "nw"],
  },
  {
    id: "19",
    walls: ["e", "se", "nw"],
  },
  {
    id: "2",
    walls: ["se"],
  },
  {
    id: "20",
    walls: ["sw", "nw"],
  },
  {
    id: "21",
    walls: ["e", "sw", "nw"],
  },
  {
    id: "22",
    walls: ["se", "sw", "nw"],
  },
  {
    id: "23",
    walls: ["e", "se", "sw", "nw"],
  },
  {
    id: "24",
    walls: ["w", "nw"],
  },
  {
    id: "25",
    walls: ["e", "w", "nw"],
  },
  {
    id: "26",
    walls: ["se", "w", "nw"],
  },
  {
    id: "27",
    walls: ["e", "se", "w", "nw"],
  },
  {
    id: "28",
    walls: ["sw", "w", "nw"],
  },
  {
    id: "29",
    walls: ["e", "sw", "w", "nw"],
  },
  {
    id: "3",
    walls: ["e", "se"],
  },
  {
    id: "30",
    walls: ["se", "sw", "w", "nw"],
  },
  {
    id: "31",
    walls: ["e", "se", "sw", "w", "nw"],
  },
  {
    id: "32",
    walls: ["ne"],
  },
  {
    id: "33",
    walls: ["e", "ne"],
  },
  {
    id: "34",
    walls: ["se", "ne"],
  },
  {
    id: "35",
    walls: ["e", "se", "ne"],
  },
  {
    id: "36",
    walls: ["sw", "ne"],
  },
  {
    id: "37",
    walls: ["e", "sw", "ne"],
  },
  {
    id: "38",
    walls: ["se", "sw", "ne"],
  },
  {
    id: "39",
    walls: ["e", "se", "sw", "ne"],
  },
  {
    id: "4",
    walls: ["sw"],
  },
  {
    id: "40",
    walls: ["w", "ne"],
  },
  {
    id: "41",
    walls: ["e", "w", "ne"],
  },
  {
    id: "42",
    walls: ["se", "w", "ne"],
  },
  {
    id: "43",
    walls: ["e", "se", "w", "ne"],
  },
  {
    id: "44",
    walls: ["sw", "w", "ne"],
  },
  {
    id: "45",
    walls: ["e", "sw", "w", "ne"],
  },
  {
    id: "46",
    walls: ["se", "sw", "w", "ne"],
  },
  {
    id: "47",
    walls: ["e", "se", "sw", "w", "ne"],
  },
  {
    id: "48",
    walls: ["nw", "ne"],
  },
  {
    id: "49",
    walls: ["e", "nw", "ne"],
  },
  {
    id: "5",
    walls: ["e", "sw"],
  },
  {
    id: "50",
    walls: ["se", "nw", "ne"],
  },
  {
    id: "51",
    walls: ["e", "se", "nw", "ne"],
  },
  {
    id: "52",
    walls: ["sw", "nw", "ne"],
  },
  {
    id: "53",
    walls: ["e", "sw", "nw", "ne"],
  },
  {
    id: "54",
    walls: ["se", "sw", "nw", "ne"],
  },
  {
    id: "55",
    walls: ["e", "se", "sw", "nw", "ne"],
  },
  {
    id: "56",
    walls: ["w", "nw", "ne"],
  },
  {
    id: "57",
    walls: ["e", "w", "nw", "ne"],
  },
  {
    id: "58",
    walls: ["se", "w", "nw", "ne"],
  },
  {
    id: "59",
    walls: ["e", "se", "w", "nw", "ne"],
  },
  {
    id: "6",
    walls: ["se", "sw"],
  },
  {
    id: "60",
    walls: ["sw", "w", "nw", "ne"],
  },
  {
    id: "61",
    walls: ["e", "sw", "w", "nw", "ne"],
  },
  {
    id: "62",
    walls: ["se", "sw", "w", "nw", "ne"],
  },
  {
    id: "63",
    walls: ["e", "se", "sw", "w", "nw", "ne"],
  },
  {
    id: "7",
    walls: ["e", "se", "sw"],
  },
  {
    id: "8",
    walls: ["w"],
  },
  {
    id: "9",
    walls: ["e", "w"],
  },
];

// tmp/pr43-reference/hex-command/pixel/art/terrain/road/dead-ends.json
var dead_ends_default = [
  [34, 1],
  [34, 8],
];

// tmp/pr43-reference/hex-command/src/presentation/terrain-connections.ts
var TERRAIN_DIRECTIONS = ["e", "se", "sw", "w", "nw", "ne"];
var CONNECTED_TERRAINS = [
  "signpost",
  "road",
  "village",
  "bridge",
  "ford",
  "landing",
  "ferry",
  "castle-wall",
  "castle-bell",
  "cave-passage",
];
var directionMask = (directions) =>
  directions.reduce((mask, direction) => mask | (1 << TERRAIN_DIRECTIONS.indexOf(direction)), 0);
function castleVariantId(walls) {
  return String(directionMask(walls));
}
var castleVariants = {
  "castle-wall": variants_default2,
  "castle-bell": variants_default,
};
function validTerrainConnections(tile) {
  if (Object.hasOwn(tile, "wallCorners") || Object.hasOwn(tile, "gates")) return false;
  const castle = tile.terrain === "castle-wall" || tile.terrain === "castle-bell";
  const validEdges = (edges) =>
    edges !== void 0 &&
    edges.every((edge) => TERRAIN_DIRECTIONS.includes(edge)) &&
    new Set(edges).size === edges.length;
  if (castle) {
    if (tile.connections !== void 0 || tile.deadEnds !== void 0 || !validEdges(tile.walls))
      return false;
    const key = castleVariantId(tile.walls);
    return castleVariants[tile.terrain].some((variant) => variant.id === key);
  }
  if (tile.walls !== void 0) return false;
  if (tile.connections === void 0)
    return !CONNECTED_TERRAINS.includes(tile.terrain) && tile.deadEnds === void 0;
  if (!tile.connections.every((direction) => TERRAIN_DIRECTIONS.includes(direction))) return false;
  const mask = directionMask(tile.connections);
  if (
    (!mask && tile.terrain !== "village" && tile.terrain !== "cave-passage") ||
    new Set(tile.connections).size !== tile.connections.length
  )
    return false;
  if (tile.deadEnds !== void 0) {
    if (
      !tile.deadEnds.every((direction) => TERRAIN_DIRECTIONS.includes(direction)) ||
      new Set(tile.deadEnds).size !== tile.deadEnds.length
    )
      return false;
    const capped = directionMask(tile.deadEnds);
    return (
      tile.terrain === "road" &&
      dead_ends_default.some(([exits, ends]) => exits === mask && ends === capped)
    );
  }
  switch (tile.terrain) {
    case "road":
    case "cave-passage":
      return true;
    case "signpost":
      return mask === 9;
    case "village":
      return tile.connections.length <= 3;
    case "bridge":
    case "ford":
      return [9, 18, 36].includes(mask);
    case "landing":
    case "ferry":
      return [1, 8].includes(mask);
    default:
      return false;
  }
}
function terrainArtNames(tile) {
  if (!validTerrainConnections(tile))
    throw new Error(`Invalid or missing terrain connections: ${tile.terrain}`);
  if (tile.terrain === "castle-wall" || tile.terrain === "castle-bell")
    return [`${tile.terrain}_${castleVariantId(tile.walls)}`];
  if (!CONNECTED_TERRAINS.includes(tile.terrain)) return [tile.terrain];
  const mask = directionMask(tile.connections);
  if (tile.terrain === "road" && tile.deadEnds?.length)
    return [`road_${mask}-end-${directionMask(tile.deadEnds)}`];
  return [`${tile.terrain}_${mask}`];
}

// tmp/pr43-reference/hex-command/scripts/content.ts
var root = new URL("../rust/data/", import.meta.url);
var floors = JSON.parse(
  readFileSync(new URL("../pixel/art/terrain/floors.json", import.meta.url), "utf8"),
);
var paths = /* @__PURE__ */ new Set();
function read(path) {
  if (!path || path.startsWith("/") || path.split("/").includes(".."))
    throw new Error(`Invalid content path: ${path}`);
  paths.add(path);
  return JSON.parse(readFileSync(new URL(path, root), "utf8"));
}
function components(path, fields) {
  const descriptor = read(path);
  for (const field of fields) {
    const component = descriptor[field];
    if (!component && field === "objectives") continue;
    if (!component) throw new Error(`${path}: missing ${field}`);
    const componentPath = posix.join(posix.dirname(path), component);
    const document = read(componentPath);
    if (field === "map") {
      for (const tile of document.tiles ?? []) {
        if (!validTerrainConnections(tile))
          throw new Error(
            `${componentPath} at ${tile.col},${tile.row}: unsupported terrain connections`,
          );
        const frame = terrainArtNames(tile)[0];
        const stem = frame === tile.terrain ? "tile" : frame.slice(tile.terrain.length + 1);
        if (
          !existsSync(new URL(`../pixel/art/terrain/${tile.terrain}/${stem}.txt`, import.meta.url))
        )
          throw new Error(
            `${componentPath} at ${tile.col},${tile.row}: missing terrain source ${frame}`,
          );
        if (tile.floor !== void 0 && !Object.hasOwn(floors.palettes, tile.floor))
          throw new Error(`${componentPath}: unknown floor ${tile.floor}`);
      }
    }
  }
}
var manifest = read("manifest.json");
for (const path of [
  ...manifest.units,
  ...manifest.items,
  manifest.terrain,
  ...(manifest.recruitment ? [manifest.recruitment] : []),
  ...manifest.races,
  ...Object.values(manifest.simulation),
])
  read(path);
var units = new Map(
  manifest.units.map((path) => {
    const unit = read(path);
    return [unit.id, unit];
  }),
);
var enabledLeaders = new Set(read(manifest.simulation.rosters).map((roster) => roster.leader));
var lineupRoot = new URL("../pixel/art/units/lineups/", import.meta.url);
for (const file of readdirSync(lineupRoot).filter((file2) => file2.endsWith(".json"))) {
  const lineup = JSON.parse(readFileSync(new URL(file, lineupRoot), "utf8"));
  const trees = [...lineup.trees];
  if (lineup.commander)
    trees.push({
      base: lineup.commander.units[0],
      branches: [{ units: lineup.commander.units.slice(1) }],
    });
  const id = (unit) => unit.gameplayId ?? unit.id;
  const entries = trees.flatMap((tree) => [
    tree.base,
    ...tree.branches.flatMap((branch) => branch.units),
  ]);
  if (!entries.some((entry) => enabledLeaders.has(id(entry)))) continue;
  for (const tree of trees) {
    const members = [tree.base, ...tree.branches.flatMap((branch) => branch.units)];
    if (!members.some((entry) => entry.gameplayId || units.has(id(entry)))) continue;
    for (const entry of members) {
      const unit = units.get(id(entry));
      const next =
        entry === tree.base
          ? tree.branches.map((branch) => id(branch.units[0]))
          : tree.branches.flatMap((branch) =>
              entry === branch.units[0] ? [id(branch.units[1])] : [],
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
for (const path of manifest.campaigns) {
  const campaign = read(path);
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
