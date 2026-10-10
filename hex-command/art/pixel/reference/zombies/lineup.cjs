// Read-only preview of the selected branches and commander progression.
const fs = require("../../../../scripts/pixel-fs.cjs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const lineup = require("../../lineups/zombies.json");
const project = path.resolve(__dirname, "../../../..");
const units = [
  ...lineup.branches.flatMap((branch) => [
    { ...lineup.base, name: `${lineup.base.name} → ${branch.role}` },
    ...branch.units,
  ]),
  ...lineup.commander.units,
];
for (const side of ["player", "enemy"]) {
  const list = path.join(__dirname, `lineup-${side}.json`);
  fs.writeFileSync(
    list,
    JSON.stringify(
      units.map((unit) => ({
        label: `${unit.name} L${unit.level}`,
        image: path.resolve(__dirname, "../../units", unit.id, `${side}.png`),
      })),
      null,
      2,
    ) + "\n",
  );
  execFileSync(
    "node",
    [
      "art/pixel/reference/human-anatomies/preview.cjs",
      list,
      path.join(__dirname, `lineup-${side}.png`),
      "3",
      "3",
    ],
    { cwd: project },
  );
  fs.unlinkSync(list);
}
