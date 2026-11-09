import assert from "node:assert/strict";
import { test } from "vitest";
import * as Lighting from "@frontend/Lighting";
import { fromJSON } from "./json-fixture.mjs";
test("Lighting uses clamped RGB pixels while preserving alpha and transparent source bytes", () => {
  const data = new Uint8ClampedArray([255, 128, 0, 255, 231, 111, 3, 0]);
  Lighting.shade(data, 2, 0, 1, 1, fromJSON("BattlefieldLighting", { time: "night", lights: [] }));
  assert.deepEqual([...data], [69, 62, 35, 255, 231, 111, 3, 0]);
  const day = new Uint8ClampedArray([255, 128, 0, 255, 231, 111, 3, 0]);
  Lighting.shade(day, 2, 0, 1, 1, fromJSON("BattlefieldLighting", { time: "day", lights: [] }));
  assert.deepEqual([...day], [202, 118, 22, 255, 231, 111, 3, 0]);
});
