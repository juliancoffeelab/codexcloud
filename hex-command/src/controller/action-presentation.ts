import type { GameView, Immutable } from "../game/view";
import type { CombatAnimation } from "../game/types";

export type ActionOutcome =
  | { kind: "completed" }
  | { kind: "cancelled"; reason: "session-replaced" | "disposed" }
  | { kind: "failed"; error: unknown };

/** Read-only identity/data; HUD inspection has no completion or cancellation methods. */
export interface ActionToken {
  readonly id: number;
  readonly before: GameView;
  readonly combat?: Immutable<CombatAnimation> | null;
}
interface Task {
  generation: number;
  settled: boolean;
  completedMotions: Set<string>;
  resolve: (outcome: ActionOutcome) => void;
}
const tasks = new WeakMap<ActionToken, Task>();
let nextId = 0;

/** The session owns cancellation; only a currently attached renderer can acknowledge. */
export class ActionPresentation {
  readonly token: ActionToken;
  readonly finished: Promise<ActionOutcome>;
  private task: Task;
  constructor(before: GameView, combat?: Immutable<CombatAnimation> | null) {
    this.token = Object.freeze({ id: ++nextId, before, combat });
    let resolve!: Task["resolve"];
    this.finished = new Promise((done) => (resolve = done));
    this.task = { generation: 0, settled: false, completedMotions: new Set(), resolve };
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
  const generation = ++task.generation;
  return {
    motionDone(key: string) {
      return task.completedMotions.has(key);
    },
    finishMotion(key: string) {
      if (task.generation === generation && !task.settled) task.completedMotions.add(key);
    },
    complete() {
      if (task.settled || task.generation !== generation) return;
      task.settled = true;
      task.resolve({ kind: "completed" });
    },
    detach() {
      if (task.generation === generation) task.generation++;
    },
  };
}
