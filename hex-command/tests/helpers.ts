import assert from "node:assert/strict";

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

export function storageWithValue(
  read: () => string,
  write: (value: string) => void,
): Storage {
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
