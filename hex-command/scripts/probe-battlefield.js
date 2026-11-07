/** Opt-in production-browser probes. Artifacts and browser state stay in tmp/. */
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { chromium } from "playwright";
import { PNG } from "pngjs";
import { preview } from "vite";

const cases = [
  "startup-phone",
  "startup-desktop",
  "interactions",
  "camera-phone",
  "details-phone",
  "selection-phone",
  "camera-desktop",
  "assets",
  "boot",
  "textures",
  "start",
  "pipeline",
  "input",
];
const chosen = process.argv.slice(2);
if (chosen.includes("--help")) {
  console.log(
    `Run after npm run build. Optional case names: ${cases.join(", ")}\nCHROMIUM_PATH selects an existing Chromium executable. Reports/captures: tmp/battlefield-probes/.`,
  );
  process.exit(0);
}
for (const name of chosen) assert(cases.includes(name), `Unknown case: ${name}`);
assert(existsSync("dist/index.html"), "Build production assets first: npm run build");
const output = `tmp/battlefield-probes/${new Date().toISOString().replaceAll(/[:.]/g, "-")}`;
await mkdir(output, { recursive: true });
const report = {
  browser: "Chromium",
  productionHTML: await readFile("dist/index.html", "utf8"),
  cases: [],
};
let server, browser;

// Uses the application's existing Rust/controller tool boundary for game commands.
function instrument(mode) {
  const probe = (window.__battleProbe = { tools: {}, layouts: [] });
  Object.defineProperty(document, "modelContext", {
    value: {
      registerTool: (tool) => (probe.tools[tool.name] = tool),
      unregisterTool: (name) => delete probe.tools[name],
    },
  });
  let previous;
  const sample = () => {
    const host = document.querySelector("#map"),
      canvas = host?.querySelector("canvas");
    const r = host?.getBoundingClientRect();
    const value = {
      host: r ? [r.x, r.y, r.width, r.height] : null,
      buffer: canvas ? [canvas.width, canvas.height] : null,
      visible: canvas ? getComputedStyle(canvas).visibility : null,
      complete: !!(
        probe.lastGame?.scene?.getScene("battlefield")?.hasFrame &&
        !probe.lastGame.scene.getScene("battlefield").loading
      ),
    };
    const key = JSON.stringify(value);
    if (key !== previous) {
      probe.layouts.push({ time: performance.now(), ...value });
      previous = key;
    }
    requestAnimationFrame(sample);
  };
  requestAnimationFrame(sample);
  let phaser,
    fired = false;
  if (mode === "input") {
    const original = ResizeObserver.prototype.observe;
    ResizeObserver.prototype.observe = function (target, ...args) {
      const result = original.call(this, target, ...args);
      if (target.classList.contains("battlefield-clip") && !fired) {
        fired = true;
        throw Error("probe: input setup");
      }
      return result;
    };
  }
  Object.defineProperty(window, "Phaser", {
    configurable: true,
    get: () => phaser,
    set(value) {
      phaser = value;
      const boot = value.Game.prototype.boot;
      value.Game.prototype.boot = function () {
        probe.lastGame = this;
        if (mode === "boot" && !fired) {
          fired = true;
          throw Error("probe: boot");
        }
        return boot.call(this);
      };
      for (const [name, method] of [
        ["textures", "texturesReady"],
        ["start", "start"],
      ]) {
        if (mode !== name) continue;
        const original = value.Game.prototype[method];
        value.Game.prototype[method] = function (...args) {
          if (!fired) {
            fired = true;
            throw Error(`probe: ${name}`);
          }
          return original.apply(this, args);
        };
      }
      if (mode === "pipeline") {
        const manager = value.Renderer.WebGL.PipelineManager.prototype,
          original = manager.add;
        manager.add = function (name, pipeline) {
          const result = original.call(this, name, pipeline);
          if (name === "BattlefieldFogBlur" && !fired) {
            fired = true;
            throw Error("probe: partial pipeline setup");
          }
          return result;
        };
      }
    },
  });
}
const command = (page, name, input) =>
  page.evaluate(({ name, input }) => __battleProbe.tools[name].execute(input), { name, input });
const settle = (page) =>
  page.waitForFunction(() => {
    const s = __battleProbe.game?.scene.getScene("battlefield");
    return s?.hasFrame && !s.loading && __battleProbe.game.canvas.style.visibility === "visible";
  });
async function connect(page, cdp) {
  await page.waitForFunction(() => window.Phaser && __battleProbe.tools.get_battle_state);
  // When upstream requests a new game, start the ordinary fixture through its
  // normal UI before probing battlefield interactions.
  const picker = page.locator("dialog.new-game-dialog[open]");
  if (await picker.count()) {
    await picker.getByRole("button", { name: "Skirmish", exact: true }).click();
    await picker.getByRole("button", { name: "Start skirmish", exact: true }).click();
  }
  await page.locator("#map canvas").waitFor({ state: "visible" });
  const p = await cdp.send("Runtime.evaluate", { expression: "Phaser.Game.prototype" });
  const q = await cdp.send("Runtime.queryObjects", { prototypeObjectId: p.result.objectId });
  const prototype = await cdp.send("Runtime.callFunctionOn", {
    objectId: q.objects.objectId,
    functionDeclaration:
      "function(){return this.find(o=>o.constructor.prototype===o && typeof o.dispose==='function')}",
  });
  const instances = await cdp.send("Runtime.queryObjects", {
    prototypeObjectId: prototype.result.objectId,
  });
  await cdp.send("Runtime.callFunctionOn", {
    objectId: instances.objects.objectId,
    functionDeclaration:
      "function(){__battleProbe.game=this.find(g=>g.canvas?.isConnected&&!g.pendingDestroy)}",
  });
  await Promise.all(
    [p.result, q.objects, prototype.result, instances.objects].map((object) =>
      cdp.send("Runtime.releaseObject", { objectId: object.objectId }),
    ),
  );
  await settle(page);
}
const dimensions = (page) =>
  page.evaluate(() => [__battleProbe.game.canvas.width, __battleProbe.game.canvas.height]);
