import type { Unit, UnitType } from "./types";
import { TYPES } from "./engine";

const actionOrder: Partial<Record<UnitType, number>> = {
  scout: 0,
  wolf: 0,
  archer: 1,
  thorncaller: 1,
  bonearcher: 1,
  adept: 1,
  soldier: 2,
  skeleton: 2,
  warden: 2,
  boar: 2,
  commander: 3,
  wildkeeper: 3,
  necromancer: 3,
};
function order(type: UnitType): number {
  const definition = TYPES[type];
  return (
    actionOrder[type] ??
    actionOrder[definition.family ?? type] ??
    (definition.leader ? 3 : definition.range > 1 ? 1 : 2)
  );
}

/** Share stable enemy sequencing between animated play and headless playtests. */
export function enemyActionIds(units: readonly Readonly<Unit>[]): string[] {
  return units
    .filter((unit) => unit.side === "enemy")
    .sort((first, second) => order(first.type) - order(second.type))
    .map((unit) => unit.id);
}
