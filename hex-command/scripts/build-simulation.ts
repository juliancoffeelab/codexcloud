/** Incremental native builds by default; explicit PGO builds serve timing comparisons. */
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpus } from "node:os";
import {
  closeSync,
  copyFileSync,
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
const args = process.argv.slice(2);
if (args.some((arg) => arg !== "--pgo")) throw new Error("Use --pgo for profile-guided builds");
const pgo = args.includes("--pgo") || env.HEX_COMMAND_SIM_PGO === "1";
const suffix = process.platform === "win32" ? ".exe" : "";
const target = resolve(root, env.CARGO_TARGET_DIR ?? "rust/target");
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
const flags =
  env.CARGO_ENCODED_RUSTFLAGS?.split("\x1f") ?? (env.RUSTFLAGS ?? "").split(/\s+/).filter(Boolean);
if (flags.some((flag) => /profile-(generate|use)/.test(flag)))
  throw new Error("Profile flags are managed by the native simulation build");
if (!flags.some((flag) => flag.includes("target-cpu"))) flags.push("-C", "target-cpu=native");
const nativeEnv = {
  ...env,
  CARGO_TARGET_DIR: target,
  ...(pgo
    ? {
        CARGO_PROFILE_RELEASE_CODEGEN_UNITS: env.CARGO_PROFILE_RELEASE_CODEGEN_UNITS ?? "1",
        CARGO_PROFILE_RELEASE_LTO: env.CARGO_PROFILE_RELEASE_LTO ?? "fat",
      }
    : {}),
};
const cargo = [
  "build",
  "--locked",
  "--manifest-path",
  "rust/Cargo.toml",
  "--bin",
  "hex-command-sim",
];
mkdirSync(join(target, "release"), { recursive: true });
// Stage either build at the launcher's stable location. Keep each compiler cache separate.
if (!pgo) {
  run("cargo", [...cargo, "--profile", "simulation"], {
    ...nativeEnv,
    HEX_SIM_PROFILE_KEY: "",
    CARGO_ENCODED_RUSTFLAGS: flags.join("\x1f"),
  });
  copyFileSync(
    join(target, "simulation/hex-command-sim" + suffix),
    join(target, "release/hex-command-sim" + suffix),
  );
  process.exit(0);
}
const cache = join(target, "ai-native");
const raw = join(cache, "raw");
mkdirSync(raw, { recursive: true });
nativeEnv.CARGO_TARGET_DIR = join(cache, "optimized");
cargo.push("--release");
const rustc = env.RUSTC ?? "rustc";
const capture = (command: string, args: string[]) =>
  execFileSync(command, args, { cwd: root, env, encoding: "utf8" }).trim();
const version = capture(rustc, ["-vV"]);
const sysroot = capture(rustc, ["--print", "sysroot"]);
const host = version.match(/^host: (.+)$/m)?.[1];
if (!host) throw new Error("Rust compiler did not report its host target");
const profdata = join(sysroot, "lib/rustlib", host, "bin", "llvm-profdata" + suffix);
if (!existsSync(profdata)) run("rustup", ["component", "add", "llvm-tools-preview"]);
if (!existsSync(profdata)) throw new Error("Matching LLVM profile tools are unavailable");
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
      "--no-report",
      "--all-pairs",
      "--campaign-players",
      "classic,rush,balanced,rotated,situational,predictor",
      "--campaign-enemies",
      "classic,rush,balanced,rotated,situational,predictor",
      "--mode",
      "full",
      "--seeds",
      "2",
      "--seed",
      "17",
      "--policies",
      "classic,rush,balanced,rotated,situational,predictor",
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
// A stable profile path lets Cargo retain unchanged dependency builds. The
// package build script tracks the profile key so a changed profile still
// rebuilds the engine, including when only its training/build settings change.
const active = join(cache, "active.profdata");
if (!existsSync(active) || !readFileSync(active).equals(readFileSync(profile))) {
  copyFileSync(profile, active);
}
run("cargo", cargo, {
  ...nativeEnv,
  HEX_SIM_PROFILE_KEY: key,
  CARGO_ENCODED_RUSTFLAGS: [...flags, "-C", "profile-use=" + active].join("\x1f"),
});
copyFileSync(
  join(nativeEnv.CARGO_TARGET_DIR, "release/hex-command-sim" + suffix),
  join(target, "release/hex-command-sim" + suffix),
);
