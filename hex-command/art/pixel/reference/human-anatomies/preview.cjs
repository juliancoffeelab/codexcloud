const fs = require("../../../../scripts/pixel-fs.cjs");
const { PNG } = require("pngjs");
const { execFileSync } = require("node:child_process");
// Render labeled native-scale contact sheets from a JSON list of {label,image}.
// Usage: node preview.cjs entries.json output.png columns scale
const [listPath, output, colsArg, scaleArg] = process.argv.slice(2);
const entries = JSON.parse(fs.readFileSync(listPath));
const cols = Number(colsArg || 2),
  scale = Number(scaleArg || 8),
  cell = 64 * scale;
const rowHeight = cell + 52,
  width = cols * (cell + 16) + 16,
  height = Math.ceil(entries.length / cols) * rowHeight + 16;
const out = new PNG({ width, height });
for (let i = 0; i < out.data.length; i += 4) out.data.set([20, 31, 26, 255], i);
const labels = [];
entries.forEach(({ label, image }, n) => {
  const p = image ? PNG.sync.read(fs.readFileSync(image)) : new PNG({ width: 48, height: 48 });
  if (p.width > 64 || p.height > 64)
    throw new Error("Expected native reference frame up to 64 pixels");
  const bx = 16 + (n % cols) * (cell + 16),
    by = 44 + Math.floor(n / cols) * rowHeight;
  const px = Math.floor((64 - p.width) / 2),
    py = Math.floor((64 - p.height) / 2);
  for (let y = 0; y < cell; y++)
    for (let x = 0; x < cell; x++) {
      const sx = Math.floor(x / scale) - px,
        sy = Math.floor(y / scale) - py;
      const di = ((by + y) * width + bx + x) * 4;
      let rgba =
        (Math.floor(x / (8 * scale)) + Math.floor(y / (8 * scale))) % 2
          ? [58, 64, 60, 255]
          : [43, 50, 46, 255];
      if (sx >= 0 && sx < p.width && sy >= 0 && sy < p.height) {
        const si = (sy * p.width + sx) * 4;
        if (p.data[si + 3]) rgba = p.data.subarray(si, si + 4);
      }
      out.data.set(rgba, di);
    }
  labels.push(
    `font-size ${Math.min(21, Math.floor(cell / (label.length * 0.64)))} text ${bx},${by - 13} '${label.replaceAll("'", "")}'`,
  );
});
const temp = output + ".base.png";
fs.writeFileSync(temp, PNG.sync.write(out));
execFileSync("magick", [
  temp,
  "-draw",
  "fill '#e1ddc8'\nfont 'DejaVu-Sans'\nfont-size 21\n" + labels.join("\n"),
  output,
]);
fs.unlinkSync(temp);
