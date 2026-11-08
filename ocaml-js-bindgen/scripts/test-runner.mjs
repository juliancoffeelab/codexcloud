import assert from "node:assert/strict";
import {
  mkdtempSync,
  readFileSync,
  rmSync,
  existsSync,
  mkdirSync,
  writeFileSync,
} from "node:fs";
import { resolve } from "node:path";
import { root, sourceFingerprint } from "./test-inputs.mjs";
import { Worker } from "node:worker_threads";
import { workerResult } from "./test-worker.mjs";
import { runChecks } from "./run-tests.mjs";

const scratch = mkdtempSync(resolve(root, "tmp/runner-"));
try {
  const inputs = resolve(scratch, "inputs");
  for (const dir of [
    "crates",
    "tests",
    "demos",
    "scripts",
    "examples/rust-to-ocaml/rust",
    "examples/rust-to-ocaml/dune.lock",
  ])
    mkdirSync(resolve(inputs, dir), { recursive: true });
  for (const file of ["Cargo.toml", "Cargo.lock", "package.json"])
    writeFileSync(resolve(inputs, file), "fixture");
  for (const extension of [
    "res",
    "resi",
    "ml",
    "html",
    "css",
    "ndjson",
    "mk",
    "in",
    "json",
    "dune",
    "png",
  ]) {
    const path = resolve(inputs, `tests/fixture.${extension}`);
    writeFileSync(path, "before");
    const fingerprint = sourceFingerprint(inputs);
    writeFileSync(path, "after");
    assert.notEqual(
      sourceFingerprint(inputs),
      fingerprint,
      `${extension} edit did not invalidate preparation`,
    );
  }
  for (const name of [
    "Makefile",
    "demos/project/dune-project",
    "demos/project/dune-workspace",
    "examples/rust-to-ocaml/dune-project",
    "examples/rust-to-ocaml/Frontend.ml",
  ]) {
    const file = resolve(inputs, name);
    mkdirSync(resolve(file, ".."), { recursive: true });
    writeFileSync(file, "before");
    const fingerprint = sourceFingerprint(inputs);
    writeFileSync(file, "after");
    assert.notEqual(
      sourceFingerprint(inputs),
      fingerprint,
      `${name} edit did not invalidate preparation`,
    );
  }
  const rescriptProject = resolve(inputs, "tests/rescript-project");
  mkdirSync(resolve(rescriptProject, "lib"), { recursive: true });
  writeFileSync(resolve(rescriptProject, "rescript.json"), "configuration");
  const output = resolve(rescriptProject, "lib/Frontend.js");
  writeFileSync(output, "first compile");
  const sourceHash = sourceFingerprint(inputs);
  writeFileSync(output, "second compile");
  assert.equal(
    sourceFingerprint(inputs),
    sourceHash,
    "compiler output invalidated preparation",
  );
  writeFileSync(
    resolve(rescriptProject, "rescript.json"),
    "changed configuration",
  );
  assert.notEqual(
    sourceFingerprint(inputs),
    sourceHash,
    "ReScript config did not invalidate preparation",
  );
  const unexpected = resolve(scratch, "unexpected");
  let warned = false;
  const originalWarn = console.warn;
  console.warn = () => {
    warned = true;
  };
  try {
    await runChecks(
      [["slow successful check", process.execPath, ["-e", "process.exit(0)"]]],
      { budgetMs: 1 },
    );
    assert(warned, "the shared advisory budget did not warn");
  } finally {
    console.warn = originalWarn;
  }
  await assert.rejects(
    runChecks([
      ["failure regression", process.execPath, ["-e", "process.exit(7)"]],
      [
        "must not run",
        process.execPath,
        [
          "-e",
          `require('node:fs').writeFileSync(${JSON.stringify(unexpected)},'bad')`,
        ],
      ],
    ]),
    /failure regression failed \(7\)/,
  );
  assert(!existsSync(unexpected));
  const worker = new Worker("setInterval(() => {}, 60000)", { eval: true });
  await assert.rejects(
    workerResult(worker, 50),
    /worker result deadline exceeded/,
  );
  assert.equal(worker.threadId, -1, "timed-out worker was not terminated");
  console.log(
    "runner: shared budget warns; failures stop subsequent checks; stalled workers remain bounded",
  );
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
