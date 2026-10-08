const path = require('node:path');
const catalog = require('../art/pixel/units/catalog.json');
const archived = require('../art/pixel/reference/zombies/catalog.json');
const dwarfArchive = require('../art/pixel/reference/dwarf-branch-references/catalog.json');
const units = path.resolve(__dirname, '../art/pixel/units');
const references = path.resolve(__dirname, '../art/pixel/reference/zombies/units');
const byStudy = new Map([
  ...Object.entries(catalog).map(([id, entry]) => [entry.study, path.join(units, id)]),
  ...Object.entries(archived).map(([id, entry]) => [entry.study, path.join(references, id)]),
  ...Object.entries(dwarfArchive).map(([id, entry]) => [entry.study, path.resolve(__dirname, '../art/pixel/reference/dwarf-branch-references/units', id)]),
]);

/** Resolve a construction study to its canonical editable unit source. */
module.exports = function sourceDirectory(study) {
  const directory = byStudy.get(study);
  if (!directory) throw new Error(`Unknown artwork study: ${study}`);
  return directory;
};
