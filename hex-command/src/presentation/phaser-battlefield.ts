import type Phaser from "phaser";
import { installPhaserWorkerPlatform } from "./phaser-worker-platform";
import { MapRendererGpu, type Region } from "./map-renderer-gpu";
import { lightPipeline } from "./phaser-light-pipeline";
import { daylight, type BattlefieldLighting } from "./battlefield-lighting";
import type { BattlefieldScene, BattlefieldUnit } from "./battlefield-scene";
import type { MotionKind } from "./motion";
import { workerTraceStart, workerTraceEnd, beginGpuTrace, endGpuTrace } from "./worker-trace";
import { hexColumnSpacing, battlefieldSpriteSize } from "../ui/geometry";

type Actor = {
  unit: BattlefieldUnit;
  body: Phaser.GameObjects.Container;
  bars: Phaser.GameObjects.Container;
  sprite?: Phaser.GameObjects.Image;
  dying: boolean;
};

/** One Phaser scene/context on the transferred worker canvas. */
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
  private outlines?: Phaser.GameObjects.RenderTexture;
  private outlineKey?: string;
  private focused: string | null = null;
  private particles!: Phaser.GameObjects.Particles.ParticleEmitter;
  private previousTime = performance.now();
  private invalid = true;
  private clip: Region = { left: 0, top: 0, right: 1, bottom: 1 };
  private effects = new Map<string, number>();

  private constructor(
    private P: typeof Phaser,
    readonly canvas: OffscreenCanvas,
  ) {
    this.terrain = new MapRendererGpu(canvas);
  }
  static async create(canvas: OffscreenCanvas) {
    const decorate = installPhaserWorkerPlatform();
    const P = (await import("phaser")).default;
    decorate(canvas);
    const result = new PhaserBattlefield(P, canvas);
    await result.boot();
    return result;
  }
  private boot() {
    return new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("Phaser worker startup timed out")), 15000);
      const owner = this;
      this.scene = new this.P.Scene("battlefield");
      (this.scene as Phaser.Scene & { create(): void }).create = function (this: Phaser.Scene) {
        const terrain = this.add.extern().setDepth(-10000);
        (terrain as Phaser.GameObjects.Extern & { render(): void }).render = () => {
          if (!owner.terrainReady) return;
          const { nativeWidth, nativeHeight } = owner.terrain.statistics;
          const clip = owner.clip;
          owner.terrain.compose({
            left: (clip.left * nativeWidth) / owner.canvas.width,
            top: (clip.top * nativeHeight) / owner.canvas.height,
            right: (clip.right * nativeWidth) / owner.canvas.width,
            bottom: (clip.bottom * nativeHeight) / owner.canvas.height,
          });
          owner.context.enable(owner.context.SCISSOR_TEST);
          owner.context.scissor(
            clip.left,
            owner.canvas.height - clip.bottom,
            clip.right - clip.left,
            clip.bottom - clip.top,
          );
        };
        const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
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
        clearTimeout(timer);
        resolve();
      };
      try {
        this.game = new this.P.Game({
          type: this.P.WEBGL,
          customEnvironment: true,
          canvas: this.canvas as unknown as HTMLCanvasElement,
          context: this.terrain.context as unknown as CanvasRenderingContext2D,
          width: 1,
          height: 1,
          parent: null,
          banner: false,
          autoFocus: false,
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
        clearTimeout(timer);
        reject(error);
      }
    });
  }
  get context() {
    return this.terrain.context;
  }
  prepare(...args: Parameters<MapRendererGpu["prepare"]>) {
    return this.terrain.prepare(...args);
  }
  setPresentationSize(size: { width: number; height: number }) {
    this.terrain.setPresentationSize(size);
    this.resize();
  }
  setTerrain(...args: Parameters<MapRendererGpu["setTerrain"]>) {
    this.terrain.setTerrain(...args);
    this.terrainReady = true;
    this.resize();
  }
  private resize() {
    const { width, height } = this.canvas;
    this.game.scale.resize(width, height);
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
    this.owners?.destroy();
    this.owners = undefined;
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
    body.add(this.scene.add.ellipse(0, 19, 38, 14, parseInt(unit.color.slice(1), 16), 0.5));
    const sprite = unit.url
      ? this.scene.add.image(0, 20 - unit.size / 2, unit.url).setPipeline("BattlefieldLights")
      : undefined;
    if (sprite) body.add(sprite);
    const bars = this.scene.add.container(unit.x, unit.y).setDepth(10000 + index);
    return { unit, body, bars, sprite, dying: false };
  }
  private updateBars(actor: Actor) {
    const { unit, bars } = actor;
    bars.removeAll(true);
    bars.add(this.scene.add.rectangle(-19, 27, 38, 4, 0x18211b).setOrigin(0));
    bars.add(
      this.scene.add
        .rectangle(-19, 27, Math.max(0, Math.min(38, unit.health)), 4, 0xa9d0a5)
        .setOrigin(0),
    );
    const state = this.scene.add.graphics();
    state.lineStyle(2, 0xdbe8d7);
    if (unit.spent) state.beginPath().moveTo(21, -20).lineTo(24, -17).lineTo(30, -24).strokePath();
    if (unit.retreating)
      state
        .beginPath()
        .moveTo(-22, -21)
        .lineTo(-30, -21)
        .lineTo(-26, -25)
        .moveTo(-30, -21)
        .lineTo(-26, -17)
        .strokePath();
    bars.add(state);
    actor.sprite?.setAlpha(unit.spent ? 0.65 : 1);
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
      if (actor.sprite && actor.unit.url !== unit.url && unit.url)
        actor.sprite.setTexture(unit.url);
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
      this.updateBars(actor);
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
    this.items?.destroy();
    this.items = this.scene.add.container(0, 0).setDepth(0);
    for (const item of snapshot.items) {
      this.items.add(this.scene.add.ellipse(item.x, item.y + 14, 24, 10, 0xd5bc68, 0.4));
      if (item.url)
        this.items.add(
          this.scene.add.image(item.x, item.y, item.url).setPipeline("BattlefieldLights"),
        );
    }
    this.owners?.destroy();
    this.owners = this.scene.add.container(0, 0).setDepth(20000);
    for (const tile of snapshot.tiles)
      if (tile.owner)
        this.owners.add(
          this.scene.add.arc(
            tile.x + 23,
            tile.y + 17,
            5,
            0,
            360,
            false,
            parseInt(tile.owner.slice(1), 16),
          ),
        );
    if (snapshot.reducedMotion) {
      this.scene.tweens.killAll();
      this.particles.killAll();
      for (const actor of this.actors.values()) {
        actor.body.setPosition(actor.unit.x, actor.unit.y).setAlpha(1);
        actor.bars.setPosition(actor.unit.x, actor.unit.y).setAlpha(1);
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
    this.outlines = this.scene.add
      .renderTexture(0, 0, this.canvas.width, this.canvas.height)
      .setOrigin(0)
      .setDisplaySize(size.width, size.height)
      .setDepth(-100);
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
  render(lighting: BattlefieldLighting = daylight, fog: readonly string[]) {
    this.lighting = lighting;
    this.fog = fog;
    for (const [id, actor] of this.actors)
      if (actor.dying && fog.includes(actor.unit.tile)) {
        this.destroyActor(actor);
        this.actors.delete(id);
      }
    const now = performance.now(),
      started = workerTraceStart();
    beginGpuTrace(this.context);
    this.drawOutlines();
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    renderer.pipelines.clear();
    const changed = this.terrain.update(lighting, fog);
    // All active sampler-array entries must reference float textures, including
    // unused slots. Native passes bind integer light tables on units 2 and 3.
    for (let unit = 0; unit < 8; unit++) {
      this.context.activeTexture(this.context.TEXTURE0 + unit);
      this.context.bindTexture(this.context.TEXTURE_2D, renderer.whiteTexture.webGLTexture);
    }
    renderer.pipelines.rebind();
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
        this.game.step(now, Math.min(100, now - this.previousTime));
      }
    }
    this.invalid = false;
    this.previousTime = now;
    endGpuTrace();
    workerTraceEnd("Map render CPU submission", started, {
      renderer: "Phaser 3.90 / worker",
      lights: lighting.lights?.length ?? 0,
      fog: fog.length,
      units: this.snapshot?.units.length ?? 0,
      tiles: this.snapshot?.tiles.length ?? 0,
      animating: this.animating,
      ...this.terrain.statistics,
    });
  }
}
