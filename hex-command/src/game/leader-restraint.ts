import type { Unit, UnitDefinition, UnitType } from "./types";

interface LeaderContext {
  units: readonly Unit[];
  definitions: Readonly<
    Record<UnitType, Readonly<Pick<UnitDefinition, "hp" | "leader" | "healing">>>
  >;
  distance(first: string, second: string): number;
  recruiting: boolean;
  requiredMovement: boolean;
}

/** Leaders follow a living escort; a calendar turn is never permission to charge. */
export function leaderMayAdvance(unit: Unit, destination: string, context: LeaderContext): boolean {
  if (destination === unit.tile || context.requiredMovement) return true;
  const enemies = context.units.filter((other) => other.side !== unit.side);
  if (!enemies.length) return true;
  const proximity = (tile: string) =>
    Math.min(...enemies.map((enemy) => context.distance(tile, enemy.tile)));
  const before = proximity(unit.tile),
    after = proximity(destination);
  const troops = context.units.filter(
    (other) =>
      other.side === unit.side &&
      !context.definitions[other.type].leader &&
      other.role !== "courier" &&
      !context.definitions[other.type].healing,
  );
  if (context.recruiting && troops.length < 4 && before > 2) return false;
  // Withdrawing or moving along the back line does not require an escort.
  if (after >= before) return true;
  const escorts = troops.filter(
    (other) =>
      other.hp >= context.definitions[other.type].hp / 2 &&
      context.distance(destination, other.tile) <= 2,
  );
  return escorts.length >= 2 && escorts.some((other) => proximity(other.tile) <= after);
}
