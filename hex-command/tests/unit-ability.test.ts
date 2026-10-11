import { test } from "vitest";
import assert from "node:assert/strict";
import { catalog, Catalog, Presentation } from "./presentation-fixture";
import type { PublicUnit } from "../rust/pkg/hex_command";
const unit: PublicUnit = {
  id: "u1",
  type: "scout",
  side: "player",
  tile: "0,0",
  hp: 1,
  moved: false,
  attacked: false,
};
test("Leader ability explains the loss condition", () => {
  assert.deepEqual(Presentation.ability(catalog, { ...unit, type: "commander" }), {
    name: "Command",
    description: Catalog.unit(catalog, "commander").description,
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
    Presentation.ability(catalog, { ...unit, type: "tidesage" }).facts.some((fact: string) =>
      fact.includes(`${Catalog.unit(catalog, "tidesage").healing} HP`),
    ),
  );
});
