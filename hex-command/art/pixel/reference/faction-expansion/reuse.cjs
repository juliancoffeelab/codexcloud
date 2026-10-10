// Extend approved L2 anatomies to L3 without changing their pose or overwriting L1/L2.
const { catalogEntry } = require("./source-directory.cjs");
const fs = require("../../../../scripts/pixel-fs.cjs"),
  path = require("node:path"),
  { execFileSync } = require("node:child_process");
const project = path.resolve(__dirname, "../../../.."),
  units = path.join(project, "art/pixel/units");
const recipes = [
  [
    "revenant",
    "skeleton-dreadguard",
    "Dreadguard",
    "skeleton",
    "Silver face guard and shoulder steel; gold shield rim.",
  ],
  [
    "deatharcher",
    "skeleton-bow-master",
    "Bone bow master",
    "skeleton",
    "Silver arm guards and gold bow bindings.",
  ],
  [
    "direwolf",
    "woodland-alpha-wolf",
    "Alpha wolf",
    "woodland",
    "Pitch-dark coat with silver-gray bristles and muzzle; approved back-fur anatomy.",
  ],
  [
    "greatboar",
    "woodland-ancient-boar",
    "Ancient boar",
    "woodland",
    "Ivory coat, dark mane, and gold-trimmed shoulder harness.",
  ],
  [
    "briarweaver",
    "woodland-thorn-oracle",
    "Thorn oracle",
    "woodland",
    "Pale ivory body garments and bronze-gold leaf mantle; orange fox fur retained.",
  ],
  [
    "grovewarden",
    "woodland-grove-oracle",
    "Grove oracle",
    "woodland",
    "Ivory robes and pale leaf mantle; antlers, face and fitted shoulders retained.",
  ],
  [
    "tridentguard",
    "merfolk-reef-champion",
    "Reef champion",
    "merfolk",
    "Lavender scales, bright silver armor and gold shield rim.",
  ],
  [
    "tidehunter",
    "merfolk-harpoon-master",
    "Harpoon master",
    "merfolk",
    "Lavender scales and silver breast guard, with the same harpoon grip and spare shafts.",
  ],
  [
    "tidesage",
    "merfolk-tide-oracle",
    "Tide oracle",
    "merfolk",
    "Lavender scales and ivory body robe, while mantle and hood remain faction colors.",
  ],
];
const catalogFile = path.join(units, "catalog.json"),
  catalog = JSON.parse(fs.readFileSync(catalogFile));
