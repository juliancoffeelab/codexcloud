import { test } from "vitest";
import assert from "node:assert/strict";
import { catalog, Catalog, Presentation } from "./presentation-fixture.mjs";
const unit = {
  experience: 0,
  id: "u1",
  type: "scout",
  side: "player",
  tile: { col: 0, row: 0 },
  hp: 1,
  moved: false,
  attacked: false,
};
test("Leader ability explains the loss condition", () => {
  assert.deepEqual(Presentation.ability(catalog, { ...unit, role: "leader" }), {
    name: "Command",
    description: Catalog.unit(catalog, unit.type).description,
    facts: ["Losing this unit ends the battle."],
  });
});
test("Ordinary ability information uses resolved native metadata", () => {
  const definition = Catalog.unit(catalog, "scout");
  assert.deepEqual(Presentation.ability(catalog, unit), {
    name: definition.ability,
    description: definition.description,
    facts: [],
  });
});
test("Healing facts use the native numeric value", () => {
  assert(
    Presentation.ability(catalog, { ...unit, type: "tidesage" }).facts.some((fact) =>
      fact.includes(`${Catalog.unit(catalog, "tidesage").healing} HP`),
    ),
  );
});
