// Extract the approved imagegen board into native 64x64 editable matrices.
// The original board has three 256px columns and two 234px rows. Each sprite
// occupies a 64x64 logical frame, rendered at 4x, seven pixels below its row.
const fs = require('../../../../../scripts/pixel-fs.cjs');
const path = require("node:path");
const { PNG } = require("pngjs");
const here = __dirname;
const image = PNG.sync.read(fs.readFileSync(path.join(here, "generated-board.png")));
const base = JSON.parse(fs.readFileSync(path.join(here, "../bases/palette.json")));
const palette = {
  ...base,
  W: [88, 108, 121, 255],
  w: [48, 63, 74, 255],
  V: [139, 160, 173, 255],
  e: [67, 147, 175, 255],
  J: [181, 229, 244, 255],
  G: [218, 169, 62, 255],
  Y: [252, 222, 118, 255],
};
// Slate belongs to robe/hood/stone, never to the faction ramp.
delete palette.M;
delete palette.a;
const candidates = Object.entries(palette).filter(([key]) => !".Rr".includes(key));
function classify(i) {
  const rgb = [...image.data.subarray(i, i + 3)];
  if (rgb.every((v, j) => Math.abs(v - [20, 31, 26][j]) < 5)) return ".";
  return candidates.reduce((best, [key, color]) => {
    const distance = rgb.reduce((sum, v, j) => sum + (v - color[j]) ** 2, 0);
    return distance < best.distance ? { key, distance } : best;
  }, { key: ".", distance: Infinity }).key;
}
for (const [column, level] of [[1, 2], [2, 3]]) {
  const rows = [];
  for (let y = 0; y < 64; y++) {
    let row = "";
    for (let x = 0; x < 64; x++) {
      const counts = new Map();
      const x0 = (column * 256 + x * 4) * image.width / 768;
      const x1 = (column * 256 + (x + 1) * 4) * image.width / 768;
      const y0 = (234 + 7 + y * 4) * image.height / 468;
      const y1 = (234 + 7 + (y + 1) * 4) * image.height / 468;
      for (let py = Math.ceil(y0); py < Math.min(image.height, y1); py++) {
        for (let px = Math.ceil(x0); px < Math.min(image.width, x1); px++) {
          const key = classify((py * image.width + px) * 4);
          counts.set(key, (counts.get(key) || 0) + 1);
        }
      }
      // Captions are outside the sprite; the model preserves the reference frame.
      row += y < 14 ? "." : [...counts].sort((a, b) => b[1] - a[1])[0]?.[0] || ".";
    }
    rows.push(row);
  }
  fs.writeFileSync(path.join(here, `stonespell-l${level}.mask`), rows.join("\n") + "\n");
}
fs.writeFileSync(path.join(here, "palette.json"), JSON.stringify(palette, null, 2) + "\n");
