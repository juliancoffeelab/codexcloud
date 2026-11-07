import type { GameView, Immutable, UnitView } from "../game/view";
import type { BattleView, CombatReport } from "../controller/view";
import { TERRAIN, TYPES } from "../game/catalog";
import type { TerrainType, UnitDefinition, UnitType } from "../game/types";
import { TERRAIN_ORDER } from "../game/authored-content";
import { terrainDamageFacts } from "./terrain-effects";
import { abilityInfo } from "../presentation/unit-ability";

export interface UnitDetailsView {
  faction: import("../../rust/pkg/hex_command").UnitFaction;
  unit: UnitView;
  definition: Immutable<UnitDefinition>;
  terrainName: string;
  actionStatus: string;
  promotions: { type: UnitType; name: string; description: string; hp: number }[];
  ability: ReturnType<typeof abilityInfo>;
  terrainEffects: {
    terrain: TerrainType;
    name: string;
    label: string;
    facts: ReturnType<typeof terrainDamageFacts>;
    className: "easy" | "slow" | "difficult" | "blocked";
  }[];
}

type UnitPanelView =
  | { kind: "hidden" }
  | { kind: "outcome"; title: string; description: string }
  | { kind: "details"; details: UnitDetailsView }
  | {
      kind: "attack";
      attacker: UnitView;
      defender: UnitView;
      game: GameView;
      detailsResetKey: string;
    }
  | {
      kind: "combat-report";
      attacker: UnitView;
      defender: UnitView;
      game: GameView;
      report: CombatReport;
    };

function actionStatus(unit: UnitView): string {
  if (unit.attacked) return "Turn finished";
  if (unit.moved) return "Can still attack";
  return "Ready";
}

/** Resolve what the selected-unit panel should show before rendering its markup. */
export function unitPanelView(
  view: Pick<BattleView, "interactionGame" | "details" | "renderRevision">,
): UnitPanelView {
  const { interactionGame: game, details } = view;
  const report = details.kind === "combat-report" ? details.report : undefined;

  // Only the explicitly chosen report follows combat phases; finished battles show their outcome.
  if (report && (!report.complete || !game.winner)) {
    return {
      kind: "combat-report",
      attacker: report.combat.units[0],
      defender: report.combat.units[1],
      game: report.game,
      report,
    };
  }

  if (game.campaign && game.winner) return { kind: "hidden" };
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

  if (details.kind !== "unit" && details.kind !== "attack") return { kind: "hidden" };
  const selectedUnit = game.unit(details.kind === "unit" ? details.unitId : details.attackerId);
  if (!selectedUnit) return { kind: "hidden" };

  const targetUnit = details.kind === "attack" ? game.unit(details.targetId) : undefined;
  const canChooseAttack = game.turn === "player" && game.faction(selectedUnit.id).controllable;
  if (canChooseAttack && targetUnit && game.targets(selectedUnit.id).includes(targetUnit)) {
    return {
      kind: "attack",
      game,
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
      faction: game.faction(selectedUnit.id),
      unit: selectedUnit,
      definition: TYPES[selectedUnit.type],
      terrainName: TERRAIN[occupiedTile.terrain].name,
      actionStatus: game.faction(selectedUnit.id).controllable
        ? actionStatus(selectedUnit)
        : "Not under your control",
      promotions: game
        .promotionOptions(selectedUnit.id)
        .filter(() => canChooseAttack)
        .map((type) => ({
          type,
          name: TYPES[type].name,
          description: TYPES[type].description,
          hp: TYPES[type].hp,
        })),
      ability: abilityInfo(selectedUnit),
      terrainEffects: TERRAIN_ORDER.map((terrain, index) => {
        const cost = game.cost(selectedUnit, { terrain });
        return {
          terrain,
          name: TERRAIN[terrain].name,
          label: Number.isFinite(cost) ? `${cost} ${cost === 1 ? "Step" : "Steps"}` : "-",
          facts: terrainDamageFacts(TYPES[selectedUnit.type].environment[index]!),
          className: !Number.isFinite(cost)
            ? "blocked"
            : cost === 1
              ? "easy"
              : cost === 2
                ? "slow"
                : "difficult",
        };
      }),
    },
  };
}
