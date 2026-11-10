import assert from "node:assert/strict";
function mockSpriteCanvas() {
  const original = Object.getOwnPropertyDescriptor(globalThis, "document");
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: {
      documentElement: { dataset: {} },
      querySelector: () => null,
      querySelectorAll: () => [],
      createElement: (name) => {
        assert.equal(name, "canvas");
        let extraction;
        const draws = [];
        const paths = [];
        const scales = [];
        let points = [];
        let clips = 0;
        const canvas = {
          width: 0,
          height: 0,
          getContext: () => context,
          toDataURL: () =>
            "data:image/png;base64," + Buffer.from(JSON.stringify(extraction)).toString("base64"),
        };
        const context = {
          imageSmoothingEnabled: true,
          save() {},
          restore() {},
          scale(x, y) {
            scales.push([x, y]);
          },
          fill() {},
          beginPath() {
            points = [];
            paths.push(points);
          },
          moveTo(x, y) {
            points.push([x, y]);
          },
          lineTo(x, y) {
            points.push([x, y]);
          },
          closePath() {},
          clip() {
            clips++;
          },
          drawImage(image, ...coordinates) {
            draws.push({ source: image.src, coordinates });
            extraction ??= {
              source: image.src,
              coordinates,
              width: canvas.width,
              height: canvas.height,
              smoothing: context.imageSmoothingEnabled,
              draws,
              paths,
              scales,
              get clips() {
                return clips;
              },
            };
          },
        };
        return canvas;
      },
    },
  });
  return () => {
    if (original) Object.defineProperty(globalThis, "document", original);
    else Reflect.deleteProperty(globalThis, "document");
  };
}
function required(value) {
  assert.ok(value !== void 0 && value !== null, "Expected fixture value");
  return value;
}
class TestStorage {
  constructor(values = /* @__PURE__ */ new Map()) {
    this.values = values;
  }
  values;
  get length() {
    return this.values.size;
  }
  clear() {
    this.values.clear();
  }
  getItem(key) {
    return this.values.get(key) ?? null;
  }
  key(index) {
    return [...this.values.keys()][index] ?? null;
  }
  removeItem(key) {
    this.values.delete(key);
  }
  setItem(key, value) {
    this.values.set(key, value);
  }
}
function storageWithValue(read, write) {
  return {
    get length() {
      return read() ? 1 : 0;
    },
    clear() {
      write("");
    },
    getItem() {
      return read() || null;
    },
    key(index) {
      return index === 0 && read() ? "hex-command-battle" : null;
    },
    removeItem() {
      write("");
    },
    setItem(_key, value) {
      write(value);
    },
  };
}
export { TestStorage, mockSpriteCanvas, required, storageWithValue };

// Explicit artwork geometry belongs to fixture construction, never to restore.
export function setTerrain(tile, terrain) {
  for (const field of ["connections", "deadEnds", "walls"]) delete tile[field];
  tile.terrain = terrain;
  if (["road", "bridge", "ford", "signpost"].includes(terrain)) tile.connections = ["e", "w"];
  else if (["village", "cave-passage"].includes(terrain)) tile.connections = [];
  else if (["landing", "ferry"].includes(terrain)) tile.connections = ["w"];
  else if (["castle-wall", "castle-bell"].includes(terrain))
    tile.walls = ["e", "se", "sw", "w", "nw", "ne"];
  return tile;
}
