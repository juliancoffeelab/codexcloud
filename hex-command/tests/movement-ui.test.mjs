import { cellKey } from "./hex-fixture.mjs";
import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, Presentation } from "./presentation-fixture.mjs";
import { required, setTerrain } from "./helpers.mjs";
test("Terrain rows use native movement and damage effects without mutating the publication", () => {
  for (const type of ["soldier", "thorncaller", "reefguard"]) {
    const game = board();
    game.edit((state) => (state.units = []));
    const unit = game.add(type, "player", "0,0");
    const before = structuredClone(game.state);
    const panel = Presentation.unit_panel(catalog, game.view, unit.id, void 0, 0);
    if (panel === 0 || panel.TAG !== 1) throw new Error("Missing unit details");
    assert.equal(panel._0.unit.id, unit.id);
    for (const row of panel._0.terrain_effects) {
      const cost =
        Native.unit_details(game.view, unit.id).terrain.find((tile) => tile.terrain === row.terrain)
          .cost ?? Infinity;
      assert.equal(
        row.label,
        Number.isFinite(cost) ? `${cost} ${cost === 1 ? "Step" : "Steps"}` : "-",
      );
      assert.equal(row.name, Catalog.terrain_type(catalog, row.terrain).name);
    }
    const forest = required(panel._0.terrain_effects.find((row) => row.terrain === "forest"));
    const rows = Native.unit_details(game.view, unit.id).terrain.find(
      (tile) => tile.terrain === "forest",
    ).defense;
    assert.deepEqual(forest.facts, Presentation.terrain_damage_facts(rows));
    assert.deepEqual(game.state, before);
  }
});
for (const definition of Catalog.terrain(catalog)) {
  test(`Terrain details describe protection and healing rather than generic movement: ${definition.id}`, () => {
    const game = board();
    const unit = required(game.state.units[0]);
    game.edit((state) =>
      setTerrain(
        required(state.tiles.find((tile) => tile.id === cellKey(unit.tile))),
        definition.id,
      ),
    );
    const panel = required(Presentation.terrain_panel(catalog, game.view, unit.id, unit.tile));
    assert(!panel.facts.some((fact) => fact.startsWith("Movement:")));
    const profile = Native.unit_details(game.view, unit.id).terrain.find(
      (row) => cellKey(row.tile) === cellKey(unit.tile),
    );
    if (profile.healing > 0)
      assert(
        panel.facts.includes(`Heals this unit by ${profile.healing} HP at the start of its turn`),
      );
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
