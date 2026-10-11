/** Generate typed bindings to the Rust-generated declaration boundary. Unsupported
 * shapes and runtime exports fail the build. Only DTO types cross this boundary.
 * Discriminated unions retain their JavaScript tags and exhaustive OCaml cases.
 */
import ts from "typescript";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
/** Only deliberately opaque Rust DTOs need a selection override. */
export interface BindingSelection {
  opaque?: readonly string[];
}
const frontendSelection: BindingSelection = {
  opaque: ["CommandError", "SavedPlanner", "PlanningSnapshot"],
};

/** Generate implementation and private interface without writing files. Keeping
 * compiler I/O outside this function lets small synthetic APIs test ABI decisions.
 */
export function generateBindings(
  program: ts.Program,
  file: string,
  selection: BindingSelection = frontendSelection,
): { implementation: string; intf: string } {
  const opaque = new Set(selection.opaque ?? []);
  const checker = program.getTypeChecker();
  const source = program.getSourceFile(file)!;
  const actual = (symbol: ts.Symbol) =>
    symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
  const exports = checker.getExportsOfModule(checker.getSymbolAtLocation(source)!).map((symbol) => {
    const target = actual(symbol);
    if (!(target.flags & ts.SymbolFlags.Type) || target.flags & ts.SymbolFlags.Value)
      throw new Error(`Only Rust DTO type exports are supported: ${symbol.name}`);
    return { symbol, type: checker.getDeclaredTypeOfSymbol(target) };
  });
  const snake = (name: string) => name.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
  const reserved = new Set([
    "type",
    "method",
    "object",
    "class",
    "open",
    "default",
    "val",
    "mutable",
    "done",
    "end",
    "begin",
    "effect",
    "match",
    "to",
    "from",
    "as",
    "assert",
    "and",
    "constraint",
    "do",
    "downto",
    "else",
    "exception",
    "external",
    "false",
    "for",
    "fun",
    "function",
    "if",
    "in",
    "include",
    "inherit",
    "initializer",
    "lazy",
    "let",
    "module",
    "new",
    "nonrec",
    "of",
    "rec",
    "sig",
    "struct",
    "then",
    "true",
    "try",
    "val",
    "virtual",
    "when",
    "while",
    "with",
    "mod",
    "land",
    "lor",
    "lxor",
    "lsl",
    "lsr",
    "asr",
  ]);
  const label = (name: string) => (reserved.has(name) ? `${name}_` : name);
  const names = new Map<ts.Type, string>();
  const definitions = new Map<string, string>();
  const enums = new Map<string, { name: string; values: string[] }>();
  const enumKey = (type: ts.Type) => {
    const parts = type.isUnion() ? type.types : [type];
    return parts.every((t) => t.isStringLiteral())
      ? parts
          .map((t) => (t as ts.StringLiteralType).value)
          .sort()
          .join("\0")
      : undefined;
  };
  for (const { symbol, type } of exports) {
    if (type.flags & ts.TypeFlags.Any) continue;
    const name = snake(symbol.name);
    names.set(type, name);
    if (opaque.has(symbol.name)) definitions.set(name, `type ${name}`);
    else {
      const key = enumKey(type);
      if (key && type.isUnion()) enums.set(key, { name, values: key.split("\0") });
    }
  }
  function fieldType(prop: ts.Symbol): ts.Type {
    return checker.getTypeOfSymbolAtLocation(
      prop,
      prop.valueDeclaration ?? prop.declarations?.[0] ?? source,
    );
  }
  function fields(type: ts.Type, hint: string, omit?: string): string {
    const props = checker.getPropertiesOfType(type).filter((p) => p.name !== omit);
    // A tag-only JS object still needs an inline-record constructor. The ignored
    // optional property is never read; no wrapper or synthetic property is added.
    if (!props.length) return "ignored : unit option;";
    return props
      .map(
        (p) =>
          `${label(p.name)} : ${typeOf(fieldType(p), `${hint}_${snake(p.name)}`)}${label(p.name) !== p.name ? ` [@mel.as ${JSON.stringify(p.name)}]` : ""};`,
      )
      .join("\n  ");
  }
  function typeOf(type: ts.Type, hint: string): string {
    if (type.flags & ts.TypeFlags.Null) return "unit Js.Nullable.t";
    if (type.flags & ts.TypeFlags.Undefined) return "unit option";
    if (names.has(type) && definitions.has(names.get(type)!)) return names.get(type)!;
    const key = enumKey(type),
      enumeration = key ? enums.get(key) : undefined;
    if (enumeration) {
      definitions.set(enumeration.name, `type ${enumeration.name} = string`);
      return enumeration.name;
    }
    if (type.flags & ts.TypeFlags.StringLike) return "string";
    if (type.flags & ts.TypeFlags.NumberLike) return "float";
    if (type.flags & ts.TypeFlags.BooleanLike) return "bool";
    if (type.flags & ts.TypeFlags.Void) return "unit";
    if (type.flags & ts.TypeFlags.Unknown) {
      definitions.set("unknown", "type unknown");
      return "unknown";
    }
    if (type.isUnion()) {
      const base = type.types.filter(
        (t) => !(t.flags & (ts.TypeFlags.Null | ts.TypeFlags.Undefined)),
      );
      if (base.length !== type.types.length) {
        const value =
          base.length === 1
            ? typeOf(base[0]!, hint)
            : typeOf(checker.getNonNullableType(type), hint);
        return `${value}${type.types.some((t) => t.flags & ts.TypeFlags.Null) ? " Js.Nullable.t" : ""}${type.types.some((t) => t.flags & ts.TypeFlags.Undefined) ? " option" : ""}`;
      }
      if (base.every((t) => t.flags & ts.TypeFlags.StringLike)) return "string";
      if (base.every((t) => t.flags & ts.TypeFlags.BooleanLike)) return "bool";
      const tag = checker.getPropertiesOfType(type).find((p) => {
        const values = base.map((t) => {
          const prop = checker.getPropertyOfType(t, p.name);
          return prop ? fieldType(prop) : undefined;
        });
        return (
          values.every(
            (value) =>
              value && value.flags & (ts.TypeFlags.StringLiteral | ts.TypeFlags.BooleanLiteral),
          ) && new Set(values.map((value) => checker.typeToString(value!))).size === base.length
        );
      });
      if (!tag) throw new Error(`Unsupported union ${hint}: ${checker.typeToString(type)}`);
      const name = names.get(type) ?? snake(hint);
      names.set(type, name);
      definitions.set(name, "");
      const constructors = new Set<string>();
      const cases = base.map((t) => {
        const value = checker.typeToString(fieldType(checker.getPropertyOfType(t, tag.name)!));
        const suffix = value.replaceAll('"', "").replaceAll("-", "_");
        if (!/^[A-Za-z0-9_]+$/.test(suffix) || constructors.has(suffix))
          throw new Error(`Unsupported tag value ${hint}: ${value}`);
        constructors.add(suffix);
        return `| ${name[0]!.toUpperCase() + name.slice(1)}_${suffix} of {\n  ${fields(t, name, tag.name)}\n} [@mel.as ${value}]`;
      });
      definitions.set(
        name,
        `type[@mel.tag ${JSON.stringify(tag.name)}] ${name} =\n${cases.join("\n")}`,
      );
      return name;
    }
    if (checker.isArrayType(type))
      return `${typeOf(checker.getTypeArguments(type as ts.TypeReference)[0]!, `${hint}_item`)} array`;
    if (checker.isTupleType(type)) {
      const parts = checker.getTypeArguments(type as ts.TypeReference);
      return `(${parts.map((part, i) => typeOf(part, `${hint}_${i}`)).join(" * ")})`;
    }
    if (checker.getSignaturesOfType(type, ts.SignatureKind.Call).length)
      throw new Error(`Callbacks are not Rust DTOs: ${hint}`);
    const indices = checker.getIndexInfosOfType(type);
    if (indices.length === 1) return `${typeOf(indices[0]!.type, `${hint}_item`)} Js.Dict.t`;
    const name = names.get(type) ?? snake(hint);
    names.set(type, name);
    if (definitions.has(name)) return name;
    definitions.set(name, "");
    const props = checker.getPropertiesOfType(type);
    if (!props.length) throw new Error(`Unsupported object ${hint}: ${checker.typeToString(type)}`);
    // Homogeneous finite Record<K,V> maps are dictionaries, not a field per key.
    if (
      type.aliasSymbol?.name === "Record" ||
      (type.aliasSymbol?.name === "Immutable" &&
        type.aliasTypeArguments?.[0]?.aliasSymbol?.name === "Record")
    ) {
      const values = props.map(fieldType);
      if (!values.every((t) => checker.typeToString(t) === checker.typeToString(values[0]!)))
        throw new Error(`Heterogeneous dictionary ${hint}`);
      definitions.delete(name);
      const result = `${typeOf(values[0]!, `${hint}_item`)} Js.Dict.t`;
      names.delete(type);
      return result;
    }
    definitions.set(name, `type ${name} = {\n  ${fields(type, name)}\n}`);
    return name;
  }
  for (const { symbol, type } of exports) {
    if (!opaque.has(symbol.name)) typeOf(type, snake(symbol.name));
  }
  const declarations = [...definitions.values()]
    .filter(Boolean)
    .map((d, i) => (i ? d.replace(/^type/, "and") : d))
    .join("\n");
  const constants = [...enums.values()].flatMap((e) =>
    e.values.map(
      (v) => `let ${e.name}_${snake(v.replaceAll("-", "_"))} : ${e.name} = ${JSON.stringify(v)}`,
    ),
  );
  const banner =
    '(* Generated by scripts/melange-bindings.ts. Do not edit. *)\n[@@@warning "-30-37"]\n';
  const implementation = banner + declarations + "\n\n" + constants.join("\n") + "\n\n\n";
  const privateTypes = declarations.replace(
    /(type|and)(\[@mel.tag [^\]]+\])? (\w+) =(\s*)(\{|string|\|)/g,
    (all, kw, attr, name, space, body) =>
      ["start", "command", "ai_options", "attack_query"].includes(name)
        ? all
        : `${kw}${attr ?? ""} ${name} = private${space}${body}`,
  );
  // Interface attributes are consumed by the compiler representation, not the PPX.
  const intf =
    banner +
    '[@@@alert "-unused"]\n' +
    privateTypes +
    "\n\n" +
    [...enums.values()]
      .flatMap((e) =>
        e.values.map((v) => `val ${e.name}_${snake(v.replaceAll("-", "_"))} : ${e.name}`),
      )
      .join("\n") +
    "\n\n\n";
  return { implementation, intf };
}

