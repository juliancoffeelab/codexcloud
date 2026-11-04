import type Phaser from "phaser";
import { MapRendererGpu, type Region } from "./map-renderer-gpu";
import { lightPipeline } from "./phaser-light-pipeline";
import { daylight, type BattlefieldLighting } from "./battlefield-lighting";
import type { BattlefieldScene, BattlefieldUnit } from "./battlefield-scene";
import type { MotionKind } from "./motion";
import { rendererTraceStart, rendererTraceEnd, beginGpuTrace, endGpuTrace } from "./render-trace";
import { hexColumnSpacing, battlefieldSpriteSize } from "../ui/geometry";

type Actor = {
  unit: BattlefieldUnit;
  body: Phaser.GameObjects.Container;
  bars: Phaser.GameObjects.Container;
  sprite?: Phaser.GameObjects.Image;
  dying: boolean;
  base: Phaser.GameObjects.Ellipse;
  healthFill: Phaser.GameObjects.Rectangle;
  state: Phaser.GameObjects.Graphics;
};

/** One Phaser scene/context on the transferred browser canvas. */
export class PhaserBattlefield {
  private game!: Phaser.Game;
  private scene!: Phaser.Scene;
  private terrain: MapRendererGpu;
  private terrainReady = false;
  private lighting: BattlefieldLighting = daylight;
  private fog: readonly string[] = [];
  private key?: string;
  private snapshot?: BattlefieldScene;
  private actors = new Map<string, Actor>();
  private items?: Phaser.GameObjects.Container;
  private owners?: Phaser.GameObjects.Container;
  private itemNodes = new Map<
    string,
    { node: Phaser.GameObjects.Container; sprite?: Phaser.GameObjects.Image; url?: string }
  >();
  private ownerNodes = new Map<string, Phaser.GameObjects.Arc>();
  private disposed = false;
  private outlines?: Phaser.GameObjects.RenderTexture;
  private outlineKey?: string;
  private focused: string | null = null;
  private particles!: Phaser.GameObjects.Particles.ParticleEmitter;
  private previousTime = performance.now();
  private invalid = true;
  private backgroundDrawn = false;
  private stage = "initialization";
  diagnostics() {
    const gl = this.context;
    return {
      stage: this.stage,
      display: { width: this.canvas.width, height: this.canvas.height },
      ...this.terrain.statistics,
      contextLost: gl.isContextLost(),
      attributes: gl.getContextAttributes(),
      vendor: gl.getParameter(gl.VENDOR),
      renderer: gl.getParameter(gl.RENDERER),
      version: gl.getParameter(gl.VERSION),
      maxTextureSize: gl.getParameter(gl.MAX_TEXTURE_SIZE),
      maxRenderbufferSize: gl.getParameter(gl.MAX_RENDERBUFFER_SIZE),
      units: this.snapshot?.units.length,
      lights: this.lighting.lights?.length,
      fog: this.fog.length,
      phaserTargets: this.targetDetails(),
      unpack: {
        premultiplyAlpha: gl.getParameter(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL),
        flipY: gl.getParameter(gl.UNPACK_FLIP_Y_WEBGL),
        alignment: gl.getParameter(gl.UNPACK_ALIGNMENT),
        rowLength: gl.getParameter(gl.UNPACK_ROW_LENGTH),
        skipRows: gl.getParameter(gl.UNPACK_SKIP_ROWS),
        skipPixels: gl.getParameter(gl.UNPACK_SKIP_PIXELS),
        bufferBound: !!gl.getParameter(gl.PIXEL_UNPACK_BUFFER_BINDING),
      },
    };
  }
  private clip: Region = { left: 0, top: 0, right: 1, bottom: 1 };
  private effects = new Map<string, number>();

