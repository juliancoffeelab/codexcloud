import type {
  Side,
  UnitType,
  TerrainType,
  DamageType,
  AttackCategory,
  MapId,
  Coordinates,
  Tile,
  Unit,
  Weapon,
  UnitDefinition,
  TerrainDefinition,
  DamageBreakdown,
  CombatPreview,
  CombatPreviewDetails,
  LastCombat,
  GameSnapshot,
  GameState,
} from "./types";
export type * from "./types";
import { MAPS, LEGACY_RIVER } from "./maps";
import { immutable } from "./immutable";
import { CAMPAIGN_UNITS } from "./campaign-units";
import { HEAVY_MOVEMENT, PATHFINDER_MOVEMENT, movementCost } from "./movement";
import { RACES, resolveUnits } from "./races";
import { progressionUnits } from "./progression";
export { RACES } from "./races";
import { CAMPAIGNS, LEGACY_ASHEN_REFUGE } from "./campaign-data";
import {
  campaignMission,
  goalProgress,
  endsRefugeMediation,
  returnsWithStory,
} from "./campaign-progress";
import type { CampaignId, CampaignState, CampaignVeteran, RecruitType } from "./types";

export const DAMAGE_TYPES = Object.freeze({
  blade: "Blade",
  pierce: "Pierce",
  impact: "Impact",
});
const weapon = (
  id: string,
  name: string,
  damage: number,
  damageType: DamageType,
  category: AttackCategory,
  range: number,
  sound = "attack-melee",
): Weapon => ({ id, name, damage, damageType, category, range, sound });
export const TYPES = immutable<Record<UnitType, UnitDefinition>>(
  resolveUnits(
    progressionUnits({
      ...CAMPAIGN_UNITS,
      commander: {
        name: "Commander",
        hp: 16,
        move: 3,
        damage: 4,
        range: 1,
        cost: 0,
        leader: true,
        race: "human",
        terrainModifiers: HEAVY_MOVEMENT,
        attacks: [
          weapon("sword", "Sword", 4, "blade", "melee", 1),
          weapon("mace", "Mace", 3, "impact", "melee", 1),
        ],
        damageModifiers: { blade: -1, pierce: 0, impact: 0 },
        ability: "Recruit",
        description: "Recruit while on your keep. New recruits act next turn.",
      },
      soldier: {
        name: "Soldier",
        race: "human",
        terrainModifiers: HEAVY_MOVEMENT,
        hp: 12,
        move: 3,
        damage: 4,
        range: 1,
        cost: 8,
        attacks: [
          weapon("sword", "Sword", 4, "blade", "melee", 1),
          weapon("mace", "Mace", 3, "impact", "melee", 1),
        ],
        damageModifiers: { blade: -1, pierce: -1, impact: 1 },
        ability: "Armor",
        description: "Takes 1 less Blade/Pierce damage and 1 more Impact damage.",
      },
      archer: {
        name: "Archer",
        race: "human",
        hp: 8,
        move: 3,
        damage: 4,
        range: 2,
        cost: 8,
        attacks: [
          weapon("bow", "Bow", 4, "pierce", "ranged", 2, "attack-bow"),
          weapon("knife", "Knife", 2, "blade", "melee", 1),
        ],
        damageModifiers: { blade: 0, pierce: 0, impact: 0 },
        ability: "Bow",
        description:
          "Strong Pierce bow reaches 2 hexes. Slower than a scout; its knife is a weak melee reply.",
      },
      scout: {
        name: "Scout",
        race: "human",
        terrainModifiers: PATHFINDER_MOVEMENT,
        hp: 8,
        move: 5,
        damage: 2,
        range: 2,
        cost: 5,
        attacks: [
          weapon("knife", "Knife", 2, "blade", "melee", 1),
          weapon("sling", "Sling", 1, "impact", "ranged", 2, "unit-archer"),
        ],
        damageModifiers: { blade: 0, pierce: 0, impact: 0 },
        ability: "Pathfinder",
        description: "Forests and hills cost only 1 movement.",
      },
    }),
  ),
);
export const TERRAIN = immutable<Record<TerrainType, TerrainDefinition>>({
  plains: {
    name: "Plains",
    cost: 1,
    passable: true,
    description: "Normal movement. No protection.",
  },
  forest: {
    damageModifiers: { ranged: -1 },
    name: "Forest",
    cost: 2,
    passable: true,
    description: "Costs 2 movement; ranged damage reduced by 1.",
  },
  hill: {
    damageModifiers: { melee: -1, ranged: -1 },
    name: "Hills",
    cost: 2,
    passable: true,
    description: "Costs 2 movement; all damage reduced by 1.",
  },
  water: { name: "Water", cost: 1, passable: false, description: "Impassable to land units." },
  bridge: {
    name: "Bridge",
    cost: 1,
    passable: true,
    description: "A crossing over water. Normal movement; no protection.",
  },
  ford: {
    damageModifiers: { melee: 1, ranged: 1 },
    name: "Shallow water",
    cost: 2,
    passable: true,
    description:
      "Costs 3 movement for soldiers and commanders, 2 for archers and scouts. Units here take 1 extra damage.",
  },
  village: {
    name: "Village",
    cost: 1,
    passable: true,
    description: "+2 gold per turn. Heals a friendly occupant by 2 HP.",
  },
  keep: {
    name: "Keep",
    cost: 1,
    passable: true,
    description: "Your commander can recruit into adjacent empty hexes.",
  },
  marsh: {
    damageModifiers: { melee: 1, ranged: 1 },
    name: "Marsh",
    cost: 2,
    passable: true,
    description:
      "Mud slows land units and exposes them to 1 extra damage. Merfolk move easily here and take 1 less damage.",
  },
  ruins: {
    damageModifiers: { ranged: -1 },
    name: "Ruins",
    cost: 1,
    passable: true,
    description:
      "Broken walls reduce ranged damage by 1 without blocking shots through the tile. Some campaign ruins contain binding stones; their objectives specify who can seal them.",
  },
  fort: {
    damageModifiers: { melee: -1, ranged: -1 },
    name: "Fort",
    cost: 2,
    passable: true,
    description:
      "Earthworks reduce all incoming damage by 1. Capture by entering; forts provide no income or healing.",
  },
  grove: {
    damageModifiers: { ranged: -1 },
    name: "Sacred grove",
    cost: 2,
    passable: true,
    description:
      "Trees reduce ranged damage by 1 and block shots through the tile. An owned grove heals its friendly occupant by 2 HP, but provides no income.",
  },
});
export function axial(tile: Coordinates) {
  return { q: tile.col - Math.floor(tile.row / 2), r: tile.row };
}
export function distance(first: Coordinates, second: Coordinates) {
  const firstAxial = axial(first);
  const secondAxial = axial(second);
  const columnDifference = Math.abs(firstAxial.q - secondAxial.q);
  const rowDifference = Math.abs(firstAxial.r - secondAxial.r);
  const diagonalDifference = Math.abs(firstAxial.q + firstAxial.r - secondAxial.q - secondAxial.r);
  return (columnDifference + rowDifference + diagonalDifference) / 2;
}

