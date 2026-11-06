/** Serialize synchronous history mutations across tabs, without owning gameplay. */
export function withProgressLock<T>(update: () => T): Promise<T> {
  return navigator.locks.request("hex-command-progress", () => update());
}
