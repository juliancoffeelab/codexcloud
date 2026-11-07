import Phaser from "phaser";
import { BattlefieldVisibility } from "./phaser-visibility";
import { BattlefieldResources } from "./phaser-resources";
import { audioReadiness, attachSound, soundReady } from "./sound";
import { errorDetails } from "../error-details";
import { BattlefieldGame } from "./phaser-runtime";
import type { BattleView, BattleCommands } from "../controller/view";
import { BattlefieldViewCache, type BattlefieldTileView } from "../ui/battlefield-view";
import { TerrainViewCache } from "./terrain-map";
import { battlefieldSpriteSize, hexPoints, mapDimensions } from "../ui/geometry";
import { battlefieldScene, type BattlefieldUnit } from "./battlefield-scene";
import {
  BattlefieldHaloPipeline,
  BattlefieldHexPipeline,
  BattlefieldFogMaskPipeline,
  BattlefieldFogBlurPipeline,
  BattlefieldFogCompositePipeline,
} from "./phaser-atmosphere";
import { BattlefieldActorLightingPipeline } from "./phaser-actor-lighting";
import { BattlefieldLightingPipeline } from "./phaser-lighting";
import { installBattlefieldInput, type BattlefieldCameraState } from "./phaser-camera";
import { lightPosition } from "./battlefield-lighting";
import { measureTrace, traceConfiguration, traceInstant } from "./performance-trace";
import type { MotionKind } from "./motion";
import type { HudArtworkCache } from "./hud-artwork";
import { BattlefieldAssets } from "./phaser-assets";
import { currentMapAudio, resourcePlan, resourcePlanKey } from "./resource-plan";
import { itemFrame } from "./art";
import { initialTerrain, terrainTexture } from "./terrain-texture";
import { BattlefieldFog } from "./phaser-fog";
import { BattlefieldTerrain } from "./phaser-terrain";

export interface BattlefieldProgress {
  identity: string;
  phase: "loading" | "preparing" | "ready";
  loaded: number;
  total: number;
  audioLoaded: number;
  audioTotal: number;
  error?: string;
}

export interface PhaserBattlefieldFrame {
  view: BattleView;
  tiles: readonly BattlefieldTileView[];
  reducedMotion: boolean;
}
type UnitActor = {
  data: BattlefieldUnit;
  root: Phaser.GameObjects.Container;
  image: Phaser.GameObjects.Image;
  base: Phaser.GameObjects.Ellipse;
  indicators: Phaser.GameObjects.Graphics;
  transition?: { kind: "move" | "hit" | "spawn"; t: number; fromX: number; fromY: number };
};
const color = (value: string) => Phaser.Display.Color.HexStringToColor(value).color;
const polygon = (tile: { x: number; y: number }) =>
  hexPoints(tile.x, tile.y)
    .split(" ")
    .map((point) => {
      const [x, y] = point.split(",").map(Number);
      return new Phaser.Math.Vector2(x!, y!);
    });

/** All battlefield objects, effects, textures, animation and input live in this Scene. */
export class BattlefieldScene extends Phaser.Scene {
  private frame?: PhaserBattlefieldFrame;
  private resourceEpoch = 0;
  private lastVisual?: {
    game: BattleView["game"];
    tiles: readonly BattlefieldTileView[];
    motion: boolean;
    sound: boolean;
    epoch: number;
  };
  private sceneGame?: BattleView["game"];
  private sceneArtwork?: BattleView["artwork"];
  private sceneData?: ReturnType<typeof battlefieldScene>;
  private tilesById = new Map<string, BattlefieldTileView>();
  private artworkMetadata?: BattleView["artwork"];
  private imageMetadata = new Map<string, Readonly<import("./art").FrameImage>>();
  private terrainView = new TerrainViewCache();
  private battlefieldView = new BattlefieldViewCache();
  private ready = false;
  private identity = "";
  private loading = false;
  private assets!: BattlefieldAssets;
  private resources!: BattlefieldResources;
  private get requests() {
    return this.assets.files;
  }
  private requiredImages = new Set<string>();
  private requiredAudio = new Set<string>();
  private planKey = "";
  private plannedGame?: BattleView["game"];
  private presented = false;
  private progressKey = "";
  private assetError?: string;
  private refreshQueued = false;
  private failed = false;
  private lighting!: BattlefieldLightingPipeline;
  private actorLighting!: BattlefieldActorLightingPipeline;
  private controls!: ReturnType<typeof installBattlefieldInput>;
  private terrain?: BattlefieldTerrain;
  private terrainUrls = new Set<string>();
  private terrainGame?: BattleView["game"];
  private actors = new Map<string, UnitActor>();
  private activeActors = new Set<UnitActor>();
  private visibility = new BattlefieldVisibility();
  private actorRoots!: Phaser.GameObjects.Container;
  private upperIndicators!: Phaser.GameObjects.Container;
  private invalidate = () => (this.game as BattlefieldGame).invalidateRender();
  private dying = new Map<Phaser.GameObjects.Container, { tile: string; key: string }>();
  private items!: Phaser.GameObjects.Container;
  private atmosphere?: BattlefieldFog;
  private atmospherePipelines: Phaser.Renderer.WebGL.WebGLPipeline[] = [];
  private outlines!: Phaser.GameObjects.Container;
  private owners!: Phaser.GameObjects.Container;
  private ownerMarkers = new Map<string, Phaser.GameObjects.Ellipse>();
  private groundItems = new Map<
    string,
    {
      root: Phaser.GameObjects.Container;
      image: Phaser.GameObjects.Image;
      anchor: { x: number; y: number };
    }
  >();
  private hexes = new Map<
    string,
    {
      root: Phaser.GameObjects.Container;
      ordinal: number;
      stroke: Phaser.GameObjects.Image;
      underlay?: Phaser.GameObjects.Image;
      tile?: BattlefieldTileView;
    }
  >();
  private pulses = new Map<string, { kind: MotionKind; t: number }>();
  private hasFrame = false;
  cameraSnapshot?: { identity: string; state: BattlefieldCameraState };

