// Real map-drafter renders with explicit authoring snapshots, not simulated battles.
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { createCanvas, loadImage } from "@napi-rs/canvas";

const destination = resolve(process.argv[2] ?? "tmp/visibility-examples");
await mkdir(destination, { recursive: true });
const cols = 11,
  rows = 7;
const cells = Array.from({ length: cols * rows }, (_, i) => ({
  col: i % cols,
  row: Math.floor(i / cols),
}));
const darkness = cells.filter(({ col }) => col >= 4);
const profile = (range, penetrates = []) => [
  { id: "view", sight: { range, penetrates }, explored: [] },
];
const map = {
  mapId: "visibility-study",
  cols,
  rows,
  lighting: { time: "day" },
  factions: [
    {
      id: "player",
      phase: "player",
      controller: { kind: "human" },
      visionGroup: "view",
      palette: "player",
      relations: {},
      recruits: [],
    },
  ],
  visionGroups: profile("unlimited"),
  units: [{ type: "ranger", faction: "player", col: 3, row: 3 }],
  tiles: cells.map(({ col, row }) => ({
    col,
    row,
    terrain: col === 7 && row === 3 ? "village" : (col + row * 2) % 5 < 2 ? "forest" : "plains",
    floor: (col + row * 2) % 5 < 2 && !(col === 7 && row === 3) ? "green-forest" : "green-plain",
    connections: [],
    owner: null,
  })),
};
const states = [
  {
    id: "day",
    title: "1. Daylight",
    notes: ["Baseline: full terrain view, no darkness condition."],
  },
  {
    id: "night",
    title: "2. Night is lighting",
    lighting: { time: "night" },
    notes: ["Same full view. Cooler, dimmer artwork does not itself restrict sight."],
  },
  {
    id: "remembered",
    title: "3. Remembered terrain",
    rememberedPreview: darkness,
    notes: [
      "Previously explored ground: dimmer, cool-tinted, still colorful.",
      "Author-supplied terrain snapshot; current occupants are not shown.",
    ],
  },
  {
    id: "unexplored",
    title: "4. Unexplored terrain",
    visionGroups: profile(2),
    notes: [
      "Outside sight, with no exploration memory: opaque dark fog.",
      "You do not know what terrain is beneath it.",
    ],
  },
  {
    id: "darkness",
    title: "5. Darkness: a hostile veil",
    lighting: { time: "night" },
    darknessPreview: darkness,
    notes: [
      "Author overview: darkness swallows color and terrain detail.",
      "Black-violet atmospheric wisps distinguish it from ordinary night.",
    ],
  },
  {
    id: "sight",
    title: "6. Ordinary sight into darkness",
    lighting: { time: "night" },
    darknessPreview: darkness,
    visionGroups: profile(5),
    restricted: true,
    notes: [
      "Sight 5; darkness reveals only within 1 hex of the ranger.",
      "Fog masks the rest; preview tint is drawn beneath fog.",
    ],
  },
  {
    id: "darkvision",
    darkvisionPreview: true,
    title: "7. Darkvision through the veil",
    lighting: { time: "night" },
    darknessPreview: darkness,
    visionGroups: profile(5, ["darkness"]),
    restricted: true,
    notes: [
      "Same sight 5. Darkness no longer blocks terrain detection.",
      "Terrain stays dark and purple: darkvision is not a light source.",
    ],
  },
  {
    id: "lantern",
    title: "8. Lantern against darkness",
    lighting: { time: "night" },
    darknessPreview: darkness.filter(
      ({ col, row }) => !(col >= 5 && col <= 8 && row >= 2 && row <= 4),
    ),
    notes: [
      "Author overview, with a real rendered tile light at the village.",
      "The cleared condition cells are explicit, not computed from glow pixels.",
    ],
  },
];
for (const state of states) {
  const snapshot = structuredClone(map);
  if (state.restricted)
    for (const tile of snapshot.tiles)
      if (tile.col >= 4)
        tile.perception = {
          remember: false,
          concealment: [{ kind: "darkness", revealWithin: 1, requires: {} }],
        };
  if (state.id === "lantern")
    snapshot.tiles.find((t) => t.col === 7 && t.row === 3).light = {
      radius: 150,
      power: 1,
      fixture: true,
    };
  const mapPath = resolve(destination, `${state.id}.map.json`);
  const overlayPath = resolve(destination, `${state.id}.overlay.json`);
  const { id, restricted, ...overlay } = state;
  await writeFile(mapPath, JSON.stringify(snapshot, null, 2));
  await writeFile(overlayPath, JSON.stringify(overlay, null, 2));
  const result = spawnSync(
    process.execPath,
    [
      "scripts/render-map.mjs",
      "--map",
      mapPath,
      "--overlay",
      overlayPath,
      "--output",
      resolve(destination, `${id}.png`),
    ],
    { stdio: "inherit" },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Rendering ${id} failed (${result.status})`);
}
const images = await Promise.all(
  states.map((state) => loadImage(resolve(destination, `${state.id}.png`))),
);
const width = Math.max(...images.map((image) => image.width));
const height = Math.max(...images.map((image) => image.height));
const sheet = createCanvas(
  width * 2 + 24,
  height * Math.ceil(states.length / 2) + 24 * (Math.ceil(states.length / 2) - 1),
);
const ctx = sheet.getContext("2d");
ctx.fillStyle = "#101827";
ctx.fillRect(0, 0, sheet.width, sheet.height);
images.forEach((image, i) =>
  ctx.drawImage(image, (i % 2) * (width + 24), Math.floor(i / 2) * (height + 24)),
);
await writeFile(resolve(destination, "comparison.png"), sheet.toBuffer("image/png"));
console.log(`Visibility comparison: ${resolve(destination, "comparison.png")}`);
