export type Immutable<T> = T extends (...args: infer A) => infer R
  ? (...args: A) => Immutable<R>
  : T extends ReadonlyMap<infer K, infer V>
    ? ReadonlyMap<K, Immutable<V>>
    : T extends readonly unknown[]
      ? { readonly [K in keyof T]: Immutable<T[K]> }
      : T extends object
        ? { readonly [K in keyof T]: Immutable<T[K]> }
        : T;

/** Freeze plain records and arrays at the publication/configuration boundary. */
export function immutable<T>(value: T): Immutable<T> {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) immutable(child);
    Object.freeze(value);
  }
  return value as Immutable<T>;
}
