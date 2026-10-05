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

export function ordinaryCombatants(
  unit: Unit,
  context: Pick<LeaderContext, "units" | "definitions">,
): Unit[] {
  return context.units.filter(
    (other) =>
      other.side === unit.side &&
      !context.definitions[other.type].leader &&
      other.role !== "courier" &&
      !context.definitions[other.type].healing,
  );
}

/** Two thresholds prevent repeated departures and returns at intermediate strengths. */
export function updateLeaderRebuilding(unit: Unit, count: number): boolean {
  if (count >= 5) unit.rebuilding = false;
  else if (count <= 2 || unit.rebuilding === undefined) unit.rebuilding = true;
  return unit.rebuilding ?? false;
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
  const troops = ordinaryCombatants(unit, context);
  if (unit.rebuilding && context.recruiting && before > 2) return false;
  // Withdrawing or moving along the back line does not require an escort.
  if (after >= before) return true;
  const escorts = troops.filter(
    (other) =>
      other.hp >= context.definitions[other.type].hp / 2 &&
      context.distance(destination, other.tile) <= 2,
  );
  return escorts.length >= 2 && escorts.some((other) => proximity(other.tile) <= after);
}
