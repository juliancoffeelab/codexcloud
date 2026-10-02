import type { DamageBreakdown } from "../game/types";

export function damageExplanation(detail: Omit<DamageBreakdown, "damageType">) {
  const multiplier = 100 - detail.resistance;
  let equation = `round(${detail.base} × ${multiplier}%)`;
  if (detail.protection) equation += ` − ${detail.protection} (cover)`;
  if (detail.exposure) equation += ` + ${detail.exposure} (shallow water)`;
  if (detail.resisted - detail.protection + detail.exposure < 1)
    equation = `max(1, ${equation})`;
  return `${equation} = ${detail.damage}`;
}
