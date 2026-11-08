import assert from "node:assert/strict";
import { test } from "vitest";
import * as Preview from "@frontend/Terrain_preview";
const frame = { url: "portrait.png" };
test("HUD artwork uses the active Phaser loader and adopts its actual decoded image", async () => {
  const cache = Preview.create();
  await assert.rejects(
    Preview.load(cache, frame),
    (error) =>
      error.MEL_EXN_ID === "Failure" && error._1 === "Artwork loader unavailable: portrait.png",
  );
  let requests = 0;
  const image = {};
  const unbind = Preview.bind(cache, async (value) => {
    requests++;
    assert.equal(value, frame);
    Preview.adopt(cache, frame.url, image);
    return image;
  });
  assert.equal(await Preview.load(cache, frame), image);
  assert.equal(await Preview.load(cache, frame), image);
  assert.equal(requests, 1);
  unbind();
  assert.equal(Preview.ready(cache, frame), image);
  Preview.close(cache);
  assert.equal(Preview.ready(cache, frame), undefined);
  await assert.rejects(
    Preview.load(cache, frame),
    (error) =>
      error.MEL_EXN_ID === "Failure" && error._1 === "Artwork loader unavailable: portrait.png",
  );
});
test("Obsolete loader cleanup cannot detach a replacement owner", async () => {
  const cache = Preview.create();
  const first = Preview.bind(cache, async () => ({ old: true }));
  const image = { current: true };
  const second = Preview.bind(cache, async () => image);
  first();
  assert.equal(await Preview.load(cache, frame), image);
  second();
  await assert.rejects(
    Preview.load(cache, frame),
    (error) =>
      error.MEL_EXN_ID === "Failure" && error._1 === "Artwork loader unavailable: portrait.png",
  );
});
