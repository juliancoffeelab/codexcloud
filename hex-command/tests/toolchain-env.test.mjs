import { test } from "vitest";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, delimiter } from "node:path";
import {
  installToolchainShell,
  toolchainEnvironment,
} from "../scripts/toolchain/toolchain-env.mjs";
function fixture(run) {
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
    mkdirSync(join(root, ".local/bin"), { recursive: true });
    mkdirSync(join(root, ".cache/npm"), { recursive: true });
    writeFileSync(join(root, ".local/bin/dune"), "installed");
    writeFileSync(join(root, ".cargo/bin/rustup"), "installed");
    const env = toolchainEnvironment({ PATH: "/usr/bin" }, root);
    assert.equal(env.CARGO_HOME, join(root, ".cargo"));
    assert.equal(env.RUSTUP_HOME, join(root, ".rustup"));
    assert.equal(env.PATH?.split(delimiter)[0], join(root, ".cargo/bin"));
    assert.equal(env.DUNE_JOBS, "4");
    assert.equal(env.npm_config_cache, join(root, ".cache/npm"));
    assert.deepEqual(toolchainEnvironment(env, root), env);
    const custom = toolchainEnvironment(
      {
        CARGO_HOME: "/chosen/cargo",
        RUSTUP_HOME: "/chosen/rustup",
        RUSTUP_TOOLCHAIN: "nightly",
        DUNE_JOBS: "2",
        CARGO_BUILD_JOBS: "3",
        NPM_CONFIG_CACHE: "/chosen/npm",
      },
      root,
    );
    assert.equal(custom.CARGO_HOME, "/chosen/cargo");
    assert.equal(custom.RUSTUP_HOME, "/chosen/rustup");
    assert.equal(custom.RUSTUP_TOOLCHAIN, "nightly");
    assert.equal(custom.DUNE_JOBS, "2");
    assert.equal(custom.CARGO_BUILD_JOBS, "3");
    assert.equal(custom.npm_config_cache, undefined);
    assert.equal(custom.NPM_CONFIG_CACHE, "/chosen/npm");
  });
});
test("An ordinary machine without workspace tooling retains its environment", () => {
  fixture((root) => {
    const env = { PATH: "/usr/bin", CARGO_BUILD_JOBS: "8" };
    assert.deepEqual(toolchainEnvironment(env, root), env);
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
