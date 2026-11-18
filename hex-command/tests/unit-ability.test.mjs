import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, Presentation } from "./presentation-fixture.mjs";
const unit = {
  items: [],
  experience: 0,
  id: "u1",
  type: "scout",
  side: "player",
  tile: { col: 0, row: 0 },
  hp: 1,
  moved: false,
  attackSpent: false,
  faction: "player",
  revealedTo: [],
  perceptionEffects: [],
};
test("Leader ability explains the loss condition", () => {
  const game = board();
  game.edit((s) => (s.units = [{ ...unit, role: "leader" }]));
  assert.deepEqual(Presentation.ability(game.view, { ...unit, role: "leader" }), {
    name: "Command",
    description: Catalog.unit(catalog, unit.type).description,
    facts: ["Losing this unit ends the battle."],
  });
});
test("Ordinary ability information uses resolved native metadata", () => {
  const definition = Catalog.unit(catalog, "scout");
  const game = board();
  game.edit((s) => (s.units = [unit]));
  assert.deepEqual(Presentation.ability(game.view, unit), {
    name: definition.abilityLabel,
    description: definition.description,
    facts: [],
  });
});
test("Healing facts use the native numeric value", () => {
  const game = board();
  game.edit((s) => (s.units = [{ ...unit, type: "tidesage" }]));
  assert(
    Presentation.ability(game.view, { ...unit, type: "tidesage" }).facts.some((fact) =>
      fact.includes(
        `${Native.unit_details(game.view, unit.id).effects.find((e) => e.effect.kind === "healing").effect.power} HP`,
      ),
    ),
  );
});