  constructor(
    private actions: () => BattleCommands,
    private describe: (label: string) => void,
    private previews: HudArtworkCache,
    private progress: (state: BattlefieldProgress) => void,
    private restoreCamera?: { identity: string; state: BattlefieldCameraState },
  ) {
    super("battlefield");
  }

  create() {
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    let cleaned = false;
    const cleanup = () => {
      if (cleaned) return;
      cleaned = true;
      if (this.controls)
        this.cameraSnapshot = {
          identity: this.identity,
          state: this.controls.snapshot(),
        };
      this.ready = false;
      this.game.events.off(Phaser.Core.Events.POST_STEP, this.animateActors, this);
      const release = (operation: () => void) => {
        try {
          operation();
        } catch (error) {
          (this.game as BattlefieldGame).recordCleanupError(error);
        }
      };
      for (const actor of this.actors.values()) {
        release(() => actor.root.destroy());
        release(() => actor.indicators.destroy());
      }
      for (const actor of this.dying.keys()) release(() => actor.destroy());
      for (const hex of this.hexes.values()) release(() => hex.root.destroy());
      for (const item of this.groundItems.values()) release(() => item.root.destroy());
      for (const marker of this.ownerMarkers.values()) release(() => marker.destroy());
      release(() => this.visibility.clear());
      for (const operation of [
        () => this.assets?.destroy(),
        () => this.tweens.killAll(),
        () => renderer.pipelines.flush(),
        () => this.lighting?.release(),
        () => renderer.pipelines.remove("BattlefieldActorLighting"),
        () => this.actorLighting?.destroy(),
        () => renderer.pipelines.remove("BattlefieldLighting"),
        () => this.lighting?.destroy(),
        () => this.atmosphere?.destroy(),
        () => this.terrain?.destroy(),
      ])
        release(operation);
      for (const pipeline of this.atmospherePipelines) {
        release(() => renderer.pipelines.remove(pipeline.name));
        release(() => pipeline.destroy());
      }
    };
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, cleanup);
    this.events.once(Phaser.Scenes.Events.DESTROY, cleanup);
    try {
      this.initialize();
    } catch (error) {
      (this.game as BattlefieldGame).fail(error);
    }
  }
  private initialize() {
    this.ready = true;
    this.cameras.main.setVisible(false);
    this.resources = new BattlefieldResources(this.game);
    this.assets = new BattlefieldAssets(
      this,
      this.resources,
      this.previews,
      (resource) => {
        if (
          !resource ||
          this.requiredImages.has(resource.key) ||
          (!this.presented && this.requiredAudio.has(resource.key))
        )
          this.queueRefresh();
      },
      (resource, error) => {
        if (
          (resource.kind === "image" && this.requiredImages.has(resource.key)) ||
          (!this.presented && this.frame?.view.sound && this.requiredAudio.has(resource.key))
        ) {
          this.failed = true;
          (this.game as BattlefieldGame).fail(error);
        } else if (resource.kind === "image") {
          this.assetError = errorDetails(error);
          this.reportLoading();
        }
      },
    );
    soundReady(this.game);
    // Keep Phaser's wheel listener non-passive, while allowing the scene to
    // prevent map gestures from reaching document scrolling.
    if (this.input.manager.mouse) this.input.manager.mouse.preventDefaultWheel = false;
    const onContextLost = () =>
      (this.game as BattlefieldGame).fail(
        new Error(
          "The battlefield WebGL context was lost. Retry map rendering to restart cleanly.",
        ),
      );
    this.game.canvas.addEventListener("webglcontextlost", onContextLost);
    this.events.once(Phaser.Scenes.Events.DESTROY, () =>
      this.game.canvas.removeEventListener("webglcontextlost", onContextLost),
    );
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    this.lighting = new BattlefieldLightingPipeline(
      this.game,
      this.cameras.main,
      this.resources,
      this,
    );
    renderer.pipelines.add("BattlefieldLighting", this.lighting);
    this.actorLighting = new BattlefieldActorLightingPipeline(this.game);
    renderer.pipelines.add("BattlefieldActorLighting", this.actorLighting);
    this.actorRoots = this.add.container(0, 0).setDepth(20000);
    this.upperIndicators = this.add.container(0, 0).setDepth(30000);
    this.items = this.add.container(0, 0).setDepth(11000);
    this.outlines = this.add.container(0, 0).setDepth(10000);
    this.owners = this.add.container(0, 0).setDepth(40000);
    for (const [name, Pipeline] of [
      ["BattlefieldHalo", BattlefieldHaloPipeline],
      ["BattlefieldFogBlur", BattlefieldFogBlurPipeline],
      ["BattlefieldFogComposite", BattlefieldFogCompositePipeline],
      ["BattlefieldHex", BattlefieldHexPipeline],
      ["BattlefieldFogMask", BattlefieldFogMaskPipeline],
    ] as const) {
      const pipeline = new Pipeline(this.game);
      this.atmospherePipelines.push(pipeline);
      renderer.pipelines.add(name, pipeline);
    }
    this.textures.addUint8Array("battlefield-white", new Uint8Array([255, 255, 255, 255]), 1, 1);
    this.controls = installBattlefieldInput(
      this,
      () => this.frame,
      this.actions,
      this.restoreCamera?.identity ===
        `${this.frame?.view.sessionVersion}:${this.frame?.view.game.mapId}`
        ? this.restoreCamera.state
        : undefined,
      this.invalidate,
    );
    this.game.events.on(Phaser.Core.Events.POST_STEP, this.animateActors, this);
    let renderStart: { id: number; time: number } | undefined;
    this.events.on(Phaser.Scenes.Events.PRE_RENDER, () => {
      const tracing = traceConfiguration();
      renderStart = tracing ? { id: tracing.id, time: performance.now() } : undefined;
    });
    this.events.on(Phaser.Scenes.Events.RENDER, () => {
      if (renderStart && traceConfiguration()?.id === renderStart.id)
        traceInstant("Battlefield CPU render submission", {
          durationMs: performance.now() - renderStart.time,
        });
      if (
        this.hasFrame &&
        !this.loading &&
        !this.failed &&
        this.game.canvas.style.visibility === "hidden"
      ) {
        this.game.canvas.style.visibility = "visible";
        this.presented = true;
        this.reportLoading();
        traceInstant("Battlefield first complete frame", { scene: this.identity });
      }
    });
    this.refreshSafely();
  }

  returnToCommander() {
    this.controls.returnToCommander();
  }

  present(
    publication: Pick<PhaserBattlefieldFrame, "view" | "reducedMotion"> &
      Partial<Pick<PhaserBattlefieldFrame, "tiles">>,
  ) {
    const { view, reducedMotion } = publication;
    if (this.artworkMetadata !== view.artwork) {
      this.artworkMetadata = view.artwork;
      this.imageMetadata = new Map(
        Object.values(view.artwork.frames).map((frame) => [frame.url, frame]),
      );
    }
    const previous = this.frame;
    const recruit = view.game.turn === "player" && view.recruitment ? view.inspectedTile : null;
    const previousRecruit =
      previous?.view.game.turn === "player" && previous.view.recruitment
        ? previous.view.inspectedTile
        : null;
    let tiles = previous?.tiles;
    if (
      !tiles ||
      previous?.view.game !== view.game ||
      previous.view.artwork !== view.artwork ||
      previous.view.selected !== view.selected ||
      previousRecruit !== recruit
    ) {
      const terrain = this.terrainView.prepare(view.game, view.artwork);
      tiles = this.battlefieldView.prepare(
        view.game,
        view.selected,
        view.artwork,
        recruit,
        terrain.tiles,
      );
    }
    if (previous?.tiles !== tiles) this.tilesById = new Map(tiles.map((tile) => [tile.id, tile]));
    this.frame = { view, reducedMotion, tiles };
    if (this.ready) this.refreshSafely();
  }
  private refreshSafely() {
    try {
      const changed = measureTrace("Phaser scene preparation", () => this.refresh());
      if (changed !== false) this.invalidate();
      this.cameras.main.setVisible(this.presented || (this.hasFrame && !this.loading));
      this.input.enabled = this.presented || (this.hasFrame && !this.loading);
      this.reportLoading();
    } catch (error) {
      this.failed = true;
      (this.game as BattlefieldGame).fail(error);
    }
  }
  private queueRefresh() {
    if (this.refreshQueued) return;
    this.refreshQueued = true;
    this.resourceEpoch++;
    this.events.once(Phaser.Scenes.Events.PRE_UPDATE, () => {
      this.refreshQueued = false;
      if (this.ready && !this.failed) this.refreshSafely();
    });
  }
  private reportLoading() {
    const audioTotal = this.frame?.view.sound ? this.requiredAudio.size : 0;
    const audioLoaded = audioTotal
      ? [...this.requiredAudio].filter((key) => this.cache.audio.exists(key)).length
      : 0;
    const total = this.requiredImages.size + audioTotal;
    const loaded =
      [...this.requiredImages].filter((key) => this.textures.exists(key)).length + audioLoaded;
    const state: BattlefieldProgress = {
      identity: this.identity,
      phase: this.presented ? "ready" : loaded < total || !total ? "loading" : "preparing",
      loaded,
      total,
      audioLoaded,
      audioTotal,
      error: this.assetError,
    };
    const key = JSON.stringify(state);
    if (key !== this.progressKey) {
      this.progressKey = key;
      this.progress(state);
    }
  }
  retryResources(kind?: "audio") {
    if (!kind) this.assetError = undefined;
    this.assets.retry(kind);
    this.reportLoading();
  }
  private reset(identity: string) {
    this.identity = identity;
    this.planKey = "";
    this.requiredAudio.clear();
    this.presented = false;
    this.assetError = undefined;
    this.cameras.main.setVisible(false);
    this.tweens.killAll();
    this.pulses.clear();
    this.hasFrame = false;
    this.game.canvas.style.visibility = "hidden";
    const units = this.frame!.view.game.units.filter(
      (unit) => this.frame!.view.game.faction(unit.id).controllable,
    );
    const commander = units.find((unit) => unit.role === "leader") ?? units[0];
    this.controls.reset([
      ...units.filter((unit) => unit !== commander).map((unit) => unit.tile),
      ...(commander ? [commander.tile] : []),
    ]);
    for (const actor of this.actors.values()) {
      actor.indicators.destroy();
      actor.root.destroy();
    }
    for (const actor of this.dying.keys()) actor.destroy();
    this.actors.clear();
    this.activeActors.clear();
    this.dying.clear();
    this.terrain?.destroy();
    this.terrain = new BattlefieldTerrain(
      this,
      terrainTexture(this.frame!.view.game.mapId, this.frame!.view.game.mapRevision),
      initialTerrain(
        this.frame!.view.game.mapId,
        this.frame!.view.game.mapRevision,
        this.frame!.view.artwork,
      ),
      this.lighting,
      this.resources,
    );
    this.terrainGame = undefined;
    for (const item of this.groundItems.values()) item.root.destroy();
    for (const hex of this.hexes.values()) hex.root.destroy();
    for (const marker of this.ownerMarkers.values()) marker.destroy();
    this.visibility.clear();
    this.items.removeAll(true);
    this.outlines.removeAll(true);
    this.hexes.clear();
    this.groundItems.clear();
    this.owners.removeAll(true);
    this.ownerMarkers.clear();
    this.atmosphere?.destroy();
    this.atmosphere = new BattlefieldFog(
      this,
      terrainTexture(this.frame!.view.game.mapId, this.frame!.view.game.mapRevision),
      this.frame!.tiles,
      this.resources,
    );
    this.restoreCamera = undefined;
    this.controls.layout();
  }
  private refresh() {
    const frame = this.frame;
    if (!frame || this.failed) return;
    if (frame.reducedMotion) {
      for (const id of this.pulses.keys()) {
        const hex = this.hexes.get(id);
        if (hex) hex.tile = undefined;
      }
      for (const pulse of this.pulses.values()) this.tweens.killTweensOf(pulse);
      this.pulses.clear();
      for (const [root, death] of this.dying) {
        this.tweens.killTweensOf(root);
        this.visibility.delete(death.key);
        root.destroy();
      }
      this.dying.clear();
    }
    const identity = `${frame.view.sessionVersion}:${frame.view.game.mapId}`;
    if (identity !== this.identity) {
      this.reset(identity);
      this.lastVisual = undefined;
      this.sceneGame = undefined;
    }
    const previous = this.lastVisual;
    // Selection/inspection changes no world resources. Preserve actors, fog,
    // terrain and the loader; only changed overlay records need publication.
    if (
      previous?.game === frame.view.game &&
      this.sceneArtwork === frame.view.artwork &&
      previous.motion === frame.reducedMotion &&
      previous.sound === frame.view.sound &&
      previous.epoch === this.resourceEpoch &&
      previous.tiles !== frame.tiles
    ) {
      for (let index = 0; index < frame.tiles.length; index++)
        if (previous.tiles[index] !== frame.tiles[index]) this.drawOutlines(frame.tiles[index]!.id);
      this.describeInspection();
      this.lastVisual = { ...previous, tiles: frame.tiles };
      return true;
    }
    if (
      previous?.game === frame.view.game &&
      previous.tiles === frame.tiles &&
      previous.motion === frame.reducedMotion &&
      previous.sound === frame.view.sound &&
      previous.epoch === this.resourceEpoch
    ) {
      this.describeInspection();
      return false;
    }
    const fog = new Set(frame.view.game.presentation.fog);
    // Conceal immediately, even while a newly published sprite is still loading.
    for (const [id, actor] of this.actors)
      if (!frame.view.game.unit(id)) {
        if (fog.has(actor.data.tile)) {
          this.tweens.killTweensOf(actor.root);
          if (actor.transition) this.tweens.killTweensOf(actor.transition);
          actor.indicators.destroy();
          actor.root.destroy();
          this.actors.delete(id);
          this.activeActors.delete(actor);
          this.visibility.delete(`actor:${id}`);
          this.visibility.delete(`indicator:${id}`);
        }
      }
    for (const [actor, death] of this.dying)
      if (fog.has(death.tile)) {
        this.visibility.delete(death.key);
        this.tweens.killTweensOf(actor);
        actor.destroy();
        this.dying.delete(actor);
      }
    this.atmosphere!.update(frame.view.game.presentation.fog, frame.tiles);
    if (
      this.sceneGame !== frame.view.game ||
      this.sceneArtwork !== frame.view.artwork ||
      !this.sceneData
    ) {
      this.sceneGame = frame.view.game;
      this.sceneArtwork = frame.view.artwork;
      this.sceneData = battlefieldScene(
        frame.view.game,
        frame.tiles,
        frame.reducedMotion,
        identity,
      );
    }
    const scene = this.sceneData;
    const urls = new Set<string>();
    if (this.terrainGame !== frame.view.game) {
      this.terrainGame = frame.view.game;
      this.terrainUrls = this.terrain!.plan(frame.tiles);
    }
    for (const url of this.terrainUrls) urls.add(url);
    for (const unit of scene.units) {
      if (!unit.url) throw new Error(`Missing unit artwork: ${unit.id}`);
      urls.add(unit.url);
    }
    for (const item of scene.items) if (item.url) urls.add(item.url);
    // Equipment belongs to the current units' visuals, including first inspection.
    for (const unit of frame.view.game.units)
      for (const item of unit.items ?? []) urls.add(itemFrame(frame.view.artwork, item.id).url);
    this.requiredImages = urls;
    if (!this.planKey || this.plannedGame !== frame.view.game) {
      this.plannedGame = frame.view.game;
      const planKey = resourcePlanKey(frame.view);
      if (planKey !== this.planKey) {
        this.planKey = planKey;
        this.requiredAudio = new Set(currentMapAudio(frame.view).map((sample) => sample.key));
        const plan = resourcePlan(frame.view);
        for (const key of this.terrain!.retiredSources) plan.delete(key);
        // Actual retained artwork remains needed until adoption/fade completes.
        const retain = (image: Phaser.GameObjects.Image) => {
          const key = image.texture.key;
          if (!plan.has(key))
            plan.set(key, {
              key,
              url: key,
              kind: "image",
              priority: 5,
              width: image.width,
              height: image.height,
            });
        };
        for (const actor of this.actors.values()) retain(actor.image);
        for (const root of this.dying.keys())
          for (const object of root.list)
            if (object instanceof Phaser.GameObjects.Image) retain(object);
        this.assets.setPlan(plan);
      }
    }
    this.assets.requireImages(urls, this.imageMetadata);
    for (const url of urls) {
      const failure = this.assets.error(url);
      if (failure) throw failure;
    }
    if (!this.presented && frame.view.sound)
      for (const key of this.requiredAudio) {
        const failure = this.assets.error(key);
        if (failure) throw failure;
      }
    this.loading =
      [...urls].some((url) => !this.textures.exists(url)) ||
      (!this.presented &&
        frame.view.sound &&
        [...this.requiredAudio].some((key) => !this.cache.audio.exists(key)));
    const terrainReady = [...this.terrainUrls].every((url) => this.textures.exists(url));
    this.actorLighting.configure(frame.view.game.presentation.lighting);
    if (terrainReady) {
      this.terrain!.update(frame.view.game.presentation.lighting);
      for (const key of this.terrain!.retiredSources)
        if (this.terrainUrls.delete(key)) {
          urls.delete(key);
          this.assets.retireImage(key);
        }
    }
    this.drawOutlines();
    const readyItems = scene.items.filter((item) => this.textures.exists(item.url));
    const liveItems = new Set(readyItems.map((item) => item.key));
    for (const [key, record] of this.groundItems)
      if (!liveItems.has(key)) {
        record.root.destroy();
        this.visibility.delete(`item:${key}`);
        this.groundItems.delete(key);
      }
    for (const [ordinal, item] of readyItems.entries()) {
      let record = this.groundItems.get(item.key);
      if (!record) {
        const anchor = { x: item.x, y: item.y },
          root = this.add.container(item.x, item.y);
        const base = this.add
          .ellipse(0, 14, 24, 10, 0xd5bc68, 0.4)
          .setSmoothness(20)
          .setPipeline("BattlefieldActorLighting", { lightPoint: anchor });
        const image = this.add
          .image(0, 0, item.url)
          .setDisplaySize(64, 64)
          .setPipeline("BattlefieldActorLighting", { lightPoint: anchor });
        root.add([base, image]);
        this.items.add(root);
        record = { root, image, anchor };
        this.groundItems.set(item.key, record);
      }
      record.anchor.x = item.x;
      record.anchor.y = item.y;
      record.root.setPosition(item.x, item.y);
      record.image.setTexture(item.url);
      this.actorLighting.prepare(record.image);
      this.visibility.set(
        `item:${item.key}`,
        this.items,
        record.root,
        { left: item.x - 32, top: item.y - 32, right: item.x + 32, bottom: item.y + 32 },
        ordinal,
      );
    }
    const readyUnits = scene.units.flatMap((unit) => {
      if (this.textures.exists(unit.url!)) return [unit];
      const previous = this.actors.get(unit.id);
      return previous ? [{ ...unit, url: previous.data.url }] : [];
    });
    this.syncUnits(readyUnits, fog);
    for (const actor of this.actors.values()) urls.add(actor.image.texture.key);
    // Fading actors retain their source texture until their Phaser tween completes.
    for (const root of this.dying.keys())
      for (const object of root.list)
        if (object instanceof Phaser.GameObjects.Image) urls.add(object.texture.key);
    this.drawOwners();
    this.describeInspection();
    traceInstant("Battlefield workload", {
      tiles: frame.tiles.length,
      units: scene.units.length,
      items: scene.items.length,
      lights: frame.view.game.presentation.lighting.lights?.length ?? 0,
      fog: fog.size,
    });
    this.hasFrame = terrainReady;
    this.lastVisual = {
      game: frame.view.game,
      tiles: frame.tiles,
      motion: frame.reducedMotion,
      sound: frame.view.sound,
      epoch: this.resourceEpoch,
    };
    return true;
  }

  private syncUnits(units: BattlefieldUnit[], fog: Set<string>) {
    const live = new Set(units.map((unit) => unit.id));
    for (const [id, actor] of this.actors)
      if (!live.has(id)) {
        this.actors.delete(id);
        this.activeActors.delete(actor);
        this.visibility.delete(`actor:${id}`);
        this.visibility.delete(`indicator:${id}`);
        this.tweens.killTweensOf(actor.root);
        if (actor.transition) this.tweens.killTweensOf(actor.transition);
        actor.indicators.destroy();
        actor.image.clearTint();
        actor.root.setPosition(actor.data.x, actor.data.y).setAlpha(1);
        if (fog.has(actor.data.tile) || this.frame!.reducedMotion) actor.root.destroy();
        else {
          this.dying.set(actor.root, { tile: actor.data.tile, key: `dying:${id}` });
          this.visibility.set(
            `dying:${id}`,
            this.actorRoots,
            actor.root,
            {
              left: actor.root.x - actor.data.size / 2,
              top: actor.root.y + 20 - actor.data.size,
              right: actor.root.x + actor.data.size / 2,
              bottom: actor.root.y + 30,
            },
            actor.root.depth,
          );
          this.tweens.add({
            targets: actor.root,
            alpha: 0,
            y: actor.root.y + 10,
            duration: 300,
            onUpdate: () => {
              const root = actor.root;
              this.visibility.set(
                `dying:${id}`,
                this.actorRoots,
                root,
                {
                  left: root.x - actor.data.size / 2,
                  top: root.y + 20 - actor.data.size,
                  right: root.x + actor.data.size / 2,
                  bottom: root.y + 26,
                },
                root.depth,
              );
              this.invalidate();
            },
            onComplete: () => {
              this.dying.delete(actor.root);
              this.visibility.delete(`dying:${id}`);
              actor.root.destroy();
              this.invalidate();
            },
          });
        }
      }
    for (const unit of units) {
      let actor = this.actors.get(unit.id);
      const previous = actor?.data;
      if (!actor) {
        const source = this.textures.get(unit.url!).getSourceImage() as HTMLImageElement;
        if (source.width !== 64 || source.height !== 64)
          throw new Error(`Units must use native 64×64 artwork: ${unit.url}`);
        const root = this.add.container(unit.x, unit.y).setDepth(20000 + unit.y);
        const base = this.add
          .ellipse(0, 19, 38, 14, color(unit.color), 0.5)
          .setSmoothness(20)
          .setPipeline("BattlefieldActorLighting", { unlit: true });
        const image = this.add
          .image(-unit.size / 2, 20 - unit.size, unit.url!)
          .setOrigin(0)
          .setDisplaySize(unit.size, unit.size)
          .setPipeline("BattlefieldActorLighting", { lightAnchor: root });
        const indicators = this.add.graphics().setDepth(30000);
        this.actorRoots.add(root);
        this.upperIndicators.add(indicators);
        root.add([base, image]);
        actor = { data: unit, root, image, base, indicators };
        this.actors.set(unit.id, actor);
        if (this.hasFrame && !this.frame!.reducedMotion) this.transition(actor, "spawn", 220);
      } else {
        if (actor.data.x !== unit.x || actor.data.y !== unit.y) {
          if (!this.frame!.reducedMotion) this.transition(actor, "move", 240);
        } else if (unit.hp < actor.data.hp && !this.frame!.reducedMotion)
          this.transition(actor, "hit", 260);
        if (actor.data.url !== unit.url || actor.data.size !== unit.size)
          actor.image.setTexture(unit.url!).setDisplaySize(unit.size, unit.size);
        actor.data = unit;
      }
      this.activeActors.add(actor);
      actor.base.setFillStyle(color(unit.color), 0.5);
      actor.root.setDepth(20000 + unit.y + unit.x * 0.00001);
      actor.image.setAlpha(unit.spent ? 0.65 : 1);
      if (this.frame!.reducedMotion) {
        if (actor.transition) this.tweens.killTweensOf(actor.transition);
        actor.transition = undefined;
        actor.root.setPosition(unit.x, unit.y).setAlpha(1);
        actor.image.clearTint();
      }
      if (
        previous?.health === unit.health &&
        previous.spent === unit.spent &&
        previous.retreating === unit.retreating
      )
        continue;
      const g = actor.indicators.clear();
      g.fillStyle(0x18211b).fillRect(-19, 27, 38, 4);
      g.fillStyle(0xa9d0a5).fillRect(-19, 27, Math.max(0, Math.min(38, unit.health)), 4);
      g.lineStyle(2, 0xdbe8d7);
      if (unit.spent) g.beginPath().moveTo(21, -20).lineTo(24, -17).lineTo(30, -24).strokePath();
      if (unit.retreating)
        g.lineStyle(2, 0xfff1c1)
          .beginPath()
          .moveTo(-22, -21)
          .lineTo(-30, -21)
          .lineTo(-26, -25)
          .moveTo(-30, -21)
          .lineTo(-26, -17)
          .strokePath();
    }
  }

  private transition(actor: UnitActor, kind: "move" | "hit" | "spawn", duration: number) {
    if (actor.transition) this.tweens.killTweensOf(actor.transition);
    const state = { kind, t: 0, fromX: actor.root.x, fromY: actor.root.y };
    actor.transition = state;
    this.activeActors.add(actor);
    this.invalidate();
    this.tweens.add({
      targets: state,
      t: 1,
      duration,
      onUpdate: this.invalidate,
      onComplete: () => {
        if (actor.transition === state) actor.transition = undefined;
        this.invalidate();
      },
    });
  }
  private animateActors() {
    if (!this.frame || !this.ready) return;
    for (const actor of this.activeActors) {
      const unit = actor.data,
        state = actor.transition,
        t = state?.t ?? 1;
      let x = unit.x,
        y = unit.y,
        alpha = 1;
      if (state?.kind === "move") {
        const ease = t * t * (3 - 2 * t);
        x = state.fromX + (unit.x - state.fromX) * ease;
        y = state.fromY + (unit.y - state.fromY) * ease - Math.sin(t * Math.PI) * 5;
      } else if (state?.kind === "spawn") {
        alpha = t;
        y -= (1 - t) * 10;
      } else if (state?.kind === "hit") x += Math.sin(t * Math.PI * 6) * (1 - t) * 4;
      const pulse = this.pulses.get(unit.tile);
      if (pulse?.kind === "attack") x += Math.sin(pulse.t * Math.PI) * 7;
      const flash = state?.kind === "hit" ? Math.sin(t * Math.PI) * 0.5 : 0;
      const shade = Math.round(255 * (1 - flash));
      actor.image.setTint((255 << 16) | (shade << 8) | shade);
      actor.root.setPosition(x, y).setAlpha(alpha);
      actor.indicators.setPosition(x, y).setAlpha(alpha);
      this.actorLighting.prepare(actor.image);
      this.visibility.set(
        `actor:${unit.id}`,
        this.actorRoots,
        actor.root,
        {
          left: x - unit.size / 2,
          top: y + 20 - unit.size,
          right: x + unit.size / 2,
          bottom: y + 26,
        },
        actor.root.depth,
      );
      this.visibility.set(
        `indicator:${unit.id}`,
        this.upperIndicators,
        actor.indicators,
        { left: x - 31, top: y - 26, right: x + 31, bottom: y + 32 },
        0,
      );
      if (!state && pulse?.kind !== "attack") this.activeActors.delete(actor);
    }
    this.actorLighting.preparePending();
    const camera = this.cameras.main,
      origin = camera.getWorldPoint(0, 0),
      edge = camera.getWorldPoint(camera.width, camera.height);
    const bounds = { left: origin.x, top: origin.y, right: edge.x, bottom: edge.y };
    this.visibility.update(bounds);
    this.terrain?.visible(bounds);
    this.atmosphere?.visible(bounds);
  }

  private drawOutlines(tileId?: string) {
    const tiles = tileId ? [this.tilesById.get(tileId)!].filter(Boolean) : this.frame!.tiles;
    if (!tileId) {
      const live = new Set(tiles.map((tile) => tile.id));
      for (const [id, hex] of this.hexes)
        if (!live.has(id)) {
          hex.root.destroy();
          this.visibility.delete(`hex:${id}`);
          this.hexes.delete(id);
        }
    }
    for (const tile of tiles) {
      // Immutable tile views carry overlay invalidation. Ordinary movement must
      // not resubmit unchanged outlines to Phaser's spatial/display lists.
      const existing = this.hexes.get(tile.id);
      if (!tileId && existing?.tile === tile) continue;
      const classes = new Set(tile.classes.split(" "));
      let ink = 0x39472b,
        width = 0.7,
        alpha = 0.3;
      if (classes.has("selected")) {
        ink = 0xffe5a1;
        width = 3;
      }
      if (classes.has("reachable")) {
        ink = 0x95bfd2;
        width = 2.4;
        alpha = 1;
      }
      if (classes.has("target")) {
        ink = 0xdb8980;
        width = 3;
        alpha = 1;
      }
      if (classes.has("recruitable")) {
        ink = 0x8ec7a2;
        width = 3;
        alpha = 1;
      }
      if (classes.has("objective")) {
        ink = classes.has("reachable") || classes.has("selected") ? 0x99c8ec : 0xffe5a1;
        if (classes.has("recruitable")) ink = 0x6fdec0;
        width = 3.5;
        alpha = 1;
      }
      const pulse = this.pulses.get(tile.id);
      if (pulse?.kind === "select") {
        // Ordinary terrain needs contrasting feedback; keep the stronger
        // movement, combat, recruitment and objective outlines intact.
        if (width === 0.7) {
          ink = 0xffe5a1;
          width = 3;
        }
        const t = pulse.t;
        alpha = t < 0.55 ? 0.45 + (0.55 * t) / 0.55 : 1 - (0.2 * (t - 0.55)) / 0.45;
      }
      const dash: readonly [number, number] | undefined = classes.has("objective")
        ? [9, 5]
        : classes.has("recruitable")
          ? [4, 3]
          : undefined;
      let hex = this.hexes.get(tile.id);
      if (!hex) {
        const stroke = this.add
          .image(tile.x, tile.y, "battlefield-white")
          .setDisplaySize(84, 90)
          .setPipeline("BattlefieldHex");
        const root = this.add.container(0, 0);
        root.add(stroke);
        this.outlines.add(root);
        hex = { root, ordinal: this.hexes.size, stroke };
        this.hexes.set(tile.id, hex);
      }
      // Keep scene objects stable. Selection tweens change one hex's opacity,
      // never destroy and reconstruct the entire grid on every animation frame.
      hex.stroke.setPosition(tile.x, tile.y).setTint(ink).setAlpha(alpha);
      hex.stroke.setPipelineData("width", width);
      hex.stroke.setPipelineData("dash", dash);
      if (dash && !hex.underlay) {
        hex.underlay = this.add
          .image(tile.x, tile.y, "battlefield-white")
          .setDisplaySize(84, 90)
          .setTint(0x172015)
          .setPipeline("BattlefieldHex", { width: 7 });
        hex.root.addAt(hex.underlay, 0);
      }
      hex.underlay?.setPosition(tile.x, tile.y).setVisible(!!dash);
      if (hex.underlay) hex.underlay.setPipelineData("dash", dash);
      this.visibility.set(
        `hex:${tile.id}`,
        this.outlines,
        hex.root,
        { left: tile.x - 42, top: tile.y - 45, right: tile.x + 42, bottom: tile.y + 45 },
        hex.ordinal,
      );
      hex.tile = tile;
    }
  }
  private drawOwners(tileId?: string) {
    const fog = new Set(this.frame!.view.game.presentation.fog);
    const tiles = tileId ? [this.tilesById.get(tileId)!].filter(Boolean) : this.frame!.tiles;
    if (!tileId) {
      const live = new Set(
        tiles.filter((tile) => tile.owner && !fog.has(tile.id)).map((tile) => tile.id),
      );
      for (const [id, marker] of this.ownerMarkers)
        if (!live.has(id)) {
          marker.destroy();
          this.visibility.delete(`owner:${id}`);
          this.ownerMarkers.delete(id);
          const pulse = this.pulses.get(id);
          if (pulse?.kind === "capture") {
            this.tweens.killTweensOf(pulse);
            this.pulses.delete(id);
          }
        }
    }
    for (const tile of tiles)
      if (tile.owner && !fog.has(tile.id)) {
        let marker = this.ownerMarkers.get(tile.id);
        const retained = !!marker;
        if (!marker) {
          marker = this.add
            .ellipse(tile.x + 23, tile.y + 17, 10, 10, color(tile.ownerColor))
            .setSmoothness(20);
          this.ownerMarkers.set(tile.id, marker);
          this.owners.add(marker);
        }
        const pulse = this.pulses.get(tile.id),
          radius = pulse?.kind === "capture" ? 5 + Math.sin(pulse.t * Math.PI) * 3 : 5;
        const fill = color(tile.ownerColor);
        if (
          retained &&
          marker.x === tile.x + 23 &&
          marker.y === tile.y + 17 &&
          marker.fillColor === fill &&
          marker.scaleX === radius / 5
        )
          continue;
        marker
          .setPosition(tile.x + 23, tile.y + 17)
          .setFillStyle(fill)
          .setScale(radius / 5);
        this.visibility.set(
          `owner:${tile.id}`,
          this.owners,
          marker,
          { left: tile.x + 15, top: tile.y + 9, right: tile.x + 31, bottom: tile.y + 25 },
          this.hexes.get(tile.id)?.ordinal ?? 0,
        );
      }
  }
  private describeInspection() {
    const frame = this.frame!;
    const tile = frame.view.inspectedTile
      ? this.tilesById.get(frame.view.inspectedTile)
      : undefined;
    const label = tile
      ? frame.view.game.presentation.fog.includes(tile.id)
        ? `Fog-covered terrain, column ${tile.col + 1}, row ${tile.row + 1}`
        : tile.label
      : "Battlefield";
    this.describe(label);
    this.game.canvas.setAttribute("aria-label", `Battlefield. ${label}`);
  }
  pulse(tileId: string, kind: MotionKind) {
    if (!this.ready || !this.frame || this.frame.reducedMotion) return;
    if (!this.tilesById.has(tileId)) return;
    const previous = this.pulses.get(tileId);
    if (previous) {
      this.tweens.killTweensOf(previous);
      this.pulses.delete(tileId);
      if (previous.kind === "capture") this.drawOwners(tileId);
      if (previous.kind === "select") this.drawOutlines(tileId);
    }
    const pulse = { kind, t: 0 };
    this.pulses.set(tileId, pulse);
    if (kind === "attack")
      for (const actor of this.actors.values())
        if (actor.data.tile === tileId) this.activeActors.add(actor);
    this.invalidate();
    const redraw = () => {
      this.invalidate();
      if (kind === "select") this.drawOutlines(tileId);
      if (kind === "capture") this.drawOwners(tileId);
    };
    this.tweens.add({
      targets: pulse,
      t: 1,
      duration: kind === "select" ? 180 : 300,
      onUpdate: redraw,
      onComplete: () => {
        if (this.pulses.get(tileId) === pulse) this.pulses.delete(tileId);
        redraw();
      },
    });
  }
}

