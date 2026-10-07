import { immutable } from "./immutable";
import { AUTHORED_RACES } from "./authored-content";
import type { Race, RaceDefinition, UnitDefinition, UnitType } from "./types";

export const RACES = immutable<Record<Race, RaceDefinition>>(AUTHORED_RACES);

/** Specifications hold unit deviations; consumers receive final race + unit stats. */
export function resolveUnits(
  specifications: Record<UnitType, UnitDefinition>,
): Record<UnitType, UnitDefinition> {
  return Object.fromEntries(
    Object.entries(specifications).map(([type, unit]) => {
      const inherited = RACES[unit.race].damageModifiers;
      return [
        type,
        {
          ...unit,
          damage: unit.attacks[0].damage,
          range: Math.max(...unit.attacks.map((attack) => attack.range)),
          damageModifiers: {
            blade: inherited.blade + unit.damageModifiers.blade,
            pierce: inherited.pierce + unit.damageModifiers.pierce,
            impact: inherited.impact + unit.damageModifiers.impact,
          },
        },
      ];
    }),
  ) as Record<UnitType, UnitDefinition>;
}
