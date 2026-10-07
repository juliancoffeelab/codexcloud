import { test } from "vitest";
import assert from "node:assert/strict";
import { TYPES } from "./game-fixture.ts";
import type { Unit } from "../src/game/types.ts";
import { abilityInfo } from "../src/presentation/unit-ability.ts";

const unit: Unit = {
  id: "u1",
  type: "scout",
  side: "player",
  tile: "0,0",
  hp: 1,
  moved: false,
  attacked: false,
};

test("Leader ability information explains the loss condition", () => {
  assert.deepEqual(abilityInfo({ ...unit, type: "commander" }), {
    name: "Command",
    description: "Losing this unit ends the battle.",
  });
});

test("Ordinary ability information uses the unit definition", () => {
  const definition = TYPES.scout;
  assert.deepEqual(abilityInfo(unit), {
    name: definition.ability,
    description: definition.description,
  });
});
