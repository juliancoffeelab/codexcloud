const path = require('node:path');
const catalog = require('../art/pixel/units/catalog.json');
const units = path.resolve(__dirname, '../art/pixel/units');
const byStudy = new Map(Object.entries(catalog).map(([id, entry]) => [entry.study, id]));

/** Resolve a construction study to its canonical editable unit source. */
module.exports = function sourceDirectory(study) {
  const id = byStudy.get(study);
  if (!id) throw new Error(`Unknown promoted artwork study: ${study}`);
  return path.join(units, id);
};
