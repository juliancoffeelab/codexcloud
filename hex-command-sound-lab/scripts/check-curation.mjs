import assert from "node:assert/strict";
import fs from "node:fs";
import { buildActions } from "../dist/matching.js";
const root = new URL("../dist/", import.meta.url);
const read = (name) => JSON.parse(fs.readFileSync(new URL(name, root)));
const catalog = read("catalog.json"),
  workbench = read("workbench.json");
const byId = new Map(catalog.map((item) => [item.id, item]));
let units = 0;
for (const faction of workbench.factions)
  for (const unit of faction.units) {
    const actions = buildActions(unit, catalog, { extras: true });
    const tap = actions.find((row) => row.key === "tap");
    assert.deepEqual(actions.find((row) => row.key === "move").suggested, [
      "current/select-1",
      "current/select-2",
      "current/select-3",
    ]);
    for (const action of actions) {
      for (const id of [...action.suggested, ...action.alternatives]) {
        assert(byId.has(id), `${unit.key}: missing ${id}`);
        assert(
          !byId.get(id).name.endsWith(" Swing"),
          `${unit.key}: rejected swing`,
        );
      }
      assert(!action.suggested.some((id) => action.alternatives.includes(id)));
      if (action.key.startsWith("attack-")) {
        assert(
          !action.suggested.some((id) => /alien_|grunt_|horse-vocals/.test(id)),
        );
        if (/fire/i.test(action.title))
          assert(action.suggested.every((id) => id.includes("spell_fire_07")));
        if (/staff/i.test(action.title))
          assert(action.suggested.every((id) => id.includes("wood_05")));
        if (/sling/i.test(action.title))
          assert.equal(action.suggested[0], "rpg-audio/bookClose");
      }
    }
    if (/^giant-wasp/.test(unit.id))
      assert(tap.suggested[0].includes("alien_03"));
    if (/^giant-spider/.test(unit.id))
      assert(tap.suggested[0].includes("alien_04"));
    if (/^ray(?:-l[23])?$/.test(unit.id))
      assert(tap.suggested[0].includes("alien_02"));
    if (/^(ghost|spectre|wraith)$/.test(unit.id))
      assert.equal(tap.suggested[0], "edits/ghost-grunt-02-trim-v1");
    const death = buildActions(unit, catalog).find(
      (row) => row.key === "death",
    );
    if (death)
      assert(death.hypothetical, `${unit.key}: death must stay a proposal`);
    units++;
  }
for (const [id, wanted] of [
  ["forest", "edits/forest-book-03-soft-v1"],
  ["grove", "edits/ruins-magic-soft-v1"],
  ["ruins", "current/terrain-hill-1"],
]) {
  const terrain = workbench.terrains.find((tile) => tile.id === id);
  assert.equal(
    buildActions(terrain, catalog).find((row) => row.key === "tap")
      .suggested[0],
    wanted,
  );
}
const profiles = read("playback-levels.json").entries;
for (const name of [
  "horse-0286-trim-v1",
  "ghost-grunt-02-trim-v1",
  "axe-katana-one-shot-v1",
  "spear-sabre-one-shot-v1",
])
  assert.equal(profiles[`edits/${name}`].preserved, false);
assert.equal(profiles["edits/forest-book-03-soft-v1"].preserved, true);
for (const group of read("curation-notes.json").groups)
  for (const id of group.ids) assert(byId.has(id));
console.log(
  `Curation references, semantic defaults and disjoint candidates passed for ${units} units.`,
);
