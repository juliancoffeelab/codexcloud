/** Historical importer annotation; no application dependency. */
export type Immutable<T> = T extends (...args: never[]) => unknown
  ? T
  : T extends readonly (infer U)[]
    ? readonly Immutable<U>[]
    : T extends object
      ? { readonly [K in keyof T]: Immutable<T[K]> }
      : T;
