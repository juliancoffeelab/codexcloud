import { readFileSync, readdirSync, realpathSync } from "node:fs";
import { dirname, extname, isAbsolute, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import ts from "typescript";
import { createHash } from "node:crypto";

/** Check the entire package, including unused files and type-only dependencies. */
export function checkMigrationBoundary(directory) {
  const root = realpathSync(directory);
  const failures = [];
  const inside = (path) => {
    const rel = relative(root, path);
    return rel !== ".." && !rel.startsWith("../") && !isAbsolute(rel);
  };
  const fail = (file, message) => failures.push(`${relative(root, file)}: ${message}`);
  const manifest = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8"));
  for (const field of [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
    "bundledDependencies",
    "bundleDependencies",
    "imports",
  ])
    if (manifest[field] && Object.keys(manifest[field]).length)
      fail(resolve(root, "package.json"), `${field} must be empty`);
  const configFile = resolve(root, "tsconfig.json");
  const config = ts.readConfigFile(configFile, ts.sys.readFile);
  if (config.error) fail(configFile, "invalid TypeScript configuration");
  else {
    const options = config.config;
    if (
      options.extends ||
      options.references?.length ||
      options.compilerOptions?.paths ||
      options.compilerOptions?.baseUrl ||
      options.compilerOptions?.typeRoots ||
      options.compilerOptions?.types?.length
    )
      fail(configFile, "configuration must be standalone, without external type resolution");
  }
  const readersFile = resolve(root, "src/formats/readers.json");
  const readers = JSON.parse(readFileSync(readersFile, "utf8"));
  const fingerprint = createHash("sha256");
  for (const source of readers.sources) {
    const path = resolve(root, "src/formats", source);
    if (!inside(path)) {
      fail(readersFile, "reader source escapes package");
      continue;
    }
    fingerprint.update(source).update(readFileSync(path));
  }
  if (fingerprint.digest("hex") !== readers.hash)
    fail(readersFile, "frozen types changed; run npm run migrations:generate");
  function reference(file, value) {
    if (!value.startsWith("./") && !value.startsWith("../")) {
      fail(file, `external dependency ${JSON.stringify(value)}`);
      return;
    }
    const path = resolve(dirname(file), value);
    if (!inside(path)) fail(file, `dependency escapes package: ${value}`);
    // Every file is checked below; reject symlink escapes even for extensionless imports.
    for (const candidate of [path, `${path}.ts`, `${path}.json`, resolve(path, "index.ts")])
      if (ts.sys.fileExists(candidate) && !inside(realpathSync(candidate)))
        fail(file, `dependency resolves outside package: ${value}`);
  }
  function walk(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const file = resolve(directory, entry.name);
      if (entry.isSymbolicLink()) {
        fail(file, "symlinks are not allowed in the frozen package");
        continue;
      }
      if (entry.isDirectory()) {
        walk(file);
        continue;
      }
      if (![".ts", ".js", ".mjs", ".cjs", ".mts", ".cts", ".tsx", ".jsx"].includes(extname(file)))
        continue;
      const source = ts.createSourceFile(
        file,
        readFileSync(file, "utf8"),
        ts.ScriptTarget.Latest,
        true,
      );
      for (const ref of source.referencedFiles) reference(file, ref.fileName);
      for (const ref of source.typeReferenceDirectives)
        fail(file, `external type reference ${ref.fileName}`);
      function module(node) {
        if (node && ts.isStringLiteralLike(node)) reference(file, node.text);
        else fail(file, "dependency must use a literal relative path");
      }
      function visit(node) {
        if (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) {
          if (node.moduleSpecifier) module(node.moduleSpecifier);
        } else if (ts.isImportTypeNode(node)) {
          module(ts.isLiteralTypeNode(node.argument) ? node.argument.literal : undefined);
        } else if (ts.isImportEqualsDeclaration(node)) {
          if (ts.isExternalModuleReference(node.moduleReference))
            module(node.moduleReference.expression);
        } else if (
          ts.isCallExpression(node) &&
          (node.expression.kind === ts.SyntaxKind.ImportKeyword ||
            (ts.isIdentifier(node.expression) && node.expression.text === "require") ||
            (ts.isPropertyAccessExpression(node.expression) &&
              node.expression.name.text === "require"))
        ) {
          module(node.arguments[0]);
        } else if (ts.isModuleDeclaration(node) && ts.isStringLiteral(node.name)) {
          fail(file, "ambient external modules are not allowed");
        }
        ts.forEachChild(node, visit);
      }
      visit(source);
    }
  }
  walk(root);
  if (failures.length) throw new Error(`Migration isolation failed:\n${failures.join("\n")}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const started = performance.now();
  checkMigrationBoundary(fileURLToPath(new URL("../packages/save-migrations", import.meta.url)));
  console.log(`Migration boundary passed (${Math.round(performance.now() - started)} ms)`);
}