  private constructor(
    private P: typeof Phaser,
    readonly canvas: HTMLCanvasElement,
  ) {
    try {
      this.terrain = new MapRendererGpu(canvas, (width, height) =>
        this.game.scale.resize(width, height),
      );
    } catch (error) {
      canvas.getContext("webgl2")?.getExtension("WEBGL_lose_context")?.loseContext();
      throw error;
    }
  }
  static async create(canvas: HTMLCanvasElement, signal?: AbortSignal) {
    const started = rendererTraceStart();
    const P = (await import("phaser")).default;
    signal?.throwIfAborted();
    const result = new PhaserBattlefield(P, canvas);
    try {
      await result.boot(signal);
    } catch (error) {
      result.destroy();
      throw error;
    }
    result.game.loop.stop();
    rendererTraceEnd("Phaser browser boot", started, { version: P.VERSION });
    return result;
  }
  private boot(signal?: AbortSignal) {
    return new Promise<void>((resolve, reject) => {
      const cleanup = () => {
        clearTimeout(timer);
        signal?.removeEventListener("abort", abort);
      };
      const fail = (error: unknown) => {
        cleanup();
        reject(error);
      };
      const abort = () => fail(signal?.reason ?? new Error("Phaser startup cancelled"));
      const timer = setTimeout(() => fail(new Error("Phaser browser startup timed out")), 15000);
      signal?.addEventListener("abort", abort, { once: true });
      const owner = this;
      this.scene = new this.P.Scene("battlefield");
      (this.scene as Phaser.Scene & { create(): void }).create = function (this: Phaser.Scene) {
        if (owner.disposed) {
          this.game.loop.stop();
          return;
        }
        try {
          const terrain = this.add.extern().setDepth(-10000);
          (terrain as Phaser.GameObjects.Extern & { render(): void }).render = () => {
            if (!owner.terrainReady) return;
            const { nativeWidth, nativeHeight } = owner.terrain.statistics;
            const clip = owner.clip;
            owner.stage = "background composition";
            owner.terrain.compose({
              left: (clip.left * nativeWidth) / owner.canvas.width,
              top: (clip.top * nativeHeight) / owner.canvas.height,
              right: (clip.right * nativeWidth) / owner.canvas.width,
              bottom: (clip.bottom * nativeHeight) / owner.canvas.height,
            });
            owner.checkStage();
            owner.stage = "Phaser sprites and indicators";
            owner.backgroundDrawn = true;
            owner.context.enable(owner.context.SCISSOR_TEST);
            owner.context.scissor(
              clip.left,
              owner.canvas.height - clip.bottom,
              clip.right - clip.left,
              clip.bottom - clip.top,
            );
          };
          const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
          // None of this scene's custom lighting, tweens or particles uses built-in
          // FX or masks. Keep their otherwise full-screen scratch buffers at 1×1.
          for (const target of [
            renderer.renderTarget,
            renderer.maskTarget,
            renderer.maskSource,
            ...renderer.pipelines.UTILITY_PIPELINE.renderTargets,
          ]) {
            target.setAutoResize(false);
          }
          renderer.renderTarget.destroy();
          renderer.renderTarget = new owner.P.Renderer.WebGL.RenderTarget(
            renderer,
            1,
            1,
            1,
            0,
            true,
            false,
            false,
          );
          renderer.pipelines.add(
            "BattlefieldLights",
            lightPipeline(owner.P, this.game, owner.terrain, () => owner.lighting),
          );
          owner.particles = this.add
            .particles(0, 0, "__WHITE", {
              lifespan: 350,
              speed: { min: 20, max: 90 },
              angle: { min: 0, max: 360 },
              scale: { start: 0.3, end: 0 },
              alpha: { start: 0.9, end: 0 },
              tint: [0xffe5a1, 0xff7438],
              frequency: -1,
              maxAliveParticles: 96,
            })
            .setDepth(30000);
          this.game.loop.stop();
          cleanup();
          resolve();
        } catch (error) {
          fail(error);
        }
      };
      try {
        this.game = new this.P.Game({
          type: this.P.WEBGL,
          canvas: this.canvas,
          context: this.terrain.context as unknown as CanvasRenderingContext2D,
          width: 1,
          height: 1,
          parent: this.canvas.parentElement ?? undefined,
          banner: false,
          callbacks: {
            preBoot: (game) => {
              this.game = game;
            },
          },
          autoFocus: false,
          disablePreFX: true,
          disablePostFX: true,
          audio: { noAudio: true },
          input: {
            keyboard: false,
            mouse: false,
            touch: false,
            gamepad: false,
            windowEvents: false,
          },
          render: {
            antialias: true,
            clearBeforeRender: false,
            transparent: true,
            preserveDrawingBuffer: true,
            maxTextures: 8,
          },
          scene: this.scene,
        });
      } catch (error) {
        fail(error);
      }
    });
  }
  attachParent() {
    const parent = this.canvas.parentElement;
    if (parent) {
      this.game.scale.parent = parent;
      this.game.scale.refresh();
    }
  }
  destroy() {
    if (this.disposed) return;
    this.disposed = true;
    const context = this.context;
    try {
      if (this.game) {
        this.game.loop.stop();
        this.game.destroy(false);
        if (this.game.scene.systemScene) {
          // Phaser defers destruction to a step, even with a stopped loop.
          this.game.step(performance.now(), 0);
        } else {
          // Phaser's runDestroy assumes systemScene exists. Before READY there
          // are no scene objects or visibility/input handlers to tear down.
          // Remove pending texture-ready callbacks so late images cannot start it.
          this.game.textures.removeAllListeners();
          this.game.events.removeAllListeners();
          const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer | null;
          if (renderer) {
            this.canvas.removeEventListener(
              "webglcontextlost",
              renderer.contextLostHandler as EventListener,
            );
            this.canvas.removeEventListener(
              "webglcontextrestored",
              renderer.contextRestoredHandler as EventListener,
            );
            renderer.removeAllListeners();
          }
        }
      }
    } finally {
      if (this.game?.loop.raf) this.game.loop.destroy();
      context.getExtension("WEBGL_lose_context")?.loseContext();
    }
  }

