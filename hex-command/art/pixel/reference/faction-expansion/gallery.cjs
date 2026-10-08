// Regenerate review sheets and paired numbered source/result overlays.
const fs = require("node:fs"),
  path = require("node:path"),
  { PNG } = require("pngjs"),
  { execFileSync } = require("node:child_process");
const root = __dirname,
  project = path.resolve(root, "../../../..");
const { sourceDirectory, catalogEntry, isArchived } = require("./source-directory.cjs");
const m = JSON.parse(fs.readFileSync(path.join(root, "manifest.json"))),
  reuse = JSON.parse(fs.readFileSync(path.join(root, "reuse.json")));
const rel = (p) => path.relative(project, p);
function board(name, entries, columns, scale) {
  const p = path.join(root, name + ".json");
  fs.writeFileSync(p, JSON.stringify(entries, null, 2) + "\n");
  execFileSync(
    "node",
    [
      "art/pixel/reference/human-anatomies/preview.cjs",
      p,
      path.join(root, name + ".png"),
      String(columns),
      String(scale),
    ],
    { cwd: project },
  );
}
if (!process.argv.includes("--html-only")) {
  for (const [name, beast, columns] of [
    ["overview-humanoids", false, 6],
    ["overview-beasts", true, 5],
  ]) {
    board(
      name,
      m.jobs
        .filter((j) => j.race.startsWith("beast-") === beast)
        .map((j, n) => ({
          label: `${n + 1}. ${j.name}`,
          image: rel(path.join(sourceDirectory(j.id), "player.png")),
        })),
      columns,
      4,
    );
  }
  for (const side of ["player", "enemy"])
    board(
      "existing-l3-" + side,
      reuse.flatMap((j) =>
        [j.base, j.id].map((id) => ({ label: id, image: `art/pixel/units/${id}/${side}.png` })),
      ),
      2,
      6,
    );
  const gridDir = path.join(root, "grids");
  fs.mkdirSync(gridDir, { recursive: true });
  for (const j of m.jobs) {
    const dir = path.join(root, "extracted", j.id),
      info = JSON.parse(fs.readFileSync(path.join(dir, "extraction.json")));
    const result = PNG.sync.read(fs.readFileSync(path.join(dir, "player.png"))),
      source = PNG.sync.read(fs.readFileSync(path.join(dir, "crop.png")));
    const scale = 12,
      margin = 36,
      size = result.width,
      body = size * scale,
      width = body + margin + 8,
      height = body + margin + 8;
    for (const [label, img] of [
      ["source", source],
      ["matrix", result],
    ]) {
      const out = new PNG({ width, height });
      for (let i = 0; i < out.data.length; i += 4) out.data.set([20, 31, 26, 255], i);
      for (let y = 0; y < body; y++)
        for (let x = 0; x < body; x++) {
          let sx, sy;
          if (label === "source") {
            sx = Math.floor(((x / scale + info.origin[0]) * source.width) / 64);
            sy = Math.floor(((y / scale + info.origin[1]) * source.height) / 64);
          } else {
            sx = Math.floor(x / scale);
            sy = Math.floor(y / scale);
          }
          const si = (sy * img.width + sx) * 4;
          const color = img.data[si + 3]
            ? img.data.subarray(si, si + 4)
            : (Math.floor(x / (8 * scale)) + Math.floor(y / (8 * scale))) % 2
              ? [58, 64, 60, 255]
              : [43, 50, 46, 255];
          out.data.set(color, ((y + margin) * width + x + margin) * 4);
        }
      const commands = ["fill '#e1ddc8' font 'DejaVu-Sans' font-size 8"];
      for (let n = 0; n < size; n++)
        commands.push(
          `text ${margin + n * scale + 1},${margin - 7} '${n}'`,
          `text 12,${margin + n * scale + 9} '${n}'`,
        );
      commands.push("stroke '#aab8aa55' stroke-width 0.5");
      for (let n = 0; n <= size; n++)
        commands.push(
          `line ${margin + n * scale},${margin} ${margin + n * scale},${margin + body}`,
          `line ${margin},${margin + n * scale} ${margin + body},${margin + n * scale}`,
        );
      const target = path.join(gridDir, j.id + "-" + label + ".png"),
        temp = target + ".base.png";
      fs.writeFileSync(temp, PNG.sync.write(out));
      execFileSync("magick", [temp, "-draw", commands.join("\n"), target]);
      fs.unlinkSync(temp);
    }
  }
}
const escape = (s) => s.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
const cards = m.jobs
  .map(
    (j) =>
      `<article data-race="${j.race}"><h2>${escape(j.name)}</h2><div class="tiers">${[1, 2, 3]
        .map((l) => {
          const id = `${j.id}${l === 1 ? "" : "-l" + l}`;
          const directory = path.relative(root, sourceDirectory(id));
          const entry = catalogEntry(id, { level: l });
          const caption =
            j.race !== "zombie"
              ? `L${l}`
              : isArchived(id)
                ? `Study L${l} (reference)`
                : entry.role === "commander"
                  ? "Commander"
                  : `Chosen L${entry.level}`;
          return `<figure><img class="unit" data-directory="${directory}" src="${directory}/player.png"><figcaption>${caption}</figcaption></figure>`;
        })
        .join(
          "",
        )}</div><p><a href="sources/${j.id}.png">Raw generation</a> · <a href="grids/${j.id}-source.png">Numbered source</a> · <a href="grids/${j.id}-matrix.png">Numbered matrix</a> · <a href="${path.relative(root, sourceDirectory(j.id))}/components.md">Components</a></p></article>`,
  )
  .join("\n");
fs.writeFileSync(
  path.join(root, "gallery.html"),
  `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Faction anatomy review</title><style>body{background:#141f1a;color:#e1ddc8;font:16px system-ui;margin:16px}h1{font-size:24px}h2{font-size:18px}a{color:#a5dac8}.controls{position:sticky;top:0;background:#141f1a;padding:12px;display:flex;gap:16px;flex-wrap:wrap}main{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,600px),1fr));gap:16px}article{border:1px solid #65735f;padding:12px}.tiers{display:flex;gap:8px}figure{margin:0;flex:1;min-width:0}img.unit{image-rendering:pixelated;width:100%;background:repeating-conic-gradient(#3a403c 0% 25%,#2b322e 0% 50%) 0/32px 32px}figcaption{text-align:center}select{font:inherit;padding:6px}p{font-size:13px}article[hidden]{display:none}</style><h1>61 anatomies · L1–L3 · both factions</h1><p>Artwork prototypes. Zombie cards include archived studies; see <a href="../zombies/README.md">the chosen zombie lineup</a>. Each numbered grid uses zero-based native matrix coordinates. Source grids show the corresponding crop before palette cleanup.</p><div class="controls"><label>Faction <select id="side"><option value="player">Teal</option><option value="enemy">Red</option></select></label><label>Group <select id="race"><option value="all">All</option>${Object.keys(
    m.races,
  )
    .map((r) => `<option value="${r}">${r}</option>`)
    .join(
      "",
    )}</select></label></div><main>${cards}</main><script>document.querySelector('#side').onchange=e=>document.querySelectorAll('img.unit').forEach(img=>img.src=img.dataset.directory+'/'+e.target.value+'.png');document.querySelector('#race').onchange=e=>document.querySelectorAll('article').forEach(a=>a.hidden=e.target.value!=='all'&&a.dataset.race!==e.target.value);</script></html>\n`,
);
console.log(
  process.argv.includes("--html-only")
    ? "Local review gallery generated."
    : "Overview sheets, paired numbered grids, and local review gallery generated.",
);
