import assert from "node:assert/strict";
import { beforeEach, test, vi } from "vitest";
import * as Motion from "@frontend/Loading_motion";

let now;
beforeEach(() => {
  now = 0;
  vi.spyOn(performance, "now").mockImplementation(() => now);
  Motion.restart(1, 0);
});

test("Presentation moves gently while waiting, then eases into actual completion", () => {
  now = 2000;
  const waiting = Motion.value();
  assert(waiting > 0 && waiting < 0.25);
  Motion.advance(2, 1);
  assert.equal(Motion.value(), waiting);
  now += 200;
  const catchingUp = Motion.value();
  assert(catchingUp > waiting && catchingUp < 0.25);
  now += 100000;
  assert(Motion.value() < 0.3);
  Motion.advance(5, 4);
  now += 100000;
  assert(Motion.value() < 1);
  Motion.advance(5, 5);
  assert(Motion.value() < 1);
  now += 140;
  assert.equal(Motion.value(), 1);
});

test("Retargeting and remounting preserve progress; a new battlefield resets explicitly", () => {
  Motion.advance(4, 3.8);
  now += 1000;
  const before = Motion.value();
  Motion.advance(4, 3.2);
  assert.equal(Motion.value(), before);
  const cancel = vi.fn();
  const animate = vi.fn(() => ({ cancel }));
  const cleanup = Motion.animate({ animate });
  const frames = animate.mock.calls[0][0];
  assert(Math.abs(Number(frames[0].transform.match(/scaleX\((.*)\)/)[1]) - before) < 0.00001);
  cleanup();
  assert.equal(cancel.mock.calls.length, 1);
  Motion.restart(3, 2);
  assert.equal(Motion.value(), 0.3);
});
