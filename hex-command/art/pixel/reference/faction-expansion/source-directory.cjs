// Keep chosen lineups active and exploratory poses in references.
const path = require("node:path");
const lineup = require("../../lineups/zombies.json");
const archived = require("../zombies/catalog.json");
const dwarfArchive = require("../dwarf-branch-references/catalog.json");
const dwarves = require("../../lineups/dwarves.json");
const merfolkArchive = require("../merfolk-archive/catalog.json");
const merfolk = require("../../lineups/merfolk.json");
const elves = require("../../lineups/elves.json");
const woodland = require("../../lineups/woodland.json");
const selected = new Map(
  [
    ...elves.trees.flatMap((tree) => [
      tree.base,
      ...tree.branches.flatMap((branch) => branch.units),
    ]),
    ...merfolk.trees.flatMap((tree) => [
      tree.base,
      ...tree.branches.flatMap((branch) => branch.units),
    ]),
    ...merfolk.commander.units,
    ...woodland.commander.units.map((unit) => ({ ...unit, role: "commander" })),
    ...(elves.commander?.units ?? []),
    ...(dwarves.commander?.units ?? []),
    lineup.base,
    ...lineup.branches.flatMap((branch) => branch.units),
    ...lineup.commander.units,
    ...dwarves.trees.flatMap((tree) => [
      tree.base,
      ...tree.branches.flatMap((branch) => branch.units),
    ]),
  ].map((unit) => [unit.id, unit]),
);

function isArchived(id) {
  return (
    Object.hasOwn(archived, id) ||
    Object.hasOwn(dwarfArchive, id) ||
    Object.hasOwn(merfolkArchive, id)
  );
}

function sourceDirectory(id) {
  return path.resolve(
    __dirname,
    Object.hasOwn(merfolkArchive, id)
      ? "../merfolk-archive/units"
      : Object.hasOwn(dwarfArchive, id)
        ? "../dwarf-branch-references/units"
        : Object.hasOwn(archived, id)
          ? "../zombies/units"
          : "../../units",
    id,
  );
}

function catalogEntry(id, generated) {
  const chosen = selected.get(id);
  if (!chosen) return generated;
  const metadata = { ...chosen };
  delete metadata.id;
  return { ...generated, ...metadata };
}

function tierIds(job) {
  if (job.level === 3) return [job.id];
  if (job.id === "dwarf-runesmith") {
    return [
      job.id,
      "dwarf-thundersmith-l2",
      "dwarf-thundersmith-l3",
      "dwarf-stonespell-l2",
      "dwarf-stonespell-l3",
    ];
  }
  return [job.id, `${job.id}-l2`, `${job.id}-l3`].filter(
    (id) => job.race !== "dwarf" || selected.has(id) || Object.hasOwn(dwarfArchive, id),
  );
}

module.exports = { sourceDirectory, catalogEntry, isArchived, tierIds };
