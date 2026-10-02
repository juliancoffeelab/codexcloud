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
  const a = axial(first);
  const b = axial(second);
  return (
    (Math.abs(a.q - b.q) +
      Math.abs(a.r - b.r) +
      Math.abs(a.q + a.r - b.q - b.r)) /
    2
  );
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
  constructor(
    mapId: string = "mirror",
    mapRevision = MAPS[mapId as MapId]?.revision ?? 1,
  ) {
    if (!Object.hasOwn(MAPS, mapId)) throw new Error("Unknown battlefield.");
    if (![1, MAPS[mapId as MapId].revision ?? 1].includes(mapRevision))
      throw new Error("Unknown battlefield revision.");
    const layout =
      mapId === "river" && mapRevision === 1
        ? LEGACY_RIVER
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
    for (const { col, row, terrain, owner = null } of layout.terrain)
      Object.assign(this.tile(`${col},${row}`)!, { terrain, owner });
    for (const { type, side, col, row } of layout.units)
      this.addUnit(type, side, `${col},${row}`);
    this.record("Capture villages for gold. Defeat the enemy commander.");
  }
  tile(id: string | null | undefined): Tile | undefined {
    return this.tiles.find((t) => t.id === id);
  }
  unit(id: string | null | undefined): Unit | undefined {
    return this.units.find((u) => u.id === id);
  }
  at(tileId: string | null | undefined): Unit | undefined {
    return this.units.find((u) => u.tile === tileId);
  }
  neighbors(id: string): Tile[] {
    const tile = this.tile(id)!;
    return this.tiles.filter((t) => distance(tile, t) === 1);
  }
  record(message: string): void {
    this.log.unshift(message);
    this.log = this.log.slice(0, 8);
  }
  addUnit(type: UnitType, side: Side, tile: string): Unit {
    const u: Unit = {
      id: `u${this.nextId++}`,
      type,
      side,
      tile,
      hp: TYPES[type].hp,
      moved: false,
      attacked: false,
    };
    this.units.push(u);
    return u;
  }
  requireActor(id: string): Unit {
    const u = this.unit(id);
    if (this.winner || !u || u.side !== this.turn)
      throw new Error("This unit cannot act now.");
    return u;
  }
  cost(unit: Pick<Unit, "type">, tile: Pick<Tile, "terrain">): number {
    if (tile.terrain === "ford")
      return ["soldier", "commander"].includes(unit.type) ? 3 : 2;
    return ["forest", "hill"].includes(tile.terrain) && unit.type === "scout"
      ? 1
      : TERRAIN[tile.terrain].cost;
  }
  reachable(id: string | null | undefined): Map<string, number> {
    const unit = this.unit(id);
    if (!unit || unit.moved || unit.attacked || this.winner) return new Map();
    const costs = new Map([[unit.tile, 0]]),
      open = [unit.tile];
    while (open.length) {
      open.sort((a, b) => costs.get(a)! - costs.get(b)!);
      const current = open.shift()!;
      for (const tile of this.neighbors(current)) {
        if (this.at(tile.id)) continue;
        const cost = costs.get(current)! + this.cost(unit, tile);
        if (
          cost > TYPES[unit.type].move ||
          cost >= (costs.get(tile.id) ?? Infinity)
        )
          continue;
        costs.set(tile.id, cost);
        if (!open.includes(tile.id)) open.push(tile.id);
      }
    }
    costs.delete(unit.tile);
    return costs;
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
    const d = distance(this.tile(from)!, this.tile(target.tile)!);
    return attacks.some((attack) => {
      if (d < 1 || d > attack.range) return false;
      if (attack.category === "ranged" && d === 2) {
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
    return this.units.filter((target) =>
      this.canAttackFrom(unit, unit.tile, target),
    );
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
    const protection =
      terrain === "hill" ||
      (terrain === "forest" && attack.category === "ranged")
        ? 1
        : 0;
    const exposure = terrain === "ford" ? 1 : 0;
    return {
      base: attack.damage,
      damageType: attack.damageType,
      resistance,
      resisted,
      protection,
      exposure,
      damage: Math.max(1, resisted - protection + exposure),
    };
  }
  damage(attacker: Unit, defender: Unit, attackId?: string): number {
    return this.damageBreakdown(attacker, defender, attackId).damage;
  }
  retaliationProfile(
    attacker: Unit,
    target: Unit,
    attack: Weapon,
  ): Weapon | null {
    return (
      this.attackOptions(target, attacker)
        .filter((response) => response.category === attack.category)
        .sort(
          (a, b) =>
            this.damage(target, attacker, b.id) -
            this.damage(target, attacker, a.id),
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
    const positioned = { ...attacker, tile: from };
    const damageDetail = this.damageBreakdown(positioned, target, attack.id);
    const response =
      target.hp > damageDetail.damage
        ? this.retaliationProfile(positioned, target, attack)
        : null;
    const retaliationDetail = response
      ? this.damageBreakdown(target, positioned, response.id)
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
    const { damage, retaliation } = this.previewDetails(
      attacker,
      target,
      attackId,
      from,
    );
    return { damage, retaliation };
  }
  bestAttack(
    attacker: Unit,
    target: Unit,
    from = attacker.tile,
  ): Weapon | null {
    const score = (attack: Weapon) => {
      const { damage, retaliation } = this.preview(
        attacker,
        target,
        attack.id,
        from,
      );
      return (
        damage +
        (damage >= target.hp ? 18 : 0) -
        retaliation * 0.7 -
        (retaliation >= attacker.hp ? 40 : 0)
      );
    };
    return (
      this.attackOptions(attacker, target, from).sort(
        (a, b) => score(b) - score(a),
      )[0] || null
    );
  }
  move(id: string, tileId: string): Unit {
    const unit = this.requireActor(id);
    if (!this.reachable(id).has(tileId))
      throw new Error("That hex is outside this unit’s move.");
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
    const attacker = this.requireActor(id),
      target = this.unit(targetId);
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
    this.record(
      `${attacker.side === "player" ? "Your" : "Enemy"} ${TYPES[attacker.type].name.toLowerCase()} used ${details.attack.name.toLowerCase()} for ${result.damage} damage${result.retaliation ? ` and took ${result.retaliation} in retaliation` : ""}.`,
    );
    for (const u of this.units.filter((u) => u.hp <= 0)) {
      if (u.type === "commander")
        this.winner = u.side === "player" ? "enemy" : "player";
      this.record(
        `${u.side === "player" ? "Your" : "Enemy"} ${TYPES[u.type].name.toLowerCase()} fell.`,
      );
    }
    this.units = this.units.filter((u) => u.hp > 0);
    return result;
  }
  recruitTiles(id: string, type: UnitType): Tile[] {
    const u = this.unit(id);
    if (
      !u ||
      u.type !== "commander" ||
      !TYPES[type] ||
      type === "commander" ||
      this.winner ||
      u.side !== this.turn ||
      this.gold[u.side] < TYPES[type].cost
    )
      return [];
    const tile = this.tile(u.tile)!;
    if (tile.terrain !== "keep" || tile.owner !== u.side) return [];
    return this.neighbors(u.tile).filter(
      (t) => t.terrain !== "water" && !this.at(t.id),
    );
  }
  recruit(id: string, type: UnitType, tileId: string): Unit {
    const commander = this.requireActor(id);
    if (!this.recruitTiles(id, type).some((t) => t.id === tileId))
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
      this.tiles.filter((t) => t.terrain === "village" && t.owner === side)
        .length * 2
    );
  }
  beginTurn(side: Side): void {
    this.turn = side;
    this.lastCombat = null;
    this.gold[side] += this.income(side);
    for (const u of this.units.filter((u) => u.side === side)) {
      u.moved = false;
      u.attacked = false;
      const tile = this.tile(u.tile)!;
      if (tile.terrain === "village" && tile.owner === side)
        u.hp = Math.min(TYPES[u.type].hp, u.hp + 2);
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
    const commander = this.units.find(
      (u) => u.side === "enemy" && u.type === "commander",
    );
    if (!commander) return;
    const type = this.units.some(
      (u) => u.side === "enemy" && u.type === "archer",
    )
      ? "soldier"
      : "archer";
    let options = this.recruitTiles(commander.id, type);
    if (options.length)
      this.recruit(
        commander.id,
        type,
        options.sort((a, b) => b.col - a.col)[0]!.id,
      );
  }
  routeDistances(unit: Unit, goals: Tile[]): Map<string, number> {
    // Long-range routing ignores occupied hexes; reachable() handles actual moves.
    // Reverse edges use the destination's cost so forests and hills count correctly.
    const costs = new Map(goals.map((tile) => [tile.id, 0]));
    const open = [...costs.keys()];
    while (open.length) {
      open.sort((a, b) => costs.get(a)! - costs.get(b)!);
      const current = open.shift()!;
      const step = this.cost(unit, this.tile(current)!);
      if (!Number.isFinite(step)) continue;
      for (const tile of this.neighbors(current)) {
        if (!Number.isFinite(this.cost(unit, tile))) continue;
        const cost = costs.get(current)! + step;
        if (cost >= (costs.get(tile.id) ?? Infinity)) continue;
        costs.set(tile.id, cost);
        if (!open.includes(tile.id)) open.push(tile.id);
      }
    }
    return costs;
  }
  enemyAct(id: string): void {
    if (this.turn !== "enemy" || this.winner) return;
    const unit = this.unit(id);
    if (!unit || unit.side !== "enemy" || unit.attacked) return;
    const rank = (target: Unit, from = unit.tile) => {
      const weapon = this.bestAttack(unit, target, from);
      if (!weapon) return -100;
      const { damage, retaliation } = this.preview(
        unit,
        target,
        weapon.id,
        from,
      );
      return (
        (target.type === "commander" ? 20 : 0) +
        (target.hp <= damage ? 12 : 0) +
        (TYPES[target.type].hp - target.hp) * 0.3 +
        damage -
        retaliation * 0.7 -
        (retaliation >= unit.hp ? 30 : 0)
      );
    };
    const attack = () => {
      const targets = this.targets(id).sort((a, b) => rank(b) - rank(a));
      if (targets.length) {
        const weapon = this.bestAttack(unit, targets[0]!);
        if (this.preview(unit, targets[0]!, weapon!.id).retaliation >= unit.hp)
          return false;
        this.attack(id, targets[0]!.id, weapon!.id);
        return true;
      }
      return false;
    };
    if (attack()) return;
    if (unit.type === "commander" && this.round < 5) return;
    const villages = this.tiles.filter(
      (t) => t.terrain === "village" && t.owner !== "enemy",
    );
    let goals =
      unit.type === "scout" && villages.length
        ? villages
        : this.units
            .filter((u) => u.side === "player")
            .map((u) => this.tile(u.tile)!);
    if (unit.hp <= 4) {
      const healing = this.tiles.filter(
        (t) => t.terrain === "village" && t.owner === "enemy",
      );
      if (healing.length) goals = healing;
    }
    if (!goals.length) return;
    const routes =
      this.mapId === "river" ? this.routeDistances(unit, goals) : null;
    const options = [unit.tile, ...this.reachable(id).keys()];
    const utility = (tileId: string) => {
      const tile = this.tile(tileId)!,
        dist = routes
          ? (routes.get(tileId) ?? 1000)
          : Math.min(...goals.map((g) => distance(tile, g)));
      const attackScore = Math.max(
        0,
        ...this.units
          .filter((t) => this.canAttackFrom(unit, tileId, t))
          .map((t) => 5 + rank(t, tileId) / 5),
      );
      const capture =
        tile.terrain === "village" && tile.owner !== "enemy"
          ? unit.type === "scout"
            ? 8
            : 5
          : 0;
      const heal =
        unit.hp <= 4 && tile.terrain === "village" && tile.owner === "enemy"
          ? 9
          : 0;
      return (
        -dist +
        attackScore +
        capture +
        heal +
        (tile.terrain === "hill" ? 0.25 : 0)
      );
    };
    options.sort((a, b) => utility(b) - utility(a));
    if (options[0] !== unit.tile) this.move(id, options[0]!);
    attack();
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
    if (Object.hasOwn(this, "lastCombat"))
      state.lastCombat = this.lastCombat ?? null;
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
      units: this.units.map((u) => ({ ...u })),
      villages: this.tiles
        .filter((t) => t.terrain === "village")
        .map((t) => ({ id: t.id, owner: t.owner })),
    };
  }
}
