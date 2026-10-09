// Render the selected advancement tree from its canonical native sprites.
const fs = require("../../../../scripts/pixel-fs.cjs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const { PNG } = require("pngjs");
const lineup = require("../../lineups/zombies.json");
const width = 1240,
  height = 968,
  tile = 192,
  scale = 3;
const out = new PNG({ width, height });
for (let i = 0; i < out.data.length; i += 4) out.data.set([20, 31, 26, 255], i);
const drawing = [
  "fill '#e1ddc8'",
  "font 'DejaVu-Sans'",
  "font-size 30",
  "text 72,48 'Chosen zombie lineup'",
  "font-size 19",
  "text 72,84 'L1'",
  "text 488,84 'L2'",
  "text 904,84 'L3'",
];
function cell(unit, x, y, label = unit.name) {
  const image = PNG.sync.read(
    fs.readFileSync(path.resolve(__dirname, "../../units", unit.id, "player.png")),
  );
  const px = Math.floor((64 - image.width) / 2),
    py = Math.floor((64 - image.height) / 2);
  for (let dy = 0; dy < tile; dy++)
    for (let dx = 0; dx < tile; dx++) {
      const sx = Math.floor(dx / scale) - px,
        sy = Math.floor(dy / scale) - py;
      let rgba =
        (Math.floor(dx / 24) + Math.floor(dy / 24)) % 2 ? [58, 64, 60, 255] : [43, 50, 46, 255];
      if (sx >= 0 && sx < image.width && sy >= 0 && sy < image.height) {
        const i = (sy * image.width + sx) * 4;
        if (image.data[i + 3]) rgba = image.data.subarray(i, i + 4);
      }
      out.data.set(rgba, ((y + dy) * width + x + dx) * 4);
    }
  drawing.push(`text ${x},${y + tile + 26} '${label}'`);
}
cell(lineup.base, 72, 356);
lineup.branches.forEach((branch, index) => {
  const y = 116 + index * 240;
  cell(branch.units[0], 488, y);
  cell(branch.units[1], 904, y);
});
cell(lineup.commander, 72, 652, "Commander: Corruption mage");
drawing.push(
  "fill 'none'",
  "stroke '#87a99a'",
  "stroke-width 3",
  "line 280,452 376,452",
  "line 376,212 376,692",
);
for (const y of [212, 452, 692]) {
  drawing.push(
    `line 376,${y} 472,${y}`,
    `line 696,${y} 888,${y}`,
    `line 460,${y - 7} 472,${y}`,
    `line 460,${y + 7} 472,${y}`,
    `line 876,${y - 7} 888,${y}`,
    `line 876,${y + 7} 888,${y}`,
  );
}
const output = path.join(__dirname, "lineup-player.png"),
  temporary = output + ".base.png";
fs.writeFileSync(temporary, PNG.sync.write(out));
execFileSync("magick", [temporary, "-draw", drawing.join("\n"), output]);
fs.unlinkSync(temporary);
