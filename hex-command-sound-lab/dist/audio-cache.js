// Shared fetch/decode cache. Prefetch has two workers and stops when its budget is full.
export class AudioCache {
  constructor(getContext, { budget = 48 * 1024 * 1024 } = {}) {
    this.getContext = getContext;
    this.budget = budget;
    this.bytes = 0;
    this.ready = new Map();
    this.pending = new Map();
    this.epoch = 0;
  }
  peek(id) {
    const buffer = this.ready.get(id);
    if (buffer) {
      this.ready.delete(id);
      this.ready.set(id, buffer);
    }
    return buffer;
  }
  async load(item) {
    const cached = this.peek(item.id);
    if (cached) return cached;
    if (this.pending.has(item.id)) return this.pending.get(item.id);
    const task = (async () => {
      const response = await fetch(item.path);
      if (!response.ok) throw Error(`Audio returned HTTP ${response.status}`);
      const buffer = await this.getContext().decodeAudioData(
        await response.arrayBuffer(),
      );
      const size = buffer.length * buffer.numberOfChannels * 4;
      // A single large source may exceed the budget, but never accumulates with others.
      while (this.ready.size && this.bytes + size > this.budget) {
        const [id, old] = this.ready.entries().next().value;
        this.bytes -= old.length * old.numberOfChannels * 4;
        this.ready.delete(id);
      }
      this.ready.set(item.id, buffer);
      this.bytes += size;
      return buffer;
    })();
    this.pending.set(item.id, task);
    try {
      return await task;
    } finally {
      this.pending.delete(item.id);
    }
  }
  cancelWarm() {
    this.epoch++;
  }
  async warm(items, update = () => {}, priority = 4) {
    const epoch = ++this.epoch;
    const queue = [
      ...new Map(items.map((item) => [item.id, item])).values(),
    ].map((item, index) => ({ item, priority: index < priority }));
    const worker = async () => {
      while (epoch === this.epoch && queue.length) {
        if (this.pending.size >= 2) {
          await Promise.race(this.pending.values()).catch(() => {});
          continue;
        }
        const next = queue.shift();
        const item = next.item;
        if (
          !next.priority &&
          !this.ready.has(item.id) &&
          this.bytes + item.duration * 44100 * 4 > this.budget &&
          this.ready.size
        )
          continue;
        try {
          await this.load(item);
          update();
        } catch {
          update();
        } // Playback retries failed preparation and reports the error.
      }
    };
    await Promise.all([worker(), worker()]);
  }
}
