import type { DamageBreakdown } from "../game/types";

export function damageExplanation(detail: Omit<DamageBreakdown, "damageType">) {
  let equation = `${detail.base}`;
  if (detail.modifier < 0) equation += ` − ${-detail.modifier} (armor)`;
  if (detail.modifier > 0) equation += ` + ${detail.modifier} (weakness)`;
  if (detail.protection) equation += ` − ${detail.protection} (cover)`;
  if (detail.exposure) equation += ` + ${detail.exposure} (shallow water)`;
  if (detail.modified - detail.protection + detail.exposure < 0) equation = `max(0, ${equation})`;
  return `${equation} = ${detail.damage}`;
}
