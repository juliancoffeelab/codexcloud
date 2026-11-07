import { measureTrace } from "../presentation/performance-trace";
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
  BoardInspection,
  NativeEngineFactory as WasmEngineFactory,
  SavedGame,
  Start,
  State,
  UndoStep,
  UndoValidation,
} from "../../rust/pkg/hex_command";

export type NativeResponse<T> = EngineResult<Immutable<T>>;

/** Typed Rust engine boundary used by the live browser controller. */
export interface GameEngine {
  view(): NativeResponse<EngineView>;
  inspect(): NativeResponse<BattleInspection>;
  geometry(): NativeResponse<BoardInspection>;
  dispatch(command: Immutable<Command>, expectedRevision?: number): NativeResponse<DispatchResult>;
  chooseAiCommand(options: AiOptions): NativeResponse<AiDecision>;
  save(): NativeResponse<SavedGame>;
  saveJson(): NativeResponse<string>;
  dispose(): EngineResult<null>;
}

export interface GameEngineFactory {
  catalog(): NativeResponse<CatalogView>;
  create(start: Start, seed: number): EngineResult<GameEngine>;
  restore(saved: Immutable<SavedGame>): EngineResult<GameEngine>;
  validateUndo(state: Immutable<State>, history: UndoStep[]): EngineResult<null>;
  dispose(): EngineResult<null>;
}

/** Signatures come from wasm-bindgen; tests can supply the same public operations. */
export type NativeBridge = Pick<
  WasmEngineFactory,
  | "catalog"
  | "create"
  | "restore"
  | "validateUndo"
  | "view"
  | "inspect"
  | "geometry"
  | "dispatchUpdate"
  | "chooseAiCommand"
  | "save"
  | "saveJson"
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

/** Generated Rust responses own their JS object graphs; injected bridges may borrow them. */
export interface NativeBridgeOptions {
  responseOwnership?: "borrowed" | "transferred";
}

function validSeed(seed: number): boolean {
  return Number.isInteger(seed) && seed >= 0 && seed <= 0xffffffff;
}

export function createNativeEngineFactory(
  bridge: NativeBridge,
  { responseOwnership = "borrowed" }: NativeBridgeOptions = {},
): GameEngineFactory {
  const detach = <T>(value: T): T =>
    responseOwnership === "transferred" ? value : structuredClone(value);
  const owned = <T>(result: EngineResult<T>): NativeResponse<T> =>
    immutable(detach(result)) as NativeResponse<T>;
  let disposed = false;
  const engines = new Set<GameEngine>();
  const alive = <T>(operation: () => EngineResult<T>): EngineResult<T> =>
    disposed ? failure({ code: "disposed" }) : call(operation);

  function engine(handle: BattleHandle): GameEngine {
    const id = handle.id;
    let released = false;
    let revision = handle.view.revision;
    let currentInspection: Immutable<BattleInspection> | undefined;
    let current: Immutable<EngineView> = immutable(detach(handle.view));
    let tileIndices = new Map(current.state.tiles.map((tile, index) => [tile.id, index]));
    const active = <T>(operation: () => EngineResult<T>): EngineResult<T> =>
      released || disposed ? failure({ code: "disposed" }) : call(operation);
    const result: GameEngine = Object.freeze({
      view: () => {
        if (responseOwnership === "transferred" && !released && !disposed)
          return immutable({ ok: true as const, value: current });
        const result = owned(active(() => bridge.view(id, revision)));
        if (result.ok) {
          current = result.value;
          tileIndices = new Map(current.state.tiles.map((tile, index) => [tile.id, index]));
        }
        return result;
      },
      inspect: () =>
        currentInspection && !released && !disposed
          ? immutable({ ok: true as const, value: currentInspection })
          : owned(active(() => bridge.inspect(id, revision))),
      geometry: () => owned(active(() => bridge.geometry(id, revision))),
      dispatch: (
        command: Immutable<Command>,
        expectedRevision = revision,
      ): NativeResponse<DispatchResult> => {
        const dispatched = measureTrace("engine.dispatch", () =>
          owned(
            active(() =>
              bridge.dispatchUpdate(
                id,
                expectedRevision,
                structuredClone(command) as Command,
                responseOwnership === "transferred" ? true : undefined,
              ),
            ),
          ),
        );
        if (!dispatched.ok) return dispatched;
        const update = dispatched.value;
        if (update.baseRevision !== current.revision)
          throw new Error("Incremental publication does not match its base revision");
        let tiles: Immutable<EngineView>["state"]["tiles"];
        switch (update.terrain.kind) {
          case "keep":
            tiles = current.state.tiles;
            break;
          case "replace":
            tiles = update.terrain.tiles;
            tileIndices = new Map(tiles.map((tile, index) => [tile.id, index]));
            break;
          case "patch": {
            const changed = [...current.state.tiles];
            for (const tile of update.terrain.tiles) {
              const index = tileIndices.get(tile.id);
              if (index === undefined)
                throw new Error(`Terrain update references unknown tile ${tile.id}`);
              changed[index] = tile;
            }
            tiles = changed;
            break;
          }
          default: {
            const impossible: never = update.terrain;
            throw new Error(`Unknown terrain publication ${impossible}`);
          }
        }
        current = immutable({
          revision: update.revision,
          state: { ...update.state, tiles },
          presentation: update.presentation,
        });
        revision = current.revision;
        currentInspection = update.inspection;
        return { ok: true, value: immutable({ view: current, combat: update.combat }) };
      },
      chooseAiCommand: (options: AiOptions) =>
        measureTrace("engine.chooseAiCommand", () =>
          owned(active(() => bridge.chooseAiCommand(id, revision, structuredClone(options)))),
        ),
      save: () => owned(active(() => bridge.save(id, revision))),
      saveJson: () => owned(active(() => bridge.saveJson(id, revision))),
      dispose: (): EngineResult<null> => {
        if (released || disposed) return { ok: true, value: null };
        const closed = call(() => bridge.disposeBattle(id));
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
    validateUndo: (state: Immutable<State>, history: UndoStep[]) =>
      alive(() => bridge.validateUndo(structuredClone({ state, history }) as UndoValidation)),
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
  options: NativeBridgeOptions = {},
  initialization?: Parameters<NativeWasmModule["default"]>[0],
): Promise<EngineResult<GameEngineFactory>> {
  try {
    const module = await loader();
    await module.default(initialization);
    return {
      ok: true,
      value: createNativeEngineFactory(new module.NativeEngineFactory(bundle), options),
    };
  } catch (error) {
    return thrown(error);
  }
}
