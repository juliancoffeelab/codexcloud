import type { UnitView } from "../game/view";
import { TYPES, DAMAGE_TYPES } from "../game/engine";
import type { DamageType } from "../game/types";

function resistanceClass(resistance: number): string {
  if (resistance > 0) return "resistant";
  if (resistance < 0) return "vulnerable";
  return "";
}

export function arsenalView(unit: UnitView) {
  const definition = TYPES[unit.type];
  const weapons = definition.attacks.map((weapon) => ({
    ...weapon,
    categoryLabel: weapon.category === "melee" ? "Melee" : "Ranged",
    damageLabel: DAMAGE_TYPES[weapon.damageType],
    rangeLabel: weapon.range === 1 ? "1 hex range" : `1–${weapon.range} hex range`,
  }));
  const damageTypes = Object.entries(DAMAGE_TYPES) as [DamageType, string][];
  const resistances = damageTypes.map(([damageType, name]) => {
    const resistance = definition.resistances[damageType] || 0;
    return { name, multiplier: 100 - resistance, className: resistanceClass(resistance) };
  });
  return { weapons, resistances };
}
