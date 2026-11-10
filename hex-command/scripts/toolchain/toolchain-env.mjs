import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
const project = fileURLToPath(new URL("../../", import.meta.url));
function toolchainEnvironment(original = process.env, workspace = "/workspace") {
  const env = { ...original };
  if (!env.CARGO_HOME && existsSync(join(workspace, ".cargo/bin/rustup"))) {
    env.CARGO_HOME = join(workspace, ".cargo");
  }
  if (!env.RUSTUP_HOME && existsSync(join(workspace, ".rustup/toolchains"))) {
    env.RUSTUP_HOME = join(workspace, ".rustup");
  }
  if (!env.XDG_CACHE_HOME && existsSync(join(workspace, ".cache/dune")))
    env.XDG_CACHE_HOME = join(workspace, ".cache");
  if (existsSync(join(workspace, ".local/bin/dune")))
    env.PATH = [
      join(workspace, ".local/bin"),
      ...(env.PATH ?? "").split(delimiter).filter((path) => path !== join(workspace, ".local/bin")),
    ].join(delimiter);
  const paths = (env.PATH ?? "").split(delimiter).filter(Boolean);
  if (env.CARGO_HOME) {
    const bin = join(env.CARGO_HOME, "bin");
    env.PATH = [bin, ...paths.filter((path) => path !== bin)].join(delimiter);
  }
  if (env.CARGO_HOME === join(workspace, ".cargo")) {
    env.NODE_USE_ENV_PROXY ??= "1";
    env.CARGO_BUILD_JOBS ??= "4";
  }
  return env;
}
function toolchainCommand(command) {
  return command;
}
const quote = (value) => `'${value.replaceAll("'", "'\\''")}'`;
function installToolchainShell(home = homedir(), workspace = "/workspace", root = project) {
  const config = join(home, ".config/hex-command/toolchain-env.sh");
  const bin = join(home, ".local/bin");
  mkdirSync(dirname(config), { recursive: true });
  mkdirSync(bin, { recursive: true });
  const cargo = join(workspace, ".cargo");
  const rustup = join(workspace, ".rustup");
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
      "",
    ].join("\n"),
  );
  const hook = `[ ! -r ${quote(config)} ] || . ${quote(config)}`;
  const profiles = [".profile", ".bashrc", ".bash_profile", ".bash_login"];
  for (const name of profiles) {
    const file = join(home, name);
    if (!existsSync(file) && name !== ".profile" && name !== ".bashrc") continue;
    const current = existsSync(file) ? readFileSync(file, "utf8") : "";
    if (!current.includes(hook))
      writeFileSync(
        file,
        `${hook}
${current}`,
      );
  }
  const marker = "# Managed by Hex Command toolchain setup";
  const wrapper = (name, target) => {
    const file = join(bin, name);
    if (existsSync(file) && !readFileSync(file, "utf8").includes(marker)) return;
    writeFileSync(
      file,
      `#!/bin/sh
${marker}
${hook}
exec ${quote(target)} "$@"
`,
      {
        mode: 493,
      },
    );
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
}
export { installToolchainShell, toolchainCommand, toolchainEnvironment };

// Emit only resolved changes, quoted for Bash; never dump the inherited environment.
if (
  process.argv[1] &&
  fileURLToPath(import.meta.url) === process.argv[1] &&
  process.argv[2] === "--shell"
) {
  const resolved = toolchainEnvironment();
  for (const key of [
    "PATH",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "XDG_CACHE_HOME",
    "NODE_USE_ENV_PROXY",
    "CARGO_BUILD_JOBS",
  ]) {
    if (resolved[key] !== undefined && resolved[key] !== process.env[key])
      console.log(`export ${key}=${quote(resolved[key])}`);
  }
}
