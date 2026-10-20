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
}

export type AttackChoiceView = AvailableAttackChoice | UnavailableAttackChoice;

function healthForecast(
  unit: UnitView,
  damage: number,
  healing = 0,
): Omit<HealthForecastView, "healthLabel" | "impactLabel" | "remainingClass"> {
  const maximumHp = TYPES[unit.type].hp;
  const remainingHp = Math.max(0, unit.hp - damage + healing);

  return {
    side: unit.side,
    remainingHp,
    currentHp: unit.hp,
    maximumHp,
    currentPercent: (unit.hp / maximumHp) * 100,
    remainingPercent: (remainingHp / maximumHp) * 100,
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

  return `${label}: ${weapon.count} × ${weapon.damage - (weapon.spread ?? 0)}–${weapon.damage + (weapon.spread ?? 0)} ${DAMAGE_TYPES[weapon.damageType].toLowerCase()}, ${weapon.name.toLowerCase()}, ${weapon.category}${terrainDescription}`;
}

/** Prepare display text and forecasts; damage and legality remain engine decisions. */
export function attackChoiceViews(
  game: GameView,
  attacker: UnitView,
  target: UnitView,
): AttackChoiceView[] {
  return TYPES[attacker.type].attacks.map((weapon) => {
    if (!game.canAttack(attacker, target, weapon.id)) {
      return {
        available: false,
        id: weapon.id,
        name: weapon.name,
        buttonLabel: `Attack with ${weapon.name} — out of reach or blocked`,
      };
    }

    const preview = game.previewDetails(attacker, target, weapon.id);
    // Available attacks have already validated both combatants' tiles.
    const attackerTerrain = game.tile(attacker.tile)!.terrain;
    const targetTerrain = game.tile(target.tile)!.terrain;
    const chance = (p: number) =>
      p === 0
        ? "0%"
        : p === 1
          ? "100%"
          : p < 0.01
            ? "<1%"
            : p > 0.99
              ? ">99%"
              : `${Math.round(p * 100)}%`;
    const warning = `, kill chance ${chance(preview.killChance)}, your death chance ${chance(preview.deathChance)}`;
    const retaliationCalculation: CombatCalculationView = {
      description: "Their attack: none",
      equation: preview.killChance === 1 ? "0 (enemy defeated)" : "0 (out of range)",
    };

    if (preview.response && preview.retaliationDetail) {
      retaliationCalculation.description = describeAttack(
        "Their attack",
        preview.response,
        preview.retaliationDetail,
        attackerTerrain,
      );
      retaliationCalculation.equation = `${damageExplanation(preview.retaliationDetail)}; only if the enemy survives`;
    }

    if (preview.thornDamage) {
      const weaponDamage = preview.retaliation - preview.thornDamage;
      retaliationCalculation.description += `; thorns reflect damage per incoming melee strike`;
      retaliationCalculation.equation += `; ${weaponDamage} + ${preview.thornDamage} (thorns: ${preview.thornDamage / weapon.count} × ${weapon.count} incoming strikes) = ${preview.retaliation}`;
    }

    const enemyHealth = healthForecast(target, preview.damage, preview.retaliationHealing);
    const enemyForecast: HealthForecastView = {
      ...enemyHealth,
      impactLabel: `Deal ${preview.damageDetail.minimum}–${preview.damageDetail.maximum} HP; ${chance(preview.killChance)} kill`,
      healthLabel: `Enemy: estimated ${enemyHealth.remainingHp} HP; ${chance(preview.killChance)} chance of death`,
      remainingClass: preview.killChance === 1 ? "fatal" : "",
    };
    const playerHealth = healthForecast(attacker, preview.retaliation, preview.healing);
    const playerForecast: HealthForecastView = {
      ...playerHealth,
      impactLabel: `Take ${preview.retaliationMin}–${preview.retaliationMax} HP; ${chance(preview.deathChance)} death${preview.healing ? `; heal ≈${preview.healing}` : ""}`,
      healthLabel: `You: estimated ${playerHealth.remainingHp} HP; ${chance(preview.deathChance)} chance of death`,
      remainingClass: preview.deathChance === 1 ? "fatal" : "",
    };
    return {
      available: true,
      id: weapon.id,
      name: `${weapon.name} (${weapon.count} × ${weapon.damage - (weapon.spread ?? 0)}–${weapon.damage + (weapon.spread ?? 0)})`,
      buttonLabel: `Attack with ${weapon.name}: deal ${preview.damageDetail.minimum}–${preview.damageDetail.maximum} damage, take ${preview.retaliationMin}–${preview.retaliationMax} damage${warning}`,
      inspectLabel: `Inspect ${weapon.name.toLowerCase()} damage calculation`,
      detailsId: `attack-details-${weapon.id}`,
      playerForecast,
      enemyForecast,
      attackCalculation: {
        description: describeAttack("Your attack", weapon, preview.damageDetail, targetTerrain),
        equation: damageExplanation(preview.damageDetail),
      },
      retaliationCalculation,
    };
  });
}
