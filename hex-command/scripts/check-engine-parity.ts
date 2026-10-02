/**
 * Compare the migrated rules with the last JavaScript revision.
 * Run: node --import tsx scripts/check-engine-parity.ts
 * Optional: --baseline /path/to/original/dist OR --ref <git-revision>
 * Default baseline is extracted into a temporary directory from the pinned
 * pre-migration commit. No generated files or legacy source are kept in the repo.
 */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Game, TYPES, TERRAIN } from "../src/game/engine";
import { loadBattle } from "../src/game/save";
import { createMoveUndo } from "../src/game/undo";
import type { MapId, Side, Unit, UnitType } from "../src/game/types";

type LegacyGame = Omit<Game, "captureState" | "restoreState">;
interface LegacyEngine {
  Game: new (mapId?: string, mapRevision?: number) => LegacyGame;
  TYPES: unknown;
  TERRAIN: unknown;
}
interface LegacyUndo {
  move(game: LegacyGame, unitId: string, tileId: string): Unit;
  undo(game: LegacyGame): { unitId: string; tileId: string } | null;
}
interface LegacySave {
  loadBattle(): { game: LegacyGame; pendingEnemy: boolean };
}
interface MutableSave {
  [key: string]: unknown;
  state: {
    [key: string]: unknown;
    units: Record<string, unknown>[];
  };
}
const maps: [MapId, number][] = [
  ["mirror", 1],
  ["river", 1],
  ["river", 2],
];
const baselineCommit = "2297aa12ba5407fb9f5e84225517dd6139ff343a";

function option(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  if (index < 0) return undefined;
  const value = process.argv[index + 1];
  if (!value || value.startsWith("--"))
    throw new Error(`Missing value for ${name}.`);
  return value;
}

function engineParity(legacy: LegacyEngine, oldUndo: () => LegacyUndo): number {
  assert.deepEqual(TYPES, legacy.TYPES);
  assert.deepEqual(TERRAIN, legacy.TERRAIN);
  let checks = 0;
  for (const [map, revision] of maps) {
    for (let seed = 1; seed <= 20; seed++) {
      let rng = seed;
      const random = (): number =>
        (rng = (Math.imul(rng, 1664525) + 1013904223) >>> 0) / 2 ** 32;
      const original = new legacy.Game(map, revision);
      const current = new Game(map, revision);
      const equal = (): void => {
        assert.deepEqual(
          structuredClone({ ...current }),
          structuredClone({ ...original }),
          `${map} revision ${revision}, seed ${seed}, comparison ${checks}`,
        );
        checks++;
      };
      equal();
      for (let turn = 0; turn < 40 && !original.winner; turn++) {
        if (original.turn === "player") {
          for (const unit of [...original.units].filter(
            (u) => u.side === "player",
          )) {
            const moves = [...original.reachable(unit.id).keys()];
            assert.deepEqual(
              [...current.reachable(unit.id)],
              [...original.reachable(unit.id)],
            );
            if (moves.length && random() < 0.85) {
              const destination = moves[Math.floor(random() * moves.length)]!;
              original.move(unit.id, destination);
              current.move(unit.id, destination);
              equal();
            }
            const targets = original.targets(unit.id);
            assert.deepEqual(current.targets(unit.id), targets);
            if (targets.length && !original.winner) {
              const target = targets[Math.floor(random() * targets.length)]!;
              const attack = original.bestAttack(unit, target)!;
              assert.deepEqual(
                current.bestAttack(
                  current.unit(unit.id)!,
                  current.unit(target.id)!,
                ),
                attack,
              );
              original.attack(unit.id, target.id, attack.id);
              current.attack(unit.id, target.id, attack.id);
              equal();
            }
            if (original.winner) break;
            const type: UnitType = random() < 0.5 ? "archer" : "soldier";
            const tiles = original.recruitTiles(unit.id, type);
            assert.deepEqual(current.recruitTiles(unit.id, type), tiles);
            if (tiles.length && random() < 0.6) {
              original.recruit(unit.id, type, tiles[0]!.id);
              current.recruit(unit.id, type, tiles[0]!.id);
              equal();
            }
          }
        } else {
          original.enemyRecruit();
          current.enemyRecruit();
          equal();
          for (const unit of [...original.units].filter(
            (u) => u.side === "enemy",
          )) {
            original.enemyAct(unit.id);
            current.enemyAct(unit.id);
            equal();
          }
        }
        if (!original.winner) {
          original.endTurn();
          current.endTurn();
          equal();
        }
      }
    }
  }
  for (const owner of [null, "enemy", "player"] satisfies (Side | null)[]) {
    const original = new legacy.Game();
    const current = new Game();
    const originalUndo = oldUndo();
    const currentUndo = createMoveUndo();
    original.tile("3,1")!.owner = owner;
    current.tile("3,1")!.owner = owner;
    const before = structuredClone({ ...current });
    originalUndo.move(original, "u3", "3,1");
    currentUndo.move(current, "u3", "3,1");
    assert.deepEqual(currentUndo.undo(current), originalUndo.undo(original));
    assert.deepEqual(structuredClone({ ...current }), before);
  }
  return checks;
}

