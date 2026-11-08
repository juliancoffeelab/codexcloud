import assert from "node:assert/strict";
import { test, vi } from "vitest";
import * as FaultFixture from "@frontend/Fault_fixture";

test("Fault descriptions retain common messages and locations while logging original diagnostics", () => {
  const logged = vi.spyOn(console, "error").mockImplementation(() => {});
  try {
    assert.deepEqual(FaultFixture.samples(), [
      "Précise λ🦀",
      "bad argument",
      'File "sample.ml", line 12, characters 3-8: Pattern matching failed',
      'File "sample.ml", line 18, characters 4-10: Assertion failed',
      'File "sample.ml", line 24, characters 5-11: Undefined recursive module',
      "Out of memory",
      "Stack overflow",
    ]);
    const generic = FaultFixture.generic();
    assert.match(generic.description, /Diagnostic(?:\/\d+)?$/);
    assert.equal(logged.mock.calls.at(-1)[0], generic.original);
    const foreign = FaultFixture.foreign();
    assert.equal(foreign.description, "Browser λ🦀 failure");
    assert.equal(logged.mock.calls.at(-1)[0], foreign.original);
    assert(foreign.original instanceof Error);
    assert(foreign.original.stack);
  } finally {
    logged.mockRestore();
  }
});
