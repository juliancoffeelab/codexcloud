/** Generate typed bindings to the Rust-generated declaration boundary. Unsupported
 * shapes fail the build; getters and methods are called directly, without shims.
 * Discriminated unions retain their JavaScript tags and exhaustive OCaml cases.
 */
import ts from "typescript";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
/** Selection policy for this project's restricted foreign boundary. Types and
 * fields are always resolved through the checker, including parameter refinements.
 */
export interface BindingSelection {
  opaque?: readonly string[];
  selected?: Record<string, string[]>;
  aliases?: Record<string, string>;
  equivalentTypes?: readonly string[];
  members?: Record<string, Record<string, string>>;
  refinements?: Record<string, Record<string, string>>;
  dictionaries?: Record<string, string>;
  factories?: readonly string[];
  staticFactory?: { owner: string; member: string; binding: string };
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
  const selected = selection.selected ?? {};
  const members = selection.members ?? {};
  const checker = program.getTypeChecker();
  const source = program.getSourceFile(file)!;
  const actual = (symbol: ts.Symbol) =>
    symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
  const exports = checker
    .getExportsOfModule(checker.getSymbolAtLocation(source)!)
    .map((symbol) => ({
      symbol,
      actual: actual(symbol),
      type: checker.getDeclaredTypeOfSymbol(actual(symbol)),
    }));
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
  const externals: string[] = [];
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
  // Engine commands expose mutable Unit/Tile; published views expose their deeply
  // readonly counterparts. Both have the same field representation and are only
  // readable through private OCaml records.
  for (const [a, b] of Object.entries(selection.aliases ?? {})) {
    names.set(exports.find((e) => e.symbol.name === a)!.type, snake(b!));
  }
  function canonical(type: ts.Type): string | undefined {
    const chosen = Object.keys(selected).find((name) => snake(name) === names.get(type));
    if (chosen) return chosen;
    const root = type.aliasSymbol?.name === "Immutable" ? type.aliasTypeArguments?.[0] : type;
    return root?.symbol?.name;
  }
  function properties(type: ts.Type): ts.Symbol[] {
    const fields = selected[canonical(type) ?? ""];
    return fields
      ? fields.map((name) => {
          const property = checker.getPropertyOfType(type, name);
          if (!property) throw new Error(`Missing selected field ${canonical(type)}.${name}`);
          return property;
        })
      : checker.getPropertiesOfType(type);
  }
  function fieldType(prop: ts.Symbol): ts.Type {
    return checker.getTypeOfSymbolAtLocation(
      prop,
      prop.valueDeclaration ?? prop.declarations?.[0] ?? source,
    );
  }
  function fields(type: ts.Type, hint: string, omit?: string): string {
    const props = properties(type).filter((p) => p.name !== omit);
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
    if (type.flags & (ts.TypeFlags.Object | ts.TypeFlags.Union) && !checker.isArrayType(type)) {
      const equivalent = exports.find(
        (e) =>
          (selection.equivalentTypes ?? []).includes(e.symbol.name) &&
          checker.isTypeAssignableTo(type, e.type) &&
          checker.isTypeAssignableTo(e.type, type),
      );
      if (equivalent) type = equivalent.type;
    }
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
    const reference = type as ts.TypeReference;
    if (reference.target?.symbol?.name === "Promise")
      return `${typeOf(checker.getTypeArguments(reference)[0]!, `${hint}_value`)} Js.Promise.t`;
    if (["Map", "ReadonlyMap"].includes(reference.target?.symbol?.name ?? "")) {
      const [key, value] = checker.getTypeArguments(reference);
      return `(${typeOf(key!, `${hint}_key`)}, ${typeOf(value!, `${hint}_value`)}) Js.Map.t`;
    }
    if (checker.getSignaturesOfType(type, ts.SignatureKind.Call).length)
      throw new Error(`Bind callbacks explicitly: ${hint}`);
    const indices = checker.getIndexInfosOfType(type);
    if (indices.length === 1) return `${typeOf(indices[0]!.type, `${hint}_item`)} Js.Dict.t`;
    const root = canonical(type);
    const name = root && selected[root] ? snake(root) : (names.get(type) ?? snake(hint));
    names.set(type, name);
    if (definitions.has(name)) return name;
    definitions.set(name, "");
    const props = properties(type);
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
  function modulePath(symbol: ts.Symbol): string {
    const path = symbol.valueDeclaration?.getSourceFile().fileName.replaceAll("\\", "/");
    if (!path?.includes("rust/pkg/"))
      throw new Error(`Foreign runtime must come from raw Rust WASM: ${symbol.name}`);
    return "@wasm/hex_command";
  }
  const signatureTypes = (
    sig: ts.Signature,
    hint: string,
    overrides: Record<string, string> = {},
  ) => {
    const args = sig.parameters.flatMap((p) => {
      const value = fieldType(p),
        name = overrides[p.name];
      if (!name) {
        if (
          checker.isTupleType(value) &&
          (p.valueDeclaration as ts.ParameterDeclaration)?.dotDotDotToken
        )
          return checker
            .getTypeArguments(value as ts.TypeReference)
            .map((t, i) => typeOf(t, `${hint}_${i}`));
        return typeOf(value, `${hint}_${snake(p.name)}`);
      }
      const replacement = exports.find((e) => e.symbol.name === name)!.type;
      if (!checker.isTypeAssignableTo(replacement, value))
        throw new Error(`Unsafe parameter refinement ${hint}.${p.name}: ${name}`);
      return typeOf(replacement, snake(name));
    });
    return [
      ...(args.length ? args : ["unit"]),
      typeOf(checker.getReturnTypeOfSignature(sig), `${hint}_result`),
    ];
  };
  for (const { symbol, actual: target, type } of exports) {
    if (target.flags & ts.SymbolFlags.Type || symbol.name === "LiveGame") {
      if (!opaque.has(symbol.name)) typeOf(type, snake(symbol.name));
      continue;
    }
    const value = checker.getTypeOfSymbolAtLocation(target, target.valueDeclaration!);
    const signatures = checker.getSignaturesOfType(value, ts.SignatureKind.Call);
    if (signatures.length > 1) throw new Error(`Overloaded export ${symbol.name}`);
    const name = snake(symbol.name);
    const dictionary = selection.dictionaries?.[symbol.name];
    let dictionaryType: string | undefined;
    if (dictionary) {
      const item = exports.find((e) => e.symbol.name === dictionary)!.type;
      for (const p of checker.getPropertiesOfType(value)) {
        // Check the selected read-only shape, including immutable collections.
        for (const field of properties(item)) {
          const actualField = checker.getPropertyOfType(fieldType(p), field.name);
          if (!actualField || !checker.isTypeAssignableTo(fieldType(actualField), fieldType(field)))
            throw new Error(`Unsafe dictionary ${symbol.name}.${p.name}.${field.name}`);
        }
      }
      dictionaryType = `${typeOf(item, snake(dictionary))} Js.Dict.t`;
    }
    const signature =
      dictionaryType ??
      (signatures.length
        ? signatureTypes(signatures[0]!, name, selection.refinements?.[name]).join(" -> ")
        : typeOf(value, name));
    externals.push(
      `external ${name} : ${signature} = ${JSON.stringify(target.name)} [@@mel.module ${JSON.stringify(modulePath(target))}]`,
    );
  }
  // Public names remain compact; member existence and full signatures are checked.
  for (const [owner, bindings] of Object.entries(members)) {
    const type = exports.find((e) => e.symbol.name === owner)!.type;
    for (const [name, member] of Object.entries(bindings)) {
      const property = checker.getPropertyOfType(type, member);
      if (!property) throw new Error(`Missing ${owner}.${member}`);
      const value = fieldType(property),
        signatures = checker.getSignaturesOfType(value, ts.SignatureKind.Call);
      if (signatures.length > 1) throw new Error(`Overloaded member ${owner}.${member}`);
      const args = signatures.length
        ? signatureTypes(signatures[0]!, name, selection.refinements?.[name])
        : [typeOf(value, `${name}_result`)];
      // Zero-argument instance methods need no dummy argument with mel.send.
      if (signatures.length && !signatures[0]!.parameters.length) args.shift();
      externals.push(
        `external ${name} : ${[snake(owner), ...args].join(" -> ")} = ${JSON.stringify(member)} [@@mel.${signatures.length ? "send" : "get"}]`,
      );
    }
  }
  if (selection.staticFactory) {
    const { owner, member, binding } = selection.staticFactory;
    const target = exports.find((e) => e.symbol.name === owner)!.actual;
    const staticType = checker.getTypeOfSymbolAtLocation(target, target.valueDeclaration!);
    const method = checker.getPropertyOfType(staticType, member);
    if (!method) throw new Error(`Missing ${owner}.${member}`);
    const signatures = checker.getSignaturesOfType(fieldType(method), ts.SignatureKind.Call);
    if (signatures.length !== 1) throw new Error(`Unsupported static factory ${owner}.${member}`);
    externals.push(
      `external ${binding} : ${signatureTypes(signatures[0]!, binding).join(" -> ")} = ${JSON.stringify(member)} [@@mel.module ${JSON.stringify(modulePath(target))}] [@@mel.scope ${JSON.stringify(owner)}]`,
    );
  }
  for (const name of selection.factories ?? []) {
    const type = exports.find((e) => e.symbol.name === name)!.type;
    const args = properties(type).map(
      (p) => `${label(p.name)}:${typeOf(fieldType(p), `${snake(name)}_${snake(p.name)}`)}`,
    );
    externals.push(
      `external make_${snake(name)} : ${[...args, "unit", snake(name)].join(" -> ")} = "" [@@mel.obj]`,
    );
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
  const implementation =
    banner + declarations + "\n\n" + constants.join("\n") + "\n\n" + externals.join("\n") + "\n";
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
    "\n\n" +
    externals.map((e) => e.replace(/^external /, "val ").split(" = ")[0]).join("\n") +
    "\n";
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
