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
          ["beasts", "undead-animals"].includes(unit.faction) || (unit.faction === "zombies" && action.key === "attack-bite") || /^(boar|greatboar|woodland-ancient-boar)$/.test(unit.id) || !action.suggested.some((id) => /alien_|grunt_|horse-vocals/.test(id)),
        );
        if (/fire/i.test(action.title))
          assert(action.suggested.every((id) => id.includes("spell_fire_07")));
        if (/staff/i.test(action.title))
          assert(action.suggested.every((id) => id.includes("wood_05")));
        if (/sling/i.test(action.title))
          assert.equal(action.suggested[0], "rpg-audio/bookClose");
      }
    }
    if (/^(wildkeeper|grovewarden)$/.test(unit.id))
      assert.equal(tap.suggested[0], "rpg-audio/cloth3");
    if (unit.faction === "undead-animals")
      for (const attack of unit.attacks) {
        if (/fire|magic|spell|claw/i.test(attack.name) || (/^zombie-bear/.test(unit.id) && /bite/i.test(attack.name))) continue;
        assert.deepEqual(actions.find(row => row.key === `attack-${attack.id}`).suggested,
          tap.suggested, `${unit.key}: undead animal attack follows its selection palette`);
      }
    if (/^elf-flanker(?:-l[23])?$|^elf-jumper-l[23]$/.test(unit.id)) {
      const row = actions.find(row => row.key === "attack-blade");
      assert.equal(row.suggested[0], "rubberduck/80-cc0-rpg-sfx/blade_01.ogg");
      assert.equal(row.attackName, /^elf-jumper/.test(unit.id) ? "Spear" : "Knives");
    }
    if (/^(ghost|wraith|spectre)$/.test(unit.id))
      assert.equal(actions.find(row => row.key === "attack-spectral-touch").suggested[0],
        "rubberduck/80-cc0-rpg-sfx/spell_02.ogg");
    for (const attack of unit.attacks) {
      const row = actions.find(row => row.key === `attack-${attack.id}`);
      if (/tail.*strike/i.test(attack.name)) assert.equal(row.suggested[0], "kyles/fish-slap/fish_drop1.ogg");
      if (/claw/i.test(attack.name)) assert.equal(row.suggested[0], "edits/knife-slice-2-no-hit-v1");
      if (/wolf/.test(unit.id) && /bite|fang/i.test(attack.name))
        assert.equal(row.suggested[0], "qubodup/dog-snarl-grunt-grumble/dog/dog-snarl.flac");
      if (unit.faction === "zombies" || /^zombie-bear/.test(unit.id)) {
        if (/bite/i.test(attack.name)) assert(row.suggested[0].endsWith("/grunt_05.ogg"));
        if (/grasp|hands/i.test(attack.name)) assert(row.suggested[0].endsWith("/burp_02.ogg"));
      }
    }
    if (/^skeleton-rider(?:-l[23])?$/.test(unit.id))
      assert.equal(actions.find(row => row.key === "attack-spear").suggested[0],
        "rubberduck/80-cc0-rpg-sfx/blade_01.ogg");
    const attacks = actions.filter(row => row.key.startsWith("attack-"));
    if (/^giant-(wasp|spider)(?:-l[23])?$/.test(unit.id))
      assert(attacks.every(row => row.suggested[0].endsWith("/spit_01.ogg")));
    if (/^treant(?:-l[23])?$/.test(unit.id))
      assert(attacks.every(row => row.suggested[0] === "impact-sounds/impactPlank_medium_004"));
    if (/^(?:fox-spearguard|treant)(?:-l[23])?$/.test(unit.id))
      assert(tap.suggested[0].endsWith("/roar_03.ogg"));
    for (const attack of unit.attacks)
      if (attack.id === "root-crush")
        assert.equal(attacks.find(row => row.key === "attack-root-crush").suggested[0],
          "rubberduck/80-cc0-rpg-sfx/spell_02.ogg");
    for (const attack of unit.attacks)
      if (/axe|halberd/i.test(attack.name) && !/pickaxe/i.test(attack.name))
        assert.equal(attacks.find(row => row.key === `attack-${attack.id}`).suggested[0],
          "edits/metal-01-soft-onset-v1");
    for (const attack of unit.attacks)
      if (/arquebus|pistol|musket|gunshot/i.test(attack.name))
        assert.equal(attacks.find(row => row.key === `attack-${attack.id}`).suggested[0], "impact-sounds/impactPlate_medium_000");
    if (/^(giant-snake|sea-serpent)(?:-l[23])?$/.test(unit.id)) {
      assert.equal(tap.suggested[0], "edits/snake-belt-texture-v2");
      assert(actions.filter(row => row.key.startsWith("attack-")).every(row => row.suggested[0] === tap.suggested[0]));
    }
    if (/^stag(?:-l[23])?$/.test(unit.id))
      assert.equal(tap.suggested[0], "edits/horse-0286-trim-v1");
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
  ["forest", "edits/forest-book-03-short-v1"],
  ["grove", "edits/grove-short-magic-v2"],
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
assert.equal(profiles["edits/forest-book-03-short-v1"].preserved, true);
for (const group of read("curation-notes.json").groups)
  for (const id of group.ids) assert(byId.has(id));
console.log(
  `Curation references, semantic defaults and disjoint candidates passed for ${units} units.`,
);

assert.equal(profiles["edits/snake-belt-texture-v2"].preserved, false);
