import Phaser from "phaser";
import { errorDetails } from "../error-details";

// These lifecycle methods exist in Phaser 3.90 but are omitted from its types.
const lifecycle = Phaser.Game.prototype as Phaser.Game & {
  texturesReady(): void;
  runDestroy(): void;
};

/** Keep asynchronous Phaser failures inside the battlefield's error boundary. */
export class BattlefieldGame extends Phaser.Game {
  private renderDirty = true;
  invalidateRender = () => {
    this.renderDirty = true;
  };
  private disposed = false;
  private failureReported = false;
  private cleanupErrors: unknown[] = [];
  get unavailable() {
    return this.disposed || this.failureReported;
  }
  recordCleanupError(error: unknown) {
    this.cleanupErrors.push(error);
  }
  private notifyFailure!: (message: string) => void;

  constructor(config: Phaser.Types.Core.GameConfig, notifyFailure: (message: string) => void) {
    super(config);
    this.notifyFailure = notifyFailure;
    for (const event of [
      Phaser.Core.Events.VISIBLE,
      Phaser.Core.Events.FOCUS,
      Phaser.Core.Events.RESUME,
    ])
      this.events.on(event, this.invalidateRender);
    this.scale.on(Phaser.Scale.Events.RESIZE, this.invalidateRender);
  }

  protected boot() {
    // Game's constructor can call boot synchronously. Defer it until this
    // subclass and the caller's runtime handle have both finished construction.
    queueMicrotask(() => this.guard(() => super.boot()));
  }
  texturesReady() {
    this.guard(() => lifecycle.texturesReady.call(this));
  }
  protected start() {
    this.guard(() => super.start());
  }
  step(time: number, delta: number) {
    this.guard(() => {
      // Pinned Phaser 3.90 Game.step seam: no native render-only skip exists.
      // Keep its exact update/event order and headlessStep's processing reset.
      // Only the clear, scene submission and post-render section is conditional.
      if ((this as unknown as { pendingDestroy: boolean }).pendingDestroy)
        return lifecycle.runDestroy.call(this);
      if (this.isPaused) return;
      const events = this.events,
        E = Phaser.Core.Events;
      events.emit(E.PRE_STEP, time, delta);
      events.emit(E.STEP, time, delta);
      this.scene.update(time, delta);
      events.emit(E.POST_STEP, time, delta);
      if (!this.renderDirty) {
        (this.scene as unknown as { isProcessing: boolean }).isProcessing = false;
        return;
      }
      this.renderDirty = false;
      this.renderer.preRender();
      events.emit(E.PRE_RENDER, this.renderer, time, delta);
      this.scene.render(this.renderer);
      this.renderer.postRender();
      events.emit(E.POST_RENDER, this.renderer, time, delta);
    });
  }
  private guard(operation: () => void) {
    if (this.disposed || this.failureReported) return;
    try {
      operation();
    } catch (error) {
      this.fail(error);
    }
  }
  fail(error: unknown) {
    if (this.failureReported || this.disposed) return;
    this.failureReported = true;
    this.loop.stop();
    // Unwind the current Phaser callback before tearing down its managers.
    queueMicrotask(() => {
      let cleanupErrors: unknown[];
      try {
        cleanupErrors = this.dispose();
      } catch (error) {
        cleanupErrors = [error];
      }
      this.notifyFailure(
        [errorDetails(error), ...cleanupErrors.map((e) => `Cleanup: ${errorDetails(e)}`)].join(
          "\n",
        ),
      );
    });
  }
  destroy(_removeCanvas = true, _noReturn = false) {
    this.dispose();
  }
  dispose(): unknown[] {
    if (this.disposed) return [];
    this.disposed = true;
    this.loop.stop();
    super.destroy(true);
    const errors = this.cleanupErrors;
    const release = (operation: () => void) => {
      try {
        operation();
      } catch (error) {
        errors.push(error);
      }
    };
    // Unbooted devices have no DOM target or listeners. Phaser 3.90's touch
    // destructor assumes a target exists; dropping these constructor-only
    // managers lets InputManager release its remaining resources normally.
    for (const key of ["keyboard", "mouse", "touch"] as const) {
      const device = this.input[key];
      if (device && !device.target) (this.input as unknown as Record<string, unknown>)[key] = null;
    }
    try {
      lifecycle.runDestroy.call(this);
    } catch {
      // Phaser's normal destructor assumes BOOT completed. A renderer/preBoot
      // failure can leave managers unbooted; release each surviving manager.
      const scenes = this.scene as Phaser.Scenes.SceneManager & { systemScene?: Phaser.Scene };
      if (scenes.systemScene) release(() => scenes.destroy());
      for (const manager of [this.input, this.sound, this.scale, this.anims, this.plugins]) {
        if (manager && (manager as unknown as { game?: unknown }).game !== null)
          release(() => manager.destroy());
      }
      const textures = this.textures as Phaser.Textures.TextureManager & {
        stamp?: Phaser.GameObjects.Image;
        _tempCanvas?: HTMLCanvasElement;
      };
      if (textures.game) {
        if (textures.stamp) release(() => textures.destroy());
        else {
          for (const key of textures.getTextureKeys()) release(() => textures.remove(key));
          textures.removeAllListeners();
          if (textures._tempCanvas) Phaser.Display.Canvas.CanvasPool.remove(textures._tempCanvas);
        }
      }
      release(() => this.renderer?.destroy());
    } finally {
      this.events.removeAllListeners();
      // runDestroy already destroys the loop on the normal path.
      if (this.loop.raf) release(() => this.loop.destroy());
      if (this.canvas) {
        release(() => Phaser.Display.Canvas.CanvasPool.remove(this.canvas));
        release(() => this.canvas.remove());
      }
      release(() => this.domContainer?.remove());
    }
    return errors;
  }
}

