import { immutable } from "./immutable";
import { WOODLAND_MOVEMENT } from "./movement";
import type { Race, RaceDefinition, UnitDefinition, UnitType } from "./types";

export const RACES = immutable<Record<Race, RaceDefinition>>({
  merfolk: {
    name: "Merfolk",
    terrainModifiers: {
      water: { passable: true, damageModifiers: { ranged: -1 } },
      ford: { cost: -1, damageModifiers: { melee: -1, ranged: -1 } },
      marsh: { cost: -1, damageModifiers: { melee: -2, ranged: -2 } },
      forest: { cost: 1, damageModifiers: { melee: 1, ranged: 2 } },
      grove: { cost: 1, damageModifiers: { melee: 1, ranged: 2 } },
      hill: { cost: 1, damageModifiers: { melee: 2, ranged: 2 } },
    },
    damageModifiers: { blade: 0, pierce: 0, impact: 0 },
  },
  human: {
    name: "Human",
    terrainModifiers: {},
    damageModifiers: { blade: 0, pierce: 0, impact: 0 },
  },
  undead: {
    name: "Undead",
    terrainModifiers: {},
    damageModifiers: { blade: 0, pierce: -1, impact: 1 },
  },
  beast: {
    name: "Woodland beast",
    terrainModifiers: WOODLAND_MOVEMENT,
    damageModifiers: { blade: 0, pierce: 1, impact: 0 },
  },
});

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
          range: unit.attacks[0].range,
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
