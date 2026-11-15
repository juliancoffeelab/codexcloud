import assert from "node:assert/strict";
import { test } from "vitest";
import * as Camera from "@frontend/Camera_view";

const layout = {
  map: { width: 2000, height: 2000 },
  base_width: 2000,
  capacity_width: 500,
  viewport: { width: 500, height: 500 },
};
function create(measured = layout) {
  const camera = Camera.create();
  Camera.configure(camera, measured);
  Camera.set_visible_bounds(camera, {
    left: 0,
    top: 0,
    right: measured.viewport.width,
    bottom: measured.viewport.height,
  });
  return camera;
}
const pan = 0,
  fit = 1;

test("Queued zooms expose complete immutable poses and preserve both anchor coordinates", () => {
  const camera = create();
  Camera.move_to(camera, fit, 500, 600);
  const before = Camera.snapshot(camera);
  assert.equal(Camera.zoom_at(camera, 2, 100, 150), true);
  assert.deepEqual(Camera.snapshot(camera).pose, { left: 1100, top: 1350, zoom: 2 });
  assert.equal(Camera.zoom_at(camera, 3, 100, 150), true);
  const after = Camera.snapshot(camera);
  assert.deepEqual(after.pose, { left: 1700, top: 2100, zoom: 3 });
  assert.deepEqual(before.pose, { left: 500, top: 600, zoom: 1 });
  assert.equal(after.content.width, 6000);
  assert.equal(after.content.height, 6000);
  assert.equal((after.pose.left + 100) / after.pose.zoom, 600);
  assert.equal((after.pose.top + 150) / after.pose.zoom, 750);
  assert.equal(Camera.zoom_at(camera, 100, 100, 150), false);
  Camera.zoom_at(camera, 0.1, 100, 150);
  assert.equal(Camera.snapshot(camera).pose.zoom, 0.5);
});

test("Layout and clipping preserve an out-of-bounds pose; pan permits inward movement without snapping", () => {
  const camera = create();
  Camera.move_to(camera, fit, 1500, 1500);
  Camera.configure(camera, { ...layout, viewport: { width: 500, height: 1000 } });
  Camera.set_visible_bounds(camera, { left: 0, top: 0, right: 500, bottom: 1000 });
  assert.equal(Camera.snapshot(camera).pose.top, 1500);
  Camera.pan_by(camera, 0, 50);
  assert.equal(Camera.snapshot(camera).pose.top, 1500);
  Camera.pan_by(camera, 0, -100);
  assert.equal(Camera.snapshot(camera).pose.top, 1400);
  Camera.move_to(camera, fit, 1500, 1500);
  assert.equal(Camera.snapshot(camera).pose.top, 1000);
  Camera.set_visible_bounds(camera, { left: 20, top: 30, right: 500, bottom: 1000 });
  Camera.move_to(camera, pan, -1000, -1000);
  assert.deepEqual(Camera.snapshot(camera).pose, { left: -20, top: -30, zoom: 1 });
});

test("Capacity resizing preserves the horizontal map center; reset starts a fresh layout", () => {
  const camera = create({
    ...layout,
    base_width: 1000,
    capacity_width: 1000,
    viewport: { width: 1000, height: 500 },
  });
  Camera.zoom_at(camera, 2, 0, 0);
  Camera.move_to(camera, fit, 300, 100);
  const old = Camera.snapshot(camera);
  const center = (old.pose.left + old.layout.capacity_width / 2) / old.display_scale;
  Camera.configure(camera, {
    ...layout,
    base_width: 800,
    capacity_width: 800,
    viewport: { width: 800, height: 500 },
  });
  const next = Camera.snapshot(camera);
  assert.equal((next.pose.left + 400) / next.display_scale, center);
  assert.equal(next.pose.top, 100);
  assert.equal(next.content.width, 1600);
  Camera.reset(camera);
  assert.equal(Camera.initialized(camera), false);
  Camera.configure(camera, layout);
  assert.deepEqual(Camera.snapshot(camera).pose, { left: 0, top: 0, zoom: 1 });
});
