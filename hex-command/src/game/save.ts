import { MAPS } from "./maps";
import { Game, TYPES } from "./engine";
import type { Side, Unit, UnitType, VillageState } from "./types";

export const SAVE_KEY = "hex-command-battle";
export const SAVE_VERSION = 1;
export interface LoadedBattle {
  game: Game;
  pendingEnemy: boolean;
}

export function saveBattle(game: Game, pendingEnemy = false): void {
  try {
    localStorage.setItem(
      SAVE_KEY,
      JSON.stringify({
        version: SAVE_VERSION,
        pendingEnemy,
        state: { ...game.snapshot(), nextId: game.nextId, log: game.log },
      }),
    );
  } catch {
    // Storage may be unavailable or full. The battle remains playable.
  }
}

const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;
const side = (value: unknown): value is Side =>
  value === "player" || value === "enemy";
const integer = (value: unknown): value is number =>
  typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
const unitType = (value: unknown): value is UnitType =>
  typeof value === "string" && Object.hasOwn(TYPES, value);

function validUnit(
  value: unknown,
  game: Game,
  nextId: number,
  ids: Set<string>,
  occupied: Set<string>,
): value is Unit {
  if (
    !record(value) ||
    typeof value.id !== "string" ||
    !/^u[1-9]\d*$/.test(value.id) ||
    Number(value.id.slice(1)) >= nextId ||
    ids.has(value.id) ||
    !unitType(value.type) ||
    !side(value.side) ||
    typeof value.tile !== "string" ||
    !game.tile(value.tile) ||
    game.tile(value.tile)!.terrain === "water" ||
    occupied.has(value.tile) ||
    !integer(value.hp) ||
    value.hp < 1 ||
    value.hp > TYPES[value.type].hp ||
    typeof value.moved !== "boolean" ||
    typeof value.attacked !== "boolean"
  )
    return false;
  return true;
}
function validVillage(
  value: unknown,
  villages: VillageState[],
): value is VillageState {
  return (
    record(value) &&
    typeof value.id === "string" &&
    villages.some((tile) => tile.id === value.id) &&
    (value.owner === null || side(value.owner))
  );
}

export function loadBattle(): LoadedBattle {
  const fresh = (): LoadedBattle => ({ game: new Game(), pendingEnemy: false });
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(SAVE_KEY) ?? "null");
    if (!record(saved) || saved.version !== SAVE_VERSION) return fresh();
    const s = saved.state;
    if (
      !record(s) ||
      (s.mapId !== undefined &&
        (typeof s.mapId !== "string" || !Object.hasOwn(MAPS, s.mapId)))
    )
      return fresh();
    // The constructor rejects unsupported revisions, including malformed persisted values.
    if (
      s.mapRevision !== undefined &&
      s.mapRevision !== null &&
      typeof s.mapRevision !== "number"
    )
      return fresh();
    const game = new Game(s.mapId as string | undefined, s.mapRevision ?? 1);
    if (
      !integer(s.round) ||
      s.round < 1 ||
      !side(s.turn) ||
      !(s.winner === null || side(s.winner)) ||
      !(s.turn === "player" || s.winner) ||
      typeof saved.pendingEnemy !== "boolean" ||
      (saved.pendingEnemy && (s.turn !== "player" || s.winner)) ||
      !record(s.gold) ||
      !integer(s.gold.player) ||
      !integer(s.gold.enemy) ||
      !integer(s.nextId) ||
      s.nextId < 1 ||
      !Array.isArray(s.units) ||
      !Array.isArray(s.villages) ||
      !Array.isArray(s.log) ||
      s.log.length > 8 ||
      !s.log.every(
        (line: unknown): line is string =>
          typeof line === "string" && line.length <= 1000,
      )
    )
      return fresh();
    const ids = new Set<string>(),
      occupied = new Set<string>();
    const units: Unit[] = [];
    for (const unit of s.units as unknown[]) {
      if (!validUnit(unit, game, s.nextId, ids, occupied)) return fresh();
      ids.add(unit.id);
      occupied.add(unit.tile);
      units.push({ ...unit });
    }
    const villages = game.tiles.filter((tile) => tile.terrain === "village");
    if (
      s.villages.length !== villages.length ||
      new Set(
        (s.villages as unknown[]).map((v) => (record(v) ? v.id : undefined)),
      ).size !== villages.length
    )
      return fresh();
    for (const village of s.villages as unknown[]) {
      if (!validVillage(village, villages)) return fresh();
      game.tile(village.id)!.owner = village.owner;
    }
    if (
      !s.winner &&
      ["player", "enemy"].some(
        (team) =>
          !units.some(
            (unit) => unit.side === team && unit.type === "commander",
          ),
      )
    )
      return fresh();
    Object.assign(game, {
      round: s.round,
      turn: s.turn,
      winner: s.winner,
      gold: { ...s.gold },
      units,
      nextId: s.nextId,
      log: [...s.log],
    });
    return { game, pendingEnemy: saved.pendingEnemy };
  } catch {
    return fresh();
  }
}
