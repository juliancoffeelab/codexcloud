/** Lifecycle of the painted loading shell and its asynchronously acquired application. */
declare const startupBrand: unique symbol;
export interface t { readonly [startupBrand]: true }
/** Melange encoding: Loading = 0, Ready = 1, Failed carries its readable message. */
export type phase = 0 | 1 | { readonly TAG: 0; readonly _0: string };
export type cleanup = () => void;
/** Schedule acquisition after painting; return a cancellation capability. */
export type schedule = (acquire: () => void) => cleanup;
/** Load owns the resources it returns; current becomes false immediately on close. */
export type load = (current: () => boolean) => Promise<cleanup>;
export function create(show: (phase: phase) => void, schedule: schedule, load: load): t;
/** Idempotently cancel queued acquisition and release acquired resources. */
export function close(startup: t): void;

/** Release owned work and show a late render failure once. */
export function fail(startup: t, message: string): void;
