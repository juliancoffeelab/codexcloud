import { test } from "vitest";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

test("Routine simulation builds stage an incremental binary without PGO tooling or training", () => {
  const root = mkdtempSync(join(tmpdir(), "hex-native-build-"));
  try {
    const home = join(root, "cargo");
    const target = join(root, "target");
    const log = join(root, "calls.jsonl");
    mkdirSync(join(home, "bin"), { recursive: true });
    writeFileSync(
      join(home, "bin/cargo"),
      `#!${process.execPath}
const fs = require("node:fs"), path = require("node:path");
const args = process.argv.slice(2);
fs.appendFileSync(process.env.BUILD_TEST_LOG, JSON.stringify({args, flags: process.env.CARGO_ENCODED_RUSTFLAGS}) + "\\n");
const legacy = args.includes("rust/legacy/Cargo.toml");
const output = path.join(process.env.CARGO_TARGET_DIR, legacy ? "release" : "simulation", legacy ? "hex-command-legacy" : "hex-command-sim");
fs.mkdirSync(path.dirname(output), {recursive:true});
fs.writeFileSync(output, legacy ? "frozen" : "incremental");
`,
      { mode: 0o755 },
    );
    for (const tool of ["rustc", "rustup"]) {
      writeFileSync(join(home, "bin", tool), "#!/bin/sh\nexit 99\n", { mode: 0o755 });
    }
    const result = spawnSync(process.execPath, ["--import", "tsx", "scripts/build-simulation.ts"], {
      encoding: "utf8",
      env: {
        ...process.env,
        CARGO_HOME: home,
        CARGO_TARGET_DIR: target,
        BUILD_TEST_LOG: log,
        HEX_COMMAND_SIM_PGO: "0",
        CARGO_ENCODED_RUSTFLAGS: "",
        RUSTFLAGS: "",
        RUSTC: join(home, "bin/rustc"),
      },
    });
    assert.equal(result.status, 0, result.stderr);
    const calls = readFileSync(log, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line) as { args: string[]; flags: string });
    assert.equal(calls.length, 2);
    assert(calls[0] && calls[1]);
    assert(calls[0].args.includes("rust/legacy/Cargo.toml"));
    assert(calls[1].args.includes("rust/Cargo.toml"));
    assert.deepEqual(calls[1].args.slice(-2), ["--profile", "simulation"]);
    assert(!calls[1].args.includes("--release"));
    assert(!calls[1].flags.includes("profile-"));
    assert.equal(readFileSync(join(target, "release/hex-command-sim"), "utf8"), "incremental");
    assert.equal(readFileSync(join(target, "release/hex-command-legacy"), "utf8"), "frozen");
    assert(!existsSync(join(target, "ai-native")));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
