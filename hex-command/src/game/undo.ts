import { stateChanges } from "../save/undo";
import type { UndoStep, UndoChange } from "../../rust/pkg/hex_command";
import type { LiveGame } from "./live-game";
import type { GameState, Unit } from "./types";
import type { Immutable } from "./immutable";
type UndoGame = Pick<
  LiveGame,
  "round" | "turn" | "winner" | "captureUndoState" | "restoreState" | "move" | "unit"
>;
export interface SavedUndoEntry {
  changes?: UndoChange[];
  round: number;
  state: Immutable<GameState>;
  unitId: string;
}
interface UndoEntry extends SavedUndoEntry {
  game: UndoGame;
  changes?: UndoChange[];
  round: number;
  state: Immutable<GameState>;
  unitId: string;
}
export interface UndoResult {
  unitId: string;
  tileId: string;
}
export interface MoveUndo {
  move(game: UndoGame, unitId: string, tileId: string): Unit;
  undo(game: UndoGame): UndoResult | null;
  clear(): void;
  canUndo(game: UndoGame): boolean;
  saved(game: UndoGame): UndoStep[];
  restore(game: UndoGame, entries: SavedUndoEntry[]): void;
}
// History belongs to this play session; combat and turn changes are barriers.
export function createMoveUndo(): MoveUndo {
  const history: UndoEntry[] = [];

  function clear(): void {
    history.length = 0;
  }

  function canUndo(game: UndoGame): boolean {
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

  function move(game: UndoGame, unitId: string, tileId: string): Unit {
    // Check any existing entries before adding a move from another battle/turn.
    canUndo(game);
    const state = game.captureUndoState();
    const unit = game.move(unitId, tileId);
    if (game.turn === "player" && !game.winner) {
      const previous = history.at(-1);
      if (previous) previous.changes = undefined;
      history.push({ game, round: game.round, state, unitId });
    }
    return unit;
  }

  function undo(game: UndoGame): UndoResult | null {
    if (!canUndo(game)) return null;
    const previous = history.pop()!;
    game.restoreState(previous.state);
    return {
      unitId: previous.unitId,
      tileId: game.unit(previous.unitId)!.tile,
    };
  }

  return {
    move,
    undo,
    clear,
    canUndo,
    saved: (game) =>
      !canUndo(game)
        ? []
        : history.map((entry, index) => {
            const latest = index === history.length - 1;
            const changes = latest
              ? stateChanges(game.captureUndoState(), entry.state)
              : (entry.changes ??= stateChanges(history[index + 1]!.state, entry.state));
            return { round: entry.round, unitId: entry.unitId, changes };
          }),
    restore: (game, entries) => {
      clear();
      for (const entry of entries) history.push({ ...entry, game });
      canUndo(game);
    },
  };
}
