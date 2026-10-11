import { test } from "vitest";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";

test("Complete combat report preserves seats, caps, unit events and paired comparisons", () => {
  const result = spawnSync(
    "python3",
    ["-m", "unittest", "discover", "-s", ".agents/skills/combat-simulation/tests"],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stdout + result.stderr);
});
