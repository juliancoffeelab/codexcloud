import { TYPES, isLeader, unitWeapons } from "../game/catalog";
import type { Unit } from "../game/types";

export interface AbilityInfo {
  name: string;
  description: string;
  facts: string[];
}

export function abilityInfo(unit: Unit): AbilityInfo {
  const type = TYPES[unit.type];
  const facts: string[] = [];
  const attacks = unitWeapons(unit);
  if (type.thorns)
    facts.push(
      `When defending, reflects ${type.thorns} damage per incoming melee strike, including killing hits.`,
    );
  if (type.thornAura)
    facts.push(
      `When defending, adjacent allies reflect ${type.thornAura} damage per incoming melee strike. The strongest aura applies and stacks with Thornshield.`,
    );
  if (attacks.some((attack) => attack.drain))
    facts.push(
      `Draining attacks restore ${Math.max(...attacks.map((attack) => attack.drain ?? 0))}% of HP removed, including retaliation, up to maximum HP.`,
    );
  if (isLeader(unit)) facts.push("Losing this unit ends the battle.");
  if (type.healing)
    facts.push(
      `Heals adjacent allies by ${type.healing} HP at the start of its side’s turn. Only the strongest healer applies.`,
    );
  return { name: isLeader(unit) ? "Command" : type.ability, description: type.description, facts };
}
