import { test } from "vitest";
import assert from "node:assert/strict";
import { TYPES } from "./game-fixture.ts";
import type { Unit } from "../src/game/types.ts";
import { abilityInfo } from "../src/presentation/unit-ability.ts";

const unit: Unit = {
  id: "u1",
  type: "scout",
  side: "player",
  tile: "hex-0001-7256137ec1",
  hp: 1,
  moved: false,
  attacked: false,
};

test("Leader ability information explains the loss condition", () => {
  assert.deepEqual(abilityInfo({ ...unit, type: "commander", role: "leader" }), {
    name: "Command",
    description: TYPES.commander.description,
    facts: ["Losing this unit ends the battle."],
  });
});

test("Ordinary ability information uses the unit definition", () => {
  const definition = TYPES.scout;
  assert.deepEqual(abilityInfo(unit), {
    name: definition.ability,
    description: definition.description,
    facts: [],
  });
});

// The value comes from the catalog, not duplicated prose or a fixed healer list.
test("Healing is presented as a numeric ability fact", () => {
  const facts = abilityInfo({ ...unit, type: "tidesage" }).facts;
  assert(facts.some((fact) => fact.includes(`${TYPES.tidesage.healing} HP`)));
});
