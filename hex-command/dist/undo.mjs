// History belongs to this play session; combat and turn changes are barriers.
export function createMoveUndo() {
  const history = [];

  function clear() {
    history.length = 0;
  }

  function canUndo(game) {
    const previous = history.at(-1);
    if (!previous) return false;
    if (
      previous.game !== game ||
      previous.round !== game.round ||
      game.turn !== "player" ||
      game.winner
    ) {
      clear();
      return false;
    }
    return true;
  }

  function move(game, unitId, tileId) {
    // Check any existing entries before adding a move from another battle/turn.
    canUndo(game);
    const state = structuredClone({ ...game });
    const unit = game.move(unitId, tileId);
    if (game.turn === "player" && !game.winner) {
      history.push({ game, round: game.round, state, unitId });
    }
    return unit;
  }

  function undo(game) {
    if (!canUndo(game)) return null;
    const previous = history.pop();
    const restored = structuredClone(previous.state);
    for (const key of Object.keys(game)) {
      if (!Object.hasOwn(restored, key)) delete game[key];
    }
    Object.assign(game, restored);
    return { unitId: previous.unitId, tileId: game.unit(previous.unitId).tile };
  }

  return { move, undo, clear, canUndo };
}