/** Native AudioFile processing with cause preservation and obsolete-decode guards.
 * Phaser 3.90's stock callback drops the decode cause and calls into destroyed
 * loaders when a decode finishes after destruction. Keep that version seam here.
 */
export class PreparedAudioFile extends Phaser.Loader.FileTypes.AudioFile {
  private abandoned = false;
  private expired = false;
  private deadline: ReturnType<typeof setTimeout> | undefined;
  constructor(
    loader: Phaser.Loader.LoaderPlugin,
    key: string,
    url: string,
    private context: AudioContext,
    private report: (cause: unknown) => void,
  ) {
    super(loader, key, { type: "wav", url }, undefined, context);
  }
  onProcess() {
    this.state = Phaser.Loader.FILE_PROCESSING;
    this.deadline = setTimeout(() => {
      if (this.abandoned) return;
      this.expired = true;
      this.report(new Error(`Audio decode deadline exceeded: ${this.key} (${this.src})`));
      this.onProcessError();
    }, 30000);
    void this.context.decodeAudioData(this.xhrLoader!.response as ArrayBuffer).then(
      (buffer) => {
        if (this.abandoned || this.expired) return;
        clearTimeout(this.deadline);
        this.data = buffer;
        this.onProcessComplete();
      },
      (cause: unknown) => {
        if (this.abandoned || this.expired) return;
        clearTimeout(this.deadline);
        this.report(new Error(`Cannot decode audio ${this.key} (${this.src})`, { cause }));
        this.onProcessError();
      },
    );
  }
  destroy() {
    this.abandoned = true;
    clearTimeout(this.deadline);
    this.resetXHR();
    if (this.xhrLoader) {
      this.xhrLoader.ontimeout = null;
      this.xhrLoader.onabort = null;
      this.xhrLoader.abort();
    }
    super.destroy();
  }
}

/** Phaser's Image element path shares decoded sources with normal HUD images.
 * Keep a finite deadline and remove obsolete DOM callbacks on cancellation. */
export class PreparedImageFile extends Phaser.Loader.FileTypes.ImageFile {
  failure?: Error;
  private deadline?: ReturnType<typeof setTimeout>;
  private complete = false;
  private abandoned = false;
  constructor(
    loader: Phaser.Loader.LoaderPlugin,
    url: string,
    private defer?: (file: PreparedImageFile) => void,
  ) {
    super(loader, { key: url, url });
    const load = this.load.bind(this);
    this.load = () => {
      this.deadline = setTimeout(() => {
        this.failure = new Error(
          `Image download/decode deadline exceeded: ${this.key} (${this.src})`,
        );
        if (this.data instanceof HTMLImageElement) {
          this.data.onload = this.data.onerror = null;
          this.data.removeAttribute("src");
        }
        this.loader.nextFile(this, false);
      }, 30000);
      load();
    };
  }
  onProcessComplete() {
    clearTimeout(this.deadline);
    this.complete = true;
    if (this.data instanceof HTMLImageElement) this.data.onload = this.data.onerror = null;
    if (this.defer) this.defer(this);
    else this.admit();
  }
  /** Decode stays in Phaser's processing set until its bounded update admission. */
  admit() {
    if (!this.abandoned) super.onProcessComplete();
  }
  destroy() {
    this.abandoned = true;
    clearTimeout(this.deadline);
    if (!this.complete && this.data instanceof HTMLImageElement) {
      this.data.onload = this.data.onerror = null;
      this.data.removeAttribute("src");
    }
    super.destroy();
  }
}

/** Phaser 3.90 cancellation seam; leave late callbacks unable to re-enter a retired loader. */
export function cancelLoaderFile(loader: Phaser.Loader.LoaderPlugin, file: Phaser.Loader.File) {
  loader.list?.delete(file);
  loader.inflight?.delete(file);
  loader.queue?.delete(file);
  file.onProcessComplete = () => {};
  file.onProcessError = () => {};
  file.resetXHR();
  if (file.xhrLoader && typeof file.xhrLoader.abort === "function") {
    file.xhrLoader.ontimeout = file.xhrLoader.onabort = null;
    file.xhrLoader.abort();
  }
  file.destroy();
}
