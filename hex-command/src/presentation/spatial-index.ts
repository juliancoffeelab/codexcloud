export interface PaintBounds {
  left: number;
  top: number;
  right: number;
  bottom: number;
}
export const overlaps = (a: PaintBounds, b: PaintBounds) =>
  a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;

/** Resident records and intersected buckets are independent of active draw lists. */
export class SpatialIndex<T> {
  private records = new Map<T, { bounds: PaintBounds; keys: string[] }>();
  private buckets = new Map<string, Set<T>>();
  constructor(private size = 256) {}
  private keys(bounds: PaintBounds) {
    const keys: string[] = [];
    for (
      let y = Math.floor(bounds.top / this.size);
      y <= Math.floor(bounds.bottom / this.size);
      y++
    )
      for (
        let x = Math.floor(bounds.left / this.size);
        x <= Math.floor(bounds.right / this.size);
        x++
      )
        keys.push(`${x}:${y}`);
    return keys;
  }
  set(record: T, bounds: PaintBounds) {
    const old = this.records.get(record);
    if (
      old &&
      old.bounds.left === bounds.left &&
      old.bounds.top === bounds.top &&
      old.bounds.right === bounds.right &&
      old.bounds.bottom === bounds.bottom
    )
      return;
    this.delete(record);
    const keys = this.keys(bounds);
    this.records.set(record, { bounds, keys });
    for (const key of keys) {
      const bucket = this.buckets.get(key) ?? new Set<T>();
      bucket.add(record);
      this.buckets.set(key, bucket);
    }
  }
  delete(record: T) {
    const old = this.records.get(record);
    if (!old) return;
    for (const key of old.keys) {
      const bucket = this.buckets.get(key)!;
      bucket.delete(record);
      if (!bucket.size) this.buckets.delete(key);
    }
    this.records.delete(record);
  }
  query(bounds: PaintBounds) {
    const found = new Set<T>();
    for (const key of this.keys(bounds))
      for (const record of this.buckets.get(key) ?? [])
        if (!found.has(record) && overlaps(this.records.get(record)!.bounds, bounds))
          found.add(record);
    return [...found];
  }
  clear() {
    this.records.clear();
    this.buckets.clear();
  }
}