/** Ordinary single Game lifecycle; the boot owner retains this retry handle. */
export function preparePhaserBattlefield() {
  let game: BattlefieldGame | undefined;
  let scene: BattlefieldScene | undefined;
  return {
    mount(
      parent: HTMLElement,
      actions: () => BattleCommands,
      reportError: (message: string) => void,
      describe: (label: string) => void,
      previews: HudArtworkCache,
      progress: (state: BattlefieldProgress) => void,
    ) {
      scene = new BattlefieldScene(actions, describe, previews, progress, scene?.cameraSnapshot);
      const owner = new BattlefieldGame(
        {
          type: Phaser.WEBGL,
          parent,
          canvasStyle: "visibility: hidden;",
          backgroundColor: "#19231c",
          pixelArt: false,
          roundPixels: false,
          antialiasGL: true,
          antialias: false,
          banner: false,
          callbacks: {
            preBoot: (owner) => {
              // Phaser 3.90's anonymous unlock listener survives locked destruction.
              // Our removable listeners resume this same native context instead.
              const manager = owner.sound as Phaser.Sound.WebAudioSoundManager & { unlock(): void };
              owner.events.off(Phaser.Core.Events.BOOT, manager.unlock, manager);
            },
          },
          input: { touch: { capture: true }, mouse: { preventDefaultWheel: true } },
          loader: {
            maxParallelDownloads: 6,
            timeout: 30000,
            maxRetries: 0,
            imageLoadType: "HTMLImageElement",
          },
          scale: { mode: Phaser.Scale.NONE, width: 1, height: 1 },
          scene: [scene],
        },
        reportError,
      );
      game = owner;
      const detach = attachSound(owner, async () => {
        const manager = owner.sound as Phaser.Sound.WebAudioSoundManager;
        if (!manager.context) return !manager.locked;
        if (manager.context.state === "closed") {
          manager.removeAll();
          const replacement = manager.createAudioContext(owner);
          // Phaser 3.90's setter unconditionally closes its previous context,
          // producing an unhandled rejection if that context is already closed.
          (manager as unknown as { context: AudioContext | null }).context = null;
          manager.setAudioContext(replacement);
        }
        if (manager.context.state !== "running") await manager.context.resume();
        if (owner !== game || !manager.context || manager.context.state !== "running") return false;
        // Phaser 3.90's own unlock handler sets this flag for its update loop.
        const permission = manager as unknown as { unlocked: boolean; locked: boolean };
        permission.unlocked = true;
        permission.locked = false;
        return true;
      });
      owner.events.once(Phaser.Core.Events.DESTROY, detach);
      return { game: owner, scene };
    },
    retryAudio() {
      const owner = game;
      const active = scene;
      if (!owner || owner.unavailable || !active) {
        audioReadiness.set({
          ready: false,
          error: errorDetails(
            new Error(
              "Battlefield audio is unavailable because its renderer stopped. Retry map rendering to restore its native audio owner.",
            ),
          ),
        });
        return;
      }
      active.retryResources("audio");
    },
    dispose() {
      game?.destroy(true);
      game = undefined;
      scene = undefined;
    },
  };
}
export type PreparedBattlefield = ReturnType<typeof preparePhaserBattlefield>;
export type createPhaserBattlefield = PreparedBattlefield["mount"];
