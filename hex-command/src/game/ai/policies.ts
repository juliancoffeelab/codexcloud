import { Game, TYPES, distance } from "../engine";
import { enemyActionIds } from "../enemy-turn";
import { enemyTileOrder } from "../enemy-order";
import { leaderMayAdvance } from "../leader-restraint";
import type { Side, Unit, UnitType } from "../types";
import {
  canPursue,
  essential,
  individualObjectives,
  objectivesFor,
  reservedTiles,
  supportObjective,
  authoredDestinations,
  finishesObjective,
} from "./objectives";
import type { AiObjective } from "./objectives";
import { diplomacyAllowsAttack, withdrawalLanes } from "./diplomacy";
import type { DiplomacyId } from "./diplomacy";

export const POLICY_IDS = ["classic", "rush", "mixed"] as const;
export type PolicyId = (typeof POLICY_IDS)[number];
export interface PolicyContext {
  seed: number;
  /** Same cap for both seats. The live classic enemy recruits once. */
  recruitLimit?: number;
  objectives?: AiObjective[];
  diplomacy?: DiplomacyId;
  /** A structured record of commands, useful for reproducing a failed game. */
  onAction?: (action: AiAction) => void;
}
export type AiAction =
  | { kind: "move"; unit: string; tile: string }
  | { kind: "attack"; unit: string; target: string; weapon: string }
  | { kind: "advance"; unit: string; type: UnitType }
  | { kind: "recruit"; unit: string; type: UnitType; tile: string };

export function seededRandom(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state += 0x6d2b79f5;
    let value = Math.imul(state ^ (state >>> 15), 1 | state);
    value ^= value + Math.imul(value ^ (value >>> 7), 61 | value);
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
  };
}
function jitter(seed: number, key: string): number {
  let hash = seed >>> 0;
  for (const char of key) hash = Math.imul(hash ^ char.charCodeAt(0), 16777619);
  return ((hash >>> 0) / 4294967296) * 0.05;
}
function role(unit: Pick<Unit, "type">): "screen" | "ranged" | "healer" {
  const definition = TYPES[unit.type];
  return definition.healing ? "healer" : definition.range > 1 ? "ranged" : "screen";
}

function policyObjectives(game: Game, context: PolicyContext): AiObjective[] {
  const goals = [...(context.objectives ?? objectivesFor(game, game.turn))];
  for (const leader of game.units.filter(
    (unit) => unit.side === game.turn && TYPES[unit.type].leader,
  )) {
    const requiredMovement = goals.some(
      (goal) => goal.kind === "escape" && goal.actor === leader.id,
    );
    const keeps = game.leaderRebuildGoals(leader, requiredMovement);
    if (keeps.length)
      goals.push({
        id: `rebuild:${leader.id}`,
        kind: "capture",
        actor: leader.id,
        tiles: keeps.map((keep) => keep.id),
        weight: 80,
      });
  }
  return goals;
}

