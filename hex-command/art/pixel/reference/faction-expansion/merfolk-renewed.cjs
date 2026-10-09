// Material ownership for the three regenerated old-roster merfolk poses.
const fs = require("../../../../scripts/pixel-fs.cjs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const root = __dirname;
const project = path.resolve(root, "../../../..");
const ids = new Set(["merfolk-trident", "merfolk-netter", "merfolk-maceguard"]);

function clean(job) {
  const dir = path.join(root, "extracted", job.id);
  const rows = fs
    .readFileSync(path.join(dir, "sampled.txt"), "utf8")
    .trimEnd()
    .split("\n")
    .map((r) => [...r]);
  const palette = JSON.parse(fs.readFileSync(path.join(dir, "palette.json")));
  const edits = [];
  // Grow skin surfaces from actual skin seeds. Weapon steel is disconnected by
  // hands/shafts and contours, so a nearby grey weapon is not a skin rectangle.
  const points = [];
  const seen = new Set();
  rows.forEach((row, y) =>
    row.forEach((c, x) => {
      if ("Hh".includes(c)) {
        points.push([x, y]);
        seen.add(`${x},${y}`);
      }
    }),
  );
  for (let i = 0; i < points.length; i++) {
    const [x, y] = points[i];
    for (const [xx, yy] of [
      [x - 1, y],
      [x + 1, y],
      [x, y - 1],
      [x, y + 1],
    ]) {
      const key = `${xx},${yy}`;
      if (seen.has(key) || !"HhVva".includes(rows[yy]?.[xx] || ".")) continue;
      seen.add(key);
      points.push([xx, yy]);
    }
  }
  for (const [x, y] of points) {
    if (!"Vva".includes(rows[y][x])) continue;
    edits.push({
      x,
      y,
      from: rows[y][x],
      to: "h",
      reason: "Connected flesh shadow uses old-roster sea-green, not grey-purple cloth",
    });
    rows[y][x] = "h";
  }
  palette.H = [89, 154, 144, 255];
  palette.h = [48, 102, 103, 255];
  fs.writeFileSync(path.join(dir, "player.txt"), rows.map((r) => r.join("")).join("\n") + "\n");
  fs.writeFileSync(path.join(dir, "palette.json"), JSON.stringify(palette, null, 2) + "\n");
  fs.writeFileSync(
    path.join(dir, "material-cleanup.json"),
    JSON.stringify({ coordinates: "native cropped frame", edits }, null, 2) + "\n",
  );
  execFileSync("python3", ["scripts/render-pixel.py", path.join(dir, "player.txt")], {
    cwd: project,
  });
}

function build(job) {
  clean(job);
  const dir = path.join(root, "extracted", job.id);
  const source = fs.readFileSync(path.join(dir, "player.txt"), "utf8").trimEnd().split("\n");
  const basePalette = JSON.parse(fs.readFileSync(path.join(dir, "palette.json")));
  const {
    origin: [ox, oy],
  } = JSON.parse(fs.readFileSync(path.join(dir, "extraction.json")));
  const catalogPath = path.join(project, "art/pixel/units/catalog.json");
  const catalog = JSON.parse(fs.readFileSync(catalogPath));
  for (const level of [1, 2, 3]) {
    const id = level === 1 ? job.id : `${job.id}-l${level}`;
    const out = path.join(project, "art/pixel/units", id);
    const rows = source.map((r) => [...r]);
    const palette = { ...basePalette, R: [185, 24, 37, 255], r: [112, 14, 24, 255] };
    const changes = [];
    if (level > 1) {
      palette.A = level === 2 ? [222, 232, 235, 255] : [244, 248, 241, 255];
      palette.a = level === 2 ? [129, 151, 160, 255] : [155, 184, 191, 255];
      if (level === 3) palette.G = [255, 204, 65, 255];
      rows.forEach((row, y) =>
        row.forEach((c, x) => {
          const sx = x + ox,
            sy = y + oy;
          // Reviewed chest harness on the new poses; shafts, grips and flesh stay
          // intact. L3 also plates the existing round shield's wooden face.
          const chest = sx >= 25 && sx <= 40 && sy >= 25 && sy <= 36;
          const shield =
            job.id === "merfolk-maceguard" && sx >= 13 && sx <= 25 && sy >= 25 && sy <= 40;
          if (!(chest || (level === 3 && shield)) || !"BbWwCc".includes(c)) return;
          const to = "bwc".includes(c) ? "a" : "A";
          changes.push({
            x,
            y,
            from: c,
            to,
            reason: shield
              ? "L3 silver shield face"
              : "Steel chest harness following generated straps",
          });
          row[x] = to;
        }),
      );
    }
    if (job.id === "merfolk-trident")
      changes.push(...require("./merfolk-tridents.cjs").apply(rows, palette));
    const player = rows.map((r) => r.join("")).join("\n") + "\n";
    fs.writeFileSync(path.join(out, "player.txt"), player);
    fs.writeFileSync(path.join(out, "enemy.txt"), player.replaceAll("T", "R").replaceAll("t", "r"));
    fs.writeFileSync(path.join(out, "palette.json"), JSON.stringify(palette, null, 2) + "\n");
    fs.writeFileSync(
      path.join(out, "changes.json"),
      JSON.stringify({ base: job.id, level, recipe: "merfolk-renewed.cjs", changes }, null, 2) +
        "\n",
    );
    fs.writeFileSync(
      path.join(out, "components.md"),
      `# ${catalog[id].name}\n\nRegenerated from old-roster merfolk anchors with a compact upright torso and one connected curled supporting tail. Native skin, hand grips, weapon shafts, net mesh and contours are retained through tiers. Teal cloth alone swaps to red. Rebuild with \`npm run art:expansion -- ${job.id}\`.\n`,
    );
    catalog[id].canvas = [rows[0].length, rows.length];
    execFileSync(
      "python3",
      ["scripts/render-pixel.py", path.join(out, "player.txt"), path.join(out, "enemy.txt")],
      { cwd: project },
    );
  }
  fs.writeFileSync(catalogPath, JSON.stringify(catalog, null, 2) + "\n");
}
module.exports = { owns: (id) => ids.has(id), clean, build };