// Importing the generator for unit tests performs no compilation or writes.
if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  const config = ts.readConfigFile("tsconfig.json", ts.sys.readFile);
  const options = ts.parseJsonConfigFileContent(config.config, ts.sys, ".").options;
  const file = "scripts/frontend-boundary.ts";
  const ambient = ts
    .parseJsonConfigFileContent(config.config, ts.sys, ".")
    .fileNames.filter((name) => name.endsWith(".d.ts"));
  const program = ts.createProgram([file, ...ambient], options);
  const diagnostics = ts.getPreEmitDiagnostics(program);
  if (diagnostics.length)
    throw new Error(
      ts.formatDiagnosticsWithColorAndContext(diagnostics, {
        getCanonicalFileName: (path) => path,
        getCurrentDirectory: () => process.cwd(),
        getNewLine: () => "\n",
      }),
    );
  const { implementation, intf } = generateBindings(program, file);
  let stale = false;
  for (const [ext, content] of [
    ["ml", implementation],
    ["mli", intf],
  ]) {
    const path = `frontend/Domain.${ext}`;
    if (process.argv.includes("--check")) {
      try {
        if (readFileSync(path, "utf8") !== content) stale = true;
      } catch {
        stale = true;
      }
    } else writeFileSync(path, content!);
  }
  if (stale) {
    console.error("Melange bindings are stale. Run npm run bindings:frontend.");
    process.exit(1);
  }
}
