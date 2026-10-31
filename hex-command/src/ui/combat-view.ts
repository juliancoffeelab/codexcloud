import type { GameView, UnitView } from "../game/view";
import { DAMAGE_TYPES, TERRAIN, TYPES, unitWeapons } from "../game/catalog";
import type {
  CombatPreviewDetails,
  DamageBreakdown,
  Side,
  TerrainType,
  Weapon,
} from "../game/types";
import { damageExplanation } from "../presentation/combat";

export interface HealthForecastView {
  side: Side;
  remainingHp: number;
  remainingLabel: string;
  remainingClass: string;
  currentHp: number;
  maximumHp: number;
  healthLabel: string;
  currentPercent: number;
  minimumPercent: number;
  maximumPercent: number;
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
  responseName: string;
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

function range(minimum: number, maximum: number): string {
  return minimum === maximum ? `${minimum}` : `${minimum}–${maximum}`;
}

function weaponRange(weapon: Weapon): string {
  const spread = weapon.spread ?? 0;
  return range(weapon.damage - spread, weapon.damage + spread);
}

function weaponLabel(weapon: Weapon, damage: DamageBreakdown): string {
  // Engine bounds cover all strikes; headings show the adjusted range per strike.
  const minimum = damage.minimum === undefined ? damage.damage : damage.minimum / weapon.count;
  const maximum = damage.maximum === undefined ? damage.damage : damage.maximum / weapon.count;
  return `(${weapon.count} × ${range(minimum, maximum)}) ${weapon.name}`;
}

function healthForecast(
  unit: UnitView,
  label: "You" | "Enemy",
  impact: "Take" | "Deal",
  deathChance: number,
  health: CombatPreviewDetails["attackerHealth"],
): HealthForecastView {
  const maximumHp = TYPES[unit.type].hp;
  const remainingHp = Math.round(health.expected * 10) / 10;
  const minimumLoss = unit.hp - health.maximum;
  const maximumLoss = unit.hp - health.minimum;
  let impactLabel: string;
  if (minimumLoss >= 0) {
    const loss = range(minimumLoss, maximumLoss);
    impactLabel = `${impact} ${minimumLoss > 0 ? `−${loss}` : loss} HP`;
  } else if (maximumLoss <= 0) {
    impactLabel = `${label === "You" ? "Heal" : "Enemy heals"} +${range(-maximumLoss, -minimumLoss)} HP`;
  } else {
    impactLabel = `Change −${maximumLoss}–+${-minimumLoss} HP`;
  }

  return {
    side: unit.side,
    remainingHp,
    remainingLabel: range(health.minimum, health.maximum),
    remainingClass: deathChance === 1 ? "fatal" : "",
    currentHp: unit.hp,
    maximumHp,
    healthLabel: `${label}: ${unit.hp} HP before, ${range(health.minimum, health.maximum)} HP after this attack, average ${remainingHp} HP${deathNotice(deathChance)}`,
    currentPercent: (unit.hp / maximumHp) * 100,
    minimumPercent: (health.minimum / maximumHp) * 100,
    maximumPercent: (health.maximum / maximumHp) * 100,
    impactLabel: `${impactLabel}${deathNotice(deathChance)}`,
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

  return `${label}: ${weapon.count} × ${weaponRange(weapon)} ${DAMAGE_TYPES[weapon.damageType].toLowerCase()}, ${weapon.name.toLowerCase()}, ${weapon.category}${terrainDescription}`;
}

function deathNotice(probability: number): string {
  if (probability === 0) return "";
  if (probability === 1) return " (dies)";
  const percent =
    probability < 0.01 ? "<1" : probability > 0.99 ? ">99" : Math.round(probability * 100);
  return ` (${percent}% chance to die)`;
}

/** Prepare display text and forecasts; damage and legality remain engine decisions. */
export function attackChoiceViews(
  game: GameView,
  attacker: UnitView,
  target: UnitView,
): AttackChoiceView[] {
  return unitWeapons(attacker).map((weapon) => {
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
      retaliationCalculation.equation = damageExplanation(preview.retaliationDetail);
    }

    if (preview.thornDamage) {
      const weaponDamage = preview.retaliation - preview.thornDamage;
      retaliationCalculation.description += `; thorns reflect damage per incoming melee strike`;
      retaliationCalculation.equation += `; ${weaponDamage} + ${preview.thornDamage} (thorns: average across executed strikes) = ${preview.retaliation}`;
    }

    const enemyForecast = healthForecast(
      target,
      "Enemy",
      "Deal",
      preview.killChance,
      preview.defenderHealth,
    );
    const playerForecast = healthForecast(
      attacker,
      "You",
      "Take",
      preview.deathChance,
      preview.attackerHealth,
    );
    const name = weaponLabel(weapon, preview.damageDetail);
    const responseName =
      preview.response && preview.retaliationDetail
        ? weaponLabel(preview.response, preview.retaliationDetail)
        : "No retaliation";
    return {
      available: true,
      responseName,
      id: weapon.id,
      name,
      buttonLabel: `Attack with ${name}. Enemy: ${responseName}. ${playerForecast.healthLabel}. ${enemyForecast.healthLabel}.`,
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