async function filmstrip(cdp, directory) {
  const frames = [],
    pending = [];
  const listener = (event) => {
    pending.push(
      (async () => {
        await cdp.send("Page.screencastFrameAck", { sessionId: event.sessionId });
        const bytes = Buffer.from(event.data, "base64"),
          png = PNG.sync.read(bytes);
        // This fixed ordinary/Bell phone fixture has non-black terrain/fog here.
        let black = 0,
          pixels = 0;
        for (let y = 110; y < Math.min(220, png.height); y++)
          for (let x = 24; x < Math.min(370, png.width); x++) {
            const i = (y * png.width + x) * 4;
            pixels++;
            if (!png.data[i] && !png.data[i + 1] && !png.data[i + 2]) black++;
          }
        const index = frames.length;
        frames.push({
          index,
          timestamp: event.metadata.timestamp,
          blackFraction: pixels ? black / pixels : 0,
        });
        await writeFile(`${directory}/frame-${String(index).padStart(3, "0")}.png`, bytes);
      })(),
    );
  };
  cdp.on("Page.screencastFrame", listener);
  await cdp.send("Page.startScreencast", {
    format: "png",
    maxWidth: 396,
    maxHeight: 707,
    everyNthFrame: 1,
  });
  return async () => {
    await cdp.send("Page.stopScreencast");
    cdp.off("Page.screencastFrame", listener);
    await Promise.all(pending);
    await writeFile(`${directory}/frames.json`, JSON.stringify(frames, null, 2));
    assert(frames.length, "No compositor frames captured");
    assert(
      !frames.some((f) => f.blackFraction > 0.98),
      "Black battlefield frame captured; inspect filmstrip",
    );
    return frames.length;
  };
}
async function startup(page, cdp, url, directory) {
  for (const kind of ["fresh", "saved"]) {
    if (kind === "fresh") await page.goto(url);
    else {
      await command(page, "start_campaign", { id: "bell-night" });
      await settle(page);
      await cdp.send("Network.clearBrowserCache");
      await page.reload();
    }
    await connect(page, cdp);
    const layouts = await page.evaluate(() => __battleProbe.layouts);
    await writeFile(`${directory}/${kind}-layout.json`, JSON.stringify(layouts, null, 2));
    const hosts = layouts.filter((l) => l.host);
    assert.equal(
      new Set(hosts.map((l) => l.host[3])).size,
      1,
      `${kind}: host height changed during startup`,
    );
    const visible = hosts.filter((l) => l.visible === "visible");
    assert(visible.length, `${kind}: scene never became visible`);
    assert(
      visible.every((l) => l.complete),
      `${kind}: canvas appeared before the complete scene`,
    );
    assert.equal(
      new Set(visible.map((l) => JSON.stringify([l.host, l.buffer]))).size,
      1,
      `${kind}: visible map changed allocation/geometry`,
    );
    await page.screenshot({ path: `${directory}/${kind}.png` });
  }
}
async function interactions(page, cdp, url, directory) {
  await page.goto(url);
  await connect(page, cdp);
  await command(page, "start_campaign", { id: "bell-night" });
  await settle(page);
  const originalSize = await dimensions(page);
  await page.evaluate(() => {
    __battleProbe.outlines = [...__battleProbe.game.scene.getScene("battlefield").outlines.list];
  });
  const stop = await filmstrip(cdp, directory);
  try {
    for (let i = 0; i < 8; i++) {
      const target = await page.evaluate(() => {
        const s = __battleProbe.game.scene.getScene("battlefield"),
          c = s.cameras.main;
        const u = s.frame.view.game.units.find((u) => u.character === "survey-captain");
        const t = s.frame.tiles.find((t) => t.id === u.tile),
          b = s.game.canvas.getBoundingClientRect();
        const p = c.matrix.transformPoint(t.x - c.scrollX, t.y - c.scrollY);
        return {
          id: u.id,
          x: b.left + (p.x * b.width) / s.game.canvas.width,
          y: b.top + (p.y * b.height) / s.game.canvas.height,
        };
      });
      await page.touchscreen.tap(target.x, target.y);
      await page.waitForFunction(
        (id) => __battleProbe.game.scene.getScene("battlefield").frame.view.selected === id,
        i % 2 ? null : target.id,
      );
      await page.waitForTimeout(100); // Observe intermediate and completed animation frames.
      assert.deepEqual(await dimensions(page), originalSize, "Selection reallocated the canvas");
    }
    assert(
      await page.evaluate(() => __battleProbe.outlines.every((o) => o.scene)),
      "Selection destroyed existing grid objects",
    );
  } finally {
    await stop();
  }
  const read = () =>
    page.evaluate(() => {
      const g = __battleProbe.game,
        c = g.scene.getScene("battlefield").cameras.main;
      return {
        zoom: c.zoomX / (g.canvas.width / g.canvas.getBoundingClientRect().width),
        x: c.scrollX,
        y: c.scrollY,
      };
    });
  const before = await read();
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [
      { x: 90, y: 200, id: 1 },
      { x: 260, y: 200, id: 2 },
    ],
  });
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchMove",
    touchPoints: [
      { x: 50, y: 200, id: 1 },
      { x: 300, y: 200, id: 2 },
    ],
  });
  await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  await page.waitForFunction(
    (zoom) =>
      __battleProbe.game.scene.getScene("battlefield").cameras.main.zoomX /
        (__battleProbe.game.canvas.width /
          __battleProbe.game.canvas.getBoundingClientRect().width) >
      zoom,
    before.zoom,
  );
  assert(
    Math.abs((await read()).zoom / before.zoom - 250 / 170) < 0.01,
    "Pinch scale drifted from finger separation",
  );
  assert.deepEqual(await dimensions(page), originalSize, "Pinch reallocated the canvas");
  const finger = await read(),
    timestamp = Date.now() / 1000;
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    timestamp,
    touchPoints: [{ x: 280, y: 240, id: 1 }],
  });
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchMove",
    timestamp: timestamp + 0.016,
    touchPoints: [{ x: 150, y: 220, id: 1 }],
  });
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchEnd",
    timestamp: timestamp + 0.032,
    touchPoints: [],
  });
  await page.waitForFunction(
    (x) => __battleProbe.game.scene.getScene("battlefield").cameras.main.scrollX > x,
    finger.x,
  );
  assert.equal(
    await page.evaluate(() => __battleProbe.game.scene.getScene("battlefield").frame.view.selected),
    null,
    "Finger drag selected a tile",
  );
  assert.deepEqual(await dimensions(page), originalSize);
  await page.keyboard.press("Escape"); // Stop release momentum before the independent wheel check.
  await page.mouse.move(180, 200);
  const pan = await read();
  await page.mouse.wheel(-120, -100);
  await page.waitForFunction(({ x, y }) => {
    const c = __battleProbe.game.scene.getScene("battlefield").cameras.main;
    return c.scrollX !== x || c.scrollY !== y;
  }, pan);
  await page.locator("#map canvas").focus();
  const keyboardZoom = (await read()).zoom;
  await page.keyboard.press("+");
  await page.waitForFunction((zoom) => {
    const g = __battleProbe.game;
    return (
      g.scene.getScene("battlefield").cameras.main.zoomX /
        (g.canvas.width / g.canvas.getBoundingClientRect().width) >
      zoom
    );
  }, keyboardZoom);
  assert.deepEqual(await dimensions(page), originalSize, "Keyboard zoom reallocated the canvas");
  await page.keyboard.press("Escape");
  await page.setViewportSize({ width: 707, height: 396 });
  await page.waitForFunction(
    (old) =>
      __battleProbe.game.canvas.width !== old[0] || __battleProbe.game.canvas.height !== old[1],
    originalSize,
  );
}
// Native phone details scrolling: no custom gesture ownership or sheet state.
async function detailsScroll(page, cdp, url, directory) {
  await page.goto(url);
  await connect(page, cdp);
  const initialCamera = await cameraSnapshot(page);
  await page.touchscreen.tap(initialCamera.commander.x, initialCamera.commander.y);
  await page.locator("aside").waitFor();
  await cameraSettled(page);
  const read = () =>
    page.evaluate(() => {
      const g = __battleProbe.game,
        c = g.scene.getScene("battlefield").cameras.main;
      const outer = document.querySelector(".battle-content"),
        aside = document.querySelector("aside");
      const header = document.querySelector(".battle-bar"),
        clip = g.canvas.parentElement;
      return {
        outerTop: outer.scrollTop,
        outerY: outer.getBoundingClientRect().top,
        detailsY: aside.getBoundingClientRect().top,
        detailsScroll: aside.scrollTop,
        headerY: header.getBoundingClientRect().top,
        rootTop: document.scrollingElement.scrollTop,
        canvas: [g.canvas.width, g.canvas.height, g.canvas.style.width, g.canvas.style.height],
        camera: [c.scrollX, c.scrollY, c.zoomX, c.zoomY],
        overscroll: getComputedStyle(outer).overscrollBehaviorY,
        headerOverscroll: getComputedStyle(header).overscrollBehaviorY,
        radius: getComputedStyle(clip).borderBottomRightRadius,
      };
    });
  const initial = await read(),
    snapshots = [{ label: "selected", ...initial }];
  assert.equal(initial.overscroll, "contain");
  assert.equal(initial.headerOverscroll, "auto");
  assert(parseFloat(initial.radius) > 0, "Visible map clip lost rounded corners");
  const swipe = async (label, from, to) => {
    await cdp.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [{ x: 190, y: from }],
    });
    for (let i = 1; i <= 20; i++) {
      await cdp.send("Input.dispatchTouchEvent", {
        type: "touchMove",
        touchPoints: [{ x: 190, y: from + ((to - from) * i) / 20 }],
      });
      await page.evaluate(() => new Promise(requestAnimationFrame));
    }
    // Sample before release: the original contact must already move the details.
    const state = await read();
    await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
    await page.waitForTimeout(250);
    snapshots.push({ label, ...state });
    assert.deepEqual(state.canvas, initial.canvas, "Details scrolling resized the canvas");
    assert.deepEqual(state.camera, initial.camera, "Details scrolling navigated the world");
    assert.equal(state.headerY, initial.headerY, "Details scrolling moved the toolbar");
    assert.equal(state.rootTop, initial.rootTop, "Details gesture escaped into the document");
    assert.equal(state.detailsScroll, 0, "Details became a separate scroll container");
    return state;
  };
  const raised = await swipe("raise", Math.min(initial.detailsY + 60, 620), initial.outerY + 40);
  assert(
    raised.outerTop > initial.outerTop + 20 && raised.detailsY < initial.detailsY - 20,
    "Upward details swipe did not raise the shared content",
  );
  const lower = await swipe("lower", Math.max(raised.detailsY + 30, initial.outerY + 40), 620);
  assert(
    lower.outerTop < raised.outerTop && lower.detailsY > raised.detailsY,
    "Downward details swipe did not reveal more map",
  );
  await page.screenshot({ path: `${directory}/details.png` });
  await page.keyboard.press("Escape");
  await page.waitForFunction(
    () =>
      document.querySelector(".workspace.map-only") &&
      document.querySelector(".battle-content").scrollTop === 0,
  );
  assert.deepEqual(
    await dimensions(page),
    initial.canvas.slice(0, 2),
    "Closing details reallocated the canvas",
  );
  await writeFile(`${directory}/scroll.json`, JSON.stringify(snapshots, null, 2));
  return snapshots;
}
// Reproduce the campaign unit that was visible before the details clipped it.
async function selectionVisibility(page, cdp, url, directory) {
  await page.goto(url);
  await connect(page, cdp);
  await command(page, "start_campaign", { id: "lantern-road" });
  await settle(page);
  for (let i = 0; i < 30; i++) {
    const pending = await page.evaluate(() => {
      const campaign = __battleProbe.game.scene.getScene("battlefield").frame.view.game.campaign;
      return campaign.dialogueIndex < campaign.dialogue.length;
    });
    if (!pending) break;
    await command(page, "advance_dialogue");
  }
  await page.waitForFunction(() => !document.querySelector("dialog[open]"));
  await cameraSettled(page);
  const read = () =>
    page.evaluate(() => {
      const g = __battleProbe.game,
        s = g.scene.getScene("battlefield"),
        c = s.cameras.main;
      const unit = s.frame.view.game.units.find((u) => u.character === "oren"),
        tile = s.frame.tiles.find((t) => t.id === unit.tile);
      const canvas = g.canvas.getBoundingClientRect(),
        host = g.canvas.parentElement.getBoundingClientRect();
      const project = (x, y) => {
        const p = c.matrix.transformPoint(x - c.scrollX, y - c.scrollY);
        return {
          x: canvas.x + (p.x * canvas.width) / g.canvas.width,
          y: canvas.y + (p.y * canvas.height) / g.canvas.height,
        };
      };
      const bounds = s.actors.get(unit.id).image.getBounds();
      return {
        id: unit.id,
        point: project(tile.x, tile.y),
        topLeft: project(bounds.x, bounds.y),
        bottomRight: project(bounds.right, bounds.bottom),
        host: host.toJSON(),
        camera: [c.scrollX, c.scrollY, c.zoomX, c.zoomY],
        buffer: [g.canvas.width, g.canvas.height],
      };
    });
  const before = await read();
  const outlines = () =>
    page.evaluate(() =>
      [...__battleProbe.game.scene.getScene("battlefield").hexes].map(([id, { stroke }]) => [
        id,
        stroke.tintTopLeft,
        stroke.alpha,
        stroke.pipelineData.width,
      ]),
    );
  const beforeContact = await outlines();
  await page.locator("#map canvas").focus();
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(resolve)));
  assert.deepEqual(await outlines(), beforeContact, "DOM focus activated a tile marker");
  await page.evaluate(() => document.activeElement.blur());
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [{ x: before.point.x, y: before.point.y }],
  });
  await page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))),
  );
  assert.deepEqual(
    await outlines(),
    beforeContact,
    "Holding a pointer activated an unrelated tile marker",
  );
  await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  await page.waitForFunction(
    (id) => __battleProbe.game.scene.getScene("battlefield").frame.view.selected === id,
    before.id,
  );
  await cameraSettled(page);
  const after = await read();
  await page.screenshot({ path: `${directory}/selected.png` });
  assert(
    after.topLeft.x >= after.host.left &&
      after.topLeft.y >= after.host.top &&
      after.bottomRight.x <= after.host.right &&
      after.bottomRight.y <= after.host.bottom,
    "Selected Oren sprite is clipped by the details viewport",
  );
  assert.deepEqual(after.buffer, before.buffer, "Selection resized the canvas");
  assert.deepEqual(after.camera.slice(2), before.camera.slice(2), "Selection changed world scale");
  await page.mouse.move(
    (after.host.left + after.host.right) / 2,
    (after.host.top + after.host.bottom) / 2,
  );
  await page.mouse.wheel(70, 100);
  await page.waitForFunction((camera) => {
    const c = __battleProbe.game.scene.getScene("battlefield").cameras.main;
    return c.scrollX !== camera[0] || c.scrollY !== camera[1];
  }, after.camera);
  const manual = await read();
  await page.evaluate(() => {
    const s = __battleProbe.game.scene.getScene("battlefield");
    s.present(s.frame); // An unchanged publication must not reclaim camera attention.
  });
  await cameraSettled(page);
  assert.deepEqual(
    (await read()).camera,
    manual.camera,
    "Unchanged selection snapped back after manual pan",
  );
  const selection = () =>
    page.evaluate(() => {
      const view = __battleProbe.game.scene.getScene("battlefield").frame.view;
      return [view.selected, view.inspectedTile];
    });
  const beforeKeys = await selection();
  for (const key of ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Enter", "Space"])
    await page.keyboard.press(key);
  assert.deepEqual(await selection(), beforeKeys, "Removed keyboard navigation changed selection");
  await writeFile(
    `${directory}/selection.json`,
    JSON.stringify({ before, after, manual }, null, 2),
  );
  return { before, after, manual };
}
// Wait for the camera viewport to consume the latest Svelte layout, then render it.
async function cameraSettled(page) {
  await settle(page);
  await page.waitForFunction(() => {
    const g = __battleProbe.game,
      c = g.scene.getScene("battlefield").cameras.main;
    const h = g.canvas.parentElement,
      b = g.canvas.getBoundingClientRect();
    const ratio = g.canvas.width / b.width;
    return (
      Math.abs(c.width / ratio - h.clientWidth) < 1 &&
      Math.abs(c.height / ratio - Math.min(b.height, h.clientHeight)) < 1
    );
  });
  await page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))),
  );
}
const cameraSnapshot = (page) =>
  page.evaluate(() => {
    const g = __battleProbe.game,
      s = g.scene.getScene("battlefield"),
      c = s.cameras.main;
    const h = g.canvas.parentElement,
      b = g.canvas.getBoundingClientRect(),
      r = h.getBoundingClientRect();
    const unit = s.frame.view.game.units.find(
      (u) => s.frame.view.game.faction(u.id).controllable && u.role === "leader",
    );
    const tile = s.frame.tiles.find((t) => t.id === unit.tile);
    const point = (t) => {
      const p = c.matrix.transformPoint(t.x - c.scrollX, t.y - c.scrollY);
      return {
        id: t.id,
        x: b.left + (p.x * b.width) / g.canvas.width,
        y: b.top + (p.y * b.height) / g.canvas.height,
      };
    };
    return {
      map: s.frame.view.game.mapId,
      selected: s.frame.view.selected,
      inspected: s.frame.view.inspectedTile,
      inspectedPoint: s.frame.tiles.find((t) => t.id === s.frame.view.inspectedTile)
        ? point(s.frame.tiles.find((t) => t.id === s.frame.view.inspectedTile))
        : null,
      zoom: c.zoomX / (g.canvas.width / b.width),
      relativeZoom: s.controls.snapshot().zoom,
      scroll: [c.scrollX, c.scrollY],
      commander: { ...point(tile), unitId: unit.id },
      host: { left: r.left, top: r.top, right: r.right, bottom: r.bottom },
      empty: s.frame.tiles
        .filter((t) => !t.occupant)
        .map(point)
        .sort((a, b) => b.x - a.x)
        .find(
          (p) =>
            p.x > r.left + 60 && p.x < r.right - 100 && p.y > r.top + 60 && p.y < r.bottom - 60,
        ),
    };
  });
