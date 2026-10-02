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

export const DAMAGE_TYPES = {
  blade: "Blade",
  pierce: "Pierce",
  impact: "Impact",
};
const weapon = (
  id: string,
  name: string,
  damage: number,
  damageType: DamageType,
  category: AttackCategory,
  range: number,
  sound = "attack-melee",
): Weapon => ({ id, name, damage, damageType, category, range, sound });
export const TYPES: Record<UnitType, UnitDefinition> = {
  commander: {
    name: "Commander",
    hp: 16,
    move: 3,
    damage: 4,
    range: 1,
    cost: 0,
    attacks: [
      weapon("sword", "Sword", 4, "blade", "melee", 1),
      weapon("mace", "Mace", 3, "impact", "melee", 1),
    ],
    resistances: { blade: 25, pierce: 0, impact: 0 },
    ability: "Recruit",
    description: "Recruit while on your keep. New recruits act next turn.",
  },
  soldier: {
    name: "Soldier",
    hp: 12,
    move: 3,
    damage: 4,
    range: 1,
    cost: 8,
    attacks: [
      weapon("sword", "Sword", 4, "blade", "melee", 1),
      weapon("mace", "Mace", 3, "impact", "melee", 1),
    ],
    resistances: { blade: 25, pierce: 25, impact: -25 },
    ability: "Armor",
    description: "Resists blades and arrows; vulnerable to impact weapons.",
  },
  archer: {
    name: "Archer",
    hp: 8,
    move: 3,
    damage: 3,
    range: 2,
    cost: 8,
    attacks: [
      weapon("bow", "Bow", 3, "pierce", "ranged", 2, "attack-bow"),
      weapon("knife", "Knife", 2, "blade", "melee", 1),
    ],
    resistances: { blade: 0, pierce: 0, impact: 0 },
    ability: "Bow",
    description: "Bow reaches 2 hexes. A knife provides a weak melee reply.",
  },
  scout: {
    name: "Scout",
    hp: 8,
    move: 5,
    damage: 2,
    range: 2,
    cost: 5,
    attacks: [
      weapon("knife", "Knife", 2, "blade", "melee", 1),
      weapon("sling", "Sling", 1, "impact", "ranged", 2, "unit-archer"),
    ],
    resistances: { blade: 0, pierce: 0, impact: 0 },
    ability: "Pathfinder",
    description: "Forests and hills cost only 1 movement.",
  },
};
export const TERRAIN: Record<TerrainType, TerrainDefinition> = {
  plains: {
    name: "Plains",
    cost: 1,
    description: "Normal movement. No protection.",
  },
  forest: {
    name: "Forest",
    cost: 2,
    description: "Costs 2 movement; ranged damage reduced by 1.",
  },
  hill: {
    name: "Hills",
    cost: 2,
    description: "Costs 2 movement; all damage reduced by 1.",
  },
  water: { name: "Water", cost: Infinity, description: "Impassable." },
  bridge: {
    name: "Bridge",
    cost: 1,
    description: "A crossing over water. Normal movement; no protection.",
  },
  ford: {
    name: "Shallow water",
    cost: 2,
    description:
      "Costs 3 movement for soldiers and commanders, 2 for archers and scouts. Units here take 1 extra damage.",
  },
  village: {
    name: "Village",
    cost: 1,
    description: "+2 gold per turn. Heals a friendly occupant by 2 HP.",
  },
  keep: {
    name: "Keep",
    cost: 1,
    description: "Your commander can recruit into adjacent empty hexes.",
  },
};
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

  constructor(mapId: string = "mirror", mapRevision = MAPS[mapId as MapId]?.revision ?? 1) {
    if (!Object.hasOwn(MAPS, mapId)) throw new Error("Unknown battlefield.");
    if (![1, MAPS[mapId as MapId].revision ?? 1].includes(mapRevision))
      throw new Error("Unknown battlefield revision.");
    const layout = mapId === "river" && mapRevision === 1 ? LEGACY_RIVER : MAPS[mapId as MapId];
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
    for (const { col, row, terrain, owner = null } of layout.terrain)
      Object.assign(this.tile(`${col},${row}`)!, { terrain, owner });
    for (const { type, side, col, row } of layout.units) this.addUnit(type, side, `${col},${row}`);
    this.record("Capture villages for gold. Defeat the enemy commander.");
  }

  tile(id: string | null | undefined): Tile | undefined {
    return this.tiles.find((tile) => tile.id === id);
  }

  unit(id: string | null | undefined): Unit | undefined {
    return this.units.find((unit) => unit.id === id);
  }

  at(tileId: string | null | undefined): Unit | undefined {
    return this.units.find((unit) => unit.tile === tileId);
  }

  neighbors(id: string): Tile[] {
    const tile = this.tile(id)!;
    return this.tiles.filter((neighbor) => distance(tile, neighbor) === 1);
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

  cost(unit: Pick<Unit, "type">, tile: Pick<Tile, "terrain">): number {
    if (tile.terrain === "ford") {
      const heavyUnit = ["soldier", "commander"].includes(unit.type);
      return heavyUnit ? 3 : 2;
    }
    const pathfinderTerrain = ["forest", "hill"].includes(tile.terrain);
    if (pathfinderTerrain && unit.type === "scout") return 1;
    return TERRAIN[tile.terrain].cost;
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
            !["forest", "hill"].includes(tile.terrain),
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
    const resistance = TYPES[defender.type].resistances[attack.damageType] || 0;
    const resisted = Math.round((attack.damage * (100 - resistance)) / 100);
    const hasTerrainProtection =
      terrain === "hill" || (terrain === "forest" && attack.category === "ranged");
    const protection = hasTerrainProtection ? 1 : 0;
    const exposure = terrain === "ford" ? 1 : 0;
    const damage = Math.max(1, resisted - protection + exposure);
    return {
      base: attack.damage,
      damageType: attack.damageType,
      resistance,
      resisted,
      protection,
      exposure,
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
      retaliation: retaliationDetail?.damage || 0,
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
    if (!this.reachable(id).has(tileId)) throw new Error("That hex is outside this unit’s move.");
    unit.tile = tileId;
    unit.moved = true;
    const tile = this.tile(tileId)!;
    if (tile.terrain === "village" && tile.owner !== unit.side) {
      tile.owner = unit.side;
      this.record(
        `${unit.side === "player" ? "Your" : "Enemy"} ${TYPES[unit.type].name.toLowerCase()} captured a village.`,
      );
    }
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
      if (defeated.type === "commander") {
        this.winner = defeated.side === "player" ? "enemy" : "player";
      }
      this.record(
        `${defeated.side === "player" ? "Your" : "Enemy"} ${TYPES[defeated.type].name.toLowerCase()} fell.`,
      );
    }
    this.units = this.units.filter((unit) => unit.hp > 0);
    return result;
  }

  recruitTiles(id: string, type: UnitType): Tile[] {
    const commander = this.unit(id);
    if (
      !commander ||
      commander.type !== "commander" ||
      !TYPES[type] ||
      type === "commander" ||
      this.winner ||
      commander.side !== this.turn ||
      this.gold[commander.side] < TYPES[type].cost
    ) {
      return [];
    }

    const keep = this.tile(commander.tile)!;
    if (keep.terrain !== "keep" || keep.owner !== commander.side) return [];
    return this.neighbors(commander.tile).filter(
      (neighbor) => neighbor.terrain !== "water" && !this.at(neighbor.id),
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
      if (tile.terrain === "village" && tile.owner === side) {
        unit.hp = Math.min(TYPES[unit.type].hp, unit.hp + 2);
      }
    }
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
    const commander = this.units.find((unit) => unit.side === "enemy" && unit.type === "commander");
    if (!commander) return;

    const hasArcher = this.units.some((unit) => unit.side === "enemy" && unit.type === "archer");
    const recruitType = hasArcher ? "soldier" : "archer";
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

    if (this.tryEnemyAttack(unit)) return;
    if (unit.type === "commander" && this.round < 5) return;

    const goals = this.enemyMovementGoals(unit);
    if (!goals.length) return;
    const routes = this.mapId === "river" ? this.routeDistances(unit, goals) : null;
    const destinations = [unit.tile, ...this.reachable(id).keys()];
    destinations.sort(
      (first, second) =>
        this.enemyPositionScore(unit, second, goals, routes) -
        this.enemyPositionScore(unit, first, goals, routes),
    );

    const destination = destinations[0]!;
    if (destination !== unit.tile) this.move(id, destination);
    this.tryEnemyAttack(unit);
  }

  private enemyAttackScore(unit: Unit, target: Unit, from = unit.tile): number {
    const attack = this.bestAttack(unit, target, from);
    if (!attack) return -100;
    const { damage, retaliation } = this.preview(unit, target, attack.id, from);

    const commanderPriority = target.type === "commander" ? 20 : 0;
    const defeatBonus = target.hp <= damage ? 12 : 0;
    const woundedTargetBonus = (TYPES[target.type].hp - target.hp) * 0.3;
    const retaliationCost = retaliation * 0.7;
    const deathPenalty = retaliation >= unit.hp ? 30 : 0;
    return (
      commanderPriority + defeatBonus + woundedTargetBonus + damage - retaliationCost - deathPenalty
    );
  }

  private tryEnemyAttack(unit: Unit): boolean {
    const targets = this.targets(unit.id).sort(
      (first, second) => this.enemyAttackScore(unit, second) - this.enemyAttackScore(unit, first),
    );
    const target = targets[0];
    if (!target) return false;

    const attack = this.bestAttack(unit, target)!;
    const preview = this.preview(unit, target, attack.id);
    if (preview.retaliation >= unit.hp) return false;

    this.attack(unit.id, target.id, attack.id);
    return true;
  }

  private enemyMovementGoals(unit: Unit): Tile[] {
    const uncapturedVillages = this.tiles.filter(
      (tile) => tile.terrain === "village" && tile.owner !== "enemy",
    );
    let goals =
      unit.type === "scout" && uncapturedVillages.length
        ? uncapturedVillages
        : this.units
            .filter((target) => target.side === "player")
            .map((target) => this.tile(target.tile)!);

    if (unit.hp <= 4) {
      const healingVillages = this.tiles.filter(
        (tile) => tile.terrain === "village" && tile.owner === "enemy",
      );
      if (healingVillages.length) goals = healingVillages;
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
    const captureValue = unit.type === "scout" ? 8 : 5;
    const captureBonus = canCaptureVillage ? captureValue : 0;
    const canHeal = unit.hp <= 4 && tile.terrain === "village" && tile.owner === "enemy";
    const healingBonus = canHeal ? 9 : 0;
    const hillBonus = tile.terrain === "hill" ? 0.25 : 0;
    return -distanceToGoal + attackOpportunity + captureBonus + healingBonus + hillBonus;
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
    return structuredClone(state);
  }

  restoreState(state: GameState): void {
    const restored = structuredClone(state);
    if (!Object.hasOwn(restored, "lastCombat")) delete this.lastCombat;
    Object.assign(this, restored);
  }

  snapshot(): GameSnapshot {
    return {
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
  }
}
