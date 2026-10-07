import { TYPES } from "../game/catalog";
import type { Unit } from "../game/types";

export interface AbilityInfo {
  name: string;
  description: string;
}

export function abilityInfo(unit: Unit): AbilityInfo {
  if (TYPES[unit.type].leader) {
    return {
      name: "Command",
      description: "Losing this unit ends the battle.",
    };
  }
  const type = TYPES[unit.type];
  return { name: type.ability, description: type.description };
}
