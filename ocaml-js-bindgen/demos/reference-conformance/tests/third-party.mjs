// An independently implemented ES-module API, consumed through typed Rust imports.
export class Counter {
  constructor(value) {
    this.value = value;
  }
  add(amount) {
    this.value += amount;
    return this.value;
  }
}
export const thrown = Object.freeze({ reason: "third-party failure" });
export function failure() {
  throw thrown;
}

export function unsigned(a, b) {
  if (a !== 4294967295 || b !== 18446744073709551615n)
    throw new Error("unsigned argument corruption");
  return b;
}
