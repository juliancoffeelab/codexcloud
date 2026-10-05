import { Game, TYPES } from "../engine";
import {
  campaignMission,
  goalProgress,
  returnsWithStory,
  endsRefugeMediation,
} from "../campaign-progress";
import { CAMPAIGNS } from "../campaign-data";
import { playPolicyTurn, seededRandom } from "./policies";
import type { AiAction, PolicyId } from "./policies";
import type { AiObjective } from "./objectives";
import type { DiplomacyId } from "./diplomacy";
import type { CampaignId, RecruitType, Side, TerrainType, Unit, UnitType } from "../types";

export const ARMY_IDS = ["human", "woodland", "undead", "coastal"] as const;
export type ArmyId = (typeof ARMY_IDS)[number];
export const LANDSCAPES = ["open", "wooded", "crossings"] as const;
export type Landscape = (typeof LANDSCAPES)[number];
const ARMIES: Record<ArmyId, { leader: UnitType; screen: RecruitType; pool: RecruitType[] }> = {
  human: {
    leader: "commander",
    screen: "swordsman",
    pool: ["swordsman", "spearman", "archer", "scout", "adept"],
  },
  woodland: { leader: "wildkeeper", screen: "boar", pool: ["boar", "wolf", "thorncaller"] },
  undead: { leader: "necromancer", screen: "skeleton", pool: ["skeleton", "bonearcher", "adept"] },
  coastal: {
    leader: "tidekeeper",
    screen: "reefguard",
    pool: ["reefguard", "harpooner", "wavemender"],
  },
};
const otherSide = (side: Side): Side => (side === "player" ? "enemy" : "player");

/** Same command boundary as gameplay; logging also captures classic's internal commands. */
class RecordedGame extends Game {
  commands: AiAction[] = [];
  casualties: Unit[] = [];
  combats: {
    unit: string;
    target: string;
    weapon: string;
    from: string;
    targetTile: string;
    damage: number;
    retaliation: number;
    mediationAtRisk: boolean;
  }[] = [];
  pools?: Record<Side, RecruitType[]>;
  /** Arena rounds contain both seats, regardless of which seat has initiative. */
  arenaFirst?: Side;
  override endTurn(): void {
    if (!this.arenaFirst) return super.endTurn();
    if (this.winner) throw new Error("The battle is over.");
    if (this.turn !== this.arenaFirst) this.round++;
    this.beginTurn(otherSide(this.turn));
  }
  override recruitmentOptions(side: Side): RecruitType[] {
    return this.pools ? [...this.pools[side]] : super.recruitmentOptions(side);
  }
  override move(id: string, tile: string) {
    const result = super.move(id, tile);
    this.commands.push({ kind: "move", unit: id, tile });
    return result;
  }
  override attack(id: string, target: string, weapon?: string) {
    const attacker = this.unit(id)!,
      defender = this.unit(target)!;
    const selected = weapon ?? this.bestAttack(attacker, defender)?.id;
    const before = [structuredClone(attacker), structuredClone(defender)];
    const mediationAtRisk = endsRefugeMediation(this, defender);
    const result = super.attack(id, target, weapon);
    this.commands.push({ kind: "attack", unit: id, target, weapon: selected! });
    this.casualties.push(...before.filter((unit) => !this.unit(unit.id)));
    this.combats.push({
      unit: id,
      target,
      weapon: selected!,
      from: before[0]!.tile,
      targetTile: before[1]!.tile,
      ...result,
      mediationAtRisk,
    });
    return result;
  }
  override recruit(id: string, type: UnitType, tile: string) {
    const result = super.recruit(id, type, tile);
    this.commands.push({ kind: "recruit", unit: id, type, tile });
    return result;
  }
  override advanceUnit(id: string, type: UnitType) {
    const result = super.advanceUnit(id, type);
    this.commands.push({ kind: "advance", unit: id, type });
    return result;
  }
}