function recruit(game: Game, policy: PolicyId, context: PolicyContext): void {
  const side = game.turn;
  const goals = context.objectives ?? objectivesFor(game, side);
  const lanes = withdrawalLanes(game, side, context.diplomacy ?? "pragmatic");
  const placementAllowed = (type: UnitType, tile: { id: string }) => {
    const recruit: Unit = {
      id: "new-recruit",
      type,
      side,
      tile: tile.id,
      hp: TYPES[type].hp,
      moved: true,
      attacked: true,
    };
    return !reservedTiles(recruit, goals).has(tile.id) && !lanes.has(tile.id);
  };
  for (let count = 0; count < (context.recruitLimit ?? 1); count++) {
    if (policy === "classic") {
      game.enemyRecruit(side, placementAllowed);
      continue;
    }
    const leader = game.units.find((unit) => unit.side === side && TYPES[unit.type].leader);
    if (!leader) return;
    const allies = game.units.filter((unit) => unit.side === side && !TYPES[unit.type].leader);
    const opponents = game.units.filter((unit) => unit.side !== side);
    const counts = { screen: 0, ranged: 0, healer: 0 };
    for (const ally of allies) counts[role(ally)]++;
    const choices = game
      .recruitmentOptions(side)
      .filter(
        (type) =>
          (policy !== "rush" || role({ type }) === "screen") &&
          game.recruitTiles(leader.id, type).some((tile) => placementAllowed(type, tile)),
      )
      .map((type) => {
        const definition = TYPES[type];
        const category = role({ type });
        const desired = category === "screen" ? 0.5 : category === "ranged" ? 0.35 : 0.15;
        const deficiency = desired * (allies.length + 1) - counts[category];
        const effectiveness = opponents.length
          ? opponents.reduce(
              (total, target) =>
                total +
                Math.max(
                  ...definition.attacks.map((weapon) =>
                    Math.max(
                      1,
                      weapon.damage + TYPES[target.type].damageModifiers[weapon.damageType],
                    ),
                  ),
                ),
              0,
            ) / opponents.length
          : definition.damage;
        const efficiency = (definition.hp * 0.35 + effectiveness * 1.5) / definition.cost;
        const score =
          policy === "rush"
            ? efficiency * 5 + definition.hp * 0.15
            : deficiency * 8 +
              efficiency * 2 +
              (category === "healer" && counts.healer === 0 ? 3 : 0);
        return { type, score: score + jitter(context.seed, `recruit:${game.round}:${type}`) };
      })
      .sort((a, b) => b.score - a.score);
    const choice = choices[0];
    if (!choice) return;
    const tiles = game
      .recruitTiles(leader.id, choice.type)
      .filter((tile) => placementAllowed(choice.type, tile));
    const enemies = opponents.map((unit) => game.tile(unit.tile)!);
    tiles.sort((a, b) => {
      const proximity = (tile: typeof a) =>
        enemies.length ? Math.min(...enemies.map((enemy) => distance(tile, enemy))) : 0;
      // Place supports behind the line, screens toward contact.
      return (role(choice) === "screen" ? 1 : -1) * (proximity(a) - proximity(b));
    });
    const tile = tiles[0]!;
    game.recruit(leader.id, choice.type, tile.id);
    context.onAction?.({ kind: "recruit", unit: leader.id, type: choice.type, tile: tile.id });
  }
}

/** Future movement is estimated with fresh action flags, through the live pathfinder. */
class Threats {
  private readonly source: ReturnType<Game["captureState"]>;
  private readonly important: Set<string>;
  private readonly shadow: Game;
  private readonly enemies: { unit: Unit; from: string[] }[];
  private readonly cache = new Map<string, number>();
  constructor(game: Game, side: Side) {
    const state = game.captureState();
    this.source = state;
    this.important = new Set(
      game.units
        .filter((unit) => unit.side === side && essential(game, unit))
        .map((unit) => unit.id),
    );
    this.shadow = new Game(game.mapId, game.mapRevision);
    this.shadow.restoreState(state);
    // Authored next-phase arrivals and healing use the real turn boundary.
    this.shadow.endTurn();
    this.enemies = this.shadow.units
      .filter((unit) => unit.side !== side)
      .map((unit) => ({
        unit,
        from: [unit.tile, ...this.shadow.reachable(unit.id).keys()],
      }));
  }
  at(unit: Unit, tile: string, eliminated?: string): number {
    const key = `${unit.id}:${unit.type}:${tile}:${eliminated ?? ""}`;
    const cached = this.cache.get(key);
    if (cached !== undefined) return cached;
    if (this.important.has(unit.id)) {
      const source = new Game(this.source.mapId, this.source.mapRevision);
      source.restoreState(this.source);
      const risk = source.forecastDanger(unit, tile, eliminated);
      this.cache.set(key, risk);
      return risk;
    }
    const target = { ...unit, tile };
    const board = this.shadow,
      enemies = this.enemies;
    const damages = enemies
      .filter(({ unit }) => unit.id !== eliminated)
      .map(({ unit: enemy, from }) => {
        let best = 0;
        for (const weapon of TYPES[enemy.type].attacks) {
          if (from.some((position) => board.canAttackFrom(enemy, position, target, weapon.id)))
            best = Math.max(best, board.damage(enemy, target, weapon.id));
        }
        return best;
      })
      .sort((a, b) => b - a);
    // Local top-two pressure estimate, NOT an exact enemy-phase forecast.
    const risk = (damages[0] ?? 0) + (damages[1] ?? 0);
    this.cache.set(key, risk);
    return risk;
  }
}

