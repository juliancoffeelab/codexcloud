import {
  allTestTasks,
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
      ["wasm", "ui", "types", "rust", "tests", "build", "art"].some((name) =>
        selected.has(name as Group),
      )
    )
      prepared();
    // Start the longest task first. Every suite runs once even when both its own
    // group and the aggregate tests group are requested.
    if (selected.has("tests")) tasks.push(...allTestTasks());
    else {
      if (selected.has("rust")) tasks.push(...rustTasks());
      if (selected.has("python")) tasks.push(...pythonTasks());
    }
    if (selected.has("style"))
      tasks.splice(selected.has("tests") ? 1 : 0, 0, nodeTask("style", "scripts/style.ts"));
    if (selected.has("content")) tasks.push(nodeTask("content", "scripts/content.ts", ["--check"]));
    const compilerGroups = ["wasm", "ui", "types"].filter((group) => selected.has(group as Group));
    if (compilerGroups.length)
      tasks.splice(
        selected.has("tests") ? 1 : 0,
        0,
        nodeTask("frontend-checks", "scripts/frontend-check.ts", compilerGroups, 1),
      );
    // Successful preparation ran the authoritative builder. Exact source and PNG
    // digests prove those outputs remain current without re-encoding every image.
    if (selected.has("art")) console.log("PASS: art (prepared source and asset digests)");
    if (selected.has("build")) console.log("PASS: build (prepared source and bundle digests)");
    process.exitCode = await runTasks(tasks);
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
