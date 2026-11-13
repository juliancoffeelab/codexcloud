import { duration, sampleRate } from "./synthesis.mjs";

const ahead = 20,
  behind = 20,
  memoryLimit = 32 * 1024 * 1024;
const key = (line) => JSON.stringify([line.character || line.speaker, line.text]);

export class VoicePlayer {
  constructor(trace, dependencies = {}) {
    this.trace = trace;
    this.workerFactory =
      dependencies.workerFactory ||
      (() => new Worker(new URL("./worker.mjs", import.meta.url), { type: "module" }));
    this.contextFactory =
      dependencies.contextFactory || (() => new AudioContext({ latencyHint: "interactive" }));
    this.enabled = true;
    try {
      this.enabled = localStorage.getItem("hex-command-voiceover-enabled") !== "false";
    } catch {
      /* Storage is optional. */
    }
    this.prepared = new Map();
    this.wanted = new Map();
    this.history = [];
    this.candidates = [];
    this.scope = null;
    this.epoch = 0;
    this.sequence = 0;
    this.bytes = 0;
    this.closed = false;
    this.gesture = (event) => {
      if (event.isTrusted && this.enabled) this.unlock();
    };
    this.visibility = () => {
      if (document.hidden) {
        this.clearOpeningDeadline();
        if (this.active) this.active.reveal(this.active.line.text);
        this.stop();
      } else {
        this.failed = false;
        this.error = null;
        this.notifyOpening();
        this.pump();
      }
    };
    for (const event of ["pointerdown", "keydown", "click"])
      document.addEventListener(event, this.gesture, true);
    document.addEventListener("visibilitychange", this.visibility);
  }

  report(name, args = {}) {
    this.trace("Voiceover " + name, args);
  }

  unlock() {
    try {
      this.context ||= this.contextFactory();
      if (this.context.state === "suspended")
        this.context
          .resume()
          .then(() => this.playActive())
          .catch((error) =>
            this.report("audio unavailable", { error: error.stack || String(error) }),
          );
      else this.playActive();
    } catch (error) {
      this.report("audio unavailable", { error: error.stack || String(error) });
    }
  }

  reset(scope) {
    if (this.scope === scope) return;
    this.stop();
    this.clearOpeningDeadline();
    this.opening = null;
    this.epoch++;
    this.scope = scope;
    this.candidates = [];
    this.missionPrepared = false;
    this.failed = false;
    this.error = null;
    this.prepared.clear();
    this.wanted.clear();
    this.history = [];
    this.bytes = 0;
  }

  prepare(scope, lines) {
    this.reset(scope);
    if (this.missionPrepared) return;
    this.missionPrepared = true;
    this.candidates = lines.filter((line) => line.text);
    this.plan(
      this.opening?.upcoming || (this.active ? this.upcoming : this.candidates.slice(0, ahead)),
    );
  }

  watchOpening(scope, lines, index, receive) {
    this.reset(scope);
    const remaining = lines.slice(index).filter((line) => line.text);
    if (!this.opening && remaining.length) {
      this.opening = {
        targets: remaining.slice(0, 2).map(key),
        upcoming: remaining.slice(0, ahead + 1),
        listeners: new Set(),
        released: false,
        started: performance.now(),
      };
      this.plan(this.opening.upcoming);
    }
    const opening = this.opening;
    if (!opening) {
      receive({ scope, ready: true, completed: 0, total: 0, error: null });
      return () => {};
    }
    this.notifyOpening();
    opening.listeners.add(receive);
    receive(opening.status);
    return () => opening.listeners.delete(receive);
  }

  clearOpeningDeadline() {
    clearTimeout(this.openingDeadline);
    this.openingDeadline = null;
  }

  notifyOpening() {
    const opening = this.opening;
    if (!opening) return;
    const completed = opening.targets.filter((id) => this.prepared.has(id)).length;
    if (!opening.released && (!this.enabled || completed === opening.targets.length)) {
      opening.released = true;
      this.report("opening ready", { ms: performance.now() - opening.started, tracks: completed });
    }
    const status = {
      scope: this.scope,
      ready: opening.released,
      completed,
      total: opening.targets.length,
      error: opening.released ? null : this.error || null,
    };
    if (status.ready || status.error || document.hidden) this.clearOpeningDeadline();
    else if (!this.openingDeadline) {
      // Count foreground waiting only; background throttling must not poison startup.
      this.openingDeadline = setTimeout(
        () =>
          this.failure(
            new Error("Opening voice preparation exceeded 30 seconds in the foreground.").stack,
          ),
        30000,
      );
    }
    if (JSON.stringify(status) !== JSON.stringify(opening.status)) {
      opening.status = status;
      for (const listener of opening.listeners) listener(status);
    }
  }

