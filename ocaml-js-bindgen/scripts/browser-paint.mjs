import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

export async function testPaint(page) {
  const sample = (x, y) =>
    page
      .locator("#pixels")
      .evaluate(
        (canvas, [x, y]) =>
          Array.from(canvas.getContext("2d").getImageData(x, y, 1, 1).data),
        [x, y],
      );
  const stroke = async (tool, from, to) => {
    await page.click(`#${tool}`);
    const box = await page.locator("#pixels").boundingBox();
    const size = await page
      .locator("#pixels")
      .evaluate((c) => [c.width, c.height]);
    const point = ([x, y]) => [
      box.x + ((x + 0.5) * box.width) / size[0],
      box.y + ((y + 0.5) * box.height) / size[1],
    ];
    await page.mouse.move(...point(from));
    await page.mouse.down();
    await page.mouse.move(...point(to), { steps: 6 });
    await page.mouse.up();
  };
  const startBox = await page.locator("#pixels").boundingBox();
  assert.equal(await page.locator("#undo").isDisabled(), true);
  await page.click("#hand");
  await page.mouse.move(startBox.x + 100, startBox.y + 100);
  await page.mouse.down();
  await page.mouse.move(startBox.x + 160, startBox.y + 140, { steps: 4 });
  await page.mouse.up();
  const pannedBox = await page.locator("#pixels").boundingBox();
  assert(Math.abs(pannedBox.x - startBox.x - 60) < 1);
  assert(Math.abs(pannedBox.y - startBox.y - 40) < 1);
  await page.click("#zoom-in");
  assert((await page.locator("#pixels").boundingBox()).width > pannedBox.width);
  assert.equal(
    await page.locator("#undo").isDisabled(),
    true,
    "view changes must not enter history",
  );
  await page.click("#fit");
  await stroke("select", [20, 20], [100, 80]);
  assert.equal(await page.locator("#selection").isVisible(), true);
  assert.equal(
    await page.locator("#undo").isDisabled(),
    true,
    "selection must not enter history",
  );
  await page.keyboard.press("Escape");
  assert.equal(await page.locator("#selection").isVisible(), false);
  await page.waitForFunction(
    () => document.querySelector("#pixels").width === 768,
  );
  assert.deepEqual(await sample(40, 40), [255, 255, 255, 255]);
  await page.locator("#color").fill("#e75b52");
  await stroke("brush", [40, 40], [160, 40]);
  assert.deepEqual(await sample(90, 40), [231, 91, 82, 255]);
  await page.click("#undo");
  assert.deepEqual(await sample(90, 40), [255, 255, 255, 255]);
  await page.click("#zoom-in");
  await stroke("select", [10, 10], [20, 20]);
  await page.keyboard.press("Escape");
  await page.click("#fit");
  assert.equal(
    await page.locator("#redo").isDisabled(),
    false,
    "view/selection changes must preserve redo",
  );
  await page.click("#redo");
  assert.deepEqual(await sample(90, 40), [231, 91, 82, 255]);
  await stroke("eraser", [90, 40], [100, 40]);
  assert.deepEqual(await sample(95, 40), [255, 255, 255, 255]);
  await stroke("rectangle", [200, 100], [300, 200]);
  assert.deepEqual(await sample(250, 100), [231, 91, 82, 255]);
  await page.locator("#color").fill("#398c86");
  await stroke("fill", [250, 150], [250, 150]);
  assert.deepEqual(await sample(250, 150), [57, 140, 134, 255]);
  assert.deepEqual(await sample(320, 150), [255, 255, 255, 255]);
  await stroke("ellipse", [350, 100], [450, 200]);
  assert.deepEqual(await sample(400, 100), [57, 140, 134, 255]);
  await stroke("line", [350, 250], [450, 250]);
  assert.deepEqual(await sample(400, 250), [57, 140, 134, 255]);
  await page.keyboard.press("Control+z");
  assert.deepEqual(await sample(400, 250), [255, 255, 255, 255]);
  await page.keyboard.press("Control+Shift+z");
  assert.deepEqual(await sample(400, 250), [57, 140, 134, 255]);

  const downloadPromise = page.waitForEvent("download");
  await page.click("#save");
  const download = await downloadPromise;
  const bytes = await readFile(await download.path());
  assert.deepEqual(
    [...bytes.subarray(0, 8)],
    [137, 80, 78, 71, 13, 10, 26, 10],
  );
  assert.equal(bytes.readUInt32BE(16), 768);
  assert.equal(bytes.readUInt32BE(20), 512);
  await page.click("#new");
  await page.setInputFiles("#file", {
    name: "painting.png",
    mimeType: "image/png",
    buffer: bytes,
  });
  await page.waitForFunction(
    () => document.querySelector("#message").textContent === "PNG opened",
  );
  assert.deepEqual(await sample(250, 150), [57, 140, 134, 255]);
  await page.setInputFiles("#file", {
    name: "broken.png",
    mimeType: "image/png",
    buffer: Buffer.from("invalid"),
  });
  await page.waitForFunction(() =>
    document.querySelector("#message").textContent.includes("Could not open"),
  );
  assert.deepEqual(await sample(250, 150), [57, 140, 134, 255]);

  await page.evaluate(() => {
    globalThis.originalRead = Blob.prototype.arrayBuffer;
    Blob.prototype.arrayBuffer = () =>
      Promise.reject(Error("simulated file failure"));
  });
  await page.setInputFiles("#file", {
    name: "denied.png",
    mimeType: "image/png",
    buffer: bytes,
  });
  await page.waitForFunction(() =>
    document.querySelector("#message").textContent.includes("Could not open"),
  );
  await page.evaluate(() => {
    Blob.prototype.arrayBuffer = originalRead;
    delete globalThis.originalRead;
  });
  assert.deepEqual(await sample(250, 150), [57, 140, 134, 255]);

  const before = await page.evaluate(async () => {
    const { initialize } = await import("./bindings.runtime.mjs");
    const e = (await initialize()).e;
    return [e.__ocamljs_live(), e.__ocamljs_host_live()];
  });
  // Keep a real file read pending, then replace the document before it completes.
  await page.evaluate(() => {
    const original = Blob.prototype.arrayBuffer;
    globalThis.fileGate = {};
    Blob.prototype.arrayBuffer = function () {
      const file = this;
      return new Promise((resolve, reject) => {
        fileGate.release = () => original.call(file).then(resolve, reject);
      });
    };
    fileGate.restore = () => {
      Blob.prototype.arrayBuffer = original;
    };
  });
  await page.setInputFiles("#file", {
    name: "late.png",
    mimeType: "image/png",
    buffer: bytes,
  });
  await page.waitForFunction(() => typeof fileGate.release === "function");
  await page.click("#new");
  await page.evaluate(() => {
    fileGate.release();
    fileGate.restore();
  });
  await page.waitForFunction(async () => {
    const { initialize } = await import("./bindings.runtime.mjs");
    return (await initialize()).jobs.size === 0;
  });
  assert.deepEqual(await sample(250, 150), [255, 255, 255, 255]);
  const after = await page.evaluate(async () => {
    const { initialize } = await import("./bindings.runtime.mjs");
    const e = (await initialize()).e;
    return [e.__ocamljs_live(), e.__ocamljs_host_live()];
  });
  assert.deepEqual(after, before, "late file result must be disposed");

  // Host-reference conformance stays in tests, not in the product toolbar.
  const interop = await page.evaluate(async () => {
    const { api, initialize } = await import("./bindings.runtime.mjs");
    const e = (await initialize()).e;
    const before = [e.__ocamljs_live(), e.__ocamljs_host_live()];
    const ok = (r, read) => {
      try {
        if (api.r_u8(r) !== 0) throw Error("unexpected image error");
        const value = read(r);
        api.finish(r);
        return value;
      } finally {
        api.close(r);
      }
    };
    const image = ok(api.call_fixture(), api.r_handle_image);
    let rejected = false;
    try {
      api.call_blend(image, image);
    } catch {
      rejected = true;
    }
    if (!rejected) throw Error("conflicting aliases accepted");
    const canvas = document.createElement("canvas");
    ok(api.call_draw(image, canvas), () => null);
    const copies = pixelProbe.copies;
    ok(api.call_draw(image, canvas), () => null);
    if (pixelProbe.copies !== copies)
      throw Error("unchanged image frame copied again");
    const task = api.task_map(api.call_upload(image), (r) =>
      ok(r, api.r_handle_texture),
    );
    const texture = await (api.task_await
      ? api.task_await(task)
      : new Promise((resolve, reject) =>
          api.task_receive(
            task,
            (value) => {
              resolve(value);
              return true;
            },
            (error) => {
              reject(error);
              return true;
            },
            (reason) => {
              reject(Error(reason));
              return true;
            },
          ),
        ));
    ok(api.call_draw_texture(texture, canvas), () => null);
    if (canvas.width !== 8 || canvas.height !== 8)
      throw Error("bitmap dimensions");
    api.dispose_Texture(texture);
    return { before, after: [e.__ocamljs_live(), e.__ocamljs_host_live()] };
  });
  assert.deepEqual(
    interop.after,
    interop.before,
    "image and native bitmap owners released",
  );

  await page.click("#new");
  await page.locator("#color").fill("#e75b52");
  await stroke("brush", [50, 50], [90, 50]);
  await stroke("select", [40, 40], [100, 60]);
  assert.equal(await page.locator("#selection-width").inputValue(), "60");
  const imagePoint = async (x, y) => {
    const box = await page.locator("#pixels").boundingBox();
    return [
      box.x + ((x + 0.5) * box.width) / 768,
      box.y + ((y + 0.5) * box.height) / 512,
    ];
  };
  await page.mouse.move(...(await imagePoint(70, 50)));
  await page.mouse.down();
  await page.mouse.move(...(await imagePoint(170, 150)), { steps: 5 });
  await page.mouse.up();
  assert.deepEqual(await sample(70, 50), [255, 255, 255, 255]);
  assert.deepEqual(await sample(170, 150), [231, 91, 82, 255]);
  await page.mouse.move(...(await imagePoint(170, 150)));
  await page.mouse.down();
  await page.mouse.move(...(await imagePoint(270, 250)), { steps: 4 });
  await page.keyboard.press("Escape");
  await page.mouse.up();
  assert.deepEqual(
    await sample(170, 150),
    [231, 91, 82, 255],
    "Escape must restore the original pixels",
  );
  assert.deepEqual(await sample(270, 250), [255, 255, 255, 255]);
  await stroke("select", [140, 140], [200, 160]);
  const corner = await page.locator('[data-handle="se"]').boundingBox();
  await page.mouse.move(
    corner.x + corner.width / 2,
    corner.y + corner.height / 2,
  );
  await page.mouse.down();
  const scaleBox = await page.locator("#pixels").boundingBox();
  await page.mouse.move(
    corner.x + corner.width / 2 + (60 * scaleBox.width) / 768,
    corner.y + corner.height / 2 + (20 * scaleBox.height) / 512,
    { steps: 5 },
  );
  await page.mouse.up();
  assert.equal(await page.locator("#selection-width").inputValue(), "120");
  assert.deepEqual(await sample(200, 160), [231, 91, 82, 255]);
  await page.click("#undo");
  assert.deepEqual(await sample(170, 150), [231, 91, 82, 255]);
  assert.deepEqual(await sample(200, 160), [255, 255, 255, 255]);
  await page.click("#undo");
  assert.deepEqual(await sample(70, 50), [231, 91, 82, 255]);
  assert.deepEqual(await sample(170, 150), [255, 255, 255, 255]);
  await page.click("#redo");
  await stroke("select", [140, 140], [200, 160]);
  await page.locator("#selection-width").fill("90");
  await page.locator("#selection-height").fill("30");
  await page.click("#resize-selection");
  assert.equal(await page.locator("#selection-width").inputValue(), "90");
  await page.click("#deselect");
  await page.keyboard.press("Control+a");
  assert.equal(await page.locator("#selection-width").inputValue(), "768");
  await page.keyboard.press("Delete");
  assert.deepEqual(await sample(170, 150), [255, 255, 255, 255]);
  await page.click("#undo");
  assert.deepEqual(await sample(170, 150), [231, 91, 82, 255]);
  await page.keyboard.press("Escape");
  await page.click("#new");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.click("#fit");
  assert(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  );
  const box = await page.locator("#pixels").boundingBox();
  const session = await page.context().newCDPSession(page);
  const point = {
    x: box.x + box.width * 0.2,
    y: box.y + box.height * 0.4,
    id: 7,
  };
  await page.click("#brush");
  await session.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [point],
  });
  await session.send("Input.dispatchTouchEvent", {
    type: "touchMove",
    touchPoints: [{ ...point, x: point.x + 40 }],
  });
  await session.send("Input.dispatchTouchEvent", {
    type: "touchEnd",
    touchPoints: [],
  });
  assert.equal(
    await page.locator("#undo").isDisabled(),
    false,
    "touch drawing must create a stroke",
  );
  const touchDrag = async (from, to) => {
    await session.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [{ x: from[0], y: from[1], id: 8 }],
    });
    await session.send("Input.dispatchTouchEvent", {
      type: "touchMove",
      touchPoints: [{ x: to[0], y: to[1], id: 8 }],
    });
    await session.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
  };
  await page.click("#hand");
  const beforePan = await page.locator("#pixels").boundingBox();
  await touchDrag(
    [beforePan.x + 80, beforePan.y + 80],
    [beforePan.x + 110, beforePan.y + 100],
  );
  const afterPan = await page.locator("#pixels").boundingBox();
  assert(Math.abs(afterPan.x - beforePan.x - 30) < 1);
  await page.click("#undo");
  assert.equal(
    await page.locator("#undo").isDisabled(),
    true,
    "touch pan must not add history",
  );
  await page.click("#redo");
  await page.click("#fit");
  await page.click("#select");
  await touchDrag(await imagePoint(100, 180), await imagePoint(300, 240));
  assert.equal(await page.locator("#selection-width").inputValue(), "200");
  const handle = await page.locator('[data-handle="se"]').boundingBox();
  assert(
    handle.width >= 27,
    "resize handle must stay touch-sized while zoomed out",
  );
  const mobileBox = await page.locator("#pixels").boundingBox();
  await touchDrag(
    [handle.x + handle.width / 2, handle.y + handle.height / 2],
    [
      handle.x + handle.width / 2 + (100 * mobileBox.width) / 768,
      handle.y + handle.height / 2 + (40 * mobileBox.height) / 512,
    ],
  );
  assert.equal(await page.locator("#selection-width").inputValue(), "300");
  assert.equal(await page.locator("#selection-height").inputValue(), "100");
  assert.deepEqual(await sample(220, 222), [231, 91, 82, 255]);
  await page.click("#undo");
  assert.deepEqual(await sample(194, 205), [231, 91, 82, 255]);
  // A second finger changes the view, never the image or its redo history.
  await page.click("#undo");
  await page.click("#brush");
  await page.click("#fit");
  const beforePinch = await page.locator("#pixels").boundingBox();
  const pinchCenter = {
    x: beforePinch.x + beforePinch.width * 0.6,
    y: beforePinch.y + beforePinch.height * 0.5,
  };
  const first = { x: pinchCenter.x - 40, y: pinchCenter.y, id: 20 };
  const second = { x: pinchCenter.x + 40, y: pinchCenter.y, id: 21 };
  await session.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [first],
  });
  await session.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [first, second],
  });
  const movedFirst = {
    ...first,
    x: pinchCenter.x - 80 + 10,
    y: pinchCenter.y + 5,
  };
  const movedSecond = {
    ...second,
    x: pinchCenter.x + 80 + 10,
    y: pinchCenter.y + 5,
  };
  await session.send("Input.dispatchTouchEvent", {
    type: "touchMove",
    touchPoints: [movedFirst, movedSecond],
  });
  await page.evaluate(() => new Promise(requestAnimationFrame));
  const afterPinch = await page.locator("#pixels").boundingBox();
  assert(
    Math.abs(afterPinch.width / beforePinch.width - 2) < 0.01,
    "pinch must zoom the canvas",
  );
  assert(
    Math.abs(afterPinch.x + afterPinch.width * 0.6 - pinchCenter.x - 10) < 1,
    "pinch must keep its image anchor under the fingers",
  );
  assert(
    Math.abs(afterPinch.y + afterPinch.height * 0.5 - pinchCenter.y - 5) < 1,
  );
  await session.send("Input.dispatchTouchEvent", {
    type: "touchEnd",
    touchPoints: [movedFirst],
  });
  await session.send("Input.dispatchTouchEvent", {
    type: "touchMove",
    touchPoints: [{ ...movedFirst, x: movedFirst.x + 30 }],
  });
  await session.send("Input.dispatchTouchEvent", {
    type: "touchEnd",
    touchPoints: [],
  });
  assert.equal(
    await page.locator("#undo").isDisabled(),
    true,
    "pinch must cancel the first finger's unfinished brush stroke",
  );
  assert.equal(
    await page.locator("#redo").isDisabled(),
    false,
    "pinch must preserve redo",
  );
  assert.deepEqual(await sample(450, 256), [255, 255, 255, 255]);
  assert.equal(
    await page.locator("#pixels").evaluate((c) => c.width),
    768,
    "pinch changes view size only",
  );
  const finalPinch = await page.locator("#pixels").boundingBox();
  assert.deepEqual(
    finalPinch,
    afterPinch,
    "the last finger cannot start a stray stroke or pan",
  );
  // Paired touches must produce one view update per frame, with no half-pair jump.
  await page.click("#hand");
  const paired = await page.evaluate(async () => {
    const workspace = document.querySelector("#workspace");
    const wrap = document.querySelector("#canvas-wrap");
    const rect = workspace.getBoundingClientRect();
    const x = rect.left + rect.width / 2,
      y = rect.top + rect.height / 2;
    const point = (id, px, py) =>
      new Touch({
        identifier: id,
        target: workspace,
        clientX: px,
        clientY: py,
      });
    const send = (name, id, px, py) =>
      workspace.dispatchEvent(
        new PointerEvent(name, {
          pointerId: id,
          pointerType: "touch",
          button: 0,
          buttons: name === "pointerup" ? 0 : 1,
          clientX: px,
          clientY: py,
          bubbles: true,
          cancelable: true,
        }),
      );
    // Synthetic pointer IDs are not browser capture targets.
    const capture = workspace.setPointerCapture;
    workspace.setPointerCapture = () => {};
    let mutations = 0;
    const observer = new MutationObserver((records) => {
      mutations += records.length;
    });
    observer.observe(wrap, { attributes: true, attributeFilter: ["style"] });
    try {
      send("pointerdown", 100, x - 40, y);
      send("pointerdown", 101, x + 40, y);
      const initial = wrap.style.transform;
      observer.takeRecords();
      mutations = 0;
      send("pointermove", 100, x - 60, y);
      const halfPair = wrap.style.transform;
      send("pointermove", 101, x + 60, y);
      const movePair = (left, right) => {
        const touches = [point(100, left, y), point(101, right, y)];
        workspace.dispatchEvent(
          new TouchEvent("touchmove", {
            touches,
            targetTouches: touches,
            bubbles: true,
          }),
        );
      };
      movePair(x - 50, x + 50);
      movePair(x - 60, x + 60);
      const beforeFrame = wrap.style.transform;
      await new Promise(requestAnimationFrame);
      const final = wrap.style.transform;
      const writes = mutations + observer.takeRecords().length;
      movePair(x - 70, x + 70);
      send("pointerup", 100, x - 70, y);
      send("pointerup", 101, x + 70, y);
      const released = wrap.style.transform;
      await new Promise(requestAnimationFrame);
      const afterRelease = wrap.style.transform;
      send("pointerdown", 100, x - 40, y);
      send("pointerdown", 101, x + 40, y);
      movePair(x - 80, x + 80);
      send("pointercancel", 100, x - 80, y);
      send("pointercancel", 101, x + 80, y);
      await new Promise(requestAnimationFrame);
      const afterCancel = wrap.style.transform;
      return {
        initial,
        halfPair,
        beforeFrame,
        final,
        writes,
        released,
        afterRelease,
        afterCancel,
      };
    } finally {
      observer.disconnect();
      workspace.setPointerCapture = capture;
    }
  });
  assert.equal(
    paired.halfPair,
    paired.initial,
    "one finger's new position must not cause a half-pair jump",
  );
  assert.equal(
    paired.beforeFrame,
    paired.initial,
    "pinch DOM writes must wait for the animation frame",
  );
  assert.notEqual(paired.final, paired.initial);
  assert.equal(
    paired.writes,
    2,
    "one zoom property and one transform write per pinch frame",
  );

  assert.notEqual(
    paired.released,
    paired.final,
    "lifting a finger must flush its latest paired position",
  );
  assert.equal(
    paired.afterRelease,
    paired.released,
    "no queued transform may run after release",
  );
  assert.equal(
    paired.afterCancel,
    paired.released,
    "cancelling must discard the pending pinch frame",
  );

  // The next independent touch still works after both pinch pointers lift.
  await page.click("#fit");
  await page.click("#brush");
  await touchDrag(await imagePoint(100, 100), await imagePoint(200, 100));
  assert.deepEqual(await sample(150, 100), [231, 91, 82, 255]);
  await session.detach();
}
