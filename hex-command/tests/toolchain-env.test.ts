import { test } from "vitest";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, delimiter } from "node:path";
import {
  installToolchainShell,
  toolchainCommand,
  toolchainEnvironment,
} from "../.agents/skills/rust-toolchain/scripts/toolchain-env";

function fixture(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "hex-toolchain-"));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("Fresh agent shells discover existing Rust homes and preserve explicit overrides", () => {
  fixture((root) => {
    mkdirSync(join(root, ".cargo/bin"), { recursive: true });
    mkdirSync(join(root, ".rustup/toolchains"), { recursive: true });
    mkdirSync(join(root, ".cache/wasm-pack"), { recursive: true });
    writeFileSync(join(root, ".cargo/bin/rustup"), "installed");
    const env = toolchainEnvironment({ PATH: "/usr/bin" }, root);
    assert.equal(env.CARGO_HOME, join(root, ".cargo"));
    assert.equal(env.RUSTUP_HOME, join(root, ".rustup"));
    assert.equal(env.PATH?.split(delimiter)[0], join(root, ".cargo/bin"));
    assert.equal(env.WASM_PACK_CACHE, join(root, ".cache/wasm-pack"));
    assert.deepEqual(toolchainEnvironment(env, root), env);
    const custom = toolchainEnvironment(
      {
        CARGO_HOME: "/chosen/cargo",
        RUSTUP_HOME: "/chosen/rustup",
        WASM_PACK_CACHE: "/chosen/cache",
      },
      root,
    );
    assert.equal(custom.CARGO_HOME, "/chosen/cargo");
    assert.equal(custom.RUSTUP_HOME, "/chosen/rustup");
    assert.equal(custom.WASM_PACK_CACHE, "/chosen/cache");
  });
});

test("An ordinary machine without workspace tooling retains its environment", () => {
  fixture((root) => {
    const env = { PATH: "/usr/bin", CARGO_BUILD_JOBS: "8" };
    assert.deepEqual(toolchainEnvironment(env, root), env);
  });
});

test("A fresh worktree uses only the cached wasm-pack version pinned by its package", () => {
  fixture((root) => {
    const project = join(root, "checkout");
    mkdirSync(join(project, "node_modules/wasm-pack"), { recursive: true });
    writeFileSync(
      join(project, "node_modules/wasm-pack/package.json"),
      JSON.stringify({ version: "0.15.0" }),
    );
    const cached = join(root, ".cache/toolchains/wasm-pack/0.15.0/wasm-pack");
    mkdirSync(join(cached, ".."), { recursive: true });
    writeFileSync(cached, "cached binary");
    assert.equal(toolchainCommand("wasm-pack", project, root), cached);
    writeFileSync(
      join(project, "node_modules/wasm-pack/package.json"),
      JSON.stringify({ version: "0.16.0" }),
    );
    assert.equal(toolchainCommand("wasm-pack", project, root), "wasm-pack");
  });
});

test("Shell setup is idempotent, loads before bashrc's early return and preserves user wrappers", () => {
  fixture((root) => {
    const home = join(root, "home");
    mkdirSync(home);
    writeFileSync(join(home, ".bashrc"), "return\n");
    mkdirSync(join(root, ".cargo/bin"), { recursive: true });
    writeFileSync(join(root, ".cargo/bin/cargo"), "installed");
    mkdirSync(join(home, ".local/bin"), { recursive: true });
    const custom = join(home, ".local/bin/cargo");
    writeFileSync(custom, "user wrapper");
    installToolchainShell(home, root, root);
    const profile = readFileSync(join(home, ".profile"), "utf8");
    const bashrc = readFileSync(join(home, ".bashrc"), "utf8");
    assert(bashrc.indexOf("toolchain-env.sh") < bashrc.indexOf("return"));
    installToolchainShell(home, root, root);
    assert.equal(readFileSync(join(home, ".profile"), "utf8"), profile);
    assert.equal(readFileSync(join(home, ".bashrc"), "utf8"), bashrc);
    assert.equal(readFileSync(custom, "utf8"), "user wrapper");
  });
});
