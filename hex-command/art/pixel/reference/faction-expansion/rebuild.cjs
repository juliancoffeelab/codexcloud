// Rebuild the saved expansion without calling imagegen or changing gameplay data.
const { execFileSync } = require("node:child_process");
const path = require("node:path");
const project = path.resolve(__dirname, "../../../..");
const ids = process.argv.slice(2);
const run = (script, ...args) =>
  execFileSync(process.execPath, [path.join(__dirname, script), ...args], {
    cwd: project,
    stdio: "inherit",
  });
run("process.cjs", "extract", ...ids);
run("clean.cjs", ...ids);
run("clean-beasts.cjs", ...ids);
run("process.cjs", "tiers", ...ids);
run("reuse.cjs");
run("process.cjs", "boards");
run("gallery.cjs");
execFileSync("npm", ["run", "art:build"], { cwd: project, stdio: "inherit" });
