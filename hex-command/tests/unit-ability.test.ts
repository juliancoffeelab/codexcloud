import { test } from "vitest";
import assert from "node:assert/strict";
import { Game, TYPES } from "./game-fixture.ts";
import { abilityInfo, type AbilityInfo } from "../src/presentation/unit-ability.ts";

for (const unit of new Game().units) {
  test(
    "Unit ability information follows its rules — " + JSON.stringify([unit.type, unit.side]),
    async () => {
      const info: AbilityInfo = abilityInfo(unit);
      if (unit.type === "commander") {
        assert.deepEqual(info, {
          name: "Command",
          description: "Losing this unit ends the battle.",
        });
      } else {
        const definition = TYPES[unit.type];
        assert.deepEqual(info, {
          name: definition.ability,
          description: definition.description,
        });
      }
    },
  );
}
