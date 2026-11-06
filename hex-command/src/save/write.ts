import { nativeValue, type LiveGame } from "../game/live-game";
import { SAVE_VERSION } from "./schema";
import type { CurrentSave, TurnSave } from "./schema";
import { validSetup } from "../game/setup";
import { storeSave } from "./storage";
import {
  describeRecovery,
  describeTurn,
  newRun,
  type ProgressIdentity,
  type TurnEntry,
} from "./metadata";

function serialize(
  game: LiveGame,
  metadata: Omit<CurrentSave, "checkpoint"> | Omit<TurnSave, "checkpoint">,
): string | undefined {
  if (game.turn === "enemy" && !game.winner && !(game.round === 1 && game.setup.first === "enemy"))
    return;
  if (
    !validSetup(game.setup) ||
    (metadata.pendingEnemy && (game.turn !== "player" || game.winner !== null))
  )
    throw new Error("Invalid current save envelope");
  if (metadata.progress.entry.kind === "turn" && metadata.undo !== undefined)
    throw new Error("Historical snapshots cannot contain move undo.");
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
/** Snapshot-only history: automatic turn boundaries and explicitly requested saves. */
export class TurnCheckpoint {
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
    manual = false,
  ): TurnCheckpoint | undefined {
    const entry = Object.freeze(describeTurn(game, identity, missionStart, parentId, manual));
    const text = serialize(game, {
      version: SAVE_VERSION,
      setup: game.setup,
      pendingEnemy: false,
      progress: { entry },
    });
    return text ? new TurnCheckpoint(entry, text) : undefined;
  }
  /** Runtime nominal check also rejects arbitrary objects presented as snapshots. */
  static text(checkpoint: TurnCheckpoint): string {
    return checkpoint.#text;
  }
}
/** Write a new current snapshot. This API never inserts historical records. */
export async function saveBattle(game: LiveGame, pendingEnemy = false): Promise<void> {
  const text = serializeBattle(game, pendingEnemy, {
    undo: [],
    progress: { entry: describeRecovery(game, newRun()) },
  });
  if (text) {
    try {
      await storeSave(text);
    } catch {
      /* Storage is optional for direct consumers. */
    }
  }
}
