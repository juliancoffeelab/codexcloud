// Pose-specific material cleanup. Coordinates use the original 64×64 slot.
// Rebuild with process.cjs extract, this script, then process.cjs tiers.
const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const root = __dirname,
  project = path.resolve(root, "../../../..");
const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json")));
const ids = process.argv.slice(2);
const rect = (x0, y0, x1, y1) => (x, y) => x >= x0 && x <= x1 && y >= y0 && y <= y1;
const metalAreas = {
  "skeleton-flanker": [
    [6, 30, 16, 40],
    [44, 32, 58, 42],
  ],
  "skeleton-impact-caster": [[10, 8, 20, 15]],
  "skeleton-scythe-caster": [[44, 6, 62, 24]],
  "skeleton-axeman": [[42, 8, 61, 27]],
  "skeleton-rider": [[49, 2, 58, 10]],
  "dwarf-axeguard": [[47, 20, 59, 32]],
  "dwarf-hammerer": [[40, 16, 57, 31]],
  "dwarf-pickaxe": [[36, 15, 55, 33]],
  "dwarf-poleaxe": [[38, 9, 58, 30]],
  "dwarf-arquebus": [[38, 25, 59, 29]],
  "dwarf-runesmith": [[46, 19, 57, 29]],
  "dwarf-goat-rider": [[36, 2, 49, 11]],
  "fox-spearguard": [[44, 5, 53, 20]],
  "merfolk-maceguard": [[45, 12, 58, 23]],
};
const helmets = {
  "dwarf-axeguard": [24, 19, 36, 25],
  "dwarf-hammerer": [23, 20, 36, 25],
  "dwarf-pickaxe": [24, 18, 37, 25],
  "dwarf-poleaxe": [23, 17, 37, 24],
  "dwarf-crossbow": [24, 15, 36, 22],
  "dwarf-arquebus": [23, 16, 37, 23],
  "dwarf-runesmith": [25, 16, 39, 25],
  "dwarf-goat-rider": [21, 9, 34, 14],
};
for (const job of manifest.jobs.filter((j) => !ids.length || ids.includes(j.id))) {
  if (job.race.startsWith("beast-")) continue; // Separate animal material ownership, see clean-beasts.cjs.
  const dir = path.join(root, "extracted", job.id),
    p = path.join(dir, "sampled.txt");
  if (!fs.existsSync(p)) continue;
  const source = fs.readFileSync(p, "utf8").trimEnd().split("\n");
  const info = JSON.parse(fs.readFileSync(path.join(dir, "extraction.json"))),
    [ox, oy] = info.origin;
  const original = Array.from({ length: 64 }, () => Array(64).fill("."));
  source.forEach((r, y) => [...r].forEach((s, x) => (original[y + oy][x + ox] = s)));
  const rows = original.map((r) => r.slice()),
    edits = [];
  const renewed = require("./dwarf-renewed.cjs");
  if (renewed.owns(job.id)) {
    if (ox !== 0 || oy !== 0 || source.length !== 64 || source[0].length !== 64)
      throw Error(job.id + ": regenerated dwarf requires its reviewed 64x64 frame");
    const palettePath = path.join(dir, "palette.json");
    const palette = JSON.parse(fs.readFileSync(palettePath));
    const changes = renewed.clean(job.id, rows, palette);
    if (
      rows.some((r, y) =>
        r.some(
          (s, x) =>
            (s === ".") !== (original[y][x] === ".") || (s === "O") !== (original[y][x] === "O"),
        ),
      )
    )
      throw Error(job.id + ": regenerated material cleanup changed outline or silhouette");
    fs.writeFileSync(path.join(dir, "player.txt"), rows.map((r) => r.join("")).join("\n") + "\n");
    fs.writeFileSync(palettePath, JSON.stringify(palette, null, 2) + "\n");
    fs.writeFileSync(
      path.join(dir, "material-cleanup.json"),
      JSON.stringify({ coordinates: "64x64 original slot", edits: changes }, null, 2) + "\n",
    );
    execFileSync("python3", ["scripts/render-pixel.py", path.join(dir, "player.txt")], {
      cwd: project,
      stdio: "pipe",
    });
    console.log(job.id + ": " + changes.length + " regenerated-pose material corrections");
    continue;
  }
  const set = (x, y, to, reason) => {
    const from = rows[y][x];
    if (from === to) return;
    rows[y][x] = to;
    edits.push({ x, y, from, to, reason });
  };
  const map = (test, from, to, reason) => {
    for (let y = 0; y < 64; y++)
      for (let x = 0; x < 64; x++)
        if (test(x, y) && from.includes(rows[y][x]))
          set(x, y, typeof to === "string" ? to : to[rows[y][x]], reason);
  };
  // These masks describe specific equipment in the saved pose, never guessed grips.
  for (const a of metalAreas[job.id] || [])
    map(
      rect(...a),
      "IiHhVv",
      { I: "A", i: "a", H: "A", h: "a", V: "a", v: "a" },
      "Remove bone/cloth quantization colors from the actual metal weapon",
    );
  if (helmets[job.id])
    map(
      rect(...helmets[job.id]),
      "IiVvBbwW",
      { I: "A", i: "a", V: "a", v: "a", B: "a", b: "a", w: "a", W: "a" },
      "Connected iron helmet planes",
    );
  if (job.race === "skeleton") {
    const cloaks = {
      "skeleton-flanker": [12, 20, 30, 29],
      "skeleton-impact-caster": [7, 25, 24, 47],
      "skeleton-scythe-caster": [6, 25, 22, 46],
      "skeleton-axeman": [8, 22, 24, 33],
    };
    if (cloaks[job.id])
      map(
        rect(...cloaks[job.id]),
        "Vv",
        { V: "T", v: "t" },
        "Remove robe-color artifacts from the actual faction cloak",
      );
    // Caster stone remains gray and deliberately differs from the polished scythe.
    map(() => true, "Hh", { H: "i", h: "i" }, "One coherent shadow material on exposed bone");
    if (job.id === "skeleton-impact-caster")
      map(rect(10, 8, 20, 15), "A", "a", "Blunt gray stone staff head");
    if (job.id === "skeleton-scythe-caster")
      map(
        rect(8, 10, 20, 23),
        "HTI",
        { H: "E", T: "E", I: "E" },
        "Remove beige and faction-color speckles from soul flame",
      );
  }
  if (job.race === "zombie") {
    const spell =
      job.id === "zombie-corruption-mage"
        ? (x, y) => (x <= 20 && y >= 17 && y <= 33) || (x >= 48 && y >= 17 && y <= 23)
        : job.id === "zombie-decay-caster"
          ? (x, y) => (x >= 42 && y >= 20 && y <= 32) || (x >= 17 && x <= 22 && y >= 15 && y <= 18)
          : job.id === "zombie-novice-caster"
            ? (x, y) => (x <= 23 && y >= 35 && y <= 42) || (x >= 42 && y >= 22 && y <= 26)
            : () => false;
    map((x, y) => !spell(x, y), "E", "H", "Reserve lime for magic; corpse flesh stays muted");
    const prop = (x, y) => {
      if (job.id === "zombie-plague-carrier") return y < 29 && x < 44;
      if (job.id === "zombie-light-cleaver") return x >= 39 && y >= 35 && y <= 45;
      if (job.id === "zombie-fighter")
        return (x <= 23 && y >= 40) || (x >= 40 && y >= 26 && y <= 43);
      if (job.id === "zombie-cleaver")
        return (x >= 46 && y <= 37) || (x <= 23 && y >= 27 && y <= 45);
      if (job.id.includes("caster") || job.id.includes("mage")) return spell(x, y);
      return false;
    };
    map(
      (x, y) => !prop(x, y),
      "Ii",
      { I: "H", i: "h" },
      "Unify corpse skin instead of accidental ivory highlights",
    );
    if (job.id === "zombie-fighter")
      map(
        rect(10, 40, 23, 50),
        "IiHh",
        { I: "A", i: "a", H: "A", h: "a" },
        "Keep cleaver blade steel",
      );
    if (job.id === "zombie-light-cleaver")
      map(rect(39, 35, 47, 45), "Ii", { I: "A", i: "a" }, "Keep kitchen cleaver steel");
    if (job.id === "zombie-cleaver")
      map(
        rect(46, 18, 59, 36),
        "IiVv",
        { I: "A", i: "a", V: "a", v: "a" },
        "Keep chipped cleaver steel",
      );
  }
  if (job.race === "elf")
    map(
      (x, y) => job.id !== "elf-caster" || x < 43 || y > 17,
      "E",
      "V",
      "Reserve leaf glow for staff crystal",
    );
  if (job.race === "merfolk") {
    map(() => true, "E", "H", "Remove glitter from skin; use a flat sea-green tissue plane");
    map(
      (x, y) => rect(20, 7, 42, 25)(x, y) || y >= 44,
      "Vv",
      { V: "h", v: "h" },
      "Gray flecks on merfolk face/tail belong to skin shadow",
    );
    if (job.id === "merfolk-maceguard") {
      map(
        rect(10, 21, 27, 42),
        "IiWwBbc",
        { I: "C", i: "c", W: "C", w: "c", B: "C", b: "c", c: "c" },
        "Solid coral shield face, remove mottled surface artifacts",
      );
    }
    if (job.id === "merfolk-netter")
      map(rect(29, 36, 39, 43), "Vv", { V: "H", v: "h" }, "Remove gray patch from upper tail");
  }
  if (job.id === "dwarf-hammerer")
    map(
      rect(44, 17, 57, 30),
      "BbWwVv",
      { B: "a", b: "a", W: "a", w: "a", V: "a", v: "a" },
      "Steel hammerhead has no leather or wooden speckling",
    );
  if (job.id === "fox-spearguard") {
    map(
      rect(17, 35, 20, 39),
      "VviAbw",
      { V: "a", v: "a", i: "a", A: "a", b: "O", w: "O" },
      "Solid iron shield boss",
    );
  }
  // Connected two-tone planes. Contours and eyes/knuckles/shaft outlines survive.
  function plane(symbols, light, dark, test = () => true) {
    for (let y = 0; y < 64; y++) {
      let x = 0;
      while (x < 64) {
        if (!test(x, y) || !symbols.includes(rows[y][x])) {
          x++;
          continue;
        }
        const start = x;
        while (x < 64 && test(x, y) && symbols.includes(rows[y][x])) x++;
        if (x - start < 3) continue;
        for (let xx = start; xx < x; xx++)
          set(
            xx,
            y,
            xx === x - 1 ? dark : light,
            "Flatten material plane with one connected contour shadow",
          );
      }
    }
  }
  plane("Tt", "T", "t");
  plane("Vv", "V", "v");
  plane("Hh", "H", "h");
  plane("Cc", "C", "c");
  if (job.race === "skeleton") plane("Ii", "I", "i");
  if (job.race === "dwarf") {
    plane(
      "BbwWVv",
      "B",
      "b",
      (x, y) =>
        !rect(...(helmets[job.id] || [0, 0, 0, 0]))(x, y) &&
        (job.id !== "dwarf-goat-rider" || y <= 28),
    );
  }
  plane("BbWw", "B", "b", (x, y) => y >= 49);
  // Only truly enclosed isolated dots, not deliberate outlines, holes or highlights.
  const snap = rows.map((r) => r.slice());
  for (let y = 1; y < 63; y++)
    for (let x = 1; x < 63; x++) {
      const ns = [
        [0, -1],
        [0, 1],
        [-1, 0],
        [1, 0],
      ].map(([dx, dy]) => snap[y + dy][x + dx]);
      if (
        job.race === "skeleton" &&
        "Vv".includes(snap[y][x]) &&
        ns.filter((s) => "Tt".includes(s)).length >= 3
      )
        set(x, y, "t", "Remove cloth-color fleck enclosed in faction cloak");
      if (
        ns.every((s) => s === ns[0]) &&
        "BVHT".includes(ns[0]) &&
        "BbWwVvHhTt".includes(snap[y][x]) &&
        snap[y][x] !== ns[0]
      )
        set(x, y, ns[0], "Remove enclosed single-pixel surface fleck");
    }
  if (rows.some((r, y) => r.some((s, x) => (s === ".") !== (original[y][x] === "."))))
    throw Error(job.id + ": material cleanup changed silhouette");
  const native =
    rows
      .slice(oy, oy + source.length)
      .map((r) => r.slice(ox, ox + source[0].length).join(""))
      .join("\n") + "\n";
  fs.writeFileSync(path.join(dir, "player.txt"), native);
  fs.writeFileSync(
    path.join(dir, "material-cleanup.json"),
    JSON.stringify({ coordinates: "64x64 original slot", edits }, null, 2) + "\n",
  );
  execFileSync("python3", ["scripts/render-pixel.py", path.join(dir, "player.txt")], {
    cwd: project,
    stdio: "pipe",
  });
  console.log(job.id + ": " + edits.length + " material edits, silhouette preserved");
}
