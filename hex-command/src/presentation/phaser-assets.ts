import Phaser from "phaser";
import { BattlefieldResources } from "./phaser-resources";
import { measureTrace } from "./performance-trace";
import { PreparedAudioFile, PreparedImageFile, cancelLoaderFile } from "./phaser-runtime";
import type { Resource } from "./resource-plan";
import type { HudArtworkCache } from "./hud-artwork";
import type { FrameImage } from "./art";
import { audioReadiness } from "./sound";
import { errorDetails } from "../error-details";

type Entry = { resource: Resource; state: "queued" | "active" | "ready" | "failed" };
/** Thin priority scheduler over Phaser Files/caches. No global completion barrier. */
export class BattlefieldAssets {
  readonly files = new Map<string, Phaser.Loader.File>();
  private plan = new Map<string, Resource>();
  private failed = new Map<string, Error>();
  private demand = new Map<string, Resource>();
  private images = new Set<string>();
  private states = new Map<string, Entry>();
  private queues: Entry[][] = Array.from({ length: 5 }, () => []);
  private cursors = [0, 0, 0, 0, 0];
  private pending = [0, 0, 0, 0, 0];
  private decoded: PreparedImageFile[] = [];
  private required = new Set<string>();
  private adoptions = new Map<string, HTMLImageElement>();
  private adoptionQueues: Entry[][] = Array.from({ length: 5 }, () => []);
  private adoptionCursors = [0, 0, 0, 0, 0];
  private audioTotal = 0;
  private audioLoaded = 0;
  private audioError?: Error;
  private waiters = new Map<
    string,
    { resolve: (image: HTMLImageElement) => void; reject: (error: unknown) => void }[]
  >();
  private detach: () => void;
  private disposed = false;