export interface PhaseRecord {
  side: Side;
  round: number;
  actions: AiAction[];
  losses: Unit[];
  combats: RecordedGame["combats"];
  living: Record<Side, number>;
  health: Record<Side, number>;
  gold: Record<Side, number>;
  before: Unit[];
  units: Unit[];
  ownership: { tile: string; owner: Side | null }[];
  goals: ReturnType<typeof goalProgress>[];
  objectives: AiObjective[];
  campaign?: ReturnType<Game["captureState"]>["campaign"];
}
function phase(
  game: RecordedGame,
  policy: PolicyId,
  seed: number,
  recruitLimit: number,
  objectives?: AiObjective[],
  diplomacy?: DiplomacyId,
): PhaseRecord {
  const before = [...game.units].map((unit) => ({ ...unit }));
  const side = game.turn,
    round = game.round;
  game.commands = [];
  game.casualties = [];
  game.combats = [];
  playPolicyTurn(game, policy, { seed, recruitLimit, objectives, diplomacy });
  game.advanceDialogue(true);
  const count = (owner: Side) => game.units.filter((unit) => unit.side === owner);
  return {
    side,
    round,
    actions: game.commands,
    losses: game.casualties,
    combats: game.combats,
    living: { player: count("player").length, enemy: count("enemy").length },
    health: {
      player: count("player").reduce((sum, unit) => sum + unit.hp, 0),
      enemy: count("enemy").reduce((sum, unit) => sum + unit.hp, 0),
    },
    gold: { ...game.gold },
    before,
    units: structuredClone(game.units),
    ownership: game.tiles.map(({ id: tile, owner }) => ({ tile, owner })),
    goals: game.campaign
      ? campaignMission(game.campaign).goals.map((goal) => goalProgress(game, goal))
      : [],
    objectives: structuredClone(objectives ?? []),
    ...(game.campaign ? { campaign: structuredClone(game.campaign) } : {}),
  };
}

