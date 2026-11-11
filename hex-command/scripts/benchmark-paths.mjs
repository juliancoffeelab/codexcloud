import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

// Benchmarks measure production artifacts unless development WASM is requested.
export function benchmarkWasmDirectory(
  root,
  profile = process.env.HEX_COMMAND_PROFILE ?? "release",
) {
  if (!["dev", "release"].includes(profile)) throw new Error("Choose dev or release WASM");
  const base = resolve(root, "_build/default/wasm");
  if (existsSync(resolve(root, "wasm/dev/dune"))) {
    const directory = resolve(base, profile);
    if (!existsSync(resolve(directory, "backend.wasm")))
      throw new Error(
        `Missing ${profile} WASM in ${root}; run node scripts/toolchain/with-toolchain.mjs bash scripts/build-wasm.sh ${profile}, then dune build --root=. --cache=enabled @melange @artwork`,
      );
    return directory;
  }
  // Keep historical compiled checkouts usable as comparison references.
  if (existsSync(resolve(base, "backend.wasm"))) {
    const marker = resolve(base, "profile");
    if (existsSync(marker) && readFileSync(marker, "utf8").trim() !== profile)
      throw new Error(`Comparison checkout ${root} must be prepared with ${profile} WASM`);
    return base;
  }
  return existsSync(resolve(root, "pkg")) ? resolve(root, "pkg") : resolve(root, "rust/pkg");
}