interface Plan {
  unit: string;
  tile: string;
  score: number;
  target?: string;
  weapon?: string;
  advance?: UnitType;
  withdrawing?: boolean;
}
function plans(
  game: Game,
  policy: "rush" | "mixed",
  context: PolicyContext,
  routes: Map<string, Map<string, number>>,
): Plan[] {
  const side = game.turn;
  const allies = game.units.filter((unit) => unit.side === side);
  const foes = game.units.filter((unit) => unit.side !== side);
  const objectives = individualObjectives(policyObjectives(game, context));
  const lanes = withdrawalLanes(game, side, context.diplomacy ?? "pragmatic");
  const threats = new Threats(game, side);
  const screens = allies.filter((unit) => role(unit) === "screen" && !TYPES[unit.type].leader);
  const movableScreens = screens.some((unit) => !unit.moved && !unit.attacked);
  const result: Plan[] = [];
  const rangeToFoe = (tile: string) =>
    foes.length
      ? Math.min(...foes.map((foe) => distance(game.tile(tile)!, game.tile(foe.tile)!)))
      : 20;
  const field = (unit: Unit, goal: AiObjective) => {
    const key = `${unit.type}:${goal.tiles.join(";")}`;
    if (!routes.has(key))
      routes.set(
        key,
        game.routeDistances(unit, goal.tiles.map((id) => game.tile(id)!).filter(Boolean)),
      );
    return routes.get(key)!;
  };
  for (const unit of allies) {
    if (unit.attacked) continue;
    const category = role(unit);
    const important = essential(game, unit);
    const beforeRisk = threats.at(unit, unit.tile);
    const promotion = game.promotionOptions(unit.id)[0];
    if (promotion && beforeRisk < unit.hp)
      result.push({ unit: unit.id, tile: unit.tile, advance: promotion, score: 14 });
    const destinations = authoredDestinations(game, unit, [
      unit.tile,
      ...game.reachable(unit.id).keys(),
    ]);
    const reserved = reservedTiles(unit, objectives);
    const requiredMovement = objectives.some(
      (goal) => goal.kind === "escape" && goal.actor === unit.id,
    );
    const rebuilding = game.leaderRebuildGoals(unit, requiredMovement);
    const minimumRisk = important
      ? Math.min(...destinations.map((tile) => threats.at(unit, tile)))
      : 0;
    const restraint = {
      units: allies.concat(foes),
      definitions: TYPES,
      distance: (first: string, second: string) => distance(game.tile(first)!, game.tile(second)!),
      recruiting: Boolean(
        TYPES[unit.type].leader &&
        game.recruitmentOptions(side).some((type) => game.recruitTiles(unit.id, type).length > 0),
      ),
      requiredMovement,
    };
    for (const tileId of destinations) {
      const tile = game.tile(tileId)!;
      let positioning =
        (reserved.has(unit.tile) ? 80 : 0) -
        (reserved.has(tileId) ? 160 : 0) -
        (lanes.has(tileId) ? 100 : 0);
      let objectiveBenefit = 0;
      const goals = rebuilding.length
        ? [
            {
              id: "rebuild",
              kind: "capture" as const,
              tiles: rebuilding.map((tile) => tile.id),
              weight: 80,
            },
          ]
        : objectives;
      for (const original of goals) {
        const assisting = !canPursue(unit, original) && Boolean(original.actor || original.family);
        if (!canPursue(unit, original) && !assisting) continue;
        const goal = assisting ? supportObjective(game, unit, original) : original;
        if (!goal.tiles.length) continue;
        const route = field(unit, goal);
        const before = route.get(unit.tile) ?? 1000,
          after = route.get(tileId) ?? 1000;
        const occupiedGoal =
          goal.kind === "capture" &&
          goal.tiles.every((id) => game.tile(id)?.owner === side && game.at(id)?.side === side);
        const weight = occupiedGoal ? goal.weight * 0.15 : goal.weight;
        objectiveBenefit = Math.max(
          objectiveBenefit,
          ((before - after) * weight) / 12 +
            (goal.tiles.includes(tileId) && tileId !== unit.tile && !occupiedGoal
              ? weight * 0.6
              : 0),
        );
      }
      positioning += objectiveBenefit;
      if (
        rebuilding.some((keep) => keep.id === unit.tile) &&
        tileId !== unit.tile &&
        threats.at(unit, tileId) >= beforeRisk
      )
        continue;
      if (
        tileId !== unit.tile &&
        objectives.some((goal) => goal.actor === unit.id && goal.tiles.includes(unit.tile))
      )
        positioning -= 24;
      const healingTile =
        ["village", "grove"].includes(tile.terrain) && (tile.owner === side || !game.at(tileId));
      if (healingTile) positioning += Math.min(2, TYPES[unit.type].hp - unit.hp) * 2;
      if (tile.terrain === "village" && tile.owner !== side) positioning += 3;
      if (policy === "mixed") {
        if (category === "healer") {
          const healing = TYPES[unit.type].healing!;
          let value = 0;
          for (const ally of allies) {
            if (ally.id === unit.id || distance(tile, game.tile(ally.tile)!) !== 1) continue;
            const otherHealing = Math.max(
              0,
              ...allies
                .filter(
                  (other) =>
                    other.id !== unit.id &&
                    other.id !== ally.id &&
                    distance(game.tile(other.tile)!, game.tile(ally.tile)!) === 1,
                )
                .map((other) => TYPES[other.type].healing ?? 0),
            );
            value +=
              Math.min(Math.max(0, healing - otherHealing), TYPES[ally.type].hp - ally.hp) * 3;
            if (role(ally) === "screen" && rangeToFoe(ally.tile) <= 4)
              value += rangeToFoe(tileId) > rangeToFoe(ally.tile) ? 3 : 0.5;
          }
          positioning += value * (movableScreens && unit.hp > TYPES[unit.type].hp / 2 ? 0.25 : 1);
        }
        if (category !== "screen" && screens.length && rangeToFoe(tileId) <= 4) {
          const front = Math.min(...screens.map((tank) => rangeToFoe(tank.tile)));
          positioning += rangeToFoe(tileId) > front ? 2 : -3;
        }
        if (category === "screen") {
          for (const protectedUnit of allies.filter(
            (ally) => essential(game, ally) || role(ally) !== "screen",
          )) {
            if (
              protectedUnit.id !== unit.id &&
              distance(tile, game.tile(protectedUnit.tile)!) === 1 &&
              rangeToFoe(tileId) <= rangeToFoe(protectedUnit.tile)
            )
              positioning += 1.5;
          }
        }
      }
      const attacks = foes.flatMap((target) =>
        game
          .attackOptions(unit, target, tileId)
          .filter((weapon) =>
            diplomacyAllowsAttack(
              game,
              unit,
              target,
              weapon.id,
              tileId,
              context.diplomacy ?? "pragmatic",
            ),
          )
          .map((weapon) => ({ target, weapon })),
      );
      const choices = [{ target: undefined, weapon: undefined }, ...attacks];
      for (const { target, weapon } of choices) {
        if (!target && tileId === unit.tile) continue;
        let combat = 0,
          retaliation = 0,
          eliminated: string | undefined,
          finishesBattle = false;
        if (target && weapon) {
          const preview = game.preview(unit, target, weapon.id, tileId);
          retaliation = preview.retaliation;
          const killed = preview.damage >= target.hp;
          eliminated = killed ? target.id : undefined;
          finishesBattle = killed && !game.campaign && Boolean(TYPES[target.type].leader);
          combat =
            Math.min(preview.damage, target.hp) * 2 +
            (killed ? 12 + TYPES[target.type].cost * 0.3 : 0) -
            retaliation * 1.8;
          if (essential(game, target)) combat += killed ? 18 : 4;
          if (
            killed &&
            (unit.experience ?? 0) + 6 * (TYPES[target.type].level ?? 1) >=
              (TYPES[unit.type].experienceNeeded ?? Infinity)
          )
            combat += 3;
          if (retaliation >= unit.hp) {
            if (important) continue;
            combat -= 18;
          }
        }
        const incoming = threats.at(unit, tileId, eliminated);
        const lethal = incoming >= unit.hp - retaliation;
        // Required actors avoid a predicted lethal approach; screens may make useful trades.
        const finishesEscort =
          finishesObjective(game, unit, tileId, objectives) && retaliation < unit.hp;
        if (
          TYPES[unit.type].leader &&
          !finishesEscort &&
          !finishesBattle &&
          !rebuilding.length &&
          !leaderMayAdvance(unit, tileId, restraint)
        )
          continue;
        if (
          important &&
          lethal &&
          incoming > 0 &&
          !finishesEscort &&
          !finishesBattle &&
          (minimumRisk < unit.hp || incoming + retaliation > minimumRisk || incoming >= beforeRisk)
        )
          continue;
        if (important && !finishesEscort && !finishesBattle)
          combat *= unit.role === "courier" ? 0.1 : 0.5;
        const dangerWeight = important
          ? 6
          : category === "screen"
            ? policy === "rush"
              ? 0.35
              : 0.55
            : 0.9;
        const score =
          positioning +
          (finishesEscort || finishesBattle ? 100 : 0) +
          combat -
          (finishesBattle ? 0 : incoming * dangerWeight) -
          (lethal && !finishesBattle ? (important ? 40 : 5) : 0) +
          Math.max(0, beforeRisk - incoming) * (important ? 3 : unit.hp <= 4 ? 1.5 : 0.2) +
          jitter(context.seed, `${game.round}:${unit.id}:${tileId}:${target?.id ?? "move"}`);
        if (score > 0.1 || (important && incoming < beforeRisk && !target))
          result.push({
            unit: unit.id,
            tile: tileId,
            target: target?.id,
            weapon: weapon?.id,
            score,
            withdrawing:
              unit.side === "enemy" &&
              !target &&
              unit.hp < TYPES[unit.type].hp &&
              rangeToFoe(tileId) > rangeToFoe(unit.tile) &&
              (incoming < beforeRisk || healingTile || Boolean(unit.retreating)),
          });
      }
    }
  }
  return result;
}

