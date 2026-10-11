import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const project = fileURLToPath(new URL("../../../../", import.meta.url));

/** Setup exports do not survive into later agent shells or worktrees. */
export function toolchainEnvironment(
  original: NodeJS.ProcessEnv = process.env,
  workspace = "/workspace",
) {
  const env = { ...original };
  if (!env.CARGO_HOME && existsSync(join(workspace, ".cargo/bin/rustup"))) {
    env.CARGO_HOME = join(workspace, ".cargo");
  }
  if (!env.RUSTUP_HOME && existsSync(join(workspace, ".rustup/toolchains"))) {
    env.RUSTUP_HOME = join(workspace, ".rustup");
  }
  const paths = (env.PATH ?? "").split(delimiter).filter(Boolean);
  if (env.CARGO_HOME) {
    const bin = join(env.CARGO_HOME, "bin");
    env.PATH = [bin, ...paths.filter((path) => path !== bin)].join(delimiter);
  }
  if (!env.WASM_PACK_CACHE && existsSync(join(workspace, ".cache/wasm-pack"))) {
    env.WASM_PACK_CACHE = join(workspace, ".cache/wasm-pack");
  }
  if (env.CARGO_HOME === join(workspace, ".cargo")) {
    env.NODE_USE_ENV_PROXY ??= "1";
    env.CARGO_BUILD_JOBS ??= "4";
  }
  return env;
}

export function toolchainCommand(command: string, root = project, workspace = "/workspace") {
  if (command !== "wasm-pack") return command;
  const filename = process.platform === "win32" ? "wasm-pack.exe" : "wasm-pack";
  const local = join(root, "node_modules/wasm-pack/binary", filename);
  if (existsSync(local)) return local;
  const manifest = join(root, "node_modules/wasm-pack/package.json");
  if (existsSync(manifest)) {
    const { version } = JSON.parse(readFileSync(manifest, "utf8")) as { version: string };
    const cached = join(workspace, ".cache/toolchains/wasm-pack", version, filename);
    if (existsSync(cached)) return cached;
  }
  return command;
}

const quote = (value: string) => `'${value.replaceAll("'", "'\\''")}'`;

/** Persist defaults for direct shell commands, including non-login shells. */
export function installToolchainShell(home = homedir(), workspace = "/workspace", root = project) {
  const config = join(home, ".config/hex-command/toolchain-env.sh");
  const bin = join(home, ".local/bin");
  mkdirSync(dirname(config), { recursive: true });
  mkdirSync(bin, { recursive: true });
  const cargo = join(workspace, ".cargo");
  const rustup = join(workspace, ".rustup");
  const cache = join(workspace, ".cache/wasm-pack");
  writeFileSync(
    config,
    [
      "# Hex Command toolchain defaults; preserve explicit overrides.",
      `if [ -x ${quote(join(cargo, "bin/rustup"))} ]; then`,
      `  export CARGO_HOME="\${CARGO_HOME:-${cargo}}"`,
      `  export RUSTUP_HOME="\${RUSTUP_HOME:-${rustup}}"`,
      '  case ":$PATH:" in *":$CARGO_HOME/bin:"*) ;; *) export PATH="$CARGO_HOME/bin:$PATH" ;; esac',
      '  export NODE_USE_ENV_PROXY="${NODE_USE_ENV_PROXY:-1}"',
      '  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"',
      "fi",
      `export WASM_PACK_CACHE="\${WASM_PACK_CACHE:-${cache}}"`,
      "",
    ].join("\n"),
  );
  const hook = `[ ! -r ${quote(config)} ] || . ${quote(config)}`;
  const profiles = [".profile", ".bashrc", ".bash_profile", ".bash_login"];
  for (const name of profiles) {
    const file = join(home, name);
    if (!existsSync(file) && name !== ".profile" && name !== ".bashrc") continue;
    const current = existsSync(file) ? readFileSync(file, "utf8") : "";
    if (!current.includes(hook)) writeFileSync(file, `${hook}\n${current}`);
  }
  const marker = "# Managed by Hex Command toolchain setup";
  const wrapper = (name: string, target: string) => {
    const file = join(bin, name);
    if (existsSync(file) && !readFileSync(file, "utf8").includes(marker)) return;
    writeFileSync(file, `#!/bin/sh\n${marker}\n${hook}\nexec ${quote(target)} "$@"\n`, {
      mode: 0o755,
    });
  };
  for (const name of [
    "cargo",
    "rustc",
    "rustup",
    "rustfmt",
    "cargo-fmt",
    "cargo-clippy",
    "rustdoc",
  ]) {
    const target = join(cargo, "bin", name);
    if (existsSync(target)) wrapper(name, target);
  }
  const manifest = join(root, "node_modules/wasm-pack/package.json");
  const local = join(root, "node_modules/wasm-pack/binary/wasm-pack");
  if (existsSync(manifest) && existsSync(local)) {
    const { version } = JSON.parse(readFileSync(manifest, "utf8")) as { version: string };
    const cached = join(workspace, ".cache/toolchains/wasm-pack", version, "wasm-pack");
    mkdirSync(dirname(cached), { recursive: true });
    copyFileSync(local, cached);
    wrapper("wasm-pack", cached);
  }
}
