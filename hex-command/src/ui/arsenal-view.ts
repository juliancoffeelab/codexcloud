import type { UnitView } from "../game/view";
import { TYPES, DAMAGE_TYPES, unitWeapons } from "../game/catalog";
import type { DamageType } from "../game/types";

function modifierClass(modifier: number): string {
  if (modifier < 0) return "resistant";
  if (modifier > 0) return "vulnerable";
  return "";
}

export function arsenalView(unit: UnitView) {
  const definition = TYPES[unit.type];
  const weapons = unitWeapons(unit).map((weapon) => ({
    ...weapon,
    rollLabel: weapon.spread
      ? `${weapon.damage - weapon.spread}–${weapon.damage + weapon.spread}`
      : `${weapon.damage}`,
    categoryLabel: weapon.category === "melee" ? "Melee" : "Ranged",
    damageLabel: DAMAGE_TYPES[weapon.damageType],
    rangeLabel: weapon.range === 1 ? "1 hex range" : `1–${weapon.range} hex range`,
  }));
  const damageTypes = Object.entries(DAMAGE_TYPES) as [DamageType, string][];
  const modifiers = damageTypes.map(([damageType, name]) => {
    const modifier = definition.damageModifiers[damageType];
    const label = modifier < 0 ? `−${-modifier}` : modifier > 0 ? `+${modifier}` : "0";
    return { name, label, className: modifierClass(modifier) };
  });
  return { weapons, modifiers };
}
