import { immutable, type Immutable } from "./immutable";
import type {
  AiDecision,
  AiOptions,
  BattleHandle,
  CatalogView,
  Command,
  DispatchResult,
  EngineError,
  EngineResult,
  EngineView,
  BattleInspection,
  NativeEngineFactory as WasmEngineFactory,
  SavedGame,
  Start,
} from "../../rust/pkg/hex_command";

export type NativeResponse<T> = EngineResult<Immutable<T>>;

/** Typed Rust engine boundary used by the live browser controller. */
export interface GameEngine {
  view(): NativeResponse<EngineView>;
  inspect(): NativeResponse<BattleInspection>;
  dispatch(command: Immutable<Command>, expectedRevision?: number): NativeResponse<DispatchResult>;
  chooseAiCommand(options: AiOptions): NativeResponse<AiDecision>;
  save(): NativeResponse<SavedGame>;
  dispose(): EngineResult<null>;
}

export interface GameEngineFactory {
  catalog(): NativeResponse<CatalogView>;
  create(start: Start, seed: number): EngineResult<GameEngine>;
  restore(saved: Immutable<SavedGame>): EngineResult<GameEngine>;
  dispose(): EngineResult<null>;
}

/** Signatures come from wasm-bindgen; tests can supply the same public operations. */
export type NativeBridge = Pick<
  WasmEngineFactory,
  | "catalog"
  | "create"
  | "restore"
  | "view"
  | "inspect"
  | "dispatch"
  | "chooseAiCommand"
  | "save"
  | "disposeBattle"
  | "free"
>;

export type NativeWasmModule = Pick<
  typeof import("../../rust/pkg/hex_command"),
  "default" | "NativeEngineFactory"
>;

function failure<T>(error: EngineError): EngineResult<T> {
  return { ok: false, error };
}

function thrown<T>(error: unknown): EngineResult<T> {
  // Factory construction can reject with an already tagged native error.
  if (error && typeof error === "object" && "ok" in error && error.ok === false) {
    return error as EngineResult<T>;
  }
  return failure({
    code: "invalidRequest",
    message: error instanceof Error ? error.message : String(error),
  });
}

function call<T>(operation: () => EngineResult<T>): EngineResult<T> {
  try {
    return operation();
  } catch (error) {
    return thrown(error);
  }
}

/** Detach before freezing, including errors; injected bridges never retain our snapshot. */
function owned<T>(result: EngineResult<T>): NativeResponse<T> {
  return immutable(structuredClone(result)) as NativeResponse<T>;
}

function validSeed(seed: number): boolean {
  return Number.isInteger(seed) && seed >= 0 && seed <= 0xffffffff;
}

export function createNativeEngineFactory(bridge: NativeBridge): GameEngineFactory {
  let disposed = false;
  const engines = new Set<GameEngine>();
  const alive = <T>(operation: () => EngineResult<T>): EngineResult<T> =>
    disposed ? failure({ code: "disposed" }) : call(operation);

  function engine(handle: BattleHandle): GameEngine {
    let released = false;
    let revision = handle.view.revision;
    const active = <T>(operation: () => EngineResult<T>): EngineResult<T> =>
      released || disposed ? failure({ code: "disposed" }) : call(operation);
    const result: GameEngine = Object.freeze({
      view: () => owned(active(() => bridge.view(handle.id, revision))),
      inspect: () => owned(active(() => bridge.inspect(handle.id, revision))),
      dispatch: (command: Immutable<Command>, expectedRevision = revision) => {
        const dispatched = active(() =>
          bridge.dispatch(handle.id, expectedRevision, structuredClone(command) as Command),
        );
        if (dispatched.ok) revision = dispatched.value.view.revision;
        return owned(dispatched);
      },
      chooseAiCommand: (options: AiOptions) =>
        owned(active(() => bridge.chooseAiCommand(handle.id, revision, structuredClone(options)))),
      save: () => owned(active(() => bridge.save(handle.id, revision))),
      dispose: (): EngineResult<null> => {
        if (released || disposed) return { ok: true, value: null };
        const closed = call(() => bridge.disposeBattle(handle.id));
        released = true;
        engines.delete(result);
        return closed;
      },
    });
    engines.add(result);
    return result;
  }

  function accept(result: EngineResult<BattleHandle>): EngineResult<GameEngine> {
    return result.ok ? { ok: true, value: engine(result.value) } : result;
  }

  return Object.freeze({
    catalog: () => owned(alive(() => bridge.catalog())),
    create: (start: Start, seed: number): EngineResult<GameEngine> => {
      if (!validSeed(seed)) {
        return failure({ code: "invalidRequest", message: "seed must be a uint32" });
      }
      return accept(alive(() => bridge.create(structuredClone(start), seed)));
    },
    restore: (saved: Immutable<SavedGame>): EngineResult<GameEngine> => {
      if (!validSeed(saved.seed)) {
        return failure({ code: "invalidSave", message: "saved seed must be a uint32" });
      }
      return accept(alive(() => bridge.restore(structuredClone(saved) as SavedGame)));
    },
    dispose: (): EngineResult<null> => {
      if (disposed) return { ok: true, value: null };
      let firstError: EngineError | undefined;
      for (const current of [...engines]) {
        const closed = current.dispose();
        if (!closed.ok) firstError ??= closed.error;
      }
      disposed = true;
      const freed = call<null>(() => {
        bridge.free();
        return { ok: true, value: null };
      });
      if (firstError) return failure(firstError);
      return freed;
    },
  });
}

/** Load generated WASM only on explicit use. The loader controls bundling and URL choice. */
export async function loadNativeEngineFactory(
  loader: () => Promise<NativeWasmModule>,
  bundle?: Record<string, string>,
): Promise<EngineResult<GameEngineFactory>> {
  try {
    const module = await loader();
    await module.default();
    return {
      ok: true,
      value: createNativeEngineFactory(new module.NativeEngineFactory(bundle)),
    };
  } catch (error) {
    return thrown(error);
  }
}