  constructor(
    private scene: Phaser.Scene,
    private resources: BattlefieldResources,
    private hud: HudArtworkCache,
    private changed: (resource?: Resource) => void,
    private report: (resource: Resource, error: Error) => void,
  ) {
    scene.events.on(Phaser.Scenes.Events.PRE_UPDATE, this.admit, this);
    this.detach = hud.bind((frame) => this.request(frame));
    scene.load.on(Phaser.Loader.Events.FILE_COMPLETE, this.completed, this);
    scene.load.on(Phaser.Loader.Events.FILE_LOAD_ERROR, this.errored, this);
    scene.load.on(Phaser.Loader.Events.COMPLETE, this.schedule, this);
  }
  private cached(resource: Resource) {
    return resource.kind === "image"
      ? this.scene.textures.exists(resource.key)
      : this.scene.cache.audio.exists(resource.key);
  }
  setPlan(plan: Map<string, Resource>) {
    this.plan = plan;
    for (const resource of this.demand.values()) this.plan.set(resource.key, resource);
    for (const [key, file] of this.files) if (!plan.has(key)) this.cancel(key, file);
    for (const key of this.images)
      if (!plan.has(key)) {
        this.scene.textures.remove(key);
        this.resources.remove(key);
        this.images.delete(key);
      }
    for (const resource of plan.values()) {
      if (resource.kind === "image" && resource.width && resource.height)
        this.resources.check(resource.key, resource.width, resource.height);
      if (resource.kind !== "image" || !resource.hud || this.scene.textures.exists(resource.key))
        continue;
      const image = this.hud.ready({ url: resource.url, width: 64, height: 64 });
      if (image) {
        const file = this.files.get(resource.key);
        if (file) this.cancel(resource.key, file);
        this.adoptions.set(resource.key, image);
      }
    }
    this.rebuild();
    this.pump();
    this.publishAudio();
  }
  private rebuild() {
    this.queues = Array.from({ length: 5 }, () => []);
    this.cursors = [0, 0, 0, 0, 0];
    this.pending = [0, 0, 0, 0, 0];
    this.states.clear();
    this.adoptionQueues = Array.from({ length: 5 }, () => []);
    this.adoptionCursors = [0, 0, 0, 0, 0];
    for (const key of this.adoptions.keys()) if (!this.plan.has(key)) this.adoptions.delete(key);
    this.audioTotal = this.audioLoaded = 0;
    this.audioError = undefined;
    for (const resource of this.plan.values()) {
      const state: Entry = {
        resource,
        state: this.cached(resource)
          ? "ready"
          : this.failed.has(resource.key)
            ? "failed"
            : this.files.has(resource.key)
              ? "active"
              : "queued",
      };
      this.states.set(resource.key, state);
      if (this.adoptions.has(resource.key)) this.adoptionQueues[resource.priority - 1]!.push(state);
      if (state.state === "queued") this.enqueue(state);
      if (resource.kind === "audio") {
        if (resource.priority === 1) {
          this.audioTotal++;
          if (state.state === "ready") this.audioLoaded++;
        }
        if (state.state === "failed") this.audioError ??= this.failed.get(resource.key);
      }
    }
  }
  private enqueue(entry: Entry) {
    entry.state = "queued";
    const priority = entry.resource.priority - 1;
    this.queues[priority]!.push(entry);
    this.pending[priority]!++;
  }
  private promote(resource: Resource) {
    const old = this.states.get(resource.key);
    if (old?.state === "queued") this.pending[old.resource.priority - 1]!--;
    const entry: Entry = {
      resource,
      state: this.cached(resource)
        ? "ready"
        : this.files.has(resource.key)
          ? "active"
          : this.failed.has(resource.key)
            ? "failed"
            : "queued",
    };
    this.states.set(resource.key, entry);
    if (this.adoptions.has(resource.key)) this.adoptionQueues[resource.priority - 1]!.push(entry);
    if (entry.state === "queued") this.enqueue(entry);
  }
  private admitDecoded(index: number) {
    const [file] = this.decoded.splice(index, 1);
    if (this.files.get(file!.key) !== file) return;
    this.resources.remove(`pending:${file!.key}`);
    this.checkImage(file!.key, file!.data as HTMLImageElement);
    measureTrace("Phaser image admission", () => file!.admit());
  }
  private admitAdoption(key: string, image: HTMLImageElement) {
    this.adoptions.delete(key);
    if (!this.plan.has(key) || this.scene.textures.exists(key)) return;
    this.checkImage(key, image);
    measureTrace("Phaser image admission", () => this.scene.textures.addImage(key, image));
    this.images.add(key);
    const entry = this.states.get(key);
    if (entry?.state === "queued") this.pending[entry.resource.priority - 1]!--;
    if (entry) entry.state = "ready";
    this.changed(this.plan.get(key));
    this.schedule();
  }
  private admit() {
    if (this.disposed || (!this.decoded.length && !this.adoptions.size)) return;
    // Foreground keeps its normal admission behavior. Decoded work is bounded
    // by the existing six in-flight slots; it must not gain an extra RAF gate.
    for (;;) {
      const index = this.decoded.findIndex((file) => this.required.has(file.key));
      if (index < 0) break;
      this.admitDecoded(index);
    }
    for (const key of this.required) {
      const image = this.adoptions.get(key);
      if (image) this.admitAdoption(key, image);
    }
    // Only speculative cache admission/upload is limited to one per update.
    let best = -1,
      priority = 6;
    for (let i = 0; i < this.decoded.length; i++) {
      const p = this.plan.get(this.decoded[i]!.key)?.priority ?? 5;
      if (p < priority) {
        priority = p;
        best = i;
      }
    }
    let adoption: Entry | undefined;
    for (let p = 0; p < Math.min(priority, 5); p++) {
      const queue = this.adoptionQueues[p]!;
      while (this.adoptionCursors[p]! < queue.length) {
        const entry = queue[this.adoptionCursors[p]!]!;
        if (
          this.states.get(entry.resource.key) === entry &&
          this.adoptions.has(entry.resource.key)
        ) {
          adoption = entry;
          break;
        }
        this.adoptionCursors[p]!++;
      }
      if (adoption) break;
    }
    if (adoption) {
      this.adoptionCursors[adoption.resource.priority - 1]!++;
      this.admitAdoption(adoption.resource.key, this.adoptions.get(adoption.resource.key)!);
    } else if (best >= 0) this.admitDecoded(best);
  }
  private checkImage(key: string, image: HTMLImageElement) {
    const expected = this.plan.get(key);
    if (
      (expected?.width !== undefined && expected.width !== image.naturalWidth) ||
      (expected?.height !== undefined && expected.height !== image.naturalHeight)
    )
      throw new Error(
        `Invalid natural artwork dimensions: ${key}; expected ${expected?.width}×${expected?.height}, received ${image.naturalWidth}×${image.naturalHeight}`,
      );
    this.resources.image(key, image);
  }
  /** A mutable page supersedes its original PNG texture. Its owner removes that
   * source from subsequent plans so native admission cannot recreate stale art. */
  retireImage(key: string) {
    this.plan.delete(key);
    const entry = this.states.get(key);
    if (entry?.state === "queued") this.pending[entry.resource.priority - 1]!--;
    this.states.delete(key);
    this.adoptions.delete(key);
    const file = this.files.get(key);
    if (file) this.cancel(key, file);
    this.scene.textures.remove(key);
    this.images.delete(key);
    this.resources.remove(key);
  }
  requireImages(keys: ReadonlySet<string>, metadata?: ReadonlyMap<string, Readonly<FrameImage>>) {
    this.required = new Set(keys);
    let changed = false;
    for (const key of keys) {
      const old = this.plan.get(key);
      if (!old || old.priority !== 1) {
        const resource: Resource = {
          ...old,
          key,
          url: key,
          kind: "image",
          priority: 1,
          width: old?.width ?? metadata?.get(key)?.width,
          height: old?.height ?? metadata?.get(key)?.height,
        };
        this.plan.set(key, resource);
        this.promote(resource);
        changed = true;
      }
    }
    if (changed) this.pump();
  }
  error(key: string) {
    return this.failed.get(key);
  }
  private schedule() {
    queueMicrotask(() => this.pump());
  }
  private pump() {
    if (this.disposed) return;
    const priority = this.pending.findIndex((count) => count > 0) + 1;
    if (priority > 0) {
      let needed =
        Math.min(this.pending[priority - 1]!, this.scene.load.maxParallelDownloads) -
        (this.scene.load.maxParallelDownloads - this.files.size);
      for (const [key, file] of this.files) {
        if (needed <= 0) break;
        if ((this.plan.get(key)?.priority ?? 5) > priority) {
          this.cancel(key, file);
          needed--;
        }
      }
    }
    while (this.files.size < this.scene.load.maxParallelDownloads) {
      const priority = this.pending.findIndex((count) => count > 0);
      if (priority < 0) break;
      const queue = this.queues[priority]!;
      let entry: Entry | undefined;
      while (this.cursors[priority]! < queue.length) {
        const candidate = queue[this.cursors[priority]!++]!;
        if (this.states.get(candidate.resource.key) === candidate && candidate.state === "queued") {
          entry = candidate;
          break;
        }
      }
      if (!entry) throw new Error("Asset FIFO count mismatch");
      this.pending[priority]!--;
      const resource = entry.resource;
      if (this.adoptions.has(resource.key)) {
        entry.state = "active";
        continue;
      }
      entry.state = "active";
      let file: Phaser.Loader.File;
      if (resource.kind === "image")
        file = new PreparedImageFile(this.scene.load, resource.url, (file) => {
          if (!this.disposed && this.files.get(file.key) === file) {
            const image = file.data as HTMLImageElement;
            this.resources.set(
              `pending:${file.key}`,
              "decoded pending images",
              image.naturalWidth * image.naturalHeight * 4,
              0,
            );
            this.decoded.push(file);
          }
        });
      else {
        const context = (this.scene.game.sound as Phaser.Sound.WebAudioSoundManager).context;
        if (!context) {
          this.failure(resource, new Error(`Web Audio unavailable for ${resource.key}`));
          continue;
        }
        file = new PreparedAudioFile(
          this.scene.load,
          resource.key,
          resource.url,
          context,
          (cause) =>
            this.failure(
              resource,
              new Error(`Audio preparation failed: ${resource.key}`, { cause }),
            ),
        );
      }
      this.files.set(resource.key, file);
      this.scene.load.addFile(file);
    }
    if (this.files.size && !this.scene.load.isLoading()) this.scene.load.start();
  }
  private completed(key: string) {
    if (!this.files.has(key) || this.disposed) return;
    const completedFile = this.files.get(key)!;
    this.files.delete(key);
    queueMicrotask(() => {
      // Phaser otherwise retains every completed File until the entire loader
      // finishes, which may be delayed by unrelated speculative downloads.
      if (
        completedFile.loader === this.scene.load &&
        completedFile.state === Phaser.Loader.FILE_PENDING_DESTROY
      ) {
        (
          this.scene.load as Phaser.Loader.LoaderPlugin & {
            _deleteQueue: Phaser.Structs.Set<Phaser.Loader.File>;
          }
        )._deleteQueue.delete(completedFile);
        completedFile.destroy();
      }
    });
    const resource = this.plan.get(key);
    const entry = this.states.get(key);
    if (entry) entry.state = "ready";
    if (resource?.kind === "audio") {
      if (resource.priority === 1) this.audioLoaded++;
      const buffer = this.scene.cache.audio.get(key) as AudioBuffer;
      this.resources.set(key, "decoded audio", buffer.length * buffer.numberOfChannels * 4, 0);
    }
    if (resource?.kind === "image") {
      this.images.add(key);
      const image = this.scene.textures.get(key).getSourceImage() as HTMLImageElement;
      if (resource.hud) this.hud.adopt(key, image);
      for (const waiter of this.waiters.get(key) ?? []) waiter.resolve(image);
      this.waiters.delete(key);
      this.demand.delete(key);
    }
    this.publishAudio();
    this.changed(resource);
    this.schedule();
  }
  private errored(file: Phaser.Loader.File) {
    if (this.files.get(file.key) !== file) return;
    this.files.delete(file.key);
    const resource = this.plan.get(file.key);
    if (resource && !this.failed.has(file.key))
      this.failure(
        resource,
        (file as PreparedImageFile).failure ??
          new Error(`Resource failed: ${file.key} (${file.src}); download or decode error`),
      );
    this.publishAudio();
    this.schedule();
  }
  private failure(resource: Resource, error: Error) {
    if (this.disposed || this.failed.has(resource.key)) return;
    this.files.delete(resource.key);
    this.failed.set(resource.key, error);
    const entry = this.states.get(resource.key);
    if (entry) entry.state = "failed";
    if (resource.kind === "audio") this.audioError ??= error;
    for (const waiter of this.waiters.get(resource.key) ?? []) waiter.reject(error);
    this.waiters.delete(resource.key);
    this.report(resource, error);
    this.publishAudio();
    this.schedule();
  }
  private publishAudio() {
    if (this.disposed) return;
    audioReadiness.set({
      ready: this.audioLoaded === this.audioTotal,
      ...(this.audioError ? { error: errorDetails(this.audioError) } : {}),
    });
  }

