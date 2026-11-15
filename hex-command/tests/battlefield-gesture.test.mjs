import assert from "node:assert/strict";
import { test } from "vitest";
import * as Pointer from "@frontend/Phaser_pointer";
import * as Gesture from "@frontend/Battlefield_gesture";

const view = { left: 100, top: 200, zoom: 1, canvas_left: 0, canvas_top: 0 };
function pointer(overrides = {}) {
  return {
    id: 0,
    identifier: 0,
    button: 0,
    buttons: 1,
    primaryDown: true,
    wasTouch: false,
    wasCanceled: false,
    x: 40,
    y: 60,
    event: { clientX: 20, clientY: 30, timeStamp: 100 },
    ...overrides,
  };
}
const sample = (p) => Pointer.sample(p);
const start = (g, p, active = [sample(p)]) => {
  const press = Pointer.press(p);
  assert.notEqual(press, undefined);
  return Gesture.press(g, press, view, active);
};
const release = (g, p, active = []) => Gesture.release(g, sample(p), active)[1];
function commands(list) {
  const values = [];
  for (; list; list = list.tl) values.push(list.hd);
  return values;
}
const taps = (list) =>
  commands(list)
    .filter((command) => command.TAG === 2)
    .map((command) => command._0);

test("Primary tap events with either held-button mask activate exactly once, including after idle cancellation", () => {
  for (const buttons of [0, 1]) {
    const g = Gesture.create();
    Gesture.cancel(g, []);
    const p = pointer({ buttons });
    assert.equal(start(g, p), true);
    const up = pointer({ buttons: 0, event: { ...p.event, timeStamp: 101 } });
    assert.deepEqual(taps(release(g, up)), [{ scene_x: 40, scene_y: 60 }]);
    assert.deepEqual(commands(release(g, up)), []);
  }
});

test("The boundary rejects control-click and secondary presses, and stores immutable positions", () => {
  assert.equal(Pointer.press(pointer({ primaryDown: false, buttons: 2 })), undefined);
  assert.equal(Pointer.press(pointer({ button: 2, buttons: 3 })), undefined);
  const g = Gesture.create();
  const p = pointer();
  start(g, p);
  p.event.clientX = 200;
  p.x = 400;
  assert.deepEqual(taps(release(g, pointer())), [{ scene_x: 40, scene_y: 60 }]);
});

test("Unmatched releases cannot activate or consume another contact; cancellation consumes the matching release", () => {
  const g = Gesture.create();
  const p = pointer();
  assert.deepEqual(commands(release(g, p)), []);
  start(g, p);
  assert.deepEqual(commands(release(g, pointer({ id: 1 }), [sample(p)])), []);
  assert.deepEqual(commands(release(g, pointer({ button: 2 }), [sample(p)])), []);
  Gesture.cancel(g, [sample(p)]);
  assert.deepEqual(commands(release(g, p)), []);
  start(g, p);
  assert.equal(taps(release(g, p)).length, 1);
});

test("Middle presses only pan; moving touch contacts retain pan/slop/inertia behavior without turning into taps", () => {
  const g = Gesture.create();
  const middle = pointer({ button: 1, buttons: 4, primaryDown: false });
  start(g, middle);
  assert.deepEqual(commands(release(g, middle)), []);
  const touch = pointer({ id: 1, identifier: 1, wasTouch: true });
  start(g, touch);
  const moved = sample(pointer({ ...touch, event: { clientX: 30, clientY: 30, timeStamp: 116 } }));
  const pan = commands(Gesture.move(g, moved, [moved]));
  assert.deepEqual(pan, [{ TAG: 0, _0: 95, _1: 200 }]);
  const coast = commands(Gesture.release(g, { ...moved, time: 120 }, [])[1]);
  assert.equal(coast.length, 1);
  assert.equal(coast[0].TAG, 3);
});

test("Pinch releases suppress the remaining finger until release, without a tap or an accidental pan", () => {
  const g = Gesture.create();
  const a = pointer({ id: 1, wasTouch: true });
  const b = pointer({ id: 2, wasTouch: true, event: { clientX: 60, clientY: 30, timeStamp: 110 } });
  start(g, a);
  start(g, b, [sample(a), sample(b)]);
  const farther = sample(pointer({ ...b, event: { clientX: 100, clientY: 30, timeStamp: 120 } }));
  assert.deepEqual(commands(Gesture.move(g, farther, [sample(a), farther])), [
    { TAG: 1, _0: 2, _1: 40, _2: 30 },
  ]);
  assert.deepEqual(commands(release(g, b, [sample(a)])), []);
  assert.deepEqual(commands(Gesture.move(g, sample(a), [sample(a)])), []);
  // A fresh second finger may restart pinching, while the remaining finger alone never taps.
  start(g, b, [sample(a), sample(b)]);
  assert.equal(commands(Gesture.move(g, farther, [sample(a), farther]))[0].TAG, 1);
  assert.deepEqual(commands(release(g, b, [sample(a)])), []);
  assert.deepEqual(commands(release(g, a)), []);
  start(g, a);
  assert.equal(taps(release(g, a)).length, 1);
});
