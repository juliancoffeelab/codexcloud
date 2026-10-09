// The Trident guard L2 head is the common merfolk trident design. Shafts,
// hands and the rest of each sprite are outside this native head region.
const fs = require("../../../../scripts/pixel-fs.cjs");
const path = require("node:path");
const project = path.resolve(__dirname, "../../../..");
const source = path.join(project, "art/pixel/units/tridentguard");
function apply(rows, palette) {
  if (rows.length !== 48 || rows.some((row) => row.length !== 48))
    throw new Error("Shared trident head requires the reviewed 48x48 merfolk frame");
  const reference = fs.readFileSync(path.join(source, "player.txt"), "utf8").trimEnd().split("\n");
  const colours = JSON.parse(fs.readFileSync(path.join(source, "palette.json")));
  const changes = [];
  for (let y = 0; y < 10; y++)
    for (let x = 30; x < 44; x++) {
      const to = reference[y][x],
        from = rows[y][x];
      if (from !== to)
        changes.push({
          x,
          y,
          from,
          to,
          reason: "Shared Trident guard head; body and hand grips preserved",
        });
      rows[y][x] = to;
    }
  palette.g = colours.g;
  return changes;
}
if (require.main === module) {
  const dir = path.join(project, "art/pixel/units/reefguard");
  for (const file of ["player.txt", "enemy.txt", "palette.json"]) {
    fs.copyFileSync(path.join(source, file), path.join(dir, file));
  }
  // Preserve the approved 48px spear source; the new hunter armour uses a padded frame.
  const hunter = path.join(project, "art/pixel/reference/merfolk-tier-designs/baseline/tidehunter");
  const harpooner = path.join(project, "art/pixel/units/harpooner");
  for (const side of ["player", "enemy"]) {
    const target = fs
      .readFileSync(path.join(harpooner, `${side}.txt`), "utf8")
      .trimEnd()
      .split("\n")
      .map((row) => [...row]);
    const reference = fs
      .readFileSync(path.join(hunter, `${side}.txt`), "utf8")
      .trimEnd()
      .split("\n");
    for (let y = 0; y < 15; y++) for (let x = 30; x < 44; x++) target[y][x] = reference[y][x];
    fs.writeFileSync(
      path.join(harpooner, `${side}.txt`),
      target.map((row) => row.join("")).join("\n") + "\n",
    );
  }
  const palette = JSON.parse(fs.readFileSync(path.join(harpooner, "palette.json")));
  const hunterPalette = JSON.parse(fs.readFileSync(path.join(hunter, "palette.json")));
  palette.g = hunterPalette.g;
  fs.writeFileSync(path.join(harpooner, "palette.json"), JSON.stringify(palette, null, 2) + "\n");
}
module.exports = { apply };
