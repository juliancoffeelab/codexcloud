import { test } from "vitest";
import assert from "node:assert/strict";
import ts from "typescript";
import { generateBindings, type BindingSelection } from "../scripts/melange-bindings";

/** These declarations are tiny compiler inputs, never files or Git fixtures. */
function bindings(source: string, selection?: BindingSelection) {
  const file = "/bindings/rust/pkg/hex_command.d.ts";
  const globals = "/bindings/globals.d.ts";
  const files = new Map([
    [file, source],
    [
      globals,
      `interface Array<T> { length: number; [index: number]: T }
       interface ReadonlyArray<T> { readonly length: number; readonly [index: number]: T }
       interface Object {} interface Function {} interface CallableFunction {}
       interface NewableFunction {} interface IArguments {} interface RegExp {}
       interface String {} interface Number {} interface Boolean {}`,
    ],
  ]);
  const host: ts.CompilerHost = {
    getSourceFile: (name, languageVersion) => {
      const text = files.get(name);
      return text === undefined
        ? undefined
        : ts.createSourceFile(name, text, languageVersion, true);
    },
    getDefaultLibFileName: () => globals,
    writeFile: () => {},
    getCurrentDirectory: () => "/bindings",
    getDirectories: () => [],
    fileExists: (name) => files.has(name),
    readFile: (name) => files.get(name),
    getCanonicalFileName: (name) => name,
    useCaseSensitiveFileNames: () => true,
    getNewLine: () => "\n",
  };
  const program = ts.createProgram([file, globals], { noLib: true, strict: true }, host);
  assert.deepEqual(
    ts.getPreEmitDiagnostics(program),
    [],
    "Synthetic DTO declarations must be valid",
  );
  return generateBindings(program, file, selection);
}

test("Rust command cases preserve tags, nullable optional payloads and tag-only commands", () => {
  const { implementation, intf } = bindings(`
    export type Command = { kind: "move"; unit: number; note?: string | null }
                        | { kind: "end-turn" };
  `);
  assert.match(implementation, /type\[@mel.tag "kind"\] command =/);
  assert.match(
    implementation,
    /Command_move of \{\n  unit : float;\n  note : string Js.Nullable.t option;\n\} \[@mel.as "move"\]/,
  );
  assert.match(
    implementation,
    /Command_end_turn of \{\n  ignored : unit option;\n\} \[@mel.as "end-turn"\]/,
  );
  assert.doesNotMatch(implementation, /kind :/);
  assert.doesNotMatch(intf, /command = private/);
});

test("Published DTOs expose closed enum values and readonly nested tuples through private types", () => {
  const { implementation, intf } = bindings(`
    export type Side = "player" | "enemy";
    export interface Published {
      readonly side: Side;
      readonly path: ReadonlyArray<readonly [number, string | null]>;
      readonly next?: Side | null;
    }
  `);
  assert.match(intf, /(?:type|and) side = private string/);
  assert.match(intf, /val side_enemy : side/);
  assert.match(intf, /val side_player : side/);
  assert.match(intf, /(?:type|and) published = private \{/);
  assert.match(implementation, /path : \(float \* string Js.Nullable.t\) array;/);
  assert.match(implementation, /next : side Js.Nullable.t option;/);
});

test("An explicitly opaque Rust DTO hides an unsupported internal shape", () => {
  const { implementation, intf } = bindings(`export type Planner = (seed: number) => number;`, {
    opaque: ["Planner"],
  });
  assert.match(implementation, /type planner\n/);
  assert.match(intf, /type planner\n/);
  assert.doesNotMatch(implementation, /seed|float|external/);
});

test("Unsupported DTO unions and callback fields fail before generating bindings", () => {
  assert.throws(() => bindings(`export type Value = string | number;`), /Unsupported union value/);
  assert.throws(
    () => bindings(`export interface Published { callback: () => void }`),
    /Callbacks are not Rust DTOs: published_callback/,
  );
});

test("Runtime functions and classes cannot cross the DTO-only boundary", () => {
  for (const source of [
    `export declare function helper(): number;`,
    `export declare class LiveGame { revision: number }`,
  ]) {
    assert.throws(() => bindings(source), /Only Rust DTO type exports are supported/);
  }
});
