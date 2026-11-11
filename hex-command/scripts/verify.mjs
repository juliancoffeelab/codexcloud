import { allTestTasks, nodeTask, verifyTasks, pythonTasks, rustTasks } from "./test-budget.mjs";
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
];
const args = process.argv
  .slice(2)
  .map((name) => (name === "format" ? "style" : name === "bindings" ? "wasm" : name));
if (args.some((name) => !groups.includes(name))) {
  console.error(`Choose checks from: ${groups.join(", ")}`);
  process.exitCode = 1;
} else {
  try {
    const selected = new Set(args.length ? args : groups);
    const needsPreparation = ["tests", "build"].some((name) => selected.has(name));
    process.exitCode = await verifyTasks(
      () => {
        const tasks = [];
        if (selected.has("tests")) tasks.push(...allTestTasks());
        else {
          if (selected.has("rust")) tasks.push(...rustTasks(needsPreparation));
          if (selected.has("python")) tasks.push(...pythonTasks());
        }
        if (selected.has("style"))
          tasks.splice(
            selected.has("tests") ? 1 : 0,
            0,
            nodeTask("style", "scripts/style.mjs", ["--check"], 1, "dune"),
          );
        if (selected.has("content"))
          tasks.push(nodeTask("content", "scripts/content.mjs", ["--check"]));
        const compilerGroups = ["wasm", "ui", "types"].filter((group) => selected.has(group));
        if (compilerGroups.length)
          tasks.splice(
            selected.has("tests") ? 1 : 0,
            0,
            nodeTask(
              "frontend-checks",
              "scripts/frontend-check.mjs",
              [...compilerGroups, ...(needsPreparation ? ["--prepared"] : [])],
              1,
              "dune",
            ),
          );
        if (selected.has("art")) console.log("PASS: art (prepared source and asset digests)");
        if (selected.has("build")) console.log("PASS: build (prepared source and bundle digests)");
        return tasks;
      },
      needsPreparation ? true : selected.has("art") ? "art" : false,
    );
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