  retry() {
    this.failed = false;
    this.error = null;
    this.clearOpeningDeadline();
    this.notifyOpening();
    this.pump();
  }

  plan(upcoming) {
    this.wanted = new Map(
      [...upcoming, ...this.history.slice(-behind).reverse()].map((line) => [key(line), line]),
    );
    let budget = memoryLimit;
    for (const [id, line] of this.wanted) {
      const bytes = Math.ceil(duration(line.text) * sampleRate) * 4;
      if (bytes > budget) this.wanted.delete(id);
      else budget -= bytes;
    }
    for (const [id, samples] of this.prepared) {
      if (!this.wanted.has(id)) {
        this.bytes -= samples.byteLength;
        this.prepared.delete(id);
      }
    }
    this.report("queue", {
      ready: this.prepared.size,
      wanted: this.wanted.size,
      bytes: this.bytes,
    });
    this.pump();
  }

  pump() {
    if (this.closed || !this.enabled || this.busy || this.failed || document.hidden) return;
    const next = [...this.wanted].find(
      ([id, line]) =>
        !this.prepared.has(id) &&
        this.bytes + Math.ceil(duration(line.text) * sampleRate) * 4 <= memoryLimit,
    );
    if (!next) return;
    try {
      if (!this.worker) {
        this.worker = this.workerFactory();
        this.worker.onmessage = ({ data }) => this.receive(data);
        this.worker.onerror = (event) => this.failure(event.error?.stack || event.message);
        this.worker.onmessageerror = () => this.failure("Worker message could not be decoded");
      }
      const [id, line] = next;
      this.busy = { id, epoch: this.epoch, request: ++this.sequence };
      this.worker.postMessage({ id: this.busy.request, epoch: this.epoch, line });
    } catch (error) {
      this.failure(error.stack || String(error));
    }
  }

  failure(error) {
    this.failed = true;
    this.busy = null;
    this.worker?.terminate();
    this.worker = null;
    this.error = error;
    this.notifyOpening();
    this.report("generation failed", { error });
    console.error("Voiceover generation failed", error);
  }

  receive(result) {
    if (this.closed || !this.busy || result.id !== this.busy.request) return;
    const job = this.busy;
    this.busy = null;
    if (result.epoch !== this.epoch) {
      this.pump();
      return;
    }
    if (result.error) {
      this.failure(result.error);
      return;
    }
    if (this.wanted.has(job.id)) {
      this.prepared.set(job.id, result.samples);
      this.bytes += result.samples.byteLength;
      this.report("generated", {
        ms: result.ms,
        samples: result.samples.length,
        ready: this.prepared.size,
        bytes: this.bytes,
      });
      this.playActive();
      this.notifyOpening();
    }
    this.pump();
  }

  speak(scope, lines, index, reveal) {
    this.reset(scope);
    this.stop();
    const line = lines[index];
    if (!line?.text) return () => {};
    const id = key(line),
      anchor = this.candidates.findIndex((candidate) => key(candidate) === id);
    this.history = lines.slice(Math.max(0, index - behind), index);
    const upcoming = [
      ...lines.slice(index, index + ahead + 1),
      ...this.candidates.slice(anchor < 0 ? 0 : anchor + 1),
    ];
    this.active = {
      line,
      id,
      reveal,
      started: performance.now(),
      duration: duration(line.text),
      spoken: false,
    };
    const active = this.active;
    this.upcoming = [...new Map(upcoming.map((item) => [key(item), item])).values()].slice(
      0,
      ahead + 1,
    );
    this.plan(this.upcoming);
    this.report("line", {
      ready: this.prepared.has(id),
      readyLines: this.prepared.size,
      bytes: this.bytes,
      seconds: active.duration,
    });
    if (this.enabled && !this.prepared.has(id))
      this.report("deadline miss", { readyLines: this.prepared.size });
    // Reveal graphemes without splitting surrogate pairs or combining characters.
    const letters =
      typeof Intl.Segmenter === "function"
        ? [...new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(line.text)].map(
            (item) => item.segment,
          )
        : Array.from(line.text);
    let previous = -1;
    const tick = () => {
      if (this.active !== active) return;
      const elapsed = (performance.now() - active.started) / 1000;
      const count = Math.min(
        letters.length,
        Math.max(1, Math.floor((letters.length * elapsed) / active.duration)),
      );
      if (count !== previous) {
        previous = count;
        reveal(letters.slice(0, count).join(""));
      }
      if (elapsed < active.duration) this.frame = requestAnimationFrame(tick);
      else {
        this.frame = null;
        this.stopSource();
      }
    };
    tick();
    this.playActive();
    return () => {
      if (this.active === active) this.stop();
    };
  }

