/** Host-native, profile-guided build of the simulation CLI; wasm builds are separate. */
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpus } from "node:os";
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readdirSync,
  readFileSync,
  unlinkSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "../.agents/skills/rust-toolchain/scripts/toolchain-env";

const root = fileURLToPath(new URL("../", import.meta.url));
const env = toolchainEnvironment();
const suffix = process.platform === "win32" ? ".exe" : "";
const target = resolve(root, env.CARGO_TARGET_DIR ?? "rust/target");
const cache = join(target, "ai-native");
const raw = join(cache, "raw");
mkdirSync(raw, { recursive: true });
const rustc = env.RUSTC ?? "rustc";
const capture = (command: string, args: string[]) =>
  execFileSync(command, args, { cwd: root, env, encoding: "utf8" }).trim();
const version = capture(rustc, ["-vV"]);
const sysroot = capture(rustc, ["--print", "sysroot"]);
const host = version.match(/^host: (.+)$/m)?.[1];
if (!host) throw new Error("Rust compiler did not report its host target");
const profdata = join(sysroot, "lib/rustlib", host, "bin", "llvm-profdata" + suffix);
function run(command: string, args: string[], childEnv = env, log?: string) {
  const descriptor = log ? openSync(log, "w") : undefined;
  try {
    const child = spawnSync(command, args, {
      cwd: root,
      env: childEnv,
      stdio: descriptor === undefined ? "inherit" : ["ignore", descriptor, descriptor],
    });
    if (child.error || child.status !== 0) {
      if (log) console.error(readFileSync(log, "utf8").split("\n").slice(-20).join("\n"));
      throw child.error ?? new Error(`${command} failed with exit code ${child.status}`);
    }
  } finally {
    if (descriptor !== undefined) closeSync(descriptor);
  }
}
if (!existsSync(profdata)) run("rustup", ["component", "add", "llvm-tools-preview"]);
if (!existsSync(profdata)) throw new Error("Matching LLVM profile tools are unavailable");
const flags =
  env.CARGO_ENCODED_RUSTFLAGS?.split("\x1f") ?? (env.RUSTFLAGS ?? "").split(/\s+/).filter(Boolean);
if (flags.some((flag) => /profile-(generate|use)/.test(flag)))
  throw new Error("Profile flags are managed by the native simulation build");
if (!flags.some((flag) => flag.includes("target-cpu"))) flags.push("-C", "target-cpu=native");
const nativeEnv = {
  ...env,
  CARGO_TARGET_DIR: target,
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: env.CARGO_PROFILE_RELEASE_CODEGEN_UNITS ?? "1",
  CARGO_PROFILE_RELEASE_LTO: env.CARGO_PROFILE_RELEASE_LTO ?? "fat",
};
function sources(directory: string, extension: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory()
      ? sources(path, extension)
      : entry.name.endsWith(extension)
        ? [path]
        : [];
  });
}
const hash = createHash("sha256").update(
  JSON.stringify({
    version,
    flags,
    cpu: cpus()[0]?.model,
    lto: nativeEnv.CARGO_PROFILE_RELEASE_LTO,
    codegen: nativeEnv.CARGO_PROFILE_RELEASE_CODEGEN_UNITS,
  }),
);
for (const path of [
  "rust/Cargo.toml",
  "rust/Cargo.lock",
  "rust/build.rs",
  "scripts/build-simulation.ts",
  ...sources(join(root, "rust/src"), ".rs"),
  ...sources(join(root, "rust/data"), ".json"),
].sort()) {
  hash
    .update(path.startsWith(root) ? path.slice(root.length) : path)
    .update("\0")
    .update(readFileSync(resolve(root, path)));
}
const key = hash.digest("hex");
const profile = join(cache, key + ".profdata");
const cargo = [
  "build",
  "--locked",
  "--release",
  "--manifest-path",
  "rust/Cargo.toml",
  "--bin",
  "hex-command-sim",
];
if (!existsSync(profile)) {
  console.log(
    "Training native compiler profile: all policies, maps and campaigns; seeds 17–18, 30 rounds.",
  );
  const instrumentedTarget = join(cache, "instrumented");
  run("cargo", cargo, {
    ...nativeEnv,
    CARGO_TARGET_DIR: instrumentedTarget,
    CARGO_ENCODED_RUSTFLAGS: [...flags, "-C", "profile-generate=" + raw].join("\x1f"),
  });
  for (const file of readdirSync(raw)) if (file.endsWith(".profraw")) unlinkSync(join(raw, file));
  run(
    join(instrumentedTarget, "release", "hex-command-sim" + suffix),
    [
      "--mode",
      "full",
      "--seeds",
      "2",
      "--seed",
      "17",
      "--workers",
      "1",
      "--data-dir",
      join(root, "rust/data"),
      "--out",
      join(cache, "training.json"),
    ],
    { ...env, LLVM_PROFILE_FILE: join(raw, "run-%p.profraw") },
    join(cache, "training.log"),
  );
  const profiles = readdirSync(raw)
    .filter((file) => file.endsWith(".profraw"))
    .map((file) => join(raw, file));
  if (!profiles.length) throw new Error("Native training produced no compiler profile");
  run(profdata, ["merge", "-o", profile, ...profiles]);
}
run("cargo", cargo, {
  ...nativeEnv,
  CARGO_ENCODED_RUSTFLAGS: [...flags, "-C", "profile-use=" + profile].join("\x1f"),
});
