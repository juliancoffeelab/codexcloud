import assert from "node:assert/strict";

/** Record the canvas extraction boundary without a browser or network. */
export function mockSpriteCanvas() {
  const original = Object.getOwnPropertyDescriptor(globalThis, "document");
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: {
      documentElement: { dataset: {} },
      querySelector: () => null,
      querySelectorAll: () => [],
      createElement: (name: string) => {
        assert.equal(name, "canvas");
        let extraction: unknown;
        const canvas = {
          width: 0,
          height: 0,
          getContext: () => context,
          toDataURL: () =>
            "data:image/png;base64," + Buffer.from(JSON.stringify(extraction)).toString("base64"),
        };
        const context = {
          imageSmoothingEnabled: true,
          beginPath() {},
          moveTo() {},
          lineTo() {},
          closePath() {},
          clip() {},
          drawImage(image: { src: string }, ...coordinates: number[]) {
            extraction = {
              source: image.src,
              coordinates,
              width: canvas.width,
              height: canvas.height,
              smoothing: context.imageSmoothingEnabled,
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

/** Assert a fixture invariant while preserving strict nullable lookup types. */
export function required<T>(value: T | undefined | null): T {
  assert.ok(value !== undefined && value !== null, "Expected fixture value");
  return value;
}

/** Browser-compatible storage adapter for deterministic Node fixtures. */
export class TestStorage implements Storage {
  constructor(readonly values = new Map<string, string>()) {}
  get length(): number {
    return this.values.size;
  }
  clear(): void {
    this.values.clear();
  }
  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }
  key(index: number): string | null {
    return [...this.values.keys()][index] ?? null;
  }
  removeItem(key: string): void {
    this.values.delete(key);
  }
  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}

export function storageWithValue(read: () => string, write: (value: string) => void): Storage {
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