async function camera(page, cdp, url, directory) {
  await page.goto(url);
  await connect(page, cdp);
  const snapshots = [];
  const record = async (label) => {
    await cameraSettled(page);
    const snapshot = await cameraSnapshot(page);
    snapshots.push({ label, ...snapshot });
    return snapshot;
  };
  const visible = (s) => {
    const { commander: p, host: h, zoom } = s;
    assert(
      p.x - 35 * zoom >= h.left &&
        p.x + 35 * zoom <= h.right &&
        p.y - 50 * zoom >= h.top &&
        p.y + 35 * zoom <= h.bottom,
      "Initial commander artwork falls outside the viewport",
    );
  };
  const click = async (p) => {
    if (await page.evaluate(() => matchMedia("(max-width: 1000px)").matches))
      await page.touchscreen.tap(p.x, p.y);
    else await page.mouse.click(p.x, p.y);
  };
  const initial = await record("fresh"),
    size = await dimensions(page);
  visible(initial);
  await click(initial.commander);
  await page.waitForFunction(
    (id) => __battleProbe.game.scene.getScene("battlefield").frame.view.selected === id,
    initial.commander.unitId,
  );
  const selected = await record("selected");
  assert(Math.abs(selected.zoom - initial.zoom) < 1e-6, "Opening unit details changed world scale");
  visible(selected);
  const stillFits =
    initial.commander.x - 35 * initial.zoom >= selected.host.left &&
    initial.commander.x + 35 * initial.zoom <= selected.host.right &&
    initial.commander.y - 50 * initial.zoom >= selected.host.top &&
    initial.commander.y + 35 * initial.zoom <= selected.host.bottom;
  if (stillFits)
    assert.deepEqual(
      selected.scroll,
      initial.scroll,
      "Already visible commander caused a camera jump",
    );
  await click(selected.commander);
  await page.waitForFunction(
    () => !__battleProbe.game.scene.getScene("battlefield").frame.view.selected,
  );
  const clear = await record("deselected");
  assert(Math.abs(clear.zoom - initial.zoom) < 1e-6, "Closing details changed world scale");
  assert(clear.empty, "No visible empty terrain available");
  await click(clear.empty);
  await page.waitForFunction(
    (id) => __battleProbe.game.scene.getScene("battlefield").frame.view.inspectedTile === id,
    clear.empty.id,
  );
  const inspected = await record("terrain");
  assert(Math.abs(inspected.zoom - initial.zoom) < 1e-6, "Inspecting terrain changed world scale");
  assert(
    inspected.inspectedPoint.x - 31 * inspected.zoom >= inspected.host.left &&
      inspected.inspectedPoint.x + 31 * inspected.zoom <= inspected.host.right &&
      inspected.inspectedPoint.y - 35 * inspected.zoom >= inspected.host.top &&
      inspected.inspectedPoint.y + 35 * inspected.zoom <= inspected.host.bottom,
    "Inspected terrain is hidden by details",
  );
  assert.deepEqual(await dimensions(page), size, "Inspection reallocated the backing canvas");
  await page.locator("#map canvas").focus();
  await page.keyboard.press("+");
  const zoomed = await record("keyboard-zoom");
  assert(Math.abs(zoomed.relativeZoom - 1.2) < 1e-6, "Keyboard zoom was lost to a queued reveal");
  await page.mouse.move(180, 200);
  await page.mouse.wheel(140, 180);
  await page.waitForFunction((old) => {
    const c = __battleProbe.game.scene.getScene("battlefield").cameras.main;
    return c.scrollX !== old[0] || c.scrollY !== old[1];
  }, zoomed.scroll);
  const manuallyPanned = await record("manual-pan");
  const viewport = page.viewportSize();
  await page.setViewportSize({
    ...viewport,
    height: viewport.height - (viewport.width > 1000 ? 200 : 24),
  });
  const resized = await record("height-resize");
  await writeFile(
    `${directory}/resize.json`,
    JSON.stringify({ before: manuallyPanned, after: resized }, null, 2),
  );
  // Phaser's render matrix stores Float32 coefficients. Independent framebuffer
  // rounding can change a projected point by a few ULPs with identical camera
  // state; reject real drift without demanding double precision from that matrix.
  const sameProjected = (a, b) =>
    Math.abs(a - b) <= 4 * 2 ** -23 * Math.max(1, Math.abs(a), Math.abs(b));
  assert(
    resized.scroll.every((value, i) => Math.abs(value - manuallyPanned.scroll[i]) < 1e-10) &&
      Math.abs(resized.zoom - manuallyPanned.zoom) < 1e-10 &&
      sameProjected(resized.commander.x, manuallyPanned.commander.x) &&
      sameProjected(resized.commander.y, manuallyPanned.commander.y),
    "Resize moved the manually panned world projection",
  );
  await page.setViewportSize(viewport);
  await cameraSettled(page);
  await command(page, "start_campaign", { id: "bell-night" });
  const bell = await record("new-bell");
  assert.equal(bell.relativeZoom, 1, "New map inherited previous zoom");
  visible(bell);
  await page.locator("#map canvas").focus();
  await page.keyboard.press("+");
  await record("bell-zoom");
  await command(page, "start_new_battle", { mapId: "mirror" });
  const mirror = await record("new-mirror");
  assert.equal(mirror.relativeZoom, 1);
  assert.deepEqual(mirror.scroll, initial.scroll, "New game inherited previous map pan");
  visible(mirror);
  await page.reload();
  await connect(page, cdp);
  const saved = await record("saved");
  visible(saved);
  assert.equal(saved.relativeZoom, 1);
  // A point just inside a rendered edge must use the same projection as its hex.
  const edge = await page.evaluate(() => {
    const g = __battleProbe.game,
      s = g.scene.getScene("battlefield"),
      c = s.cameras.main;
    const b = g.canvas.getBoundingClientRect(),
      h = document.querySelector("#map").getBoundingClientRect();
    for (const t of s.frame.tiles.filter((t) => !t.occupant && t.col >= 3)) {
      const p = c.matrix.transformPoint(t.x - 30.625 + 0.2 - c.scrollX, t.y - c.scrollY);
      const x = b.left + (p.x * b.width) / g.canvas.width,
        y = b.top + (p.y * b.height) / g.canvas.height;
      if (x > h.left + 10 && x < h.right - 10 && y > h.top + 10 && y < h.bottom - 10)
        return { id: t.id, x, y };
    }
  });
  assert(edge, "No visible edge fixture");
  await click(edge);
  await page.waitForFunction(
    (id) => __battleProbe.game.scene.getScene("battlefield").frame.view.inspectedTile === id,
    edge.id,
  );
  await record("edge-hit");
  await command(page, "start_campaign", { id: "bell-night" });
  await cameraSettled(page);
  if (page.viewportSize().width <= 1000) {
    // Read the expanded mobile briefing first; collapse it for a visible map tap.
    await page.locator("#mission-briefing-button").click();
    await page.evaluate(() => __battleProbe.game.scale.updateBounds());
  }
  const briefing = await page.evaluate(async () => {
    const g = __battleProbe.game,
      s = g.scene.getScene("battlefield"),
      c = s.cameras.main;
    document.querySelector("#mission-briefing-button").click();
    await Promise.resolve();
    await Promise.resolve();
    const b = g.canvas.getBoundingClientRect(),
      h = document.querySelector("#map").getBoundingClientRect();
    const point = s.frame.tiles
      .map((t) => {
        const p = c.matrix.transformPoint(t.x - c.scrollX, t.y - c.scrollY);
        return {
          id: t.id,
          x: b.left + (p.x * b.width) / g.canvas.width,
          y: b.top + (p.y * b.height) / g.canvas.height,
        };
      })
      .find(
        (p) =>
          p.x > h.left + 30 && p.x < h.right - 30 && p.y > h.top + 30 && p.y < innerHeight - 30,
      );
    if (point)
      for (const type of ["mousedown", "mouseup"])
        g.canvas.dispatchEvent(
          new MouseEvent(type, {
            bubbles: true,
            clientX: point.x,
            clientY: point.y,
            button: 0,
            buttons: type === "mousedown" ? 1 : 0,
          }),
        );
    return point;
  });
  assert(briefing, "Briefing toggle left no visible map point");
  await page.waitForFunction(
    (id) => __battleProbe.game.scene.getScene("battlefield").frame.view.inspectedTile === id,
    briefing.id,
  );
  await record("immediate-briefing-hit");
  await writeFile(`${directory}/camera.json`, JSON.stringify(snapshots, null, 2));
  await page.screenshot({ path: `${directory}/camera.png` });
  return snapshots;
}
async function assets(page, cdp, url) {
  await page.goto(url);
  await connect(page, cdp);
  const held = [];
  const originalURLs = new Set(
    (await page.evaluate(() => __battleProbe.game.textures.getTextureKeys())).map(
      (key) => new URL(key, url).href,
    ),
  );
  let restoredRequests = 0;
  await page.route("**/assets/sprites/*.png", (r) => {
    // Replacing a scene may legitimately evict its old textures. Let that
    // scene reload its own artwork; only unrelated new art stays stalled.
    if (originalURLs.has(r.request().url())) {
      restoredRequests++;
      return r.continue();
    }
    held.push(r);
  });
  const state = await command(page, "get_battle_state");
  const captain = state.units.find((u) => u.side === "player" && u.role === "leader");
  const soldier = state.units.find((u) => u.side === "player" && u.type === "soldier");
  const choices = await command(page, "get_unit_actions", { unitId: captain.id });
  await command(page, "recruit_unit", {
    commanderId: captain.id,
    type: "swordsman",
    tileId: choices.recruitment.swordsman[0],
  });
  await settle(page);
  const recruited = (await command(page, "get_battle_state")).units.find(
    (unit) => unit.side === "player" && unit.type === "swordsman",
  );
  assert(
    await page.evaluate((id) => {
      const s = __battleProbe.game.scene.getScene("battlefield");
      return s.actors.has(id) && !s.loading;
    }, recruited.id),
    "Predictable recruitment did not use prepared Phaser textures",
  );
  const moves = await command(page, "get_unit_actions", { unitId: soldier.id });
  await command(page, "move_unit", { unitId: soldier.id, tileId: moves.moves[0].tile });
  await command(page, "start_campaign", { id: "bell-night" });
  await page.waitForFunction(() => {
    const s = __battleProbe.game.scene.getScene("battlefield");
    return [...s.requests.values()].some((file) => s.requiredImages.has(file.key));
  });
  await page.evaluate(() => {
    __battleProbe.stalled = [
      ...__battleProbe.game.scene.getScene("battlefield").requests.values(),
    ].find((file) => __battleProbe.game.scene.getScene("battlefield").requiredImages.has(file.key));
  });
  const bell = await command(page, "get_battle_state");
  const ready = bell.units.find((unit) => unit.side === "player" && unit.role === "leader");
  await page.waitForFunction(
    (id) => __battleProbe.game.scene.getScene("battlefield").actors.has(id),
    ready.id,
  );
  // Presentation-only input fixture; the authoritative Rust session is untouched.
  await page.evaluate((id) => {
    const s = __battleProbe.game.scene.getScene("battlefield"),
      original = s.frame;
    const units = original.view.game.units.map((u) => (u.id === id ? { ...u, hp: 5 } : u));
    const tiles = original.tiles.map((t) =>
      t.occupant?.unit.id === id
        ? {
            ...t,
            occupant: {
              ...t.occupant,
              unit: units.find((u) => u.id === id),
              healthWidth: (t.occupant.healthWidth * 5) / t.occupant.unit.hp,
            },
          }
        : t,
    );
    try {
      s.present({
        ...original,
        view: {
          ...original.view,
          game: { ...original.view.game, units, unit: (key) => units.find((u) => u.id === key) },
        },
        tiles,
      });
      if (s.actors.get(id).data.hp !== 5) throw Error("Published HP remained stale");
    } finally {
      s.present(original);
    }
  }, ready.id);
  await command(page, "start_new_battle", { mapId: "mirror" });
  await settle(page);
  assert(
    await page.evaluate(
      () =>
        __battleProbe.stalled.loader === null &&
        (!__battleProbe.stalled.xhrLoader || __battleProbe.stalled.xhrLoader.readyState === 0),
    ),
    "Obsolete image preparation was not released",
  );
  for (let i = 0; i < 3; i++) {
    await command(page, "start_campaign", { id: "bell-night" });
    await page.waitForFunction(() => {
      const s = __battleProbe.game.scene.getScene("battlefield");
      return [...s.requests.values()].some((file) => s.requiredImages.has(file.key));
    });
    await command(page, "start_new_battle", { mapId: "mirror" });
    await settle(page);
    assert.equal(
      await page.locator("#map canvas").count(),
      1,
      "Rapid replacement duplicated the canvas",
    );
  }
  await Promise.all(
    held.map((route) => route.fulfill({ status: 500, body: "obsolete failure" }).catch(() => {})),
  );
  await page.unroute("**/assets/sprites/*.png");
  await page.waitForTimeout(100); // Allow canceled requests' late response events to arrive.
  assert.equal(
    await page.locator(".map-render-error").count(),
    0,
    "Obsolete error poisoned the current scene",
  );
  // A current failure must still surface and recover.
  await page.route("**/assets/sprites/*.png", (r) => r.abort());
  await command(page, "start_campaign", { id: "bell-night" });
  await page.getByRole("button", { name: "Retry map rendering" }).waitFor();
  assert.equal(await page.locator("#map canvas").count(), 0);
  await page.unroute("**/assets/sprites/*.png");
  await page.getByRole("button", { name: "Retry map rendering" }).click();
  await connect(page, cdp);
  await page.evaluate(() =>
    __battleProbe.game.renderer.gl.getExtension("WEBGL_lose_context").loseContext(),
  );
  await page.getByRole("button", { name: "Retry map rendering" }).waitFor();
  await page.getByRole("button", { name: "Retry map rendering" }).click();
  await connect(page, cdp);
  assert.equal(await page.locator("#map canvas").count(), 1);
  return { replacements: 3, restoredRequests };
}
async function failure(page, url) {
  await page.goto(url);
  await page.getByRole("button", { name: "Retry map rendering" }).waitFor();
  const diagnostic = await page.locator(".map-render-error").innerText();
  assert(diagnostic.includes("probe:"));
  assert(!diagnostic.includes("Cleanup:"), `Partial cleanup failed: ${diagnostic}`);
  const old = await page.evaluateHandle(() => __battleProbe.lastGame);
  assert(await old.evaluate((g) => !g.loop.running && !g.canvas?.isConnected));
  assert.equal(await page.locator("#map canvas").count(), 0);
  await page.getByRole("button", { name: "Retry map rendering" }).click();
  await page.waitForFunction(
    () =>
      __battleProbe.lastGame?.canvas?.isConnected &&
      __battleProbe.lastGame.canvas.style.visibility === "visible",
  );
  assert(await old.evaluate((g) => !g.loop.running && (!g.renderer || g.renderer.gl === null)));
  assert.equal(await page.locator("#map canvas").count(), 1);
  await old.dispose();
}
try {
  server = await preview({
    logLevel: "error",
    preview: { host: "127.0.0.1", port: 0, strictPort: true },
  });
  const url = `http://127.0.0.1:${server.httpServer.address().port}`;
  browser = await chromium.launch({
    executablePath:
      process.env.CHROMIUM_PATH ||
      (existsSync("/usr/bin/chromium") ? "/usr/bin/chromium" : undefined),
    args: ["--no-sandbox", "--enable-unsafe-swiftshader"],
  });
  report.browserVersion = browser.version();
  for (const name of chosen.length ? chosen : cases) {
    const directory = `${output}/${name}`;
    await mkdir(directory);
    const desktop = name.endsWith("-desktop");
    const context = await browser.newContext({
      viewport: desktop ? { width: 1280, height: 900 } : { width: 396, height: 707 },
      deviceScaleFactor: desktop ? 2 : 2.727272727,
      isMobile: !desktop,
      hasTouch: true,
    });
    const page = await context.newPage(),
      errors = [];
    page.on("pageerror", (e) => errors.push(e.stack ?? e.message));
    await context.addInitScript(
      instrument,
      ["boot", "textures", "start", "pipeline", "input"].includes(name) ? name : undefined,
    );
    await context.tracing.start({ screenshots: true, snapshots: true });
    const cdp = await context.newCDPSession(page),
      result = { name };
    const started = performance.now();
    try {
      if (name.startsWith("startup")) await startup(page, cdp, url, directory);
      else if (name === "interactions") await interactions(page, cdp, url, directory);
      else if (name === "details-phone")
        result.details = await detailsScroll(page, cdp, url, directory);
      else if (name === "selection-phone")
        result.details = await selectionVisibility(page, cdp, url, directory);
      else if (name.startsWith("camera")) result.details = await camera(page, cdp, url, directory);
      else if (name === "assets") result.details = await assets(page, cdp, url);
      else await failure(page, url);
      assert.deepEqual(errors, [], "Uncaught browser exceptions");
      result.passed = true;
    } catch (error) {
      result.passed = false;
      result.error = String(error.stack ?? error);
      result.scene = await page
        .evaluate(() => {
          const g = __battleProbe.game ?? __battleProbe.lastGame,
            s = g?.scene?.getScene("battlefield");
          return {
            canvasConnected: !!g?.canvas?.isConnected,
            running: g?.loop?.running,
            identity: s?.identity,
            loading: s?.loading,
            failed: s?.failed,
            cleanupErrors: g?.cleanupErrors?.map((error) => String(error.stack ?? error)),
            pending: s?.requests ? [...s.requests.keys()] : [],
            published: s?.frame?.view.game.units.map(({ id, hp, tile }) => ({ id, hp, tile })),
            displayed: s?.actors
              ? [...s.actors.values()].map(({ data: { id, hp, tile } }) => ({ id, hp, tile }))
              : [],
          };
        })
        .catch(() => ({ unavailable: true }));
      await page.screenshot({ path: `${directory}/failure.png` }).catch(() => {});
    } finally {
      result.runtimeMs = performance.now() - started;
      result.pageErrors = errors;
      report.cases.push(result);
      await context.tracing.stop({ path: `${directory}/trace.zip` });
      await context.close();
      await writeFile(`${output}/report.json`, JSON.stringify(report, null, 2));
    }
    console.log(
      `${result.passed ? "PASS" : "FAIL"} ${name}${result.error ? `: ${result.error}` : ""}`,
    );
  }
  if (report.cases.some((c) => !c.passed)) process.exitCode = 1;
} finally {
  await browser?.close();
  await new Promise((resolve) => (server ? server.httpServer.close(resolve) : resolve()));
  console.log(`Artifacts: ${output}`);
}