export class Game {
  #tilesById = new Map<string, Tile>();
  #adjacentTiles = new Map<string, readonly Tile[]>();
  mapRevision: number;
  mapId: MapId;
  cols: number;
  rows: number;
  round: number;
  turn: Side;
  gold: Record<Side, number>;
  winner: Side | null;
  nextId: number;
  log: string[];
  tiles: Tile[];
  units: Unit[];
  declare lastCombat?: LastCombat | null;
  declare campaign?: CampaignState;

  constructor(mapId: string = "mirror", mapRevision = MAPS[mapId as MapId]?.revision ?? 1) {
    if (!Object.hasOwn(MAPS, mapId)) throw new Error("Unknown battlefield.");
    if (![1, MAPS[mapId as MapId].revision ?? 1].includes(mapRevision))
      throw new Error("Unknown battlefield revision.");
    const layout =
      mapId === "river" && mapRevision === 1
        ? LEGACY_RIVER
        : mapId === "ashen-refuge" && mapRevision === 1
          ? LEGACY_ASHEN_REFUGE
          : MAPS[mapId as MapId];
    this.mapRevision = mapRevision;
    this.mapId = mapId as MapId;
    this.cols = layout.cols;
    this.rows = layout.rows;
    this.round = 1;
    this.turn = "player";
    this.gold = { player: 12, enemy: 12 };
    this.winner = null;
    this.nextId = 1;
    this.log = [];
    this.tiles = [];
    this.units = [];
    for (let row = 0; row < this.rows; row++)
      for (let col = 0; col < this.cols; col++)
        this.tiles.push({
          id: `${col},${row}`,
          col,
          row,
          terrain: "plains",
          owner: null,
        });
    this.indexTiles();
    for (const { col, row, terrain, owner = null } of layout.terrain)
      Object.assign(this.tile(`${col},${row}`)!, { terrain, owner });
    for (const { type, side, col, row, name, role } of layout.units) {
      const unit = this.addUnit(type, side, `${col},${row}`);
      if (name) unit.name = name;
      if (role) unit.role = role;
    }
    this.record("Capture villages for gold. Defeat the enemy commander.");
  }

  tile(id: string | null | undefined): Tile | undefined {
    return id == null ? undefined : this.#tilesById.get(id);
  }

