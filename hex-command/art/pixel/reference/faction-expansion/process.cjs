// Native extraction and material cleanup for the saved four-slot generations.
// Usage: node process.cjs extract <id...> | tiers <id...> | boards
const fs = require("node:fs");
const path = require("node:path");
const { PNG } = require("pngjs");
const { execFileSync } = require("node:child_process");
const { sourceDirectory, catalogEntry, isArchived, tierIds } = require("./source-directory.cjs");
const root = __dirname,
  project = path.resolve(root, "../../../..");
const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json")));
const dwarfBases = new Set(
  require("../dwarf-base-cleanup/sources.json").poses.map((pose) => `dwarf-${pose}`),
);
const [mode, ...ids] = process.argv.slice(2);
const selected = manifest.jobs.filter((j) => !ids.length || ids.includes(j.id));
const groups = {
  O: "outline",
  B: "leather",
  b: "leather",
  W: "wood",
  w: "wood",
  T: "faction",
  t: "faction",
  V: "cloth",
  v: "cloth",
  H: "skin",
  h: "skin",
  I: "bone",
  i: "bone",
  A: "metal",
  a: "metal",
  G: "gold",
  E: "effect",
};
function writeJSON(p, v) {
  fs.writeFileSync(p, JSON.stringify(v, null, 2) + "\n");
}
function components(rows, predicate, diagonal = false) {
  const seen = new Set(),
    found = [],
    height = rows.length,
    width = rows[0].length;
  for (let y = 0; y < height; y++)
    for (let x = 0; x < width; x++) {
      const key = y * width + x;
      if (seen.has(key) || !predicate(rows[y][x])) continue;
      const list = [[x, y]];
      seen.add(key);
      for (let n = 0; n < list.length; n++) {
        const [a, b] = list[n];
        for (let dy = -1; dy <= 1; dy++)
          for (let dx = -1; dx <= 1; dx++) {
            if ((!dx && !dy) || (!diagonal && Math.abs(dx) + Math.abs(dy) !== 1)) continue;
            const xx = a + dx,
              yy = b + dy,
              k = yy * width + xx;
            if (
              xx < 0 ||
              xx >= width ||
              yy < 0 ||
              yy >= height ||
              seen.has(k) ||
              !predicate(rows[yy][xx])
            )
              continue;
            seen.add(k);
            list.push([xx, yy]);
          }
      }
      found.push(list);
    }
  return found;
}
function render(dir) {
  execFileSync(
    "python3",
    [
      "scripts/render-pixel.py",
      path.join(dir, "player.txt"),
      ...(fs.existsSync(path.join(dir, "enemy.txt")) ? [path.join(dir, "enemy.txt")] : []),
    ],
    { cwd: project, stdio: "pipe" },
  );
}
function extract(job) {
  const dir = path.join(root, "extracted", job.id);
  fs.mkdirSync(dir, { recursive: true });
  const source = PNG.sync.read(fs.readFileSync(path.join(root, "sources", job.id + ".png"))),
    palette = job.palette || manifest.races[job.race].palette;
  const x0 = Math.floor(source.width / 2),
    y0 = Math.floor(source.height / 2),
    crop = new PNG({ width: source.width - x0, height: source.height - y0 });
  for (let y = 0; y < crop.height; y++)
    for (let x = 0; x < crop.width; x++) {
      const i = ((y0 + y) * source.width + x0 + x) * 4;
      crop.data.set(source.data.subarray(i, i + 4), (y * crop.width + x) * 4);
    }
  fs.writeFileSync(path.join(dir, "crop.png"), PNG.sync.write(crop));
  const colors = Object.entries(palette).filter(([s]) => s !== ".");
  const symbols = new Array(crop.width * crop.height);
  for (let p = 0; p < symbols.length; p++) {
    const i = p * 4,
      r = crop.data[i],
      g = crop.data[i + 1],
      b = crop.data[i + 2];
    if (!crop.data[i + 3] || (r > 160 && b > 150 && g < 90)) {
      symbols[p] = ".";
      continue;
    }
    let best = ".",
      distance = (r - 255) ** 2 + g * g + (b - 255) ** 2;
    for (const [s, c] of colors) {
      const d = (r - c[0]) ** 2 + (g - c[1]) ** 2 + (b - c[2]) ** 2;
      if (d < distance) {
        best = s;
        distance = d;
      }
    }
    symbols[p] = best;
  }
  const raw = [];
  for (let y = 0; y < 64; y++) {
    const row = [];
    for (let x = 0; x < 64; x++) {
      const counts = {},
        left = Math.floor((x * crop.width) / 64),
        right = Math.floor(((x + 1) * crop.width) / 64),
        top = Math.floor((y * crop.height) / 64),
        bottom = Math.floor(((y + 1) * crop.height) / 64);
      let fg = 0,
        total = 0;
      for (let yy = top; yy < bottom; yy++)
        for (let xx = left; xx < right; xx++) {
          total++;
          const s = symbols[yy * crop.width + xx];
          if (s !== ".") {
            fg++;
            counts[s] = (counts[s] || 0) + 1;
          }
        }
      let best = ".",
        max = 0;
      if (fg / total >= 0.35)
        for (const [s, n] of Object.entries(counts))
          if (n > max) {
            best = s;
            max = n;
          }
      row.push(best);
    }
    raw.push(row);
  }
  const changes = [],
    rows = raw.map((r) => r.slice());
  function change(x, y, to, reason) {
    if (rows[y][x] === to) return;
    changes.push({ x, y, from: rows[y][x], to, reason });
    rows[y][x] = to;
  }
  // Only disconnected one/two-pixel debris: main weapon/body geometry is untouched.
  for (const component of components(rows, (s) => s !== ".", true))
    if (component.length <= 2 || component.every(([, y]) => y < 4))
      for (const [x, y] of component)
        change(
          x,
          y,
          ".",
          "Disconnected generation debris or slot label; connected weapon tips are retained",
        );
  const before = rows.map((r) => r.slice());
  for (let y = 1; y < 63; y++)
    for (let x = 1; x < 63; x++) {
      const s = before[y][x];
      if (s === "." || s === "O" || s === "G" || s === "E") continue;
      const neighbors = [];
      for (let dy = -1; dy <= 1; dy++)
        for (let dx = -1; dx <= 1; dx++) if (dx || dy) neighbors.push(before[y + dy][x + dx]);
      const counts = {};
      for (const n of neighbors) if (n !== "." && n !== "O") counts[n] = (counts[n] || 0) + 1;
      let best = s,
        max = 0;
      for (const [n, c] of Object.entries(counts))
        if (c > max) {
          best = n;
          max = c;
        }
      if (max >= 6 && best !== s)
        change(x, y, best, "Enclosed single-pixel material fleck in a broad flat surface");
    }
  let minX = 64,
    minY = 64,
    maxX = -1,
    maxY = -1;
  for (let y = 0; y < 64; y++)
    for (let x = 0; x < 64; x++)
      if (rows[y][x] !== ".") {
        minX = Math.min(x, minX);
        minY = Math.min(y, minY);
        maxX = Math.max(x, maxX);
        maxY = Math.max(y, maxY);
      }
  let size = job.size,
    ox = 8,
    oy = 8;
  if (size === 64 || minX < 8 || maxX >= 56 || minY < 8 || maxY >= 56) {
    size = 64;
    ox = oy = 0;
  }
  const native = rows.slice(oy, oy + size).map((r) => r.slice(ox, ox + size).join(""));
  fs.writeFileSync(
    path.join(dir, "extracted-64.txt"),
    raw.map((r) => r.join("")).join("\n") + "\n",
  );
  fs.writeFileSync(path.join(dir, "player.txt"), native.join("\n") + "\n");
  writeJSON(path.join(dir, "palette.json"), palette);
  fs.writeFileSync(path.join(dir, "sampled.txt"), native.join("\n") + "\n");
  writeJSON(path.join(dir, "cleanup.json"), {
    coordinates: "64x64 logical slot before native crop",
    changes,
  });
  writeJSON(path.join(dir, "extraction.json"), {
    source: [source.width, source.height],
    crop: [x0, y0, crop.width, crop.height],
    native: [size, size],
    origin: [ox, oy],
    bounds: [minX, minY, maxX, maxY],
    retainedFullFrame: size !== job.size,
    foregroundComponents: components(rows, (s) => s !== ".", true).map((c) => c.length),
  });
  render(dir);
  job.status = "extracted";
  job.canvas = [size, size];
  console.log(`${job.id}: ${size}x${size}, ${changes.length} recorded cleanup edits`);
}
function tiers(job) {
  if (job.level === 3) {
    const dir = path.join(root, "extracted", job.id);
    const out = sourceDirectory(job.id);
    fs.mkdirSync(out, { recursive: true });
    const player = fs.readFileSync(path.join(dir, "player.txt"), "utf8");
    const palette = JSON.parse(fs.readFileSync(path.join(dir, "palette.json")));
    palette.R = [185, 24, 37, 255];
    palette.r = [112, 14, 24, 255];
    fs.writeFileSync(path.join(out, "player.txt"), player);
    fs.writeFileSync(path.join(out, "enemy.txt"), player.replaceAll("T", "R").replaceAll("t", "r"));
    writeJSON(path.join(out, "palette.json"), palette);
    writeJSON(path.join(out, "changes.json"), {
      base: "tideseer",
      level: 3,
      source: `faction-expansion/sources/${job.id}.png`,
      changes: [],
    });
    fs.writeFileSync(
      path.join(out, "components.md"),
      `# ${job.name}\n\nL3 leader artwork after Tidekeeper and Tide seer. Sea-green body and compact curled tail, gold reef crown, ivory shell shoulder guards and shield, faction tabard and mantle, upright trident. Artwork only: gameplay advancement and stats are unchanged. Rebuild with \`npm run art:expansion -- ${job.id}\`.\n`,
    );
    const catalogPath = path.join(project, "art/pixel/units/catalog.json");
    const catalog = JSON.parse(fs.readFileSync(catalogPath));
    const rows = player.trimEnd().split("\n");
    catalog[job.id] = {
      name: job.name,
      level: 3,
      study: `faction-expansion/${job.id}`,
      canvas: [rows[0].length, rows.length],
    };
    writeJSON(catalogPath, catalog);
    render(out);
    job.status = "tiers";
    return;
  }
  if (job.recipe === "merfolk-renewed") {
    require("./merfolk-renewed.cjs").build(job);
    job.status = "tiers";
    return;
  }
  if (job.id === "dwarf-runesmith") {
    execFileSync("python3", ["art/pixel/reference/rune-line/build.py"], {
      cwd: project,
      stdio: "inherit",
    });
    job.status = "tiers";
    console.log("dwarf-runesmith: rune apprentice, Thundersmith and Stonespell, both factions");
    return;
  }
  if (dwarfBases.has(job.id)) {
    execFileSync("python3", ["art/pixel/reference/dwarf-base-cleanup/build.py", job.id], {
      cwd: project,
      stdio: "inherit",
    });
    execFileSync("python3", ["art/pixel/reference/dwarf-upgrades/build.py", job.id], {
      cwd: project,
      stdio: "inherit",
    });
    job.status = "tiers";
    console.log(`${job.id}: cleaned base and whole-surface L2/L3 upgrades, both factions`);
    return;
  }
  const catalogPath = path.join(project, "art/pixel/units/catalog.json"),
    catalog = JSON.parse(fs.readFileSync(catalogPath)),
    archivePath = path.join(root, "../zombies/catalog.json"),
    archived = JSON.parse(fs.readFileSync(archivePath));
  for (let level = 1; level <= 3; level++) {
    const id = level === 1 ? job.id : `${job.id}-l${level}`,
      out = sourceDirectory(id);
    fs.mkdirSync(out, { recursive: true });
    const chains = {
      "zombie-wretch": ["zombie-wretch", "zombie-walker", "zombie-shambler"],
      "zombie-light-cleaver": ["zombie-light-cleaver", "zombie-fighter", "zombie-cleaver"],
      "zombie-novice-caster": [
        "zombie-novice-caster",
        "zombie-decay-caster",
        "zombie-corruption-mage",
      ],
    };
    const sourceId = chains[job.id]?.[level - 1] || job.id,
      dir = path.join(root, "extracted", sourceId);
    const base = fs.readFileSync(path.join(dir, "player.txt"), "utf8").trimEnd().split("\n"),
      basePalette = JSON.parse(fs.readFileSync(path.join(dir, "palette.json"))),
      origin = JSON.parse(fs.readFileSync(path.join(dir, "extraction.json"))).origin;
    const palette = JSON.parse(JSON.stringify(basePalette)),
      rows = base.map((r) => r.split("")),
      changes = [];
    // Tier changes use body cloth and equipment symbols, never skin, bone, wood or grips.
    if (level === 2) {
      palette.A = [222, 232, 235, 255];
      palette.a = [129, 151, 160, 255];
      if (job.race === "skeleton") {
        palette.V = [108, 87, 126, 255];
        palette.v = [64, 47, 81, 255];
      }
      if (job.race === "zombie") {
        palette.V = [82, 87, 55, 255];
        palette.v = [45, 52, 33, 255];
      }
      if (job.race === "elf") {
        palette.V = [164, 179, 109, 255];
        palette.v = [100, 122, 63, 255];
      }
      if (job.race === "dwarf") {
        palette.V = [140, 161, 173, 255];
        palette.v = [78, 99, 113, 255];
      }
      if (job.race === "woodland") {
        palette.V = [184, 189, 129, 255];
        palette.v = [105, 126, 65, 255];
      }
      if (job.race === "merfolk") {
        palette.H = [136, 116, 178, 255];
        palette.h = [78, 61, 117, 255];
        palette.V = [159, 172, 199, 255];
        palette.v = [86, 109, 143, 255];
      }
    }
    if (level === 3) {
      palette.A = [244, 248, 241, 255];
      palette.a = [155, 184, 191, 255];
      palette.G = [255, 204, 65, 255];
      if (job.race === "skeleton") {
        palette.V = [155, 51, 78, 255];
        palette.v = [85, 28, 53, 255];
      }
      if (job.race === "zombie") {
        palette.V = [103, 64, 120, 255];
        palette.v = [55, 35, 71, 255];
      }
      if (job.race === "elf") {
        palette.V = [232, 227, 184, 255];
        palette.v = [159, 173, 107, 255];
      }
      if (job.race === "dwarf") {
        palette.V = [235, 230, 209, 255];
        palette.v = [157, 171, 168, 255];
      }
      if (job.race === "woodland") {
        palette.V = [237, 230, 194, 255];
        palette.v = [151, 177, 111, 255];
      }
      if (job.race === "merfolk") {
        palette.H = [164, 143, 201, 255];
        palette.h = [95, 73, 141, 255];
        palette.V = [226, 227, 211, 255];
        palette.v = [139, 165, 179, 255];
      }
    }
    changes.push(...require("./upgrades.cjs")({ job, level, rows, palette, origin }));
    if (job.tierColors?.[level - 1]) Object.assign(palette, job.tierColors[level - 1]);
    palette.R = [185, 24, 37, 255];
    palette.r = [112, 14, 24, 255];
    const player = rows.map((r) => r.join("")).join("\n") + "\n",
      enemy = player.replaceAll("T", "R").replaceAll("t", "r");
    fs.writeFileSync(path.join(out, "player.txt"), player);
    fs.writeFileSync(path.join(out, "enemy.txt"), enemy);
    writeJSON(path.join(out, "palette.json"), palette);
    writeJSON(path.join(out, "changes.json"), {
      base: sourceId,
      level,
      changes,
      paletteChanges: Object.fromEntries(
        Object.entries(palette).filter(
          ([s, c]) => JSON.stringify(c) !== JSON.stringify(basePalette[s]),
        ),
      ),
    });
    const sourceJob = manifest.jobs.find((j) => j.id === sourceId);
    const entry = catalogEntry(id, {
      name: job.name + (level > 1 ? ` L${level}` : ""),
      level,
      study: `faction-expansion/${id}`,
      canvas: [rows[0].length, rows.length],
    });
    const title =
      job.race !== "zombie" || isArchived(id)
        ? `${job.name} — L${level}`
        : entry.role === "commander"
          ? `${entry.name} — commander`
          : `${entry.name} — L${entry.level}`;
    const materials = job.race.startsWith("beast-")
      ? "H/h natural coat, scales or feathers; I/i bone, belly, claws or suckers; B/b remaining fur on zombie bear; L/l treant leaves; M/m dragon wing membrane."
      : "H/h tissue, I/i bone or ivory, B/b leather, W/w wood, V/v body clothing, C/c coral equipment, A/a metal, E spell effect.";
    fs.writeFileSync(
      path.join(out, "components.md"),
      `# ${title}\n\n${sourceJob.pose}\n\nAnatomy source: ${sourceId}. Race appearance: ${manifest.races[job.race].vibe}\n\nNative canvas: ${rows[0].length}×${rows.length}. Source generation and extraction are in art/pixel/reference/faction-expansion/. Both factions share the same geometry; T/t faction cloth becomes R/r. O is outline; G is gold. ${materials} Tier edits preserve this source pose and grips.\n${job.race === "zombie" ? `\nSource recipe tier: L${level}. Chosen lineup tiers and commander role are recorded in art/pixel/lineups/zombies.json.\n` : ""}`,
    );
    render(out);
    if (isArchived(id)) archived[id] = entry;
    else catalog[id] = entry;
  }
  writeJSON(catalogPath, catalog);
  writeJSON(archivePath, archived);
  job.status = "tiers";
  console.log(`${job.id}: L1-L3, both factions`);
}
function boards() {
  for (const race of Object.keys(manifest.races)) {
    if (ids.length && !ids.includes(race)) continue;
    const jobs = manifest.jobs.filter(
      (j) => j.race === race && fs.existsSync(path.join(root, "extracted", j.id, "player.png")),
    );
    if (!jobs.length) continue;
    const bases = jobs.map((j) => ({
      label: j.name,
      image: path.relative(project, path.join(root, "extracted", j.id, "player.png")),
    }));
    const p = path.join(root, `${race}-bases.json`);
    writeJSON(p, bases);
    execFileSync(
      "node",
      [
        "art/pixel/reference/human-anatomies/preview.cjs",
        p,
        path.join(root, `${race}-bases.png`),
        "3",
        "6",
      ],
      { cwd: project },
    );
    for (const side of ["player", "enemy"]) {
      let entries =
        race === "dwarf"
          ? require("../../lineups/dwarves.json")
              .trees.flatMap((tree) => [
                tree.base,
                ...tree.branches.flatMap((branch) => branch.units),
              ])
              .map((unit) => ({
                label: `${unit.name} L${unit.level}`,
                image: path.relative(project, path.join(sourceDirectory(unit.id), `${side}.png`)),
              }))
          : jobs
              .filter((j) => j.status === "tiers")
              .flatMap((j) =>
                tierIds(j).map((id) => ({
                  label: `${catalogEntry(id, { name: j.name }).name} L${id === j.id ? 1 : Number(id.slice(-1))}`,
                  image: path.relative(project, path.join(sourceDirectory(id), `${side}.png`)),
                })),
              );
      if (race === "merfolk") {
        const lineup = require("../../lineups/merfolk.json");
        entries = [
          ...lineup.trees.flatMap((tree) =>
            tree.branches.flatMap((branch) => [
              { ...tree.base, name: `${tree.base.name} → ${branch.role}` },
              ...branch.units,
            ]),
          ),
          ...lineup.commander.units.map((unit) => ({ ...unit, name: `${unit.name} (commander)` })),
        ].map((unit) => ({
          label: `${unit.name} L${unit.level}`,
          image: path.relative(project, path.join(sourceDirectory(unit.id), `${side}.png`)),
        }));
      }
      if (!entries.length) continue;
      const q = path.join(root, `${race}-${side}.json`);
      writeJSON(q, entries);
      execFileSync(
        "node",
        [
          "art/pixel/reference/human-anatomies/preview.cjs",
          q,
          path.join(root, `${race}-${side}.png`),
          "3",
          race === "merfolk" ? "3" : "6",
        ],
        { cwd: project },
      );
    }
  }
}
if (mode === "extract") for (const job of selected) extract(job);
else if (mode === "tiers") for (const job of selected) tiers(job);
else if (mode === "boards") boards();
else throw new Error("Expected extract, tiers or boards");
writeJSON(path.join(root, "manifest.json"), manifest);
