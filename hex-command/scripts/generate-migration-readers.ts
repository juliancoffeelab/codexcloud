/** Build frozen wire readers from migration-owned types, never application types. */
import ts from "typescript";
import { readFileSync, writeFileSync, rmSync, mkdirSync } from "node:fs";
import { resolve, relative } from "node:path";
import { createHash } from "node:crypto";
import { SAVE_VERSIONS } from "../packages/save-migrations/src/formats/history";
const root = resolve("packages/save-migrations/src/formats");
// The scratch module imports only the frozen package; generated readers stay self-contained.
mkdirSync("tmp", { recursive: true });
const path = resolve("tmp/migration-reader-roots.ts");
const versions = SAVE_VERSIONS;
const roots = versions
  .map((v) => `export type Save${v} = Save<${v}>;`)
  .concat([1, 2, 3, 4].map((v) => `export type History${v} = History<${v}>;`))
  .concat([
    "export type Fragments = FragmentTypes;",
    "export type Native1 = NativeSave1;",
    "export type Javascript1 = JavascriptSave1;",
  ]);
writeFileSync(
  path,
  'import type {Save,History,Fragments as FragmentTypes,NativeSave1,JavascriptSave1} from "../packages/save-migrations/src/formats/history";\n' +
    roots.join("\n"),
);
try {
  const program = ts.createProgram([path], {
    strict: true,
    noUncheckedIndexedAccess: true,
    target: ts.ScriptTarget.ES2022,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    module: ts.ModuleKind.ESNext,
    skipLibCheck: true,
    resolveJsonModule: true,
    types: [],
  });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  if (diagnostics.length)
    throw new Error(
      ts.formatDiagnosticsWithColorAndContext(diagnostics, {
        getCurrentDirectory: () => process.cwd(),
        getCanonicalFileName: (f) => f,
        getNewLine: () => "\n",
      }),
    );
  const checker = program.getTypeChecker(),
    source = program.getSourceFile(path)!;
  type Shape = {
    kind: string;
    value?: string | number | boolean;
    members?: number[];
    element?: number;
    items?: number[];
    properties?: Record<string, { type: number; optional: boolean }>;
    index?: number;
  };
  const shapes: Shape[] = [];
  const seen = new Map<ts.Type, number>();
  function schema(type: ts.Type): number {
    const old = seen.get(type);
    if (old !== undefined) return old;
    const id = shapes.length;
    seen.set(type, id);
    const s: Shape = { kind: "pending" };
    shapes.push(s);
    const f = type.flags;
    if (f & ts.TypeFlags.Any)
      throw Error("any is not a frozen wire type: " + checker.typeToString(type));
    if (f & ts.TypeFlags.Unknown) s.kind = "unknown";
    else if (f & ts.TypeFlags.Never) s.kind = "never";
    else if (f & ts.TypeFlags.Undefined) s.kind = "undefined";
    else if (f & ts.TypeFlags.Null) s.kind = "null";
    else if (type.isStringLiteral() || type.isNumberLiteral()) {
      s.kind = "literal";
      s.value = type.value;
    } else if (f & ts.TypeFlags.BooleanLiteral) {
      s.kind = "literal";
      s.value = checker.typeToString(type) === "true";
    } else if (f & ts.TypeFlags.String) s.kind = "string";
    else if (f & ts.TypeFlags.Number) s.kind = "number";
    else if (f & ts.TypeFlags.Boolean) s.kind = "boolean";
    else if (type.isUnion()) {
      s.kind = "union";
      s.members = type.types.map(schema);
    } else if (checker.isTupleType(type)) {
      s.kind = "tuple";
      s.items = checker.getTypeArguments(type as ts.TypeReference).map(schema);
    } else if (checker.isArrayType(type)) {
      s.kind = "array";
      s.element = schema(checker.getTypeArguments(type as ts.TypeReference)[0]!);
    } else if (f & (ts.TypeFlags.Object | ts.TypeFlags.Intersection)) {
      s.kind = "object";
      s.properties = {};
      for (const p of checker.getPropertiesOfType(type)) {
        const declaration = p.valueDeclaration ?? p.declarations?.[0] ?? source;
        s.properties[p.name] = {
          type: schema(checker.getTypeOfSymbolAtLocation(p, declaration)),
          optional: !!(p.flags & ts.SymbolFlags.Optional),
        };
      }
      const index = checker.getIndexTypeOfType(type, ts.IndexKind.String);
      if (index) s.index = schema(index);
    } else throw Error("Unsupported frozen type " + checker.typeToString(type));
    return id;
  }
  const entries: Record<string, number> = {};
  for (const node of source.statements)
    if (ts.isTypeAliasDeclaration(node))
      entries[node.name.text] = schema(checker.getTypeAtLocation(node));
  // Fingerprint the exact frozen type inputs, so the fast boundary check detects stale readers.
  const files = program
    .getSourceFiles()
    .filter(
      (f) => f.fileName.startsWith(resolve(root, "..")) && !f.fileName.endsWith("reader-roots.ts"),
    )
    .map((f) => f.fileName)
    .sort();
  const hash = createHash("sha256");
  for (const f of files) hash.update(relative(root, f)).update(readFileSync(f));
  writeFileSync(
    resolve(root, "readers.json"),
    JSON.stringify({
      sources: files.map((f) => relative(root, f)),
      hash: hash.digest("hex"),
      entries,
      shapes,
    }) + "\n",
  );
  console.log("Generated frozen reader shapes:", shapes.length);
} finally {
  rmSync(path, { force: true });
}
