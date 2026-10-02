import assert from "node:assert/strict";
import { Game, TYPES } from "../src/game/engine.ts";
import { abilityInfo, type AbilityInfo } from "../src/presentation/unit-ability.ts";

const game = new Game();
for (const unit of game.units) {
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
}
console.log("PASS: commanders show command information; other unit abilities match their rules");
