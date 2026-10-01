export const TYPES = {
  commander: {
    name: "Commander",
    hp: 16,
    move: 3,
    damage: 4,
    range: 1,
    cost: 0,
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
    ability: "Shield",
    description: "Takes 1 less damage from ranged attacks.",
  },
  archer: {
    name: "Archer",
    hp: 8,
    move: 3,
    damage: 3,
    range: 2,
    cost: 10,
    ability: "Bow",
    description: "Range 2. Forests and hills block shots through them.",
  },
  scout: {
    name: "Scout",
    hp: 8,
    move: 5,
    damage: 2,
    range: 1,
    cost: 7,
    ability: "Pathfinder",
    description: "Forests cost only 1 movement.",
  },
};
export const TERRAIN = {
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
export function axial(tile) {
  return { q: tile.col - Math.floor(tile.row / 2), r: tile.row };
}
export function distance(a, b) {
  a = axial(a);
  b = axial(b);
  return (
    (Math.abs(a.q - b.q) +
      Math.abs(a.r - b.r) +
      Math.abs(a.q + a.r - b.q - b.r)) /
    2
  );
}
export class Game {
  constructor() {
    this.round = 1;
    this.turn = "player";
    this.gold = { player: 12, enemy: 12 };
    this.winner = null;
    this.nextId = 1;
    this.log = [];
    this.tiles = [];
    this.units = [];
    for (let row = 0; row < 7; row++)
      for (let col = 0; col < 11; col++)
        this.tiles.push({
          id: `${col},${row}`,
          col,
          row,
          terrain: "plains",
          owner: null,
        });
    for (const [col, row] of [
      [4, 2],
      [5, 2],
      [6, 2],
      [4, 3],
      [5, 3],
      [6, 3],
      [4, 4],
      [5, 4],
      [6, 4],
    ])
      this.tile(`${col},${row}`).terrain = "forest";
    for (const [col, row] of [
      [2, 2],
      [2, 4],
      [8, 2],
      [8, 4],
      [5, 0],
      [5, 6],
    ])
      this.tile(`${col},${row}`).terrain = "hill";
    for (const [col, row] of [
      [0, 0],
      [1, 0],
      [0, 6],
      [1, 6],
      [9, 0],
      [10, 0],
      [9, 6],
      [10, 6],
    ])
      this.tile(`${col},${row}`).terrain = "water";
    for (const [col, row] of [
      [3, 1],
      [3, 5],
      [7, 1],
      [7, 5],
    ])
      this.tile(`${col},${row}`).terrain = "village";
    for (const [col, row, side] of [
      [0, 3, "player"],
      [10, 3, "enemy"],
    ]) {
      const t = this.tile(`${col},${row}`);
      t.terrain = "keep";
      t.owner = side;
    }
    for (const [side, col] of [
      ["player", 0],
      ["enemy", 10],
    ]) {
      this.addUnit("commander", side, `${col},3`);
      this.addUnit("soldier", side, `${side === "player" ? 1 : 9},3`);
      this.addUnit("scout", side, `${side === "player" ? 1 : 9},2`);
    }
    this.record("Capture villages for gold. Defeat the enemy commander.");
  }
  tile(id) {
    return this.tiles.find((t) => t.id === id);
  }
  unit(id) {
    return this.units.find((u) => u.id === id);
  }
  at(tileId) {
    return this.units.find((u) => u.tile === tileId);
  }
  neighbors(id) {
    const tile = this.tile(id);
    return this.tiles.filter((t) => distance(tile, t) === 1);
  }
  record(message) {
    this.log.unshift(message);
    this.log = this.log.slice(0, 8);
  }
  addUnit(type, side, tile) {
    const u = {
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
  requireActor(id) {
    const u = this.unit(id);
    if (this.winner || !u || u.side !== this.turn)
      throw new Error("This unit cannot act now.");
    return u;
  }
  cost(unit, tile) {
    return tile.terrain === "forest" && unit.type === "scout"
      ? 1
      : TERRAIN[tile.terrain].cost;
  }
  reachable(id) {
    const unit = this.unit(id);
    if (!unit || unit.moved || unit.attacked || this.winner) return new Map();
    const costs = new Map([[unit.tile, 0]]),
      open = [unit.tile];
    while (open.length) {
      open.sort((a, b) => costs.get(a) - costs.get(b));
      const current = open.shift();
      for (const tile of this.neighbors(current)) {
        if (this.at(tile.id)) continue;
        const cost = costs.get(current) + this.cost(unit, tile);
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
  canAttackFrom(unit, from, target) {
    const d = distance(this.tile(from), this.tile(target.tile));
    if (d < 1 || d > TYPES[unit.type].range || unit.side === target.side)
      return false;
    if (unit.type === "archer" && d === 2) {
      return this.neighbors(from).some(
        (t) =>
          distance(t, this.tile(target.tile)) === 1 &&
          !["forest", "hill"].includes(t.terrain),
      );
    }
    return true;
  }
  targets(id) {
    const unit = this.unit(id);
    if (!unit || unit.attacked || this.winner) return [];
    return this.units.filter((t) => this.canAttackFrom(unit, unit.tile, t));
  }
  damage(attacker, defender) {
    const terrain = this.tile(defender.tile).terrain,
      ranged = attacker.type === "archer";
    let protection =
      terrain === "hill" ? 1 : terrain === "forest" && ranged ? 1 : 0;
    if (defender.type === "soldier" && ranged) protection++;
    return Math.max(1, TYPES[attacker.type].damage - protection);
  }
  preview(attacker, target) {
    const damage = this.damage(attacker, target);
    const retaliation =
      target.hp > damage &&
      TYPES[attacker.type].range === 1 &&
      TYPES[target.type].range === 1 &&
      distance(this.tile(attacker.tile), this.tile(target.tile)) === 1
        ? this.damage(target, attacker)
        : 0;
    return { damage, retaliation };
  }
  move(id, tileId) {
    const unit = this.requireActor(id);
    if (!this.reachable(id).has(tileId))
      throw new Error("That hex is outside this unit’s move.");
    unit.tile = tileId;
    unit.moved = true;
    const tile = this.tile(tileId);
    if (tile.terrain === "village" && tile.owner !== unit.side) {
      tile.owner = unit.side;
      this.record(
        `${unit.side === "player" ? "Your" : "Enemy"} ${TYPES[unit.type].name.toLowerCase()} captured a village.`,
      );
    }
    return unit;
  }
  attack(id, targetId) {
    const attacker = this.requireActor(id),
      target = this.unit(targetId);
    if (!target || !this.targets(id).includes(target))
      throw new Error("That enemy is out of range.");
    const result = this.preview(attacker, target);
    attacker.moved = true;
    attacker.attacked = true;
    target.hp -= result.damage;
    if (target.hp > 0) attacker.hp -= result.retaliation;
    this.record(
      `${attacker.side === "player" ? "Your" : "Enemy"} ${TYPES[attacker.type].name.toLowerCase()} dealt ${result.damage} damage${result.retaliation ? ` and took ${result.retaliation} in retaliation` : ""}.`,
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
  recruitTiles(id, type) {
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
    const tile = this.tile(u.tile);
    if (tile.terrain !== "keep" || tile.owner !== u.side) return [];
    return this.neighbors(u.tile).filter(
      (t) => t.terrain !== "water" && !this.at(t.id),
    );
  }
  recruit(id, type, tileId) {
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
  income(side) {
    return (
      this.tiles.filter((t) => t.terrain === "village" && t.owner === side)
        .length * 2
    );
  }
  beginTurn(side) {
    this.turn = side;
    this.gold[side] += this.income(side);
    for (const u of this.units.filter((u) => u.side === side)) {
      u.moved = false;
      u.attacked = false;
      const tile = this.tile(u.tile);
      if (tile.terrain === "village" && tile.owner === side)
        u.hp = Math.min(TYPES[u.type].hp, u.hp + 2);
    }
  }
  endTurn() {
    if (this.winner) throw new Error("The battle is over.");
    if (this.turn === "player") this.beginTurn("enemy");
    else {
      this.round++;
      this.beginTurn("player");
    }
  }
  enemyRecruit() {
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
        options.sort((a, b) => b.col - a.col)[0].id,
      );
  }
  enemyAct(id) {
    if (this.turn !== "enemy" || this.winner) return;
    const unit = this.unit(id);
    if (!unit || unit.side !== "enemy" || unit.attacked) return;
    const rank = (target) =>
      (target.type === "commander" ? 20 : 0) +
      (target.hp <= this.damage(unit, target) ? 12 : 0) +
      (TYPES[target.type].hp - target.hp);
    const attack = () => {
      const targets = this.targets(id).sort((a, b) => rank(b) - rank(a));
      if (targets.length) {
        this.attack(id, targets[0].id);
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
            .map((u) => this.tile(u.tile));
    if (unit.hp <= 4) {
      const healing = this.tiles.filter(
        (t) => t.terrain === "village" && t.owner === "enemy",
      );
      if (healing.length) goals = healing;
    }
    if (!goals.length) return;
    const options = [unit.tile, ...this.reachable(id).keys()];
    const utility = (tileId) => {
      const tile = this.tile(tileId),
        dist = Math.min(...goals.map((g) => distance(tile, g)));
      const attackScore = Math.max(
        0,
        ...this.units
          .filter((t) => this.canAttackFrom(unit, tileId, t))
          .map((t) => 5 + rank(t) / 5),
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
    if (options[0] !== unit.tile) this.move(id, options[0]);
    attack();
  }
  snapshot() {
    return {
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
