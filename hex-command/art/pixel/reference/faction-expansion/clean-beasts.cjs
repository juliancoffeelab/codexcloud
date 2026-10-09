// Material ownership for the saved animal poses, in their original 64px slots.
// Does not change alpha, limb connections, eyes, mouth holes, or wing contours.
const fs = require("../../../../scripts/pixel-fs.cjs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const root = __dirname,
  project = path.resolve(root, "../../../..");
const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json")));
const ids = process.argv.slice(2);
const rect = (x, y, a, b, c, d) => x >= a && x <= c && y >= b && y <= d;
for (const job of manifest.jobs.filter(
  (j) => j.race.startsWith("beast-") && (!ids.length || ids.includes(j.id)),
)) {
  const dir = path.join(root, "extracted", job.id);
  const original = fs
    .readFileSync(path.join(dir, "sampled.txt"), "utf8")
    .trimEnd()
    .split("\n")
    .map((r) => [...r]);
  const rows = original.map((r) => r.slice()),
    edits = [];
  function set(x, y, to, reason) {
    const from = rows[y][x];
    if (from !== to) {
      rows[y][x] = to;
      edits.push({ x, y, from, to, reason });
    }
  }
  const bone = job.id.startsWith("skeletal-") || job.id === "bone-dragon";
  for (let y = 0; y < 64; y++)
    for (let x = 0; x < 64; x++) {
      const s = rows[y][x];
      if (
        job.id === "giant-spider" &&
        "Ii".includes(s) &&
        !(x >= 36 && x <= 50 && y >= 33 && y <= 45)
      ) {
        set(
          x,
          y,
          s === "I" ? "H" : "h",
          "Chitin legs and abdomen; reserve ivory for attached fangs",
        );
        continue;
      }
      if (job.id === "giant-crab") {
        if ("IiG".includes(s) && rect(x, y, 18, 9, 46, 31)) {
          set(x, y, "H", "Flat coral carapace instead of ivory/gold texture spots");
          continue;
        }
        if ("Vv".includes(s)) continue; // Dark natural leg hinges, not faction fabric.
      }
      if (
        job.id === "giant-tortoise" &&
        "Ii".includes(s) &&
        y >= 39 &&
        y < 50 &&
        (x < 44 || y >= 43)
      ) {
        set(
          x,
          y,
          s === "I" ? "H" : "h",
          "Olive leg tissue; ivory is reserved for beak and connected toe claws",
        );
        continue;
      }
      if (job.id === "giant-wasp") {
        const wing =
          (y <= 20 && x <= 44) ||
          (y >= 20 && y <= 28 && x <= 27) ||
          (y <= 23 && x >= 32 && x <= 43);
        if (wing && s !== "." && s !== "O") {
          set(
            x,
            y,
            "VvBbWw".includes(s) ? "O" : "I",
            "Flat ivory wing membrane with a dark connected rim, no speckles",
          );
          continue;
        }
        if ("BbWwVv".includes(s)) {
          set(
            x,
            y,
            "BWV".includes(s) ? "V" : "v",
            "Dark wasp chitin distinct from gold abdominal bands",
          );
          continue;
        }
      }
      if (
        s === "." ||
        s === "O" ||
        s === "I" ||
        s === "i" ||
        s === "G" ||
        s === "L" ||
        s === "l" ||
        s === "M" ||
        s === "m"
      )
        continue;
      // A ray's diamond is blue tissue; only the narrow central harness is faction cloth.
      if (job.id === "ray" && "Tt".includes(s) && !rect(x, y, 26, 26, 35, 44)) {
        set(x, y, s === "T" ? "H" : "h", "Ray wing tissue outside its central harness");
        continue;
      }
      if ("Tt".includes(s)) continue;
      if (bone) {
        if ("HhVvAaE".includes(s))
          set(
            x,
            y,
            "HVAE".includes(s) ? "I" : "i",
            "Single ivory bone material; preserve dark gaps",
          );
        continue;
      }
      // The retained fur patches on the dead bear remain brown instead of dead flesh.
      if (job.id === "zombie-bear" && "BbWw".includes(s)) {
        set(x, y, "BW".includes(s) ? "B" : "b", "Two-tone remaining brown fur");
        continue;
      }
      if ("BbWwVvAaE".includes(s))
        set(
          x,
          y,
          "BWVAE".includes(s) ? "H" : "h",
          "One natural coat/scale/feather material instead of quantization aliases",
        );
    }
  // Remove isolated surface speckling, retaining connected shadows and all outline cells.
  const bodyAreas = {
    bear: [5, 26, 38, 47],
    lynx: [7, 31, 36, 41],
    stag: [9, 31, 35, 41],
    "giant-tortoise": [7, 21, 41, 38],
    dragon: [20, 31, 40, 46],
    basilisk: [15, 26, 41, 42],
  };
  if (bodyAreas[job.id]) {
    const snap = rows.map((r) => r.slice());
    for (let y = 1; y < 63; y++)
      for (let x = 1; x < 63; x++)
        if (snap[y][x] === "O" && rect(x, y, ...bodyAreas[job.id])) {
          const ns = [];
          for (let dy = -1; dy <= 1; dy++)
            for (let dx = -1; dx <= 1; dx++) if (dx || dy) ns.push(snap[y + dy][x + dx]);
          if (ns.every((s) => "Hh".includes(s)))
            set(
              x,
              y,
              ns.filter((s) => s === "H").length >= 4 ? "H" : "h",
              "Remove enclosed texture dot away from face and anatomical seams",
            );
        }
  }
  for (let pass = 0; pass < 2; pass++) {
    const snap = rows.map((r) => r.slice());
    for (let y = 1; y < 63; y++)
      for (let x = 1; x < 63; x++) {
        const s = snap[y][x];
        if (!"HhBbLlMm".includes(s)) continue;
        const ns = [];
        for (let dy = -1; dy <= 1; dy++)
          for (let dx = -1; dx <= 1; dx++) if (dx || dy) ns.push(snap[y + dy][x + dx]);
        const pair = s.toUpperCase() + s.toLowerCase();
        const other = pair[0] === s ? pair[1] : pair[0];
        if (ns.filter((n) => n === other).length >= 6)
          set(x, y, other, "Remove isolated natural-material fleck inside a connected plane");
      }
  }
  // Thin horizontal islands of the same material form one lit plane with a contour shadow.
  // Fully shadowed far limbs stay shadowed; ivory belly/claws and black anatomical gaps survive.
  for (const pair of ["Hh", "Bb", "Tt", "Ll", "Mm"])
    for (let y = 0; y < 64; y++) {
      let x = 0;
      while (x < 64) {
        if (!pair.includes(rows[y][x])) {
          x++;
          continue;
        }
        const start = x;
        while (x < 64 && pair.includes(rows[y][x])) x++;
        if (x - start < 4) continue;
        const light = rows[y].slice(start, x).filter((s) => s === pair[0]).length;
        if (light === 0 || light === x - start) continue;
        // Keep substantial shadow regions; remove alternating quantization on predominantly lit runs.
        if (light >= (x - start) * 0.65)
          for (let xx = start; xx < x; xx++)
            set(
              xx,
              y,
              xx === x - 1 ? pair[1] : pair[0],
              "Smooth a broad lit material plane, preserve its right contour shadow",
            );
      }
    }
  if (rows.some((r, y) => r.some((s, x) => (s === ".") !== (original[y][x] === "."))))
    throw Error(job.id + ": silhouette changed");
  fs.writeFileSync(path.join(dir, "player.txt"), rows.map((r) => r.join("")).join("\n") + "\n");
  fs.writeFileSync(
    path.join(dir, "material-cleanup.json"),
    JSON.stringify({ coordinates: "64x64 original slot", edits }, null, 2) + "\n",
  );
  execFileSync("python3", ["scripts/render-pixel.py", path.join(dir, "player.txt")], {
    cwd: project,
    stdio: "pipe",
  });
  console.log(job.id + ": " + edits.length + " material edits; alpha unchanged");
}
