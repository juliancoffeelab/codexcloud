import {
  allTestTasks,
  cpuSlots,
  nodeTask,
  prepared,
  pythonTasks,
  runTasks,
  rustTasks,
  type Task,
} from "./test-budget";

const groups = [
  "style",
  "content",
  "wasm",
  "ui",
  "types",
  "rust",
  "python",
  "art",
  "tests",
  "build",
] as const;
type Group = (typeof groups)[number];
const args = process.argv
  .slice(2)
  .map((name) => (name === "format" ? "style" : name === "bindings" ? "wasm" : name));
if (args.some((name) => !groups.includes(name as Group))) {
  console.error(`Choose checks from: ${groups.join(", ")}`);
  process.exitCode = 1;
} else {
  try {
    const selected = new Set<Group>((args.length ? args : groups) as Group[]);
    const tasks: Task[] = [];
    if (
      ["wasm", "ui", "types", "rust", "tests", "build"].some((name) => selected.has(name as Group))
    )
      prepared();
    // Start the longest task first. Every suite runs once even when both its own
    // group and the aggregate tests group are requested.
    if (selected.has("tests")) tasks.push(...allTestTasks());
    else {
      if (selected.has("rust")) tasks.push(...rustTasks());
      if (selected.has("python")) tasks.push(...pythonTasks());
    }
    if (selected.has("style")) tasks.push(nodeTask("style", "scripts/style.ts"));
    if (selected.has("content")) tasks.push(nodeTask("content", "scripts/content.ts", ["--check"]));
    if (selected.has("wasm"))
      tasks.push(
        nodeTask(
          "wasm-types",
          "node_modules/typescript/bin/tsc",
          [
            "--noEmit",
            "--strict",
            "--target",
            "ES2022",
            "--module",
            "ESNext",
            "--moduleResolution",
            "Bundler",
            "rust/pkg/hex_command.d.ts",
          ],
          Math.min(2, cpuSlots),
        ),
      );
    if (selected.has("ui"))
      tasks.push(
        nodeTask(
          "frontend-bindings",
          "scripts/melange-bindings.ts",
          ["--check"],
          Math.min(2, cpuSlots),
        ),
      );
    if (selected.has("types"))
      tasks.push(
        nodeTask(
          "types",
          "node_modules/typescript/bin/tsc",
          ["-p", "tsconfig.tests.json"],
          Math.min(2, cpuSlots),
        ),
      );
    if (selected.has("art"))
      tasks.push({
        name: "art",
        command: "python3",
        args: ["scripts/build-pixel-art.py", "--check"],
      });
    if (selected.has("build"))
      tasks.push(
        nodeTask("build", "node_modules/vite/bin/vite.js", ["build"], Math.min(2, cpuSlots)),
      );
    process.exitCode = await runTasks(tasks);
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
