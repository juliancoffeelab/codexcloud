import { nativeValue, type LiveGame } from "../game/live-game";
import { SAVE_VERSION } from "./schema";
import type { CurrentSave, BattleSave, ManualSave } from "./schema";
import { validSetup } from "../game/setup";
import { storeSave } from "./storage";
import {
  describeRecovery,
  describeManual,
  describeTurn,
  newRun,
  type ProgressIdentity,
  type TurnEntry,
  type ManualEntry,
} from "./metadata";

function serialize(game: LiveGame, metadata: Omit<BattleSave, "checkpoint">): string | undefined {
  if (game.turn === "enemy" && !game.winner && !(game.round === 1 && game.setup.first === "enemy"))
    return;
  if (
    !validSetup(game.setup) ||
    (metadata.pendingEnemy && (game.turn !== "player" || game.winner !== null))
  )
    throw new Error("Invalid current save envelope");
  const checkpoint = nativeValue(game.serializedCheckpoint());
  const text = JSON.stringify(metadata);
  return `${text.slice(0, -1)},"checkpoint":${checkpoint}}`;
}
export function serializeBattle(
  game: LiveGame,
  pendingEnemy: boolean,
  extra: Pick<CurrentSave, "undo" | "progress">,
): string | undefined {
  return serialize(game, { version: SAVE_VERSION, setup: game.setup, pendingEnemy, ...extra });
}
/** The only history-write value: captured at the controller's turn boundaries. */
export class TurnStartCheckpoint {
  readonly #text: string;
  private constructor(
    readonly entry: TurnEntry,
    text: string,
  ) {
    this.#text = text;
    Object.freeze(this);
  }
  static capture(
    game: LiveGame,
    identity: ProgressIdentity,
    missionStart: boolean,
    parentId: string,
  ): TurnStartCheckpoint | undefined {
    const entry = Object.freeze(describeTurn(game, identity, missionStart, parentId));
    const text = serialize(game, {
      version: SAVE_VERSION,
      setup: game.setup,
      pendingEnemy: false,
      progress: { entry },
    });
    return text ? new TurnStartCheckpoint(entry, text) : undefined;
  }
  /** Runtime nominal check also rejects arbitrary objects presented as snapshots. */
  static text(checkpoint: TurnStartCheckpoint): string {
    return checkpoint.#text;
  }
}
/** Write a new current snapshot. This API never inserts historical records. */
export function saveBattle(game: LiveGame, pendingEnemy = false): void {
  const text = serializeBattle(game, pendingEnemy, {
    undo: [],
    progress: { entry: describeRecovery(game, newRun()) },
  });
  if (text) {
    try {
      storeSave(text);
    } catch {
      /* Storage is optional for direct consumers. */
    }
  }
}

/** An explicitly requested immutable snapshot, distinct from automatic turn starts. */
export class ManualCheckpoint {
  readonly #text: string;
  private constructor(
    readonly entry: ManualEntry,
    text: string,
  ) {
    this.#text = text;
    Object.freeze(this);
  }
  static capture(
    game: LiveGame,
    identity: ProgressIdentity,
    undo: ManualSave["undo"],
  ): ManualCheckpoint {
    if (game.turn !== "player" && !game.winner)
      throw new Error("Wait for the enemy turn to finish before saving.");
    const entry = Object.freeze(describeManual(game, identity));
    const text = serialize(game, {
      version: SAVE_VERSION,
      setup: game.setup,
      pendingEnemy: false,
      undo,
      progress: { entry },
    });
    if (!text) throw new Error("This position cannot be saved yet.");
    return new ManualCheckpoint(entry, text);
  }
  static text(checkpoint: ManualCheckpoint): string {
    return checkpoint.#text;
  }
}
