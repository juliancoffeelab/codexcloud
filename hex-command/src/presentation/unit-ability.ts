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
  if (type.thorns)
    facts.push(
      `When defending, reflects ${type.thorns} damage per incoming melee strike, including killing hits.`,
    );
  if (type.thornAura)
    facts.push(
      `When defending, adjacent allies reflect ${type.thornAura} damage per incoming melee strike. The strongest aura applies and stacks with Thornshield.`,
    );
  if (type.attacks.some((attack) => attack.drain))
    facts.push(
      `Draining attacks restore ${Math.max(...type.attacks.map((attack) => attack.drain ?? 0))}% of HP removed, including retaliation, up to maximum HP.`,
    );
  if (type.leader || unit.role === "leader") facts.push("Losing this unit ends the battle.");
  if (type.healing)
    facts.push(
      `Heals adjacent allies by ${type.healing} HP at the start of its side’s turn. Only the strongest healer applies.`,
    );
  return { name: type.leader ? "Command" : type.ability, description: type.description, facts };
}
