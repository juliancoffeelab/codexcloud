import type { BattleView } from "../controller/battle";
import { TERRAIN, TYPES } from "../game/engine";
import type { Unit, UnitDefinition } from "../game/types";
import { abilityInfo } from "../presentation/unit-ability";

export interface UnitDetailsView {
  unit: Unit;
  definition: UnitDefinition;
  terrainName: string;
  actionStatus: string;
  ability: ReturnType<typeof abilityInfo>;
}

type UnitPanelView =
  | { kind: "hidden" }
  | { kind: "outcome"; title: string; description: string }
  | { kind: "details"; details: UnitDetailsView }
  | { kind: "combat"; attacker: Unit; defender: Unit; detailsResetKey: string };

function actionStatus(unit: Unit): string {
  if (unit.attacked) return "Turn finished";
  if (unit.moved) return "Can still attack";
  return "Ready";
}

/** Resolve what the selected-unit panel should show before rendering its markup. */
export function unitPanelView(view: BattleView): UnitPanelView {
  const { game } = view;

  if (game.winner === "player") {
    return {
      kind: "outcome",
      title: "The field is yours.",
      description: "Your army defeated the enemy commander.",
    };
  }
  if (game.winner === "enemy") {
    return {
      kind: "outcome",
      title: "An army needs a leader.",
      description: "Keep your commander protected and use villages to grow your army.",
    };
  }

  const selectedUnit = game.unit(view.selected);
  if (!selectedUnit) return { kind: "hidden" };

  const targetUnit = game.unit(view.target);
  const canChooseAttack = game.turn === "player" && selectedUnit.side === "player";
  if (canChooseAttack && targetUnit && game.targets(selectedUnit.id).includes(targetUnit)) {
    return {
      kind: "combat",
      attacker: selectedUnit,
      defender: targetUnit,
      // Any battlefield redraw closes the optional damage calculations.
      detailsResetKey: `${selectedUnit.id}-${targetUnit.id}-${view.renderRevision}`,
    };
  }

  const occupiedTile = game.tile(selectedUnit.tile);
  if (!occupiedTile) throw new Error("The selected unit has no battlefield tile.");

  return {
    kind: "details",
    details: {
      unit: selectedUnit,
      definition: TYPES[selectedUnit.type],
      terrainName: TERRAIN[occupiedTile.terrain].name,
      actionStatus: actionStatus(selectedUnit),
      ability: abilityInfo(selectedUnit),
    },
  };
}