export interface ArenaConfig {
  seed: number;
  armies: Record<Side, ArmyId>;
  landscape: Landscape;
  player: PolicyId;
  enemy: PolicyId;
  first: Side;
  maxRounds: number;
  recruitLimit: number;
}
export function createArena(
  config: Pick<ArenaConfig, "seed" | "armies" | "landscape" | "first">,
): RecordedGame {
  const random = seededRandom(config.seed);
  // Classic's shipped river AI uses route distances, unlike its open-map heuristic.
  const game = new RecordedGame(config.landscape === "crossings" ? "river" : "mirror");
  const state = game.captureState();
  state.cols = state.rows = 9;
  state.tiles = [];
  // Radius-four axial hexagon: every tile has an exact 180-degree partner.
  const id = (q: number, r: number) => `${q + Math.floor(r / 2) + 4},${r + 4}`;
  for (let r = -4; r <= 4; r++)
    for (let q = -4; q <= 4; q++) {
      if (Math.abs(q + r) > 4) continue;
      const tileId = id(q, r);
      const [col, row] = tileId.split(",").map(Number) as [number, number];
      state.tiles.push({ id: tileId, col, row, terrain: "plains", owner: null });
    }
  const used = new Set<string>();
  for (const tile of state.tiles) {
    if (used.has(tile.id)) continue;
    const partner = state.tiles.find(
      (other) => other.row === 8 - tile.row && other.col === 8 - tile.col - (tile.row % 2),
    )!;
    used.add(tile.id);
    used.add(partner.id);
    const draw = random();
    let terrain: TerrainType = "plains";
    if (config.landscape === "wooded")
      terrain = draw < 0.25 ? "forest" : draw < 0.32 ? "grove" : draw < 0.4 ? "hill" : "plains";
    if (config.landscape === "crossings") {
      const q = tile.col - Math.floor((tile.row - 4) / 2) - 4;
      terrain = q === 0 ? "water" : draw < 0.2 ? "marsh" : draw < 0.3 ? "forest" : "plains";
    }
    tile.terrain = partner.terrain = terrain;
  }
  const terrain = (q: number, r: number, kind: TerrainType, owner: Side | null = null) => {
    const tile = state.tiles.find((tile) => tile.id === id(q, r))!;
    tile.terrain = kind;
    tile.owner = owner;
  };
  terrain(-3, 0, "keep", "player");
  terrain(3, 0, "keep", "enemy");
  terrain(0, 0, "fort");
  terrain(0, -2, "village");
  terrain(0, 2, "village");
  terrain(-2, 0, "plains");
  terrain(2, 0, "plains");
  terrain(-3, 1, "plains");
  terrain(3, -1, "plains");
  terrain(-2, -1, "village");
  terrain(2, 1, "village");
  terrain(-1, 3, "ruins");
  terrain(1, -3, "ruins");
  if (config.landscape === "crossings") {
    terrain(0, -1, "bridge");
    terrain(0, 1, "bridge");
  }
  state.units = [];
  state.nextId = 1;
  const add = (type: UnitType, side: Side, q: number, r: number) =>
    state.units.push({
      id: `u${state.nextId++}`,
      type,
      side,
      tile: id(q, r),
      hp: TYPES[type].hp,
      moved: false,
      attacked: false,
      experience: 0,
      ...(TYPES[type].leader ? { role: "leader" as const } : {}),
    });
  for (const side of ["player", "enemy"] as const) {
    const army = ARMIES[config.armies[side]];
    const sign = side === "player" ? -1 : 1;
    add(army.leader, side, sign * 3, 0);
    add(army.screen, side, sign * 2, 0);
    add(army.screen, side, sign * 3, -sign);
  }
  const gold = 18 + Math.floor(random() * 3) * 3;
  state.gold = { player: gold, enemy: gold };
  state.round = 1;
  state.turn = config.first;
  state.winner = null;
  state.log = [];
  delete state.campaign;
  delete state.lastCombat;
  game.restoreState(state);
  game.pools = {
    player: [...ARMIES[config.armies.player].pool],
    enemy: [...ARMIES[config.armies.enemy].pool],
  };
  game.arenaFirst = config.first;
  return game;
}

export interface ArenaResult {
  config: ArenaConfig;
  winner: Side | null;
  reason: "leader" | "territory" | "cap";
  rounds: number;
  phases: PhaseRecord[];
  objectives: Record<Side, { controlled: number; held: number; optionalVisits: number }>;
  initial: ReturnType<Game["captureState"]>;
  final: ReturnType<Game["captureState"]>;
}
export function runArena(config: ArenaConfig): ArenaResult {
  const game = createArena(config);
  const initial = game.captureState();
  // Each seat receives its first turn's income before its first decisions.
  game.beginTurn(config.first);
  const territory = ["4,4", "3,2", "5,6"];
  const held = { player: 0, enemy: 0 };
  const visits = { player: new Set<string>(), enemy: new Set<string>() };
  const phases: PhaseRecord[] = [];
  let reason: ArenaResult["reason"] = "cap";
  let rounds = 0;
  for (let index = 0; index < config.maxRounds * 2 && !game.winner; index++) {
    const side = game.turn;
    const objectives: AiObjective[] = [
      { id: "required-territory", kind: "capture", tiles: territory, weight: 24 },
      ...game.tiles
        .filter((tile) => tile.terrain === "village" && tile.owner !== side)
        .map((tile) => ({
          id: `village:${tile.id}`,
          kind: "capture" as const,
          tiles: [tile.id],
          weight: 8,
        })),
      {
        id: "opposing-leader",
        kind: "defeat",
        tiles: game.units
          .filter((unit) => unit.side !== side && TYPES[unit.type].leader)
          .map((unit) => unit.tile),
        weight: 12,
      },
      ...game.tiles
        .filter((tile) => tile.terrain === "ruins" && !visits[side].has(tile.id))
        .map((tile) => ({
          id: `optional:${tile.id}`,
          kind: "visit" as const,
          tiles: [tile.id],
          weight: 4,
        })),
    ];
    const record = phase(game, config[side], config.seed, config.recruitLimit, objectives);
    phases.push(record);
    for (const command of record.actions)
      if (command.kind === "move" && game.tile(command.tile)?.terrain === "ruins")
        visits[side].add(command.tile);
    rounds = Math.ceil(phases.length / 2);
    if (game.winner) {
      reason = "leader";
      break;
    }
    for (const owner of ["player", "enemy"] as const) {
      if (!territory.every((tile) => game.tile(tile)!.owner === owner)) held[owner] = 0;
      else if (owner !== side) held[owner]++;
      if (held[owner] >= 2) {
        game.winner = owner;
        reason = "territory";
      }
    }
    if (!game.winner && index + 1 < config.maxRounds * 2) game.endTurn();
  }
  return {
    config,
    winner: game.winner,
    reason,
    rounds,
    phases,
    initial,
    final: game.captureState(),
    objectives: Object.fromEntries(
      (["player", "enemy"] as const).map((side) => [
        side,
        {
          controlled: territory.filter((tile) => game.tile(tile)!.owner === side).length,
          held: held[side],
          optionalVisits: visits[side].size,
        },
      ]),
    ) as ArenaResult["objectives"],
  };
}