function saveParity(legacy: LegacySave): number {
  const previousStorage = Object.getOwnPropertyDescriptor(
    globalThis,
    "localStorage",
  );
  const entries = new Map<string, string>();
  const storage: Storage = {
    get length() {
      return entries.size;
    },
    clear: () => entries.clear(),
    key: (index) => [...entries.keys()][index] ?? null,
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => {
      entries.set(key, value);
    },
    removeItem: (key) => {
      entries.delete(key);
    },
  };
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: storage,
  });
  let count = 0;
  try {
    for (const [map, revision] of maps) {
      const game = new Game(map, revision);
      game.move(
        "u3",
        map === "mirror" ? "3,1" : revision === 1 ? "3,4" : "4,1",
      );
      const base: MutableSave = {
        version: 1,
        pendingEnemy: false,
        state: {
          ...game.snapshot(),
          units: game.units.map((unit) => ({ ...unit })),
          nextId: game.nextId,
          log: game.log,
        },
      };
      const compare = (saved: MutableSave): void => {
        storage.setItem("hex-command-battle", JSON.stringify(saved));
        const actual = loadBattle();
        const expected = legacy.loadBattle();
        assert.deepEqual(
          { ...actual, game: { ...actual.game } },
          { ...expected, game: { ...expected.game } },
        );
        count++;
      };
      compare(base);
      for (const key of ["version", "pendingEnemy"]) {
        for (const malformed of [null, "wrong", [], {}, 0, 2, true]) {
          const copy = structuredClone(base);
          copy[key] = malformed;
          compare(copy);
        }
      }
      for (const key of [
        "mapId",
        "mapRevision",
        "round",
        "turn",
        "winner",
        "gold",
        "nextId",
        "log",
        "units",
        "villages",
      ]) {
        for (const malformed of [null, "wrong", [], {}, 0, -1, 2, true]) {
          const copy = structuredClone(base);
          copy.state[key] = malformed;
          compare(copy);
        }
      }
      for (const key of [
        "id",
        "type",
        "side",
        "tile",
        "hp",
        "moved",
        "attacked",
      ]) {
        for (const malformed of [null, "wrong", [], {}, 0, -1, 99, true]) {
          const copy = structuredClone(base);
          copy.state.units[0]![key] = malformed;
          compare(copy);
        }
      }
    }
  } finally {
    if (previousStorage)
      Object.defineProperty(globalThis, "localStorage", previousStorage);
    else Reflect.deleteProperty(globalThis, "localStorage");
  }
  return count;
}

const project = resolve(dirname(fileURLToPath(import.meta.url)), "..");
let temporary: string | undefined;
try {
  let baseline = option("--baseline");
  if (!baseline) {
    temporary = mkdtempSync(join(tmpdir(), "hex-command-rules-baseline-"));
    baseline = temporary;
    const revision = option("--ref") ?? baselineCommit;
    const prefix = execFileSync("git", ["rev-parse", "--show-prefix"], {
      cwd: project,
      encoding: "utf8",
    }).trim();
    for (const name of ["engine", "maps", "save", "undo"]) {
      const source = execFileSync(
        "git",
        ["show", `${revision}:${prefix}dist/${name}.mjs`],
        { cwd: project },
      );
      writeFileSync(join(baseline, `${name}.mjs`), source);
    }
  }
  const moduleURL = (name: string): string =>
    pathToFileURL(resolve(baseline, `${name}.mjs`)).href;
  const legacy = (await import(moduleURL("engine"))) as LegacyEngine;
  const undo = (await import(moduleURL("undo"))) as {
    createMoveUndo(): LegacyUndo;
  };
  const save = (await import(moduleURL("save"))) as LegacySave;
  const states = engineParity(legacy, undo.createMoveUndo);
  const saves = saveParity(save);
  console.log(
    `PASS: ${states} exact state comparisons across 60 deterministic battles, ${saves} save acceptance/state comparisons, and move undo parity.`,
  );
} finally {
  if (temporary) rmSync(temporary, { recursive: true, force: true });
}
