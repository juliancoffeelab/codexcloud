// Compose three actual native sprites and one empty pink slot. No generation calls.
const fs = require("../../../../scripts/pixel-fs.cjs");
const path = require("node:path");
const { PNG } = require("pngjs");
const { execFileSync } = require("node:child_process");
const root = __dirname,
  art = path.resolve(root, "../..");
module.exports = function (jobs, races) {
  const templateRoot = path.resolve(root, "../../../../tmp/faction-expansion/templates");
  fs.mkdirSync(templateRoot, { recursive: true });
  for (const job of jobs) {
    const def = races[job.race],
      out = new PNG({ width: 1024, height: 1024 });
    for (let i = 0; i < out.data.length; i += 4) out.data.set([255, 0, 255, 255], i);
    (job.anchors || def.anchors).forEach((id, n) => {
      const src = PNG.sync.read(fs.readFileSync(path.join(art, "units", id, "player.png")));
      const ox = (n % 2) * 512 + (512 - src.width * 8) / 2,
        oy = Math.floor(n / 2) * 512 + (512 - src.height * 8) / 2;
      for (let y = 0; y < src.height * 8; y++)
        for (let x = 0; x < src.width * 8; x++) {
          const i = (Math.floor(y / 8) * src.width + Math.floor(x / 8)) * 4;
          if (src.data[i + 3])
            out.data.set(src.data.subarray(i, i + 4), ((oy + y) * 1024 + ox + x) * 4);
        }
    });
    Object.values(job.palette || def.palette)
      .filter((c) => c[3])
      .forEach((c, n) => {
        for (let y = 0; y < 15; y++)
          for (let x = 0; x < 20; x++) out.data.set(c, ((487 + y) * 1024 + 8 + n * 26 + x) * 4);
      });
    const base = path.join(templateRoot, job.id + ".base.png"),
      target = path.join(templateRoot, job.id + ".png");
    fs.writeFileSync(base, PNG.sync.write(out));
    execFileSync("magick", [
      base,
      "-draw",
      "fill '#ffffff'\nfont 'DejaVu-Sans-Bold'\nfont-size 17\ntext 15,27 'STYLE ANCHOR 1'\ntext 527,27 'STYLE ANCHOR 2'\ntext 15,539 'STYLE ANCHOR 3'\n" +
        `text 527,539 '${job.name.toUpperCase()} — FILL HERE'\n` +
        "stroke '#141414'\nstroke-width 2\nline 512,0 512,1023\nline 0,512 1023,512",
      target,
    ]);
    fs.unlinkSync(base);
  }
};
