import { Game, TYPES } from './engine.mjs';

export const SAVE_KEY = 'hex-command-battle';
export const SAVE_VERSION = 1;

export function saveBattle(game, pendingEnemy = false) {
  try {
    localStorage.setItem(SAVE_KEY, JSON.stringify({
      version: SAVE_VERSION, pendingEnemy,
      state: { ...game.snapshot(), nextId: game.nextId, log: game.log },
    }));
  } catch {
    // Storage may be unavailable or full. The battle remains playable.
  }
}

export function loadBattle() {
  const fresh = () => ({ game: new Game(), pendingEnemy: false });
  try {
    const saved = JSON.parse(localStorage.getItem(SAVE_KEY));
    if (!saved || saved.version !== SAVE_VERSION) return fresh();
    const s = saved.state;
    const game = new Game();
    const side = value => ['player', 'enemy'].includes(value);
    const integer = value => Number.isSafeInteger(value) && value >= 0;
    if (!s || !integer(s.round) || s.round < 1 || !side(s.turn) ||
        !(s.winner === null || side(s.winner)) ||
        !(s.turn === 'player' || s.winner) ||
        typeof saved.pendingEnemy !== 'boolean' ||
        (saved.pendingEnemy && (s.turn !== 'player' || s.winner)) ||
        !integer(s.gold?.player) || !integer(s.gold?.enemy) ||
        !integer(s.nextId) || s.nextId < 1 ||
        !Array.isArray(s.units) || !Array.isArray(s.villages) ||
        !Array.isArray(s.log) || s.log.length > 8 ||
        s.log.some(line => typeof line !== 'string' || line.length > 1000)) return fresh();
    const ids = new Set(), occupied = new Set();
    for (const unit of s.units) {
      if (!unit || !/^u[1-9]\d*$/.test(unit.id) ||
          Number(unit.id.slice(1)) >= s.nextId || ids.has(unit.id) ||
          !Object.hasOwn(TYPES, unit.type) || !side(unit.side) ||
          !game.tile(unit.tile) || game.tile(unit.tile).terrain === 'water' ||
          occupied.has(unit.tile) || !integer(unit.hp) || unit.hp < 1 ||
          unit.hp > TYPES[unit.type].hp || typeof unit.moved !== 'boolean' ||
          typeof unit.attacked !== 'boolean') return fresh();
      ids.add(unit.id); occupied.add(unit.tile);
    }
    const villages = game.tiles.filter(tile => tile.terrain === 'village');
    if (s.villages.length !== villages.length ||
        new Set(s.villages.map(v => v?.id)).size !== villages.length) return fresh();
    for (const village of s.villages) {
      if (!village || !villages.some(tile => tile.id === village.id) ||
          !(village.owner === null || side(village.owner))) return fresh();
      game.tile(village.id).owner = village.owner;
    }
    if (!s.winner && ['player', 'enemy'].some(team =>
      !s.units.some(unit => unit.side === team && unit.type === 'commander'))) return fresh();
    Object.assign(game, { round: s.round, turn: s.turn, winner: s.winner,
      gold: { ...s.gold }, units: s.units.map(unit => ({ ...unit })),
      nextId: s.nextId, log: [...s.log] });
    return { game, pendingEnemy: saved.pendingEnemy };
  } catch {
    return fresh();
  }
}