/** Chooses commands only. Turn transitions, combat, XP, access and victory stay in Game. */
export function playPolicyTurn(game: Game, policy: PolicyId, context: PolicyContext): void {
  if (game.winner) return;
  const side = game.turn;
  for (const unit of game.units.filter((unit) => unit.side === side && unit.retreating))
    if (unit.hp === TYPES[unit.type].hp) delete unit.retreating;
  recruit(game, policy, context);
  if (policy === "classic") {
    // Keep the live enemy's exact stable ordering. For player-side testing, use the same order.
    const roster = game.units
      .filter((unit) => unit.side === side)
      .map((unit) => ({ ...unit, side: "enemy" as const }));
    // Live enemy duties remain in Game; explicit tests/player campaigns add goals.
    const goals =
      context.objectives || (side === "player" && game.campaign)
        ? policyObjectives(game, context)
        : policyObjectives(game, { ...context, objectives: [] });
    const ids = enemyActionIds(roster);
    if (goals)
      ids.sort(
        (a, b) =>
          Number(reservedTiles(game.unit(b)!, goals).has(game.unit(b)!.tile)) -
          Number(reservedTiles(game.unit(a)!, goals).has(game.unit(a)!.tile)),
      );
    for (const id of ids) {
      if (game.winner) break;
      const unit = game.unit(id);
      if (!unit) continue;
      const eligible =
        goals &&
        individualObjectives(goals).filter(
          (goal) =>
            canPursue(unit, goal) &&
            (goal.kind !== "capture" || goal.tiles.some((tile) => game.tile(tile)?.owner !== side)),
        );
      const order = enemyTileOrder(game.tiles, game.units, unit, true);
      const best = eligible
        ?.map((goal) => ({
          goal,
          distance:
            Math.min(
              ...goal.tiles.map((tile) => distance(game.tile(unit.tile)!, game.tile(tile)!)),
            ) / goal.weight,
        }))
        .sort((a, b) => {
          const nearest = (goal: AiObjective) =>
            goal.tiles
              .map((tile) => game.tile(tile)!)
              .sort(
                (first, second) =>
                  distance(game.tile(unit.tile)!, first) -
                    distance(game.tile(unit.tile)!, second) || order(first, second),
              )[0]!;
          return a.distance - b.distance || order(nearest(a.goal), nearest(b.goal));
        })[0]?.goal;
      game.enemyAct(
        id,
        side,
        reservedTiles(unit, goals ?? []).has(unit.tile)
          ? game
              .neighbors(unit.tile)
              .filter((tile) => !reservedTiles(unit, goals ?? []).has(tile.id))
          : best?.tiles.map((tile) => game.tile(tile)!).filter(Boolean),
        best?.kind === "escape",
        classicRules(game, unit, goals ?? [], context),
      );
    }
    return;
  }
  const routes = new Map<string, Map<string, number>>();
  // Every successful command consumes a real action; no fictional waits or unlimited search.
  const actionLimit = game.units.filter((unit) => unit.side === side).length * 3;
  for (let actions = 0; actions < actionLimit && !game.winner; actions++) {
    const candidates = plans(game, policy, context, routes).sort((a, b) => b.score - a.score);
    const choice = candidates[0];
    if (!choice) break;
    const unit = game.unit(choice.unit)!;
    if (choice.advance) {
      game.advanceUnit(unit.id, choice.advance);
      context.onAction?.({ kind: "advance", unit: unit.id, type: choice.advance });
    } else {
      if (choice.tile !== unit.tile) {
        game.move(unit.id, choice.tile);
        if (choice.withdrawing) unit.retreating = true;
        else delete unit.retreating;
        context.onAction?.({ kind: "move", unit: unit.id, tile: choice.tile });
      }
      if (choice.target && choice.weapon && !game.winner) {
        delete unit.retreating;
        game.attack(unit.id, choice.target, choice.weapon);
        context.onAction?.({
          kind: "attack",
          unit: unit.id,
          target: choice.target,
          weapon: choice.weapon,
        });
      }
    }
  }
}

