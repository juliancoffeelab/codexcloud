// Keep chosen zombie designs active and exploratory variants in references.
const path = require("node:path");
const lineup = require("../../lineups/zombies.json");
const archived = require("../zombies/catalog.json");
const selected = new Map(
  [lineup.base, ...lineup.branches.flatMap((branch) => branch.units), lineup.commander].map(
    (unit) => [unit.id, unit],
  ),
);

function isArchived(id) {
  return Object.hasOwn(archived, id);
}

function sourceDirectory(id) {
  return path.resolve(__dirname, isArchived(id) ? "../zombies/units" : "../../units", id);
}

function catalogEntry(id, generated) {
  const chosen = selected.get(id);
  if (!chosen) return generated;
  const metadata = { ...chosen };
  delete metadata.id;
  return { ...generated, ...metadata };
}

module.exports = { sourceDirectory, catalogEntry, isArchived };
