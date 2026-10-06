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

type RecruitmentRole = "tank" | "flanker" | "ranged" | "healer";
function recruitmentRole(type: UnitType): RecruitmentRole {
  const definition = TYPES[type];
  if (definition.healing) return "healer";
  // Fast melee specialists remain flankers after gaining a ranged sidearm.
  if (definition.move >= 5 && definition.attacks[0].category === "melee") return "flanker";
  return definition.range > 1 ? "ranged" : "tank";
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
    const counts = { tank: 0, flanker: 0, ranged: 0, healer: 0 };
    for (const ally of allies) counts[recruitmentRole(ally.type)]++;
    const pool = game.recruitmentOptions(side);
    const shares = { tank: 0.35, flanker: 0.15, ranged: 0.35, healer: 0.15 };
    const availableRoles = new Set(pool.map(recruitmentRole));
    const totalShare = [...availableRoles].reduce((total, category) => total + shares[category], 0);
    const choices = pool
      .filter(
        (type) =>
          (policy !== "rush" || role({ type }) === "screen") &&
          (policy === "mixed"
            ? game.recruitPlacementTiles(leader.id, type)
            : game.recruitTiles(leader.id, type)
          ).some((tile) => placementAllowed(type, tile)),
      )
      .map((type) => {
        const definition = TYPES[type];
        const category = recruitmentRole(type);
        const desired = shares[category] / totalShare;
        const deficiency = desired * (allies.length + 1) - counts[category];
        const recruit: Unit = {
          id: "planned-recruit",
          type,
          side,
          tile: leader.tile,
          hp: definition.hp,
          moved: true,
          attacked: true,
        };
        // Value the weapon the formation can deliver, with the target's actual cover.
        const attacks = definition.attacks.filter((weapon) =>
          category === "ranged" || category === "healer"
            ? weapon.category === "ranged"
            : weapon.category === "melee",
        );
        const effectiveness = opponents.length
          ? opponents.reduce(
              (total, target) =>
                total +
                Math.max(0, ...attacks.map((weapon) => game.damage(recruit, target, weapon.id))),
              0,
            ) / opponents.length
          : definition.damage;
        const efficiency = (definition.hp * 0.35 + effectiveness * 1.5) / definition.cost;
        const score =
          policy === "rush"
            ? efficiency * 5 + definition.hp * 0.15
            : deficiency * 8 +
              efficiency * 2 +
              (category === "tank" && counts.tank === 0 ? 6 : 0) +
              (category === "healer" && counts.healer === 0 ? 3 : 0);
        return { type, score: score + jitter(context.seed, `recruit:${game.round}:${type}`) };
      })
      .sort((a, b) => b.score - a.score);
    const preferred = choices[0];
    if (!preferred) return;
    if (TYPES[preferred.type].cost > game.gold[side] && game.income(side) > 0) {
      const missingTank = recruitmentRole(preferred.type) === "tank" && counts.tank === 0;
      // Preserve a replacement tank's budget; other roles save at most two income ticks.
      if (missingTank || TYPES[preferred.type].cost - game.gold[side] <= game.income(side) * 2)
        return;
    }
    const choice = choices.find((candidate) => TYPES[candidate.type].cost <= game.gold[side]);
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
  private readonly source: Game;
  private readonly important: Set<string>;
  private readonly shadow: Game;
  private readonly enemies: { unit: Unit; from: Set<string> }[];
  private readonly cache = new Map<string, number>();
  private readonly forecasts = new Map<string, ReturnType<Game["dangerForecast"]>>();
  private readonly origins = new Map<string, string[]>();
  constructor(game: Game, side: Side) {
    this.source = game;
    this.important = new Set(
      game.units
        .filter((unit) => unit.side === side && essential(game, unit))
        .map((unit) => unit.id),
    );
    this.shadow = game.fork();
    // Authored next-phase arrivals and healing use the real turn boundary.
    this.shadow.endTurn();
    this.enemies = this.shadow.units
      .filter((unit) => unit.side !== side)
      .map((unit) => ({
        unit,
        from: new Set([unit.tile, ...this.shadow.reachable(unit.id).keys()]),
      }));
  }
  at(unit: Unit, tile: string, eliminated?: string): number {
    const key = `${unit.id}:${unit.type}:${tile}:${eliminated ?? ""}`;
    const cached = this.cache.get(key);
    if (cached !== undefined) return cached;
    if (this.important.has(unit.id)) {
      let forecast = this.forecasts.get(unit.id);
      if (!forecast) {
        forecast = this.source.dangerForecast(unit);
        this.forecasts.set(unit.id, forecast);
      }
      const risk = forecast(tile, eliminated);
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
          const key = `${tile}:${weapon.category}:${weapon.range}`;
          let origins = this.origins.get(key);
          if (!origins) {
            origins = board.attackOrigins(tile, weapon);
            this.origins.set(key, origins);
          }
          if (origins.some((position) => from.has(position)))
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
  commitments: Map<string, string>,
): Plan[] {
  const side = game.turn;
  const allies = game.units.filter((unit) => unit.side === side);
  const foes = game.units.filter((unit) => unit.side !== side);
  const objectives = individualObjectives(policyObjectives(game, context));
  if (allies.every((unit) => unit.attacked)) return [];
  const lanes = withdrawalLanes(game, side, context.diplomacy ?? "pragmatic");
  const threats = new Threats(game, side);
  const screens = allies.filter((unit) => role(unit) === "screen" && !TYPES[unit.type].leader);
  const movableScreens = screens.some((unit) => !unit.moved && !unit.attacked);
  let best: Plan | undefined;
  const ranges = new Map<string, number>();
  const firingOrigins = new Map<string, Set<string>>();
  const canFire = (
    target: Unit,
    weapon: (typeof TYPES)[UnitType]["attacks"][number],
    tile: string,
  ) => {
    const key = `${target.tile}:${weapon.category}:${weapon.range}`;
    let origins = firingOrigins.get(key);
    if (!origins) {
      origins = new Set(game.attackOrigins(target.tile, weapon));
      firingOrigins.set(key, origins);
    }
    return origins.has(tile);
  };
  const rangeToFoe = (tile: string) => {
    let range = ranges.get(tile);
    if (range === undefined) {
      range = foes.length
        ? Math.min(...foes.map((foe) => distance(game.tile(tile)!, game.tile(foe.tile)!)))
        : 20;
      ranges.set(tile, range);
    }
    return range;
  };
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
    if (promotion && beforeRisk < unit.hp && (!best || best.score < 14))
      best = { unit: unit.id, tile: unit.tile, advance: promotion, score: 14 };
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
    const goals: AiObjective[] = rebuilding.length
      ? [
          {
            id: "rebuild",
            kind: "capture",
            actor: unit.id,
            tiles: rebuilding.map((tile) => tile.id),
            weight: 80,
          },
        ]
      : objectives;
    const duties = goals.flatMap((original) => {
      if (
        original.kind === "capture" &&
        !original.actor &&
        original.tiles.every((id) => {
          const occupant = game.at(id);
          return (
            game.tile(id)?.owner === side &&
            (!original.hold || (occupant?.side === side && occupant.id !== unit.id))
          );
        })
      )
        return [];
      // Rebuilding is the leader's duty, not a reason to recall the whole army.
      if (original.id.startsWith("rebuild:") && original.actor !== unit.id) return [];
      const assisting = !canPursue(unit, original) && Boolean(original.actor || original.family);
      if (!canPursue(unit, original) && !assisting) return [];
      const goal = assisting ? supportObjective(game, unit, original) : original;
      const route = field(unit, goal);
      const before = route.get(unit.tile);
      return before !== undefined && Number.isFinite(before) && goal.tiles.length
        ? [
            {
              goal,
              route,
              before,
              assigned: !assisting && Boolean(original.actor || original.family),
              priority: (before + 3) / goal.weight,
            },
          ]
        : [];
    });
    const eligibleDuties = duties.some((duty) => duty.assigned)
      ? duties.filter((duty) => duty.assigned)
      : duties;
    const order = enemyTileOrder(game.tiles, game.units, unit, true);
    const nearest = (goal: AiObjective) =>
      goal.tiles
        .map((id) => game.tile(id)!)
        .sort(
          (a, b) =>
            distance(game.tile(unit.tile)!, a) - distance(game.tile(unit.tile)!, b) || order(a, b),
        )[0]!;
    // Select a duty before comparing moves. A move cannot switch objectives just
    // to hide its retreat from the one this unit was advancing toward.
    const duty =
      eligibleDuties.find(({ goal }) => goal.id === commitments.get(unit.id)) ??
      eligibleDuties.sort(
        (a, b) => a.priority - b.priority || order(nearest(a.goal), nearest(b.goal)),
      )[0];
    if (duty) commitments.set(unit.id, duty.goal.id);
    else commitments.delete(unit.id);
    const dangerWeight = important
      ? 6
      : category === "screen"
        ? policy === "rush"
          ? 0.35
          : 0.55
        : 0.9;
    const unitPlans: Plan[] = [];
    let waitScore = -Infinity;
    // Evaluate waiting even when an authored post requires a different destination.
    for (const tileId of new Set([unit.tile, ...destinations])) {
      const tile = game.tile(tileId)!;
      let positioning =
        (reserved.has(unit.tile) ? 80 : 0) -
        (reserved.has(tileId) ? 160 : 0) -
        (lanes.has(tileId) ? 100 : 0);
      if (duty) {
        const after = duty.route.get(tileId);
        if (after === undefined || !Number.isFinite(after)) continue;
        const { goal, before } = duty;
        positioning +=
          ((before - after) * goal.weight) / 4 +
          (Number(goal.tiles.includes(tileId)) - Number(goal.tiles.includes(unit.tile))) *
            goal.weight *
            0.6;
      }
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
      if (tileId === unit.tile)
        waitScore =
          positioning -
          beforeRisk * dangerWeight -
          (beforeRisk >= unit.hp ? (important ? 40 : 5) : 0);
      if (!destinations.includes(tileId)) continue;
      const attacks = foes.flatMap((target) =>
        TYPES[unit.type].attacks
          .filter((weapon) => canFire(target, weapon, tileId))
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
        const score =
          positioning +
          (finishesEscort || finishesBattle ? 100 : 0) +
          combat -
          (finishesBattle ? 0 : incoming * dangerWeight) -
          (lethal && !finishesBattle ? (important ? 40 : 5) : 0) +
          Math.max(0, beforeRisk - incoming) * (important ? 3 : unit.hp <= 4 ? 1.5 : 0.2) +
          jitter(context.seed, `${game.round}:${unit.id}:${tileId}:${target?.id ?? "move"}`);
        unitPlans.push({
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
    // Formation, healing and safe ground already available by waiting are not
    // fresh gains. Tiny tie-breaking jitter alone must never cause a move.
    for (const plan of unitPlans) {
      const score = plan.score - waitScore;
      // Strict comparison retains the first equal-score plan, as stable sorting did.
      if (score > 0.1 && (!best || score > best.score)) best = { ...plan, score };
    }
  }
  return best ? [best] : [];
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
  const commitments = new Map<string, string>();
  // Every successful command consumes a real action; no fictional waits or unlimited search.
  const actionLimit = game.units.filter((unit) => unit.side === side).length * 3;
  for (let actions = 0; actions < actionLimit && !game.winner; actions++) {
    const candidates = plans(game, policy, context, routes, commitments);
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