export interface CampaignConfig {
  diplomacy: DiplomacyId;
  campaign: CampaignId;
  player: PolicyId;
  enemy: PolicyId;
  seed: number;
  maxRounds: number;
  recruitLimit: number;
}
export interface CampaignResult {
  config: CampaignConfig;
  completed: boolean;
  missions: {
    id: string;
    winner: Side | null;
    round: number;
    goals: ReturnType<typeof goalProgress>[];
    phases: PhaseRecord[];
    initial: ReturnType<Game["captureState"]>;
    final: ReturnType<Game["captureState"]>;
  }[];
}
function recorded(source: Game): RecordedGame {
  const game = new RecordedGame(source.mapId, source.mapRevision);
  game.restoreState(source.captureState());
  return game;
}
export function runCampaign(config: CampaignConfig): CampaignResult {
  let game = recorded(Game.startCampaign(config.campaign));
  const missions: CampaignResult["missions"] = [];
  for (let mission = 0; mission < CAMPAIGNS[config.campaign].missions.length; mission++) {
    game.advanceDialogue(true);
    const initial = game.captureState();
    const phases: PhaseRecord[] = [];
    for (let index = 0; index < config.maxRounds * 2 && !game.winner; index++) {
      phases.push(
        phase(
          game,
          config[game.turn],
          config.seed,
          config.recruitLimit,
          undefined,
          config.diplomacy,
        ),
      );
      if (!game.winner && index + 1 < config.maxRounds * 2) game.endTurn();
    }
    const definition = campaignMission(game.campaign!);
    missions.push({
      id: game.mapId,
      winner: game.winner,
      round: game.round,
      goals: definition.goals.map((goal) => goalProgress(game, goal)),
      phases,
      initial,
      final: game.captureState(),
    });
    if (game.winner !== "player" || game.campaign?.completed) break;
    const recall = game.units
      .filter(
        (unit) =>
          unit.side === "player" &&
          !TYPES[unit.type].leader &&
          unit.role !== "courier" &&
          !definition.requiredCompanions?.includes(unit.name ?? "") &&
          !returnsWithStory(game.campaign!, unit),
      )
      .sort((a, b) => veteranValue(b) - veteranValue(a))
      .slice(0, 4)
      .map((unit) => unit.id);
    game = recorded(game.nextCampaignMission(recall));
  }
  return { config, completed: game.campaign?.completed ?? false, missions };
}
function veteranValue(unit: Unit): number {
  const definition = TYPES[unit.type];
  return (
    definition.hp +
    definition.damage * 2 +
    (definition.healing ?? 0) * 5 +
    (definition.level ?? 1) * 3 +
    (unit.experience ?? 0) / 3
  );
}