  request(frame: Readonly<FrameImage>): Promise<HTMLImageElement> {
    const existing = this.hud.ready(frame);
    if (existing) return Promise.resolve(existing);
    const resource: Resource = {
      key: frame.url,
      url: frame.url,
      kind: "image",
      priority: 1,
      hud: true,
      width: frame.width,
      height: frame.height,
    };
    this.demand.set(frame.url, resource);
    this.plan.set(frame.url, resource);
    this.failed.delete(frame.url);
    if (this.scene.textures.exists(frame.url)) {
      const image = this.scene.textures.get(frame.url).getSourceImage() as HTMLImageElement;
      this.hud.adopt(frame.url, image);
      this.demand.delete(frame.url);
      return Promise.resolve(image);
    }
    const promise = new Promise<HTMLImageElement>((resolve, reject) => {
      const waiters = this.waiters.get(frame.url) ?? [];
      waiters.push({ resolve, reject });
      this.waiters.set(frame.url, waiters);
    });
    this.promote(resource);
    this.pump();
    return promise;
  }
  retry(kind?: Resource["kind"]) {
    for (const key of this.failed.keys())
      if (!kind || this.plan.get(key)?.kind === kind) this.failed.delete(key);
    this.rebuild();
    this.publishAudio();
    this.pump();
    this.changed();
  }
  private cancel(key: string, file: Phaser.Loader.File) {
    this.files.delete(key);
    this.decoded = this.decoded.filter((file) => file.key !== key);
    this.resources.remove(`pending:${key}`);
    const entry = this.states.get(key);
    if (!this.disposed && entry?.state === "active" && this.plan.has(key)) this.enqueue(entry);
    cancelLoaderFile(this.scene.load, file);
  }
  destroy() {
    this.disposed = true;
    this.detach();
    this.scene.events.off(Phaser.Scenes.Events.PRE_UPDATE, this.admit, this);
    this.decoded = [];
    this.adoptions.clear();
    this.scene.load.off(Phaser.Loader.Events.FILE_COMPLETE, this.completed, this);
    this.scene.load.off(Phaser.Loader.Events.FILE_LOAD_ERROR, this.errored, this);
    this.scene.load.off(Phaser.Loader.Events.COMPLETE, this.schedule, this);
    for (const [key, file] of this.files) this.cancel(key, file);
    for (const waiters of this.waiters.values())
      for (const waiter of waiters)
        waiter.reject(new DOMException("Artwork preparation canceled", "AbortError"));
    this.waiters.clear();
  }
}