  private indexTiles(): void {
    this.#tilesById = new Map(this.tiles.map((tile) => [tile.id, tile]));
    const coordinates = new Map(this.tiles.map((tile) => [`${tile.col},${tile.row}`, tile]));
    const order = new Map(this.tiles.map((tile, index) => [tile.id, index]));
    this.#adjacentTiles = new Map(
      this.tiles.map((tile) => {
        const { col, row } = tile;
        const diagonal = row % 2 === 0 ? -1 : 1;
        const positions = [
          [col, row - 1],
          [col + diagonal, row - 1],
          [col - 1, row],
          [col + 1, row],
          [col, row + 1],
          [col + diagonal, row + 1],
        ];
        const neighbors = positions
          .map(([column, row]) => coordinates.get(`${column},${row}`))
          .filter((neighbor): neighbor is Tile => neighbor !== undefined)
          // Preserve the old array order, including equal-cost routing ties.
          .sort((first, second) => order.get(first.id)! - order.get(second.id)!);
        return [tile.id, neighbors];
      }),
    );
  }

  unit(id: string | null | undefined): Unit | undefined {
    return this.units.find((unit) => unit.id === id);
  }

  at(tileId: string | null | undefined): Unit | undefined {
    return this.units.find((unit) => unit.tile === tileId);
  }

  neighbors(id: string): Tile[] {
    // Callers may sort/filter their result without changing cached adjacency.
    return [...(this.#adjacentTiles.get(id) ?? [])];
  }

  record(message: string): void {
    this.log.unshift(message);
    this.log = this.log.slice(0, 8);
  }

  addUnit(type: UnitType, side: Side, tile: string): Unit {
    const unit: Unit = {
      id: `u${this.nextId++}`,
      type,
      side,
      tile,
      hp: TYPES[type].hp,
      moved: false,
      attacked: false,
      experience: 0,
    };
    this.units.push(unit);
    return unit;
  }

  requireActor(id: string): Unit {
    const unit = this.unit(id);
    if (this.winner || !unit || unit.side !== this.turn) {
      throw new Error("This unit cannot act now.");
    }
    return unit;
  }

  promotionOptions(id: string): UnitType[] {
    const unit = this.unit(id);
    if (!unit || this.winner || unit.side !== this.turn || unit.moved || unit.attacked) return [];
    const definition = TYPES[unit.type];
    return (unit.experience ?? 0) >= (definition.experienceNeeded ?? Infinity)
      ? [...(definition.advancesTo ?? [])].filter((type) =>
          Number.isFinite(this.cost({ type }, this.tile(unit.tile)!)),
        )
      : [];
  }

  advanceUnit(id: string, type: UnitType): Unit {
    const unit = this.requireActor(id);
    if (!this.promotionOptions(id).includes(type))
      throw new Error(
        "This advancement is not available. Advance a ready unit after it earns enough experience.",
      );
    const previous = TYPES[unit.type];
    const wounds = previous.hp - unit.hp;
    unit.experience = Math.max(0, (unit.experience ?? 0) - (previous.experienceNeeded ?? 0));
    unit.type = type;
    unit.hp = Math.max(1, TYPES[type].hp - wounds);
    unit.moved = true;
    unit.attacked = true;
    this.record(`${unit.name ?? previous.name} advanced to ${TYPES[type].name}.`);
    return unit;
  }

  private earnExperience(unit: Unit, damage: number, opponent: Unit): void {
    const limit = TYPES[unit.type].experienceNeeded ?? 0;
    if (unit.hp <= 0 || damage <= 0 || !limit) return;
    const reward = opponent.hp <= 0 ? 6 * (TYPES[opponent.type].level ?? 1) : 1;
    unit.experience = Math.min(limit, (unit.experience ?? 0) + reward);
  }

  cost(unit: Pick<Unit, "type">, tile: Pick<Tile, "terrain">): number {
    const definition = TYPES[unit.type];
    return movementCost(TERRAIN[tile.terrain], tile.terrain, RACES[definition.race], definition);
  }

  reachable(id: string | null | undefined): Map<string, number> {
    const unit = this.unit(id);
    if (!unit || unit.moved || unit.attacked || this.winner) return new Map();

    const movementCosts = new Map([[unit.tile, 0]]);
    const pendingTiles = [unit.tile];
    while (pendingTiles.length) {
      // Stable sorting preserves the existing choice between equal-cost routes.
      pendingTiles.sort((first, second) => movementCosts.get(first)! - movementCosts.get(second)!);
      const currentTileId = pendingTiles.shift()!;
      for (const neighbor of this.neighbors(currentTileId)) {
        if (this.at(neighbor.id)) continue;
        const movementCost = movementCosts.get(currentTileId)! + this.cost(unit, neighbor);
        const exceedsMovement = movementCost > TYPES[unit.type].move;
        const knownCost = movementCosts.get(neighbor.id) ?? Infinity;
        if (exceedsMovement || movementCost >= knownCost) continue;

        movementCosts.set(neighbor.id, movementCost);
        if (!pendingTiles.includes(neighbor.id)) pendingTiles.push(neighbor.id);
      }
    }
    movementCosts.delete(unit.tile);
    return movementCosts;
  }

  attackProfile(unit: Unit, attackId: string): Weapon | undefined {
    return TYPES[unit.type].attacks.find((attack) => attack.id === attackId);
  }

  canAttackFrom(
    unit: Unit | undefined,
    from: string,
    target: Unit | undefined,
    attackId: string | null = null,
  ): boolean {
    if (
      !unit ||
      !target ||
      !this.tile(from) ||
      !this.tile(target.tile) ||
      unit.side === target.side
    )
      return false;
    const attacks =
      attackId == null
        ? TYPES[unit.type].attacks
        : [this.attackProfile(unit, attackId)].filter(
            (attack): attack is Weapon => attack !== undefined,
          );
    const targetDistance = distance(this.tile(from)!, this.tile(target.tile)!);
    return attacks.some((attack) => {
      if (targetDistance < 1 || targetDistance > attack.range) return false;
      if (attack.category === "ranged" && targetDistance === 2) {
        return this.neighbors(from).some(
          (tile) =>
            distance(tile, this.tile(target.tile)!) === 1 &&
            !["forest", "hill", "grove"].includes(tile.terrain),
        );
      }
      return true;
    });
  }

  attackOptions(unit: Unit, target: Unit, from = unit.tile): Weapon[] {
    return TYPES[unit.type].attacks.filter((attack) =>
      this.canAttackFrom(unit, from, target, attack.id),
    );
  }

  targets(id: string | null | undefined): Unit[] {
    const unit = this.unit(id);
    if (!unit || unit.attacked || this.winner) return [];
    return this.units.filter((target) => this.canAttackFrom(unit, unit.tile, target));
  }

  damageBreakdown(
    attacker: Unit,
    defender: Unit,
    attackId = TYPES[attacker.type].attacks[0].id,
  ): DamageBreakdown {
    const attack = this.attackProfile(attacker, attackId);
    if (!attack) throw new Error("Unknown attack for this unit.");
    const terrain = this.tile(defender.tile)!.terrain;
    const modifier = TYPES[defender.type].damageModifiers[attack.damageType];
    const modified = attack.damage + modifier;
    const definition = TYPES[defender.type];
    const environment = [
      TERRAIN[terrain].damageModifiers,
      RACES[definition.race].terrainModifiers[terrain]?.damageModifiers,
      definition.terrainModifiers?.[terrain]?.damageModifiers,
    ].reduce(
      (total, modifiers) =>
        total + (modifiers?.[attack.category] ?? 0) + (modifiers?.[attack.damageType] ?? 0),
      0,
    );
    const protection = Math.max(0, -environment);
    const exposure = Math.max(0, environment);
    const damage = Math.max(0, modified - protection + exposure);
    return {
      base: attack.damage,
      damageType: attack.damageType,
      modifier,
      modified,
      protection,
      exposure,
      terrain,
      damage,
    };
  }

  damage(attacker: Unit, defender: Unit, attackId?: string): number {
    return this.damageBreakdown(attacker, defender, attackId).damage;
  }

  retaliationProfile(attacker: Unit, target: Unit, attack: Weapon): Weapon | null {
    return (
      this.attackOptions(target, attacker)
        .filter((response) => response.category === attack.category)
        .sort(
          (first, second) =>
            this.damage(target, attacker, second.id) - this.damage(target, attacker, first.id),
        )[0] || null
    );
  }

  previewDetails(
    attacker: Unit,
    target: Unit,
    attackId?: string | null,
    from = attacker.tile,
  ): CombatPreviewDetails {
    const attack =
      attackId == null
        ? this.bestAttack(attacker, target, from)
        : this.attackProfile(attacker, attackId);
    if (!attack || !this.canAttackFrom(attacker, from, target, attack.id))
      throw new Error("That attack cannot reach this enemy.");
    // A forecast from a prospective tile must not move the actual attacker.
    const positionedAttacker = { ...attacker, tile: from };
    const damageDetail = this.damageBreakdown(positionedAttacker, target, attack.id);
    const targetSurvives = target.hp > damageDetail.damage;
    const response = targetSurvives
      ? this.retaliationProfile(positionedAttacker, target, attack)
      : null;
    const retaliationDetail = response
      ? this.damageBreakdown(target, positionedAttacker, response.id)
      : null;
    return {
      attack,
      response,
      damageDetail,
      retaliationDetail,
      damage: damageDetail.damage,
      retaliation: retaliationDetail?.damage ?? 0,
    };
  }

  preview(
    attacker: Unit,
    target: Unit,
    attackId?: string | null,
    from = attacker.tile,
  ): CombatPreview {
    const { damage, retaliation } = this.previewDetails(attacker, target, attackId, from);
    return { damage, retaliation };
  }

  bestAttack(attacker: Unit, target: Unit, from = attacker.tile): Weapon | null {
    const score = (attack: Weapon) => {
      const { damage, retaliation } = this.preview(attacker, target, attack.id, from);
      const defeatBonus = damage >= target.hp ? 18 : 0;
      const retaliationCost = retaliation * 0.7;
      const deathPenalty = retaliation >= attacker.hp ? 40 : 0;
      return damage + defeatBonus - retaliationCost - deathPenalty;
    };
    return (
      this.attackOptions(attacker, target, from).sort(
        (first, second) => score(second) - score(first),
      )[0] || null
    );
  }

  move(id: string, tileId: string): Unit {
    const unit = this.requireActor(id);
    if (unit.tile === tileId) throw new Error("Choose a different hex to move.");
    if (!this.reachable(id).has(tileId)) throw new Error("That hex is outside this unit’s move.");
    unit.tile = tileId;
    unit.moved = true;
    const tile = this.tile(tileId)!;
    if (["village", "grove", "fort", "ruins"].includes(tile.terrain) && tile.owner !== unit.side) {
      tile.owner = unit.side;
      this.record(
        `${unit.side === "player" ? "Your" : "Enemy"} ${TYPES[unit.type].name.toLowerCase()} ${tile.terrain === "village" ? "captured a village" : `claimed ${TERRAIN[tile.terrain].name.toLowerCase()}`}.`,
      );
    }
    this.evaluateCampaign(false, unit.side === "player" ? tileId : undefined);
    return unit;
  }

  attack(id: string, targetId: string, attackId?: string): CombatPreview {
    const attacker = this.requireActor(id);
    const target = this.unit(targetId);
    if (!target || !this.targets(id).includes(target))
      throw new Error("That enemy is out of range.");
    const details = this.previewDetails(attacker, target, attackId);
    const result = { damage: details.damage, retaliation: details.retaliation };
    this.lastCombat = {
      attackerId: id,
      attackId: details.attack.id,
      sound: details.attack.sound,
    };
    attacker.moved = true;
    attacker.attacked = true;
    target.hp -= result.damage;
    if (target.hp > 0) attacker.hp -= result.retaliation;
    this.earnExperience(attacker, result.damage, target);
    this.earnExperience(target, result.retaliation, attacker);
    const sideLabel = attacker.side === "player" ? "Your" : "Enemy";
    const unitName = TYPES[attacker.type].name.toLowerCase();
    const attackName = details.attack.name.toLowerCase();
    const retaliationMessage = result.retaliation
      ? ` and took ${result.retaliation} in retaliation`
      : "";
    this.record(
      `${sideLabel} ${unitName} used ${attackName} for ${result.damage} damage${retaliationMessage}.`,
    );
    for (const defeated of this.units.filter((unit) => unit.hp <= 0)) {
      if (
        this.campaign &&
        defeated.side === "player" &&
        defeated.name &&
        CAMPAIGNS[this.campaign.id].missions.some((chapter) =>
          chapter.map.units.some(
            (incoming) => incoming.side === "player" && incoming.name === defeated.name,
          ),
        )
      ) {
        const mark = `fallen:${defeated.name}`;
        if (!this.campaign.marks.includes(mark)) this.campaign.marks.push(mark);
      }
      if (this.campaign?.id === "green-accord" && defeated.name === "Captain Dain") {
        if (!this.campaign.marks.includes("dain-fell")) this.campaign.marks.push("dain-fell");
      }
      if (endsRefugeMediation(this, defeated)) this.escalateRefuge();
      if (TYPES[defeated.type].leader && !this.campaign) {
        this.winner = defeated.side === "player" ? "enemy" : "player";
      }
      this.record(
        `${defeated.side === "player" ? "Your" : "Enemy"} ${TYPES[defeated.type].name.toLowerCase()} fell.`,
      );
    }
    this.units = this.units.filter((unit) => unit.hp > 0);
    this.evaluateCampaign();
    return result;
  }

  recruitTiles(id: string, type: UnitType): Tile[] {
    const commander = this.unit(id);
    if (
      !commander ||
      !TYPES[commander.type].leader ||
      !TYPES[type] ||
      TYPES[type].leader ||
      !this.recruitmentOptions(commander.side).includes(type as RecruitType) ||
      this.winner ||
      commander.side !== this.turn ||
      this.gold[commander.side] < TYPES[type].cost
    ) {
      return [];
    }

    const keep = this.tile(commander.tile)!;
    if (keep.terrain !== "keep" || keep.owner !== commander.side) return [];
    return this.neighbors(commander.tile).filter(
      (neighbor) => Number.isFinite(this.cost({ type }, neighbor)) && !this.at(neighbor.id),
    );
  }

  recruit(id: string, type: UnitType, tileId: string): Unit {
    const commander = this.requireActor(id);
    if (!this.recruitTiles(id, type).some((tile) => tile.id === tileId))
      throw new Error("Cannot recruit there. Check gold and available space.");
    this.gold[commander.side] -= TYPES[type].cost;
    const unit = this.addUnit(type, commander.side, tileId);
    unit.moved = true;
    unit.attacked = true;
    this.record(
      `${commander.side === "player" ? "You" : "Enemy"} recruited a ${TYPES[type].name.toLowerCase()}. It acts next turn.`,
    );
    return unit;
  }

  income(side: Side): number {
    return (
      this.tiles.filter((tile) => tile.terrain === "village" && tile.owner === side).length * 2
    );
  }

  beginTurn(side: Side): void {
    this.turn = side;
    this.lastCombat = null;
    this.gold[side] += this.income(side);
    for (const unit of this.units.filter((unit) => unit.side === side)) {
      unit.moved = false;
      unit.attacked = false;
      const tile = this.tile(unit.tile)!;
      if (["village", "grove"].includes(tile.terrain) && tile.owner === side) {
        unit.hp = Math.min(TYPES[unit.type].hp, unit.hp + 2);
      }
    }
    // Strongest adjacent healer only. Resolve from the same start-of-turn roster.
    for (const unit of this.units.filter((unit) => unit.side === side)) {
      const healing = Math.max(
        0,
        ...this.units
          .filter(
            (ally) =>
              ally.side === side &&
              ally.id !== unit.id &&
              distance(this.tile(ally.tile)!, this.tile(unit.tile)!) === 1,
          )
          .map((ally) => TYPES[ally.type].healing ?? 0),
      );
      unit.hp = Math.min(TYPES[unit.type].hp, unit.hp + healing);
    }
    if (side === "enemy") this.spawnCampaignWaves();
    this.evaluateCampaign(side === "player");
  }

  endTurn(): void {
    if (this.winner) throw new Error("The battle is over.");
    if (this.turn === "player") this.beginTurn("enemy");
    else {
      this.round++;
      this.beginTurn("player");
    }
  }

  enemyRecruit(): void {
    if (this.turn !== "enemy" || this.winner) return;
    const commander = this.units.find((unit) => unit.side === "enemy" && TYPES[unit.type].leader);
    if (!commander) return;

    const hasArcher = this.units.some(
      (unit) => unit.side === "enemy" && TYPES[unit.type].family === "archer",
    );
    const options = this.recruitmentOptions("enemy");
    const recruitType = this.campaign
      ? options[(this.round - 1) % options.length]!
      : hasArcher
        ? "swordsman"
        : "archer";
    const recruitTiles = this.recruitTiles(commander.id, recruitType);
    if (!recruitTiles.length) return;

    recruitTiles.sort((first, second) => second.col - first.col);
    this.recruit(commander.id, recruitType, recruitTiles[0]!.id);
  }

  routeDistances(unit: Unit, goals: Tile[]): Map<string, number> {
    // Long-range routing ignores occupied hexes; reachable() handles actual moves.
    // Search backward from the goals: entering the current tile is the forward
    // movement cost, rather than entering the neighbor we're considering now.
    const distanceToGoal = new Map(goals.map((tile) => [tile.id, 0]));
    const pendingTiles = [...distanceToGoal.keys()];
    while (pendingTiles.length) {
      pendingTiles.sort(
        (first, second) => distanceToGoal.get(first)! - distanceToGoal.get(second)!,
      );
      const currentTileId = pendingTiles.shift()!;
      const entryCost = this.cost(unit, this.tile(currentTileId)!);
      if (!Number.isFinite(entryCost)) continue;

      for (const neighbor of this.neighbors(currentTileId)) {
        if (!Number.isFinite(this.cost(unit, neighbor))) continue;
        const routeCost = distanceToGoal.get(currentTileId)! + entryCost;
        const knownCost = distanceToGoal.get(neighbor.id) ?? Infinity;
        if (routeCost >= knownCost) continue;

        distanceToGoal.set(neighbor.id, routeCost);
        if (!pendingTiles.includes(neighbor.id)) pendingTiles.push(neighbor.id);
      }
    }
    return distanceToGoal;
  }

  enemyAct(id: string): void {
    if (this.turn !== "enemy" || this.winner) return;
    const unit = this.unit(id);
    if (!unit || unit.side !== "enemy" || unit.attacked) return;
    const advancement = this.promotionOptions(id)[0];
    if (advancement) {
      this.advanceUnit(id, advancement);
      return;
    }

    const retreat = this.tryEnemyRetreat(unit);
    if (retreat === "acted") return;
    if (this.tryEnemyAttack(unit, retreat === "cornered")) return;
    if (TYPES[unit.type].leader && this.round < 5) return;

    const goals = this.enemyMovementGoals(unit);
    if (!goals.length) return;
    const routes =
      this.mapId === "river" || this.campaign ? this.routeDistances(unit, goals) : null;
    let destinations = [unit.tile, ...this.reachable(id).keys()];
    if (this.campaign && campaignMission(this.campaign).defendObjectives && routes) {
      // A post is held from its hex and immediate approaches. Troops outside
      // patrol range return by the nearest available route; retreat remains free.
      const approach = Math.min(...destinations.map((tile) => routes.get(tile) ?? Infinity));
      const patrolLimit = Math.max(1, approach);
      destinations = destinations.filter((tile) => (routes.get(tile) ?? Infinity) <= patrolLimit);
    }
    const scores = new Map(
      destinations.map((tile) => [tile, this.enemyPositionScore(unit, tile, goals, routes)]),
    );
    destinations.sort((first, second) => scores.get(second)! - scores.get(first)!);

    const destination = destinations[0]!;
    if (destination !== unit.tile) this.move(id, destination);
    this.tryEnemyAttack(unit);
  }

  /** Compare positions when moving; incoming danger alone does not trigger retreat. */
  private enemyDanger(unit: Unit, tileId: string): number {
    const positioned = { ...unit, tile: tileId };
    const damage = this.units
      .filter((opponent) => opponent.side === "player")
      .map((opponent) =>
        Math.max(
          0,
          ...this.attackOptions(opponent, positioned).map((weapon) =>
            this.damage(opponent, positioned, weapon.id),
          ),
        ),
      )
      .sort((a, b) => b - a);
    return (damage[0] ?? 0) + (damage[1] ?? 0);
  }

  private tryEnemyRetreat(unit: Unit): "acted" | "cornered" | "none" {
    const enemies = this.units.filter((opponent) => opponent.side === "player");
    const retaliations = this.targets(unit.id).flatMap((target) =>
      this.attackOptions(unit, target).map(
        (attack) => this.preview(unit, target, attack.id).retaliation,
      ),
    );
    // A safe weapon or a killing blow lets the troop fight, even while wounded.
    if (!enemies.length || retaliations.some((damage) => damage < unit.hp)) {
      delete unit.retreating;
      return "none";
    }
    const trappedAttack = retaliations.length > 0;
    const villages = this.tiles.filter(
      (tile) =>
        tile.terrain === "village" &&
        tile.owner === "enemy" &&
        (!this.at(tile.id) || this.at(tile.id)?.id === unit.id) &&
        this.enemyDanger(unit, tile.id) === 0,
    );
    const routes = villages.length ? this.routeDistances(unit, villages) : null;
    const recovering = unit.retreating && unit.hp < TYPES[unit.type].hp && routes?.has(unit.tile);
    if (!trappedAttack && !recovering) {
      delete unit.retreating;
      return "none";
    }
    if (recovering && villages.some((tile) => tile.id === unit.tile)) {
      unit.attacked = true;
      return "acted";
    }
    const score = (id: string): number => {
      const tile = this.tile(id)!;
      const threat = this.enemyDanger(unit, id);
      const separation = Math.min(
        ...enemies.map((opponent) => distance(tile, this.tile(opponent.tile)!)),
      );
      const healing = villages.some((village) => village.id === id) ? 20 : 0;
      return -threat * 20 + Math.min(separation, 8) * 2 + healing - (routes?.get(id) ?? 0);
    };
    const destinations = [unit.tile, ...this.reachable(unit.id).keys()];
    const scores = new Map(destinations.map((tile) => [tile, score(tile)]));
    destinations.sort((a, b) => scores.get(b)! - scores.get(a)!);
    const destination = destinations[0]!;
    if (destination !== unit.tile && scores.get(destination)! > scores.get(unit.tile)!) {
      this.move(unit.id, destination);
      unit.retreating = true;
      unit.attacked = true;
      this.record(`${unit.name ?? TYPES[unit.type].name} is retreating.`);
      return "acted";
    }
    // A failed escape is a stand, not a withdrawal; mediation observes this marker.
    delete unit.retreating;
    return trappedAttack ? "cornered" : "none";
  }

  private enemyAttackScore(
    unit: Unit,
    target: Unit,
    from = unit.tile,
    attack = this.bestAttack(unit, target, from),
  ): number {
    if (!attack) return -100;
    const { damage, retaliation } = this.preview(unit, target, attack.id, from);

    const commanderPriority = TYPES[target.type].leader ? 20 : target.role === "courier" ? 12 : 0;
    const defeatBonus = target.hp <= damage ? 12 : 0;
    const woundedTargetBonus = (TYPES[target.type].hp - target.hp) * 0.3;
    const retaliationCost = retaliation * 0.7;
    const deathPenalty = retaliation >= unit.hp ? 30 : 0;
    return (
      commanderPriority + defeatBonus + woundedTargetBonus + damage - retaliationCost - deathPenalty
    );
  }

  private tryEnemyAttack(unit: Unit, desperate = false): boolean {
    const choices = this.targets(unit.id)
      .flatMap((target) => this.attackOptions(unit, target).map((attack) => ({ target, attack })))
      .filter(
        ({ target, attack }) =>
          desperate || this.preview(unit, target, attack.id).retaliation < unit.hp,
      )
      .sort(
        (first, second) =>
          this.enemyAttackScore(unit, second.target, unit.tile, second.attack) -
          this.enemyAttackScore(unit, first.target, unit.tile, first.attack),
      );
    const choice = choices[0];
    if (!choice) return false;

    this.attack(unit.id, choice.target.id, choice.attack.id);
    return true;
  }

  private enemyMovementGoals(unit: Unit): Tile[] {
    const uncapturedVillages = this.tiles.filter(
      (tile) => tile.terrain === "village" && tile.owner !== "enemy",
    );
    let goals =
      TYPES[unit.type].family === "scout" && uncapturedVillages.length
        ? uncapturedVillages
        : this.units
            .filter((target) => target.side === "player")
            .map((target) => this.tile(target.tile)!);

    if (this.campaign) {
      const mission = campaignMission(this.campaign);
      const territory = mission.goals.flatMap((goal) =>
        goal.kind === "control"
          ? goal.tiles
          : goal.kind === "hold" || (goal.kind === "refuge" && !this.campaign?.refuge?.escalated)
            ? [goal.tile]
            : [],
      );
      const contested = territory.filter((id) => this.tile(id)?.owner !== "enemy");
      if (contested.length) goals = contested.map((id) => this.tile(id)!);
      else if (mission.defendObjectives && territory.length)
        goals = territory.map((id) => this.tile(id)!);
    }

    return goals;
  }

  private enemyPositionScore(
    unit: Unit,
    tileId: string,
    goals: Tile[],
    routes: Map<string, number> | null,
  ): number {
    const tile = this.tile(tileId)!;
    const distanceToGoal = routes
      ? (routes.get(tileId) ?? 1000)
      : Math.min(...goals.map((goal) => distance(tile, goal)));
    const attackOpportunity = Math.max(
      0,
      ...this.units
        .filter((target) => this.canAttackFrom(unit, tileId, target))
        .map((target) => 5 + this.enemyAttackScore(unit, target, tileId) / 5),
    );

    const canCaptureVillage = tile.terrain === "village" && tile.owner !== "enemy";
    const captureValue = TYPES[unit.type].family === "scout" ? 8 : 5;
    const captureBonus = canCaptureVillage ? captureValue : 0;
    const canHeal = unit.hp <= 4 && tile.terrain === "village" && tile.owner === "enemy";
    const healingBonus = canHeal ? 9 : 0;
    const hillBonus = tile.terrain === "hill" ? 0.25 : 0;
    const danger = this.enemyDanger(unit, tileId);
    const dangerPenalty = danger * 0.5;
    return (
      -distanceToGoal + attackOpportunity + captureBonus + healingBonus + hillBonus - dangerPenalty
    );
  }

  captureState(): GameState {
    const state: GameState = {
      mapRevision: this.mapRevision,
      mapId: this.mapId,
      cols: this.cols,
      rows: this.rows,
      round: this.round,
      turn: this.turn,
      gold: this.gold,
      winner: this.winner,
      nextId: this.nextId,
      log: this.log,
      tiles: this.tiles,
      units: this.units,
    };
    if (Object.hasOwn(this, "lastCombat")) state.lastCombat = this.lastCombat ?? null;
    if (this.campaign) state.campaign = this.campaign;
    return structuredClone(state);
  }

  restoreState(state: GameState): void {
    const restored = structuredClone(state);
    if (!Object.hasOwn(restored, "lastCombat")) delete this.lastCombat;
    if (!Object.hasOwn(restored, "campaign")) delete this.campaign;
    Object.assign(this, restored);
    this.indexTiles();
  }

  snapshot(): GameSnapshot {
    const snapshot: GameSnapshot = {
      mapId: this.mapId,
      mapRevision: this.mapRevision,
      round: this.round,
      turn: this.turn,
      gold: { ...this.gold },
      winner: this.winner,
      units: this.units.map((unit) => ({ ...unit })),
      villages: this.tiles
        .filter((tile) => tile.terrain === "village")
        .map((tile) => ({ id: tile.id, owner: tile.owner })),
    };
    if (this.campaign) snapshot.campaign = structuredClone(this.campaign);
    const sites = this.tiles
      .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
      .map((tile) => ({ id: tile.id, owner: tile.owner }));
    if (this.campaign || sites.length) snapshot.sites = sites;
    return snapshot;
  }

  recruitmentOptions(side: Side): RecruitType[] {
    if (!this.campaign) return ["swordsman", "spearman", "archer", "scout"];
    const mission = campaignMission(this.campaign);
    return [...(side === "player" ? mission.recruits : mission.enemyRecruits)];
  }

  static startCampaign(
    id: CampaignId,
    missionIndex = 0,
    recall: RecruitType[] = [],
    carryGold = 0,
    choices: NonNullable<CampaignState["choices"]> = [],
    veterans: CampaignVeteran[] = [],
    storyVeterans: CampaignVeteran[] = [],
    fallenCompanions: string[] = [],
  ): Game {
    const definition = CAMPAIGNS[id];
    const mission = definition?.missions[missionIndex];
    if (
      !mission ||
      !Number.isInteger(missionIndex) ||
      !Number.isSafeInteger(carryGold) ||
      carryGold < 0
    )
      throw new Error("Unknown campaign mission.");
    if (recall.length > 4 || recall.some((type) => !TYPES[type] || TYPES[type].leader))
      throw new Error("Choose up to four surviving recruits.");
    const game = new Game(mission.map.id);
    game.campaign = {
      id,
      mission: missionIndex,
      recall: [...recall],
      veterans: recall.map((type, index) =>
        structuredClone(veterans[index] ?? { type, experience: 0 }),
      ),
      storyVeterans: structuredClone(storyVeterans),
      fallenCompanions: [...fallenCompanions],
      carryGold: Math.min(8, Math.max(0, carryGold)),
      marks: [],
      dialogue: mission.intro.map((line) => ({ ...line })),
      dialogueIndex: 0,
      outcome: null,
      completed: false,
      ...(choices.length ? { choices: [...choices] } : {}),
      ...(mission.refuge
        ? { refuge: { heldTurns: 0, lastCountedRound: 1, escalated: false } }
        : {}),
      ...(mission.goals.some((goal) => goal.kind === "vigil")
        ? { vigil: { heldTurns: 0, lastCountedRound: 1 } }
        : {}),
      holds: Object.fromEntries(
        mission.goals
          .filter((goal) => goal.kind === "hold")
          .map((goal) => [goal.tile, { heldTurns: 0, lastCountedRound: 1 }]),
      ),
    };
    const effectiveMission = campaignMission(game.campaign);
    game.units = game.units.filter(
      (unit) => unit.side !== "player" || !unit.name || !fallenCompanions.includes(unit.name),
    );
    game.campaign.dialogue = effectiveMission.intro.map((line) => ({ ...line }));
    if (
      (choices.includes("refuge-war") || choices.includes("green-dain-dead")) &&
      mission.withoutDain
    ) {
      game.units = game.units.filter((unit) => unit.name !== "Captain Dain");
      for (const incoming of mission.withoutDain.reinforcements) {
        const unit = game.addUnit(incoming.type, incoming.side, `${incoming.col},${incoming.row}`);
        if (incoming.name) unit.name = incoming.name;
        if (incoming.role) unit.role = incoming.role;
      }
      game.campaign.dialogue = effectiveMission.intro.map((line) => ({ ...line }));
    }
    game.gold = { player: mission.gold + game.campaign.carryGold, enemy: mission.enemyGold };
    for (const veteran of storyVeterans) {
      const returning = game.units.find(
        (unit) =>
          unit.side === "player" &&
          (veteran.role === "leader"
            ? unit.role === "leader"
            : veteran.name && unit.name === veteran.name),
      );
      if (!returning || TYPES[returning.type].family !== TYPES[veteran.type].family) continue;
      returning.type = veteran.type;
      returning.experience = veteran.experience;
      returning.hp = TYPES[veteran.type].hp;
    }
    const leader = game.units.find((unit) => unit.side === "player" && unit.role === "leader")!;
    const slots = game.tiles
      .filter((tile) => !game.at(tile.id) && distance(tile, game.tile(leader.tile)!) <= 3)
      .sort((a, b) => distance(a, game.tile(leader.tile)!) - distance(b, game.tile(leader.tile)!));
    for (const [recallIndex, type] of recall.entries()) {
      const index = slots.findIndex((tile) => Number.isFinite(game.cost({ type }, tile)));
      const tile = index < 0 ? undefined : slots.splice(index, 1)[0];
      if (!tile) throw new Error("No room for the returning company.");
      const returning = game.addUnit(type, "player", tile.id);
      const veteran = veterans[recallIndex];
      if (veteran?.type === type) {
        returning.experience = veteran.experience;
        if (veteran.name) returning.name = veteran.name;
      }
    }
    for (const unit of game.units) {
      const tile = game.tile(unit.tile)!;
      if (["village", "grove", "ruins", "fort"].includes(tile.terrain)) tile.owner = unit.side;
    }
    game.log = [];
    game.record(campaignMission(game.campaign).briefing);
    return game;
  }

  nextCampaignMission(recallIds: string[]): Game {
    if (!this.campaign || this.winner !== "player" || this.campaign.completed)
      throw new Error("Win this mission before continuing.");
    if (new Set(recallIds).size !== recallIds.length || recallIds.length > 4)
      throw new Error("Choose up to four different survivors.");
    const roster = recallIds.map((id) => {
      const unit = this.unit(id);
      const isCompanion =
        unit?.name && campaignMission(this.campaign!).requiredCompanions?.includes(unit.name);
      if (
        !unit ||
        unit.side !== "player" ||
        TYPES[unit.type].leader ||
        unit.role === "courier" ||
        isCompanion ||
        returnsWithStory(this.campaign!, unit)
      )
        throw new Error("Only surviving recruits can return.");
      return {
        type: unit.type,
        experience: unit.experience ?? 0,
        ...(unit.name ? { name: unit.name } : {}),
      } satisfies CampaignVeteran;
    });
    const choices = [...(this.campaign.choices ?? [])];
    if (this.campaign.marks.includes("dain-fell") && !choices.includes("green-dain-dead"))
      choices.push("green-dain-dead");
    return Game.startCampaign(
      this.campaign.id,
      this.campaign.mission + 1,
      roster.map((veteran) => veteran.type as RecruitType),
      this.gold.player,
      choices,
      roster,
      this.units
        .filter(
          (unit) =>
            unit.side === "player" &&
            (unit.role === "leader" || returnsWithStory(this.campaign!, unit)),
        )
        .map((unit) => ({
          type: unit.type,
          experience: unit.experience ?? 0,
          ...(unit.name ? { name: unit.name } : {}),
          ...(unit.role ? { role: unit.role } : {}),
        })),
      [
        ...new Set([
          ...(this.campaign.fallenCompanions ?? []),
          ...this.campaign.marks
            .filter((mark) => mark.startsWith("fallen:"))
            .map((mark) => mark.slice(7)),
          ...campaignMission(this.campaign)
            .map.units.filter(
              (incoming) =>
                incoming.side === "player" &&
                incoming.name &&
                incoming.role !== "leader" &&
                !this.units.some((unit) => unit.side === "player" && unit.name === incoming.name),
            )
            .map((incoming) => incoming.name!),
        ]),
      ],
    );
  }

  retryCampaignMission(): Game {
    if (!this.campaign) throw new Error("No campaign is active.");
    return Game.startCampaign(
      this.campaign.id,
      this.campaign.mission,
      this.campaign.recall,
      this.campaign.carryGold,
      campaignMission(this.campaign).refuge ? [] : (this.campaign.choices ?? []),
      this.campaign.veterans ?? [],
      this.campaign.storyVeterans ?? [],
      this.campaign.fallenCompanions ?? [],
    );
  }

  advanceDialogue(skip = false): void {
    if (!this.campaign) return;
    this.campaign.dialogueIndex = skip
      ? this.campaign.dialogue.length
      : Math.min(this.campaign.dialogue.length, this.campaign.dialogueIndex + 1);
  }

  previousDialogue(): void {
    const campaign = this.campaign;
    if (!campaign || campaign.dialogueIndex >= campaign.dialogue.length) return;
    campaign.dialogueIndex = Math.max(0, campaign.dialogueIndex - 1);
  }

  private spawnCampaignWaves(): void {
    if (!this.campaign || this.winner) return;
    const campaign = this.campaign;
    campaignMission(campaign).waves.forEach((wave, index) => {
      const marker = `wave:${index}`;
      if (this.round < wave.round || campaign.marks.includes(marker)) return;
      campaign.marks.push(marker);
      if (wave.requiresUnsealed && campaign.marks.includes(`sealed:${wave.requiresUnsealed}`))
        return;
      let arrived = 0;
      for (const incoming of wave.units) {
        const origin = this.tile(`${incoming.col},${incoming.row}`)!;
        const destination = this.tiles
          .filter(
            (tile) =>
              Number.isFinite(this.cost({ type: incoming.type }, tile)) &&
              !this.at(tile.id) &&
              distance(tile, origin) <= 2,
          )
          .sort((a, b) => distance(a, origin) - distance(b, origin))[0];
        if (destination) {
          this.addUnit(incoming.type, incoming.side, destination.id);
          arrived++;
        }
      }
      if (arrived) this.record(wave.label);
    });
  }

  private evaluateCampaign(atTurnStart = false, enteredTile?: string): void {
    if (!this.campaign || this.winner) return;
    const campaign = this.campaign;
    const mission = campaignMission(campaign);
    const leader = this.units.find((unit) => unit.side === "player" && unit.role === "leader");
    const needsCourier = mission.goals.some((goal) => goal.kind === "escort");
    const courier = this.units.find((unit) => unit.side === "player" && unit.role === "courier");
    const missingCompanion = mission.requiredCompanions?.find(
      (name) => !this.units.some((unit) => unit.side === "player" && unit.name === name),
    );
    if (!leader || missingCompanion || (needsCourier && !courier)) {
      this.winner = "enemy";
      campaign.outcome = !leader
        ? "Your leader fell. The company cannot continue without them."
        : missingCompanion
          ? `${missingCompanion} fell. The company cannot continue without them.`
          : "The courier fell before reaching safety.";
      campaign.dialogue.push({
        speaker: "The company",
        portrait: CAMPAIGNS[campaign.id].leader,
        text: campaign.outcome,
      });
      this.record(campaign.outcome);
      return;
    }
    const vigilGoal = mission.goals.find((goal) => goal.kind === "vigil");
    if (vigilGoal?.kind === "vigil") {
      campaign.vigil ??= { heldTurns: 0, lastCountedRound: this.round };
      const present = this.units.some(
        (unit) =>
          unit.side === "player" &&
          unit.name === vigilGoal.companion &&
          unit.tile === vigilGoal.tile,
      );
      if (!present || this.tile(vigilGoal.tile)?.owner !== "player") {
        campaign.vigil.heldTurns = 0;
        campaign.vigil.lastCountedRound = this.round;
      } else if (atTurnStart && this.round > campaign.vigil.lastCountedRound) {
        campaign.vigil.heldTurns = Math.min(vigilGoal.turns, campaign.vigil.heldTurns + 1);
        campaign.vigil.lastCountedRound = this.round;
      }
    }
    if (mission.refuge) {
      campaign.refuge ??= { heldTurns: 0, lastCountedRound: this.round, escalated: false };
      if (!this.units.some((unit) => unit.side === "enemy" && unit.role === "leader"))
        this.escalateRefuge();
      const refugeGoal = mission.goals.find((goal) => goal.kind === "refuge");
      if (refugeGoal?.kind === "refuge" && !campaign.refuge.escalated) {
        if (this.tile(refugeGoal.tile)?.owner !== "player") campaign.refuge.heldTurns = 0;
        else if (atTurnStart && this.round > campaign.refuge.lastCountedRound) {
          campaign.refuge.heldTurns++;
        }
        if (atTurnStart) campaign.refuge.lastCountedRound = this.round;
      }
    }
    for (const goal of mission.goals) {
      if (goal.kind === "hold") {
        campaign.holds ??= {};
        const progress = (campaign.holds[goal.tile] ??= {
          heldTurns: 0,
          lastCountedRound: this.round,
        });
        if (this.tile(goal.tile)?.owner !== "player") {
          progress.heldTurns = 0;
          progress.lastCountedRound = this.round;
        } else if (atTurnStart && this.round > progress.lastCountedRound) {
          progress.heldTurns = Math.min(goal.turns, progress.heldTurns + 1);
          progress.lastCountedRound = this.round;
        }
      }
      if (goal.kind !== "seal") continue;
      for (const id of goal.tiles) {
        const occupant = this.at(id);
        if (
          !campaign.marks.includes(`sealed:${id}`) &&
          occupant?.side === "player" &&
          (!goal.unitType ||
            occupant.type === goal.unitType ||
            TYPES[occupant.type].family === TYPES[goal.unitType].family)
        ) {
          campaign.marks.push(`sealed:${id}`);
          this.record("A binding stone was sealed. It will no longer raise the dead.");
        }
      }
    }
    if (this.turn === "player") {
      for (const event of mission.events) {
        const marker = `event:${event.id}`;
        const triggered =
          event.trigger.kind === "visit"
            ? enteredTile === event.trigger.tile
            : this.round >= event.trigger.round;
        if (!triggered || campaign.marks.includes(marker)) continue;
        campaign.marks.push(marker);
        this.gold.player += event.gold ?? 0;
        campaign.dialogue.push(
          ...structuredClone(
            campaign.refuge?.escalated && event.warDialogue ? event.warDialogue : event.dialogue,
          ),
        );
        this.record(event.label ?? "The company found a new story.");
      }
    }
    if (mission.goals.every((goal) => goalProgress(this, goal).done)) {
      this.winner = "player";
      campaign.outcome = "Mission complete. The company can rest and regroup.";
      const refugeChoice = mission.refuge
        ? campaign.refuge?.escalated
          ? "refuge-war"
          : "refuge-peace"
        : null;
      if (refugeChoice) campaign.choices = [...(campaign.choices ?? []), refugeChoice];
      campaign.dialogue.push(
        ...structuredClone(
          refugeChoice === "refuge-war" ? mission.refuge!.warVictory : mission.victory,
        ),
      );
      campaign.completed = campaign.mission === CAMPAIGNS[campaign.id].missions.length - 1;
      if (campaign.completed) {
        const dainSurvived = this.units.some(
          (unit) => unit.side === "player" && unit.name === "Captain Dain",
        );
        campaign.dialogue.push(
          ...structuredClone(
            CAMPAIGNS[campaign.id].ending.filter(
              (line) =>
                (dainSurvived &&
                  !campaign.choices?.includes("refuge-war") &&
                  !campaign.choices?.includes("green-dain-dead") &&
                  !campaign.marks.includes("dain-fell")) ||
                (line.speaker !== "Dain" && line.speaker !== "Captain Dain"),
            ),
          ),
        );
      }
      this.record(
        campaign.completed ? `${CAMPAIGNS[campaign.id].name} is complete.` : "Mission complete.",
      );
    }
  }

  private escalateRefuge(): void {
    if (!this.campaign || !campaignMission(this.campaign).refuge) return;
    this.campaign.refuge ??= { heldTurns: 0, lastCountedRound: this.round, escalated: false };
    if (this.campaign.refuge.escalated) return;
    this.campaign.refuge.escalated = true;
    this.campaign.dialogue.push(
      ...structuredClone(campaignMission(this.campaign).refuge!.escalation),
    );
    this.record("The chance of mediation is lost. Defeat the remaining patrol.");
  }
}