for (const [base, id, name, race, note] of recipes) {
  const source = path.join(units, base),
    out = path.join(units, id);
  fs.mkdirSync(out, { recursive: true });
  const palette = JSON.parse(fs.readFileSync(path.join(source, "palette.json"))),
    before = structuredClone(palette),
    changes = [];
  Object.assign(palette, {
    A: [243, 247, 239, 255],
    a: [151, 174, 181, 255],
    G: [255, 203, 66, 255],
    g: [185, 126, 34, 255],
  });
  if (race === "skeleton")
    Object.assign(palette, { S: [233, 239, 229, 255], s: [139, 160, 168, 255] });
  if (base === "direwolf")
    Object.assign(palette, {
      H: [27, 32, 39, 255],
      D: [13, 17, 23, 255],
      F: [81, 94, 108, 255],
      G: [162, 178, 190, 255],
    });
  if (base === "greatboar")
    Object.assign(palette, {
      H: [215, 210, 194, 255],
      S: [239, 234, 214, 255],
      D: [150, 147, 132, 255],
      M: [44, 41, 38, 255],
    });
  if (race === "merfolk")
    Object.assign(palette, {
      S: [162, 139, 204, 255],
      s: [92, 69, 132, 255],
      H: [196, 178, 226, 255],
    });
  if (base === "briarweaver")
    Object.assign(palette, { V: [231, 183, 65, 255], v: [143, 103, 35, 255] });
  if (base === "grovewarden")
    Object.assign(palette, { V: [218, 222, 156, 255], v: [129, 151, 80, 255] });
  for (const side of ["player", "enemy"]) {
    const rows = fs
      .readFileSync(path.join(source, side + ".txt"), "utf8")
      .trimEnd()
      .split("\n")
      .map((r) => [...r]);
    const region = (x0, y0, x1, y1, mapping, reason) => {
      for (let y = y0; y <= y1; y++)
        for (let x = x0; x <= x1; x++) {
          const from = rows[y]?.[x],
            to = mapping[from];
          if (to && to !== from) {
            rows[y][x] = to;
            changes.push({ side, x, y, from, to, reason });
          }
        }
    };
    if (base === "revenant")
      region(
        31,
        18,
        42,
        34,
        { A: "G", a: "g", S: "G", s: "g" },
        "Gold shield rim, with faction shield face retained",
      );
    if (base === "deatharcher") {
      region(
        33,
        9,
        39,
        13,
        { A: "G", a: "g", S: "G", s: "g", W: "G", w: "g" },
        "Upper gold bow binding",
      );
      region(
        32,
        35,
        39,
        37,
        { A: "G", a: "g", S: "G", s: "g", W: "G", w: "g" },
        "Lower gold bow binding",
      );
      region(20, 12, 24, 13, { B: "A", b: "a" }, "Silver brow band, eye sockets preserved");
    }
    if (base === "greatboar")
      region(
        20,
        23,
        29,
        30,
        { L: "G", l: "g" },
        "Gold shoulder-harness plates, preserve cloth and coat",
      );
    if (base === "briarweaver" || base === "grovewarden")
      region(
        13,
        25,
        28,
        40,
        { B: "C", b: "c" },
        "Ivory body garment inside existing silhouette; boots and staff remain wood/leather",
      );
    if (base === "tidehunter")
      region(
        20,
        19,
        32,
        31,
        { B: "A", b: "a" },
        "Fitted silver breast guard; gripping hand and shaft unchanged",
      );
    if (base === "tidesage")
      region(
        20,
        22,
        30,
        38,
        { T: "C", t: "c", R: "C", r: "c" },
        "Ivory body robe beneath faction hood and mantle",
      );
    fs.writeFileSync(path.join(out, side + ".txt"), rows.map((r) => r.join("")).join("\n") + "\n");
  }
  fs.writeFileSync(path.join(out, "palette.json"), JSON.stringify(palette, null, 2) + "\n");
  fs.writeFileSync(
    path.join(out, "changes.json"),
    JSON.stringify(
      {
        base,
        level: 3,
        changes,
        paletteChanges: Object.fromEntries(
          Object.entries(palette).filter(
            ([s, c]) => JSON.stringify(c) !== JSON.stringify(before[s]),
          ),
        ),
      },
      null,
      2,
    ) + "\n",
  );
  fs.writeFileSync(
    path.join(out, "components.md"),
    `# ${name} — L3\n\n${note}\n\nInherits anatomy and material meanings from [${base}](../${base}/components.md). Both factions preserve that complete silhouette and all grips. Changes are recorded in changes.json; construction recipe is reference/faction-expansion/reuse.cjs.\n`,
  );
  execFileSync(
    "python3",
    ["scripts/render-pixel.py", path.join(out, "player.txt"), path.join(out, "enemy.txt")],
    { cwd: project, stdio: "pipe" },
  );
  catalog[id] = catalogEntry(id, {
    name,
    level: 3,
    study: "faction-expansion/" + id,
    canvas: [48, 48],
  });
}
fs.writeFileSync(catalogFile, JSON.stringify(catalog, null, 2) + "\n");
fs.writeFileSync(
  path.join(__dirname, "reuse.json"),
  JSON.stringify(
    recipes.map(([base, id, name, race, note]) => ({ base, id, name, race, note })),
    null,
    2,
  ) + "\n",
);
console.log("Nine existing L2 anatomies extended to L3, both factions.");
