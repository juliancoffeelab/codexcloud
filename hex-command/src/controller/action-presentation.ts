import type { Immutable } from "../game/view";
import type { CombatAnimation } from "../game/types";

export type ActionOutcome =
  | { kind: "completed" }
  | { kind: "cancelled"; reason: "session-replaced" | "disposed" }
  | { kind: "failed"; error: unknown };

/** Read-only identity/data; HUD inspection has no completion or cancellation methods. */
export interface ActionToken {
  readonly id: number;
  readonly combat?: Immutable<CombatAnimation> | null;
}
interface Task {
  settled: boolean;
  resolve: (outcome: ActionOutcome) => void;
}
const tasks = new WeakMap<ActionToken, Task>();
let nextId = 0;

/** The session owns cancellation; the renderer acknowledges completion. */
export class ActionPresentation {
  readonly token: ActionToken;
  readonly finished: Promise<ActionOutcome>;
  private task: Task;
  constructor(combat?: Immutable<CombatAnimation> | null) {
    this.token = Object.freeze({ id: ++nextId, combat });
    let resolve!: Task["resolve"];
    this.finished = new Promise((done) => (resolve = done));
    this.task = { settled: false, resolve };
    tasks.set(this.token, this.task);
  }
  fail(error: unknown) {
    if (this.task.settled) return;
    this.task.settled = true;
    this.task.resolve({ kind: "failed", error });
  }
  cancel(reason: "session-replaced" | "disposed") {
    if (this.task.settled) return;
    this.task.settled = true;
    this.task.resolve({ kind: "cancelled", reason });
  }
}

export function claimAction(token: ActionToken) {
  const task = tasks.get(token);
  if (!task) throw new Error("Unknown action presentation");
  return {
    complete() {
      if (task.settled) return;
      task.settled = true;
      task.resolve({ kind: "completed" });
    },
  };
}
