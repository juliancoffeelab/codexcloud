import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, Presentation } from "./presentation-fixture";
import { required } from "./helpers";
test("Terrain rows use native movement and damage effects without mutating the publication", () => {
  for (const type of ["soldier", "thorncaller", "reefguard"]) {
    const game = board();
    game.edit((state) => (state.units = []));
    const unit = game.add(type, "player", "0,0");
    const before = structuredClone(game.state);
    const panel = Presentation.unit_panel(catalog, game.view, unit.id, undefined, 0);
    if (panel === 0 || panel.TAG !== 1) throw new Error("Missing unit details");
    assert.equal(panel._0.unit.id, unit.id);
    for (const row of panel._0.terrain_effects) {
      const cost = Native.movement_cost(game.view, type, row.terrain);
      assert.equal(
        row.label,
        Number.isFinite(cost) ? `${cost} ${cost === 1 ? "Step" : "Steps"}` : "-",
      );
      assert.equal(row.name, Catalog.terrain_type(catalog, row.terrain).name);
    }
    const forest = required(panel._0.terrain_effects.find((row) => row.terrain === "forest"));
    if (type === "reefguard") assert.deepEqual(forest.facts, [{ label: "", value: 1 }]);
    if (type === "thorncaller")
      assert.deepEqual(forest.facts, [
        { label: "Melee", value: -1 },
        { label: "Ranged", value: -2 },
      ]);
    assert.deepEqual(game.state, before);
  }
});
for (const definition of Catalog.terrain(catalog)) {
  test(`Terrain details describe protection and healing rather than generic movement: ${definition.id}`, () => {
    const game = board();
    const unit = required(game.state.units[0]);
    game.edit(
      (state) =>
        (required(state.tiles.find((tile) => tile.id === unit.tile)).terrain = definition.id),
    );
    const panel = required(Presentation.terrain_panel(catalog, game.view, unit.id, unit.tile));
    assert(!panel.facts.some((fact: string) => fact.startsWith("Movement:")));
    if (definition.id === "hill")
      assert(panel.facts.includes("Land troops: −1 damage received. Merfolk: +1 damage received."));
    if (definition.id === "grove")
      assert(panel.facts.includes("Heals its owner’s occupant by 4 HP at the start of its turn"));
  });
}
test("Terrain facts distinguish attack category and damage type", () => {
  assert.deepEqual(
    Presentation.terrain_damage_facts([
      [0, 0, 0],
      [0, 0, 0],
    ]),
    [{ label: "", value: 0 }],
  );
  assert.deepEqual(
    Presentation.terrain_damage_facts([
      [0, -2, 1],
      [0, 0, 0],
    ]),
    [
      { label: "Melee Pierce", value: -2 },
      { label: "Melee Impact", value: 1 },
    ],
  );
});
