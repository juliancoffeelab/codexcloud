import assert from "node:assert/strict";
import { test, vi } from "vitest";
import * as Viewport from "@frontend/Viewport";
import { required } from "./helpers";

const rect = (left: number, right: number, top = 0, bottom = 40): Viewport.Rect => ({
  left,
  right,
  top,
  bottom,
});
const area = rect(100, 300, 20, 220);
const extent: Viewport.Extent = {
  width: 200,
  height: 200,
  scroll_width: 500,
  scroll_height: 500,
  scroll_left: 50,
  scroll_top: 60,
};

test("Inspection union gets12px margins, scrolls only when needed and prioritizes the final bound if it cannot fit", () => {
  assert.deepEqual(
    Viewport.position(area, extent, [rect(130, 150, 40, 80), rect(250, 270, 150, 190)]),
    { left: 50, top: 60 },
  );
  assert.deepEqual(
    Viewport.position(area, extent, [rect(80, 120, 40, 80), rect(150, 180, 150, 190)]),
    { left: 18, top: 60 },
  );
  assert.deepEqual(Viewport.position(area, extent, [rect(90, 170, 10, 90)]), { left: 28, top: 38 });
  assert.deepEqual(Viewport.position(area, extent, [rect(270, 310, 200, 240)]), {
    left: 72,
    top: 92,
  });
  assert.deepEqual(
    Viewport.position(area, extent, [rect(0, 50, 40, 80), rect(180, 230, 80, 120)]),
    { left: 50, top: 60 },
  );
  assert.deepEqual(Viewport.position(area, extent, [rect(180, 230, 80, 120), rect(0, 50, 0, 50)]), {
    left: 0,
    top: 28,
  });
});

test("Visibility never scrolls outside native bounds or moves a hidden/unmeasurable viewport", () => {
  assert.deepEqual(
    Viewport.position(area, { ...extent, scroll_left: 295, scroll_top: 295 }, [
      rect(600, 650, 600, 650),
    ]),
    { left: 300, top: 300 },
  );
  assert.deepEqual(Viewport.position(area, extent, [rect(-300, -250, -300, -250)]), {
    left: 0,
    top: 0,
  });
  assert.deepEqual(Viewport.position(area, { ...extent, width: 0 }, [rect(-300, -250)]), {
    left: 50,
    top: 60,
  });
  assert.deepEqual(Viewport.position(area, extent, []), { left: 50, top: 60 });
  assert.deepEqual(Viewport.position(area, extent, [rect(NaN, 100)]), { left: 50, top: 60 });
});

class Node {
  constructor(
    public rectangle: Viewport.Rect,
    readonly attrs: Record<string, string> = {},
  ) {}
  getBoundingClientRect() {
    return this.rectangle;
  }
  getAttribute(name: string) {
    return this.attrs[name] ?? null;
  }
}
class Scroller extends Node {
  clientWidth = 200;
  clientHeight = 200;
  scrollWidth = 1000;
  scrollHeight = 1000;
  scrollLeft = 100;
  scrollTop = 100;
}
const element = (node: object) => node as Element;
const settled = () => Promise.resolve();

test("Native reveal measures matching tile and unit nodes safely, favors inspected duplicates, and observes later resizes", async () => {
  const observers: Observer[] = [];
  class Observer {
    disconnected = false;
    observed: unknown;
    constructor(readonly callback: () => void) {
      observers.push(this);
    }
    observe(node: unknown) {
      this.observed = node;
    }
    disconnect() {
      this.disconnected = true;
    }
  }
  vi.stubGlobal("ResizeObserver", Observer);
  const inspected = '0,0"] [data-other="';
  const scroller = new Scroller(rect(0, 200, 0, 200));
  const board = {
    querySelectorAll(selector: string) {
      assert.equal(selector, "[data-tile], [data-unit-tile]");
      return [
        new Node(rect(-100, -50, 40, 80), { "data-tile": inspected }),
        new Node(rect(400, 450, 40, 80), { "data-tile": "target" }),
        new Node(rect(-110, -40, 30, 90), { "data-unit-tile": inspected }),
      ];
    },
  };
  const close = Viewport.reveal_map(element(scroller), element(board), [
    inspected,
    "target",
    inspected,
  ]);
  try {
    assert.equal(scroller.scrollLeft, 100, "Layout must settle before measurement");
    await settled();
    assert.equal(
      scroller.scrollLeft,
      0,
      "Final inspected ID takes priority over a distant combatant",
    );
    assert.equal(required(observers[0]).observed, scroller);
    scroller.scrollLeft = 100;
    required(observers[0]).callback();
    assert.equal(scroller.scrollLeft, 0);
    close();
    scroller.scrollLeft = 100;
    required(observers[0]).callback();
    assert.equal(scroller.scrollLeft, 100);
    assert(required(observers[0]).disconnected);
  } finally {
    close();
    vi.unstubAllGlobals();
  }
});

test("Effect cleanup fences queued measurement and late resize callbacks; missing observation remains usable", async () => {
  let callback: (() => void) | undefined;
  class Observer {
    constructor(run: () => void) {
      callback = run;
    }
    observe() {}
    disconnect() {}
  }
  vi.stubGlobal("ResizeObserver", Observer);
  const scroller = new Scroller(rect(0, 200, 0, 200));
  const board = {
    querySelectorAll() {
      return [new Node(rect(300, 350, 40, 80), { "data-tile": "a" })];
    },
  };
  const close = Viewport.reveal_map(element(scroller), element(board), ["a"]);
  close();
  await settled();
  required(callback)();
  assert.equal(scroller.scrollLeft, 100);
  vi.stubGlobal("ResizeObserver", undefined);
  const active = Viewport.reveal_map(element(scroller), element(board), ["a"]);
  try {
    await settled();
    assert.equal(scroller.scrollLeft, 262);
  } finally {
    active();
    vi.unstubAllGlobals();
  }
});

test("Attack choices reveal after layout only on mobile, with nearest scroll and stale cleanup guards", async () => {
  const calls: unknown[] = [];
  let mobile = true;
  vi.stubGlobal("matchMedia", (query: string) => {
    assert.equal(query, "(max-width:1000px)");
    return { matches: mobile };
  });
  const panel = {
    querySelector(selector: string) {
      assert.equal(selector, ".attack-actions");
      return {
        scrollIntoView(options: unknown) {
          calls.push(options);
        },
      };
    },
  };
  try {
    const cancelled = Viewport.reveal_attack(element(panel));
    cancelled();
    await settled();
    assert.deepEqual(calls, []);
    const active = Viewport.reveal_attack(element(panel));
    assert.deepEqual(calls, []);
    await settled();
    assert.deepEqual(calls, [{ block: "nearest" }]);
    active();
    mobile = false;
    Viewport.reveal_attack(element(panel));
    await settled();
    assert.equal(calls.length, 1);
    mobile = true;
    Viewport.reveal_attack(
      element({
        querySelector() {
          return {};
        },
      }),
    );
    await settled();
    assert.equal(calls.length, 1);
  } finally {
    vi.unstubAllGlobals();
  }
});