  get context() {
    return this.terrain.context;
  }
  prepare(...args: Parameters<MapRendererGpu["prepare"]>) {
    return this.terrain.prepare(...args);
  }
  setPresentationSize(size: { width: number; height: number }) {
    this.outlines?.destroy();
    this.outlines = undefined;
    this.outlineKey = undefined;
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    renderer.pipelines.clear();
    try {
      this.terrain.setPresentationSize(size);
      this.resize();
    } finally {
      this.restorePhaser();
    }
  }
  setTerrain(...args: Parameters<MapRendererGpu["setTerrain"]>) {
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    renderer.pipelines.clear();
    try {
      this.terrain.setTerrain(...args);
      this.terrainReady = true;
      this.resize();
    } finally {
      this.restorePhaser();
    }
  }
  private restorePhaser() {
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    // Native passes bypass Phaser's binding caches. Restore all sampler slots
    // and its program cache before DOM scale events can update shader uniforms.
    for (let unit = 0; unit < 8; unit++) {
      this.context.activeTexture(this.context.TEXTURE0 + unit);
      this.context.bindTexture(this.context.TEXTURE_2D, renderer.whiteTexture.webGLTexture);
    }
    renderer.pipelines.rebind();
  }
  private resize() {
    const { width, height } = this.canvas;
    if (this.game.scale.width !== width || this.game.scale.height !== height)
      this.game.scale.resize(width, height);
    const error = this.context.getError();
    if (error !== this.context.NO_ERROR)
      throw new Error(`Phaser GPU resize failed (${error}) at ${width}×${height}`);
    const size = this.terrain.dimensions;
    this.scene.cameras.main
      .setOrigin(0, 0)
      .setViewport(0, 0, width, height)
      .setScroll(0, 0)
      .setZoom(width / size.width, height / size.height);
    this.outlineKey = undefined;
    this.invalid = true;
  }
  hasSprite(url: string, key: string) {
    return key === this.key && this.game.textures.exists(url);
  }
  prepareBattlefield(key: string) {
    if (key === this.key) return;
    this.scene.tweens.killAll();
    for (const actor of this.actors.values()) this.destroyActor(actor);
    this.actors.clear();
    this.items?.destroy();
    this.items = undefined;
    this.itemNodes.clear();
    this.owners?.destroy();
    this.owners = undefined;
    this.ownerNodes.clear();
    this.outlines?.destroy();
    this.outlines = undefined;
    this.particles.killAll();
    this.effects.clear();
    for (const texture of this.game.textures.getTextureKeys()) this.game.textures.remove(texture);
    this.snapshot = undefined;
    this.focused = null;
    this.outlineKey = undefined;
    this.key = key;
  }
  uploadSprite(url: string, key: string, image: ImageBitmap) {
    if (key !== this.key || this.game.textures.exists(url)) return;
    if (image.width !== 64 || image.height !== 64)
      throw new Error(`Battlefield sprites must be exactly 64×64: ${url}`);
    this.game.textures
      .addImage(url, image as unknown as HTMLImageElement)!
      .setFilter(this.P.Textures.FilterMode.NEAREST);
  }
  private destroyActor(actor: Actor) {
    this.scene.tweens.killTweensOf([actor.body, actor.bars, actor.sprite].filter(Boolean));
    actor.body.destroy();
    actor.bars.destroy();
  }
  private createActor(unit: BattlefieldUnit, index: number) {
    const body = this.scene.add.container(unit.x, unit.y).setDepth(100 + index);
    const base = this.scene.add.ellipse(0, 19, 38, 14, parseInt(unit.color.slice(1), 16), 0.5);
    body.add(base);
    const sprite = unit.url
      ? this.scene.add
          .image(0, 20 - unit.size / 2, unit.url)
          .setDisplaySize(unit.size, unit.size)
          .setPipeline("BattlefieldLights")
      : undefined;
    if (sprite) body.add(sprite);
    const bars = this.scene.add.container(unit.x, unit.y).setDepth(10000 + index);
    bars.add(this.scene.add.rectangle(-19, 27, 38, 4, 0x18211b).setOrigin(0));
    const healthFill = this.scene.add.rectangle(-19, 27, 38, 4, 0xa9d0a5).setOrigin(0);
    const state = this.scene.add.graphics();
    bars.add([healthFill, state]);
    const actor = { unit, body, bars, sprite, base, healthFill, state, dying: false };
    this.updateBars(actor);
    return actor;
  }
  private updateBars(actor: Actor, previous?: BattlefieldUnit) {
    const { unit, state } = actor;
    if (!previous || previous.health !== unit.health)
      actor.healthFill.setSize(Math.max(0, Math.min(38, unit.health)), 4);
    if (!previous || previous.spent !== unit.spent || previous.retreating !== unit.retreating) {
      state.clear().lineStyle(2, 0xdbe8d7);
      if (unit.spent)
        state.beginPath().moveTo(21, -20).lineTo(24, -17).lineTo(30, -24).strokePath();
      if (unit.retreating)
        state
          .beginPath()
          .moveTo(-22, -21)
          .lineTo(-30, -21)
          .lineTo(-26, -25)
          .moveTo(-30, -21)
          .lineTo(-26, -17)
          .strokePath();
      actor.sprite?.setAlpha(unit.spent ? 0.65 : 1);
    }
  }
  setBattlefield(snapshot: BattlefieldScene, _images: ReadonlyMap<string, ImageBitmap>) {
    this.prepareBattlefield(snapshot.key);
    this.invalid = true;
    const previous = this.snapshot;
    this.snapshot = snapshot;
    if (snapshot.reducedMotion)
      for (const [id, actor] of this.actors)
        if (actor.dying) {
          this.destroyActor(actor);
          this.actors.delete(id);
        }
    const present = new Set(snapshot.units.map((unit) => unit.id));
    snapshot.units.forEach((unit, index) => {
      let actor = this.actors.get(unit.id);
      if (actor?.dying) {
        this.destroyActor(actor);
        this.actors.delete(unit.id);
        actor = undefined;
      }
      if (!actor) {
        actor = this.createActor(unit, index);
        this.actors.set(unit.id, actor);
        if (previous && !snapshot.reducedMotion) {
          actor.body.setAlpha(0);
          actor.bars.setAlpha(0);
          this.scene.tweens.add({ targets: [actor.body, actor.bars], alpha: 1, duration: 180 });
        }
      } else if (actor.unit.x !== unit.x || actor.unit.y !== unit.y) {
        this.scene.tweens.killTweensOf([actor.body, actor.bars]);
        actor.body.setAlpha(1);
        actor.bars.setAlpha(1);
        if (snapshot.reducedMotion) {
          actor.body.setPosition(unit.x, unit.y);
          actor.bars.setPosition(unit.x, unit.y);
        } else
          this.scene.tweens.add({
            targets: [actor.body, actor.bars],
            x: unit.x,
            y: unit.y,
            duration: 240,
            ease: "Sine.easeInOut",
          });
      }
      const old = actor.unit;
      if (old.url !== unit.url) {
        if (unit.url) {
          if (actor.sprite) actor.sprite.setTexture(unit.url);
          else {
            actor.sprite = this.scene.add
              .image(0, 20 - unit.size / 2, unit.url)
              .setPipeline("BattlefieldLights");
            actor.body.add(actor.sprite);
          }
        } else {
          actor.sprite?.destroy();
          actor.sprite = undefined;
        }
      }
      if (actor.sprite && (old.url !== unit.url || old.size !== unit.size))
        actor.sprite
          .setDisplaySize(unit.size, unit.size)
          .setY(20 - unit.size / 2)
          .setAlpha(unit.spent ? 0.65 : 1);
      if (old.color !== unit.color) actor.base.setFillStyle(parseInt(unit.color.slice(1), 16), 0.5);
      if (actor.sprite && unit.hp < actor.unit.hp && !snapshot.reducedMotion) {
        this.scene.tweens.add({
          targets: actor.sprite,
          x: 4,
          duration: 45,
          yoyo: true,
          repeat: 2,
          onComplete: () => actor!.sprite?.setX(0),
        });
      }
      actor.unit = unit;
      actor.body.setDepth(100 + index);
      actor.bars.setDepth(10000 + index);
      this.updateBars(actor, old);
    });
    for (const [id, actor] of this.actors) {
      if (present.has(id) || actor.dying) continue;
      if (snapshot.reducedMotion || snapshot.fog.includes(actor.unit.tile)) {
        this.destroyActor(actor);
        this.actors.delete(id);
      } else {
        actor.dying = true;
        actor.bars.setVisible(false);
        this.scene.tweens.killTweensOf(actor.body);
        this.scene.tweens.add({
          targets: actor.body,
          alpha: 0,
          y: actor.body.y + 10,
          duration: 300,
          onComplete: () => {
            this.destroyActor(actor);
            if (this.actors.get(id) === actor) this.actors.delete(id);
          },
        });
      }
    }
    this.items ??= this.scene.add.container(0, 0).setDepth(0);
    const itemKeys = new Set<string>();
    for (const item of snapshot.items) {
      const key = `${item.x},${item.y}`;
      itemKeys.add(key);
      let entry = this.itemNodes.get(key);
      if (!entry) {
        const node = this.scene.add.container(item.x, item.y);
        node.add(this.scene.add.ellipse(0, 14, 24, 10, 0xd5bc68, 0.4));
        this.items.add(node);
        entry = { node };
        this.itemNodes.set(key, entry);
      }
      if (entry.url !== item.url) {
        if (item.url) {
          if (entry.sprite) entry.sprite.setTexture(item.url);
          else {
            entry.sprite = this.scene.add.image(0, 0, item.url).setPipeline("BattlefieldLights");
            entry.node.add(entry.sprite);
          }
        } else {
          entry.sprite?.destroy();
          entry.sprite = undefined;
        }
        entry.url = item.url;
      }
    }
    for (const [key, entry] of this.itemNodes)
      if (!itemKeys.has(key)) {
        entry.node.destroy();
        this.itemNodes.delete(key);
      }
    this.owners ??= this.scene.add.container(0, 0).setDepth(20000);
    const ownerKeys = new Set<string>();
    for (const tile of snapshot.tiles)
      if (tile.owner) {
        ownerKeys.add(tile.id);
        let badge = this.ownerNodes.get(tile.id);
        if (!badge) {
          badge = this.scene.add
            .arc(tile.x + 23, tile.y + 17, 5, 0, 360, false, parseInt(tile.owner.slice(1), 16))
            .setName(tile.id);
          this.owners.add(badge);
          this.ownerNodes.set(tile.id, badge);
        } else
          badge
            .setPosition(tile.x + 23, tile.y + 17)
            .setFillStyle(parseInt(tile.owner.slice(1), 16));
      }
    for (const [key, badge] of this.ownerNodes)
      if (!ownerKeys.has(key)) {
        badge.destroy();
        this.ownerNodes.delete(key);
      }
    if (snapshot.reducedMotion) {
      this.scene.tweens.killAll();
      this.particles.killAll();
      this.effects.clear();
      for (const actor of this.actors.values()) {
        actor.body.setPosition(actor.unit.x, actor.unit.y).setAlpha(1);
        actor.bars.setPosition(actor.unit.x, actor.unit.y).setAlpha(1);
        actor.sprite?.setX(0);
      }
    }
  }
  inspect(_hovered: string | null, focused: string | null) {
    if (focused !== this.focused) {
      this.focused = focused;
      this.outlineKey = undefined;
      this.invalid = true;
    }
  }
  pulse(tile: string, kind: MotionKind) {
    if (this.snapshot?.reducedMotion) return;
    const center = this.snapshot?.tiles.find((t) => t.id === tile);
    if (!center) return;
    if (kind === "attack") this.particles.explode(12, center.x, center.y - 10);
    this.effects.set(tile, performance.now() + 400);
    const actor = [...this.actors.values()].find((a) => a.unit.tile === tile && !a.dying);
    if (actor && kind === "select")
      this.scene.tweens.add({ targets: actor.body, alpha: 0.55, duration: 90, yoyo: true });
    if (kind === "capture" && this.owners) {
      const badge = this.owners.getByName(tile);
      if (badge) this.scene.tweens.add({ targets: badge, alpha: 0.5, duration: 150, yoyo: true });
    }
  }
  private drawOutlines() {
    const snapshot = this.snapshot;
    if (!snapshot) return;
    const key = JSON.stringify([
      this.canvas.width,
      this.canvas.height,
      this.focused,
      snapshot.tiles.map((t) => [t.id, t.classes]),
    ]);
    if (key === this.outlineKey) return;
    this.outlineKey = key;
    this.outlines?.destroy();
    const g = this.scene.make.graphics({ x: 0, y: 0 }, false);
    const stroke = (
      points: { x: number; y: number }[],
      width: number,
      color: number,
      alpha: number,
      dash?: readonly [number, number],
    ) => {
      g.lineStyle(width, color, alpha);
      if (!dash) {
        g.strokePoints(points, true);
        return;
      }
      let distance = 0;
      for (let i = 0; i < 6; i++) {
        const a = points[i]!,
          b = points[(i + 1) % 6]!,
          length = Math.hypot(b.x - a.x, b.y - a.y);
        for (let at = 0; at < length;) {
          const phase = (distance + at) % (dash[0] + dash[1]),
            on = phase < dash[0],
            end = Math.min(length, at + (on ? dash[0] - phase : dash[0] + dash[1] - phase));
          if (on)
            g.lineBetween(
              a.x + ((b.x - a.x) * at) / length,
              a.y + ((b.y - a.y) * at) / length,
              a.x + ((b.x - a.x) * end) / length,
              a.y + ((b.y - a.y) * end) / length,
            );
          at = end + 1e-7;
        }
        distance += length;
      }
    };
    for (const tile of snapshot.tiles) {
      const c = new Set(tile.classes.split(" "));
      let color = 0x39472b,
        alpha = 0.3,
        width = 0.7;
      if (c.has("selected") || tile.id === this.focused) {
        color = 0xffe5a1;
        width = 3;
      }
      if (c.has("reachable")) {
        color = 0x95bfd2;
        alpha = 1;
        width = 2.4;
      }
      if (c.has("target")) {
        color = 0xdb8980;
        alpha = 1;
        width = 3;
      }
      if (c.has("recruitable")) {
        color = 0x8ec7a2;
        alpha = 1;
        width = 3;
      }
      if (c.has("objective")) {
        color = c.has("selected") || c.has("reachable") ? 0x99c8ec : 0xffe5a1;
        if (c.has("recruitable")) color = 0x6fdec0;
        alpha = 1;
        width = 3.5;
      }
      const dash: readonly [number, number] | undefined = c.has("objective")
        ? [9, 5]
        : c.has("recruitable")
          ? [4, 3]
          : undefined;
      const points = Array.from({ length: 6 }, (_, i) => {
        const a = ((60 * i - 30) * Math.PI) / 180;
        return {
          x: tile.x + (hexColumnSpacing / Math.sqrt(3)) * Math.cos(a),
          y: tile.y + (battlefieldSpriteSize / 2) * Math.sin(a),
        };
      });
      if (dash) stroke(points, 7, 0x172015, 1, dash);
      stroke(points, width, color, alpha, dash);
    }
    const size = this.terrain.dimensions;
    g.setScale(this.canvas.width / size.width, this.canvas.height / size.height);
    this.stage = "outline target allocation";
    this.outlines = this.scene.add
      .renderTexture(0, 0, 1, 1)
      .setOrigin(0)
      .setDisplaySize(size.width, size.height)
      .setDepth(-100);
    const texture = this.outlines.texture as Phaser.Textures.DynamicTexture;
    const previousTarget = texture.renderTarget;
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    texture.renderTarget = new this.P.Renderer.WebGL.RenderTarget(
      renderer,
      this.canvas.width,
      this.canvas.height,
      1,
      0,
      false,
      false,
      false,
      false,
    );
    texture.setFromRenderTarget();
    previousTarget.destroy();
    this.outlines
      .resize(this.canvas.width, this.canvas.height, false)
      .setDisplaySize(size.width, size.height);
    // DynamicTexture.draw uses Phaser's capture target. Grow that one on demand,
    // without subscribing it (or unused FX/mask buffers) to display resizes.
    const capture = (this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer).renderTarget;
    capture.setAutoResize(true).resize(this.canvas.width, this.canvas.height).setAutoResize(false);
    this.stage = "outline target allocation";
    this.checkStage();
    this.stage = "outline cache draw";
    this.outlines.draw(g);
    g.destroy();
  }
  get animating() {
    return (
      this.scene.tweens.getTweens().length > 0 ||
      this.particles.getAliveParticleCount() > 0 ||
      this.effects.size > 0
    );
  }
  invalidatePresentation() {
    this.invalid = true;
    this.previousTime = performance.now();
    this.game.resume();
  }
  render(lighting: BattlefieldLighting = daylight, fog: readonly string[]) {
    this.lighting = lighting;
    this.fog = fog;
    for (const [id, actor] of this.actors)
      if (actor.dying && fog.includes(actor.unit.tile)) {
        this.destroyActor(actor);
        this.actors.delete(id);
      }
    const now = performance.now(),
      started = rendererTraceStart();
    this.backgroundDrawn = false;
    try {
      this.stage = "GPU trace query";
      beginGpuTrace(this.context);
      this.checkStage();
      this.stage = "outline cache";
      this.drawOutlines();
      this.checkStage();
      const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
      renderer.pipelines.clear();
      this.stage = "terrain/light/fog cache update";
      const changed = this.terrain.update(lighting, fog);
      this.checkStage();
      this.restorePhaser();
      const size = this.terrain.dimensions;
      let region: Region | null = this.invalid
        ? { left: 0, top: 0, right: size.width, bottom: size.height }
        : null;
      const include = (next: Region) => {
        region = region
          ? {
              left: Math.min(region.left, next.left),
              top: Math.min(region.top, next.top),
              right: Math.max(region.right, next.right),
              bottom: Math.max(region.bottom, next.bottom),
            }
          : next;
      };
      if (changed) {
        const native = this.terrain.statistics;
        include({
          left: (changed.left * size.width) / native.nativeWidth,
          top: (changed.top * size.height) / native.nativeHeight,
          right: (changed.right * size.width) / native.nativeWidth,
          bottom: (changed.bottom * size.height) / native.nativeHeight,
        });
      }
      for (const actor of this.actors.values())
        if (
          actor.dying ||
          this.scene.tweens.isTweening(actor.body) ||
          (actor.sprite && this.scene.tweens.isTweening(actor.sprite))
        ) {
          include({
            left: Math.min(actor.body.x, actor.unit.x) - 65,
            top: Math.min(actor.body.y, actor.unit.y) - 90,
            right: Math.max(actor.body.x, actor.unit.x) + 65,
            bottom: Math.max(actor.body.y, actor.unit.y) + 65,
          });
        }
      for (const [tile, expires] of this.effects) {
        const center = this.snapshot?.tiles.find((t) => t.id === tile);
        if (center)
          include({
            left: center.x - 110,
            top: center.y - 110,
            right: center.x + 110,
            bottom: center.y + 110,
          });
        if (now >= expires) this.effects.delete(tile);
      }
      if (region) {
        const sx = this.canvas.width / size.width,
          sy = this.canvas.height / size.height;
        const bounds = region as Region;
        this.clip = {
          left: Math.max(0, Math.floor(bounds.left * sx)),
          top: Math.max(0, Math.floor(bounds.top * sy)),
          right: Math.min(this.canvas.width, Math.ceil(bounds.right * sx)),
          bottom: Math.min(this.canvas.height, Math.ceil(bounds.bottom * sy)),
        };
        const { left, top, right, bottom } = this.clip;
        if (right > left && bottom > top) {
          this.scene.cameras.main
            .setViewport(left, top, right - left, bottom - top)
            .setScroll(left / sx, top / sy)
            .setZoom(sx, sy);
          this.stage = "Phaser scene draw";
          this.game.step(now, Math.min(100, now - this.previousTime));
          this.checkStage();
          if (!this.backgroundDrawn) {
            endGpuTrace();
            this.invalid = true;
            throw new Error("Phaser did not draw the battlefield. Retry map rendering.");
          }
          // Phaser.flush submits its batches; it does not call WebGL.flush.
          // Explicitly submit on-demand browser frames before acknowledging them.
          this.stage = "WebGL submission / frame verification";
          this.context.flush();
          this.checkStage();
        }
      }
      this.invalid = false;
      this.previousTime = now;
      endGpuTrace();
      rendererTraceEnd("Map render CPU submission", started, {
        renderer: "Phaser 3.90 / browser",
        lights: lighting.lights?.length ?? 0,
        fog: fog.length,
        units: this.snapshot?.units.length ?? 0,
        tiles: this.snapshot?.tiles.length ?? 0,
        animating: this.animating,
        dirty: !!region,
        items: this.snapshot?.items.length ?? 0,
        dirtyPixels: region
          ? Math.max(0, this.clip.right - this.clip.left) *
            Math.max(0, this.clip.bottom - this.clip.top)
          : 0,
        tweens: this.scene.tweens.getTweens().length,
        particles: this.particles.getAliveParticleCount(),
        textures: this.game.textures.getTextureKeys().length,
        phaserScratchPixels: this.scratchPixels(),
        ...this.terrain.statistics,
      });
    } finally {
      endGpuTrace();
    }
  }

