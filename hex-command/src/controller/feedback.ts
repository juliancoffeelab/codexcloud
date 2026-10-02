import type { GameSnapshot, Unit } from "../game/types";
import type { MotionKind } from "../presentation/motion";

/** Describe visible changes without playing sound or touching the document. */
export function describeChanges(before: GameSnapshot, after: GameSnapshot) {
  const previousUnits = new Map(before.units.map((unit) => [unit.id, unit]));
  const tileEffects = new Map<string, MotionKind>();
  let recruited = false;
  let attacked = false;
  let movedUnit: Unit | null = null;

  for (const unit of after.units) {
    const previous = previousUnits.get(unit.id);
    if (!previous) {
      tileEffects.set(unit.tile, "recruit");
      recruited = true;
    } else {
      if (previous.tile !== unit.tile) {
        tileEffects.set(unit.tile, "move");
        movedUnit = unit;
      }
      if (previous.hp > unit.hp) {
        tileEffects.set(unit.tile, "attack");
        attacked = true;
      }
    }
    previousUnits.delete(unit.id);
  }

  for (const fallen of previousUnits.values()) {
    tileEffects.set(fallen.tile, "attack");
    attacked = true;
  }

  const capturedTiles = after.villages
    .filter((village) => {
      const previous = before.villages.find((tile) => tile.id === village.id);
      return village.owner && village.owner !== previous?.owner;
    })
    .map((village) => village.id);

  // A surviving attacker has spent its attack; a fallen attacker is absent.
  const attacker =
    after.units.find((unit) => {
      const previous = before.units.find((old) => old.id === unit.id);
      return unit.side === before.turn && unit.attacked && previous && !previous.attacked;
    }) ??
    before.units.find((unit) => {
      return unit.side === before.turn && !after.units.some((alive) => alive.id === unit.id);
    });

  return {
    tileEffects,
    capturedTiles,
    turnChanged: before.turn !== after.turn || before.round !== after.round,
    newWinner: after.winner !== before.winner ? after.winner : null,
    attacked,
    recruited,
    movedUnit,
    attacker,
  };
}
