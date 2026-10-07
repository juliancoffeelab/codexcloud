import { DAMAGE_TYPES } from "../game/catalog";

const categories = ["Melee", "Ranged"];
const damageTypes = ["blade", "pierce", "impact"] as const;
export function signedTerrainValue(value: number): string {
  return value < 0 ? `−${-value}` : `+${value}`;
}

/** Describe Rust's resolved terrain contribution, excluding permanent armor. */
export function terrainDamageFacts(
  environment: readonly (readonly number[])[],
): { label: string; value: number }[] {
  const values = environment.flat();
  if (values.every((value) => value === values[0])) return [{ label: "", value: values[0]! }];
  return environment.flatMap((row, category) => {
    if (row.every((value) => value === 0)) return [];
    if (row.every((value) => value === row[0]))
      return [{ label: categories[category]!, value: row[0]! }];
    return row.flatMap((value, damage) =>
      value
        ? [{ label: `${categories[category]} ${DAMAGE_TYPES[damageTypes[damage]!]}`, value }]
        : [],
    );
  });
}