  private checkStage() {
    const error = this.context.getError();
    if (error !== this.context.NO_ERROR) {
      this.invalid = true;
      endGpuTrace();
      throw new Error(
        `Map GPU stage failed (${error}): ${this.stage} at ${this.canvas.width}×${this.canvas.height}`,
      );
    }
  }
  private targetDetails() {
    const renderer = this.game?.renderer as Phaser.Renderer.WebGL.WebGLRenderer | undefined;
    if (!renderer?.renderTarget) return [];
    return [
      { name: "capture", target: renderer.renderTarget },
      { name: "mask", target: renderer.maskTarget },
      { name: "mask source", target: renderer.maskSource },
      ...renderer.pipelines.renderTargets.map((target, i) => ({ name: `pipeline ${i}`, target })),
      ...renderer.pipelines.UTILITY_PIPELINE.renderTargets.map((target, i) => ({
        name: `utility ${i}`,
        target,
      })),
      {
        name: "outlines",
        target: (this.outlines?.texture as Phaser.Textures.DynamicTexture | undefined)
          ?.renderTarget,
      },
    ].flatMap(({ name, target }) =>
      target
        ? [
            {
              name,
              width: target.width,
              height: target.height,
              depthStencil: target.hasDepthBuffer,
            },
          ]
        : [],
    );
  }
  private scratchPixels() {
    return this.targetDetails()
      .filter((target) => target.name !== "outlines")
      .reduce((sum, target) => sum + target.width * target.height, 0);
  }
}