function classicRules(game: Game, unit: Unit, goals: AiObjective[], context: PolicyContext) {
  const reserved = reservedTiles(unit, goals);
  const lanes = withdrawalLanes(game, unit.side, context.diplomacy ?? "pragmatic");
  const important = essential(game, unit);
  const threats = important ? new Threats(game, unit.side) : undefined;
  const risk = (tile: string, eliminated?: string) => threats?.at(unit, tile, eliminated) ?? 0;
  const minimum = important
    ? Math.min(...[unit.tile, ...game.reachable(unit.id).keys()].map((tile) => risk(tile)))
    : 0;
  const holdingEndpoint = goals.some(
    (goal) => goal.actor === unit.id && goal.tiles.includes(unit.tile),
  );
  return {
    positionPenalty: (_actor: Unit, tile: string) =>
      (reserved.has(tile) ? 160 : 0) +
      (lanes.has(tile) ? 100 : 0) +
      (holdingEndpoint && tile !== unit.tile ? 100 : 0) +
      (finishesObjective(game, unit, tile, goals) ? -1000 : risk(tile) * 12),
    movementAllowed: (_actor: Unit, tile: string) =>
      !important ||
      finishesObjective(game, unit, tile, goals) ||
      risk(tile) < unit.hp ||
      risk(tile) <= minimum,
    attackAllowed: (_actor: Unit, target: Unit, weapon: string, from: string) => {
      if (
        !diplomacyAllowsAttack(game, unit, target, weapon, from, context.diplomacy ?? "pragmatic")
      )
        return false;
      if (!important) return true;
      const preview = game.preview(unit, target, weapon, from);
      if (preview.retaliation >= unit.hp) return false;
      if (preview.damage >= target.hp && !game.campaign && TYPES[target.type].leader) return true;
      const incoming = risk(from, preview.damage >= target.hp ? target.id : undefined);
      return (
        incoming + preview.retaliation < unit.hp &&
        (unit.role !== "courier" || incoming + preview.retaliation === 0)
      );
    },
  };
}
