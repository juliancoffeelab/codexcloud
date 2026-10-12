import type { GameView, UnitView } from "../game/view";
import { DAMAGE_TYPES, TERRAIN, TYPES } from "../game/catalog";
import type { DamageBreakdown, Side, TerrainType, Weapon } from "../game/types";
import { damageExplanation } from "../presentation/combat";

export interface HealthForecastView {
  side: Side;
  remainingHp: number;
  remainingClass: string;
  currentHp: number;
  maximumHp: number;
  healthLabel: string;
  currentPercent: number;
  remainingPercent: number;
  impactLabel: string;
}

export interface CombatCalculationView {
  description: string;
  equation: string;
}

interface AttackChoiceBase {
  id: string;
  name: string;
  buttonLabel: string;
}

export interface AvailableAttackChoice extends AttackChoiceBase {
  available: true;
  inspectLabel: string;
  detailsId: string;
  playerForecast: HealthForecastView;
  enemyForecast: HealthForecastView;
  attackCalculation: CombatCalculationView;
  retaliationCalculation: CombatCalculationView;
}

export interface UnavailableAttackChoice extends AttackChoiceBase {
  available: false;
  reason: string;
}

export type AttackChoiceView = AvailableAttackChoice | UnavailableAttackChoice;

function healthForecast(
  unit: UnitView,
  damage: number,
  label: "You" | "Enemy",
  impact: "Take" | "Deal",
): HealthForecastView {
  const maximumHp = TYPES[unit.type].hp;
  const remainingHp = Math.max(0, unit.hp - damage);

  return {
    side: unit.side,
    remainingHp,
    remainingClass: remainingHp === 0 ? "fatal" : "",
    currentHp: unit.hp,
    maximumHp,
    healthLabel: `${label}: ${unit.hp} to ${remainingHp} HP after this attack`,
    currentPercent: (unit.hp / maximumHp) * 100,
    remainingPercent: (remainingHp / maximumHp) * 100,
    impactLabel: `${impact} ${damage ? `−${damage}` : 0} HP${remainingHp === 0 ? " (dies)" : ""}`,
  };
}

function describeAttack(
  label: string,
  weapon: Weapon,
  damage: DamageBreakdown,
  terrain: TerrainType,
): string {
  const terrainDescription =
    damage.protection || damage.exposure
      ? `, against terrain: ${TERRAIN[terrain].name.toLowerCase()}`
      : "";

  return `${label}: ${weapon.damage} ${DAMAGE_TYPES[weapon.damageType].toLowerCase()}, ${weapon.name.toLowerCase()}, ${weapon.category}${terrainDescription}`;
}

function outcomeWarning(
  attacker: UnitView,
  target: UnitView,
  damage: number,
  retaliation: number,
): string {
  if (retaliation >= attacker.hp) return ", you fall";
  if (damage >= target.hp) return ", enemy falls";
  return "";
}

/** Prepare display text and forecasts; damage and legality remain engine decisions. */
export function attackChoiceViews(
  game: GameView,
  attacker: UnitView,
  target: UnitView,
): AttackChoiceView[] {
  return TYPES[attacker.type].attacks.map((weapon) => {
    if (!game.canAttackFrom(attacker, attacker.tile, target, weapon.id)) {
      const reason =
        attacker.attacked || attacker.remainingAttacks?.[weapon.id] === 0
          ? "No uses remaining"
          : "Out of reach or blocked";
      return {
        available: false,
        reason,
        id: weapon.id,
        name: weapon.name,
        buttonLabel: `Attack with ${weapon.name} — ${reason.toLowerCase()}`,
      };
    }

    const preview = game.previewDetails(attacker, target, weapon.id);
    // Available attacks have already validated both combatants' tiles.
    const attackerTerrain = game.tile(attacker.tile)!.terrain;
    const targetTerrain = game.tile(target.tile)!.terrain;
    const warning = outcomeWarning(attacker, target, preview.damage, preview.retaliation);
    const retaliationCalculation: CombatCalculationView = {
      description: "Their attack: none",
      equation: preview.damage >= target.hp ? "0 (enemy defeated)" : "0 (out of range)",
    };

    if (preview.response && preview.retaliationDetail) {
      retaliationCalculation.description = describeAttack(
        "Their attack",
        preview.response,
        preview.retaliationDetail,
        attackerTerrain,
      );
      retaliationCalculation.equation = damageExplanation(preview.retaliationDetail);
    }

    const enemyForecast = healthForecast(target, preview.damage, "Enemy", "Deal");
    return {
      available: true,
      id: weapon.id,
      name: `${weapon.name} (${preview.remaining} left)`,
      buttonLabel: `Attack with ${weapon.name}: deal ${preview.damage} damage, take ${preview.retaliation} damage${warning}`,
      inspectLabel: `Inspect ${weapon.name.toLowerCase()} damage calculation`,
      detailsId: `attack-details-${weapon.id}`,
      playerForecast: healthForecast(attacker, preview.retaliation, "You", "Take"),
      enemyForecast,
      attackCalculation: {
        description: describeAttack("Your attack", weapon, preview.damageDetail, targetTerrain),
        equation: damageExplanation(preview.damageDetail),
      },
      retaliationCalculation,
    };
  });
}
