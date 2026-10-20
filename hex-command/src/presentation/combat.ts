import type { DamageBreakdown } from "../game/types";

export function damageExplanation(detail: Omit<DamageBreakdown, "damageType">) {
  const variable = detail.rollMin !== undefined && detail.rollMin !== detail.rollMax;
  let equation = variable ? `${detail.rollMin}–${detail.rollMax}` : `${detail.base}`;
  if (detail.modifier < 0) equation += ` − ${-detail.modifier} (armor)`;
  if (detail.modifier > 0) equation += ` + ${detail.modifier} (weakness)`;
  if (detail.protection) equation += ` − ${detail.protection} (cover)`;
  if (detail.exposure)
    equation += ` + ${detail.exposure} (${detail.terrain && detail.terrain !== "ford" ? detail.terrain : "shallow water"})`;
  if ((detail.rollMin ?? detail.base) + detail.modifier - detail.protection + detail.exposure < 0)
    equation = `max(0, ${equation})`;
  if (variable) {
    const counted = detail.count && detail.count > 1 ? `(${equation}) × ${detail.count}` : equation;
    return `${counted} = ${detail.minimum}–${detail.maximum} (average ${detail.expected!.toFixed(1)})`;
  }
  return detail.count && detail.count > 1
    ? `(${equation}) × ${detail.count} = ${detail.total}`
    : `${equation} = ${detail.damage}`;
}
