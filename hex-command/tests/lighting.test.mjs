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

test("Underground ambience preserves outdoor daylight, uses local lights, and invalidates moved zones", () => {
  const center = (col, row) => [
    (39 + 56 * (col + (row % 2) / 2)) / (64 / 70),
    (38 + 48 * row) / (64 / 70),
  ];
  const cell = { col: 3, row: 3 };
  const light = { ...cell, radius: 90, power: 0.7, fixture: false };
  const region = fromJSON("BattlefieldLighting", {
    time: "day",
    underground: [cell],
    lights: [light],
  });
  const day = fromJSON("BattlefieldLighting", { time: "day", lights: [] });
  const night = fromJSON("BattlefieldLighting", { time: "night", lights: [light] });
  const inside = center(3, 3),
    outside = center(8, 3);
  assert.deepEqual(Lighting.matrix(...inside, region), Lighting.matrix(...inside, night));
  assert.deepEqual(Lighting.matrix(...outside, region), Lighting.matrix(...outside, day));
  const moved = fromJSON("BattlefieldLighting", {
    time: "day",
    underground: [{ col: 8, row: 3 }],
    lights: [],
  });
  assert.deepEqual(Lighting.matrix(...inside, moved), Lighting.matrix(...inside, day));
  assert.equal(Lighting.ambient(...outside, moved), 1);
  const boundary = [inside[0] + 28 / (64 / 70), inside[1]];
  const darkness = Lighting.ambient(...boundary, region);
  assert.ok(darkness > 0 && darkness < 1, "entrance boundary blends between the two ambiences");
});
