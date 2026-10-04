import type { Unit, UnitType } from "./types";

const actionOrder: Record<UnitType, number> = {
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

/** Share stable enemy sequencing between animated play and headless playtests. */
export function enemyActionIds(units: readonly Readonly<Unit>[]): string[] {
  return units
    .filter((unit) => unit.side === "enemy")
    .sort((first, second) => actionOrder[first.type] - actionOrder[second.type])
    .map((unit) => unit.id);
}
