import { TYPES } from "../game/catalog";
import type { Unit } from "../game/types";

export interface AbilityInfo {
  name: string;
  description: string;
  facts: string[];
}

export function abilityInfo(unit: Unit): AbilityInfo {
  const type = TYPES[unit.type];
  const facts: string[] = [];
  if (type.leader || unit.role === "leader") facts.push("Losing this unit ends the battle.");
  if (type.healing)
    facts.push(
      `Heals adjacent allies by ${type.healing} HP at the start of its side’s turn. Only the strongest healer applies.`,
    );
  return { name: type.leader ? "Command" : type.ability, description: type.description, facts };
}