  playActive() {
    const active = this.active;
    if (
      !active ||
      active.spoken ||
      !this.enabled ||
      this.closed ||
      document.hidden ||
      this.context?.state !== "running"
    )
      return;
    const samples = this.prepared.get(active.id);
    if (!samples) return;
    const offset = (performance.now() - active.started) / 1000;
    if (offset >= active.duration) return;
    try {
      this.stopSource();
      const buffer = this.context.createBuffer(1, samples.length, sampleRate);
      buffer.copyToChannel(samples, 0);
      const source = this.context.createBufferSource();
      source.buffer = buffer;
      const gain = this.context.createGain();
      gain.gain.setValueAtTime(0, this.context.currentTime);
      gain.gain.linearRampToValueAtTime(1, this.context.currentTime + 0.006);
      source.connect(gain);
      gain.connect(this.context.destination);
      source.onended = () => {
        source.disconnect();
        gain.disconnect();
        if (this.source === source) this.source = null;
      };
      this.source = source;
      this.sourceGain = gain;
      active.spoken = true;
      source.start(0, offset);
      this.report("playing", { waitMs: offset * 1000, seconds: active.duration - offset });
    } catch (error) {
      this.report("audio unavailable", { error: error.stack || String(error) });
    }
  }

  stopSource() {
    if (this.source) {
      const at = this.context.currentTime;
      this.sourceGain.gain.cancelScheduledValues(at);
      this.sourceGain.gain.setValueAtTime(this.sourceGain.gain.value, at);
      this.sourceGain.gain.linearRampToValueAtTime(0, at + 0.006);
      this.source.stop(at + 0.006);
      this.source = null;
      this.sourceGain = null;
    }
  }

  stop() {
    if (this.frame != null) cancelAnimationFrame(this.frame);
    this.frame = null;
    this.active = null;
    this.stopSource();
  }

  setEnabled(enabled) {
    this.enabled = enabled;
    try {
      localStorage.setItem("hex-command-voiceover-enabled", String(enabled));
    } catch {
      /* Storage is optional. */
    }
    if (!enabled) this.stopSource();
    else {
      this.failed = false;
      this.unlock();
      this.pump();
    }
    this.notifyOpening();
  }

  close() {
    this.closed = true;
    this.clearOpeningDeadline();
    this.stop();
    this.worker?.terminate();
    this.prepared.clear();
    this.wanted.clear();
    this.bytes = 0;
    for (const event of ["pointerdown", "keydown", "click"])
      document.removeEventListener(event, this.gesture, true);
    document.removeEventListener("visibilitychange", this.visibility);
    this.context?.close().catch(() => {});
  }
}

export const create = (trace) => new VoicePlayer(trace);
export const enabled = (player) => player.enabled;
export const setEnabled = (player, value) => player.setEnabled(value);
export const close = (player) => player.close();
export const prepare = (player, scope, lines) => player.prepare(scope, lines);
export const needsPreparation = (player, scope) =>
  player.scope !== scope || !player.missionPrepared;
export const speak = (player, scope, lines, index, reveal) =>
  player.speak(scope, lines, index, reveal);
export const watchOpening = (player, scope, lines, index, receive) =>
  player.watchOpening(scope, lines, index, receive);
export const retry = (player) => player.retry();
