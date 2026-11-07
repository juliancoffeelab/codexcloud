import Phaser from "phaser";
import { detachRoot } from "./phaser-visibility";
import { TERRAIN } from "../game/catalog";
import type { BattlefieldTileView } from "../ui/battlefield-view";
import { battlefieldSpriteSize } from "../ui/geometry";
import type { TerrainMapSprite, TerrainPage } from "./terrain-map";
import {
  changedLights,
  lightPosition,
  lightingKey,
  type BattlefieldLighting,
} from "./battlefield-lighting";
import { BattlefieldLightingPipeline } from "./phaser-lighting";
import { BattlefieldResources } from "./phaser-resources";
import { SpatialIndex, overlaps, type PaintBounds } from "./spatial-index";
import { measureTrace } from "./performance-trace";

const scale = 64 / battlefieldSpriteSize;
export type TerrainCell = Pick<
  BattlefieldTileView,
  | "id"
  | "x"
  | "y"
  | "terrain"
  | "terrainFrame"
  | "spriteX"
  | "spriteY"
  | "spriteWidth"
  | "spriteHeight"
>;
const bounds = (t: TerrainCell): PaintBounds => ({
  left: t.spriteX * scale,
  top: t.spriteY * scale,
  right: (t.spriteX + t.spriteWidth) * scale,
  bottom: (t.spriteY + t.spriteHeight) * scale,
});
const same = (a: TerrainCell, b: TerrainCell) =>
  a.terrainFrame === b.terrainFrame &&
  a.terrain === b.terrain &&
  a.spriteX === b.spriteX &&
  a.spriteY === b.spriteY &&
  a.spriteWidth === b.spriteWidth &&
  a.spriteHeight === b.spriteHeight &&
  a.x === b.x &&
  a.y === b.y;
type PageRecord = {
  page: TerrainPage;
  unlit?: Phaser.GameObjects.Image;
  lit?: Phaser.GameObjects.RenderTexture;
  display?: Phaser.GameObjects.Image;
  dirty: boolean;
};
const region = (page: TerrainPage): PaintBounds => ({
  left: page.x - page.gutter,
  top: page.y - page.gutter,
  right: page.x + page.interiorWidth + page.gutter,
  bottom: page.y + page.interiorHeight + page.gutter,
});

/** All pages remain resident. Only their disjoint interiors are presented. */
export class BattlefieldTerrain {
  readonly retiredSources = new Set<string>();
  private visibilitySignature = "";
  private version = 0;
  readonly root: Phaser.GameObjects.Container;
  private pages: PageRecord[];
  private active = new Set<PageRecord>();
  private pageIndex = new SpatialIndex<PageRecord>(1020);
  private tileIndex = new SpatialIndex<TerrainCell>(128);
  private cells = new Map<string, TerrainCell>();
  private current: readonly TerrainCell[] = [];
  private dirty = new Set<PageRecord>();
  private lighting?: BattlefieldLighting;
  private signature = "";
  private lightIndex = new SpatialIndex<number>(256);
  constructor(
    private scene: Phaser.Scene,
    private manifest: TerrainMapSprite,
    initial: readonly TerrainCell[],
    private pipeline: BattlefieldLightingPipeline,
    private resources: BattlefieldResources,
  ) {
    resources.area("terrain manifest", manifest.width, manifest.height);
    this.root = scene.add.container(0, 0);
    this.pages = manifest.pages.map((page) => ({ page, dirty: false }));
    for (const record of this.pages) {
      const page = record.page;
      resources.check("terrain page", page.width, page.height, true);
      resources.check("double-width coefficient page", page.width * 2, page.height);
      if (
        ![page.x, page.y, page.interiorWidth, page.interiorHeight, page.gutter].every(
          Number.isSafeInteger,
        ) ||
        page.x < 0 ||
        page.y < 0 ||
        page.interiorWidth <= 0 ||
        page.interiorHeight <= 0 ||
        page.x + page.interiorWidth > manifest.width ||
        page.y + page.interiorHeight > manifest.height
      )
        throw new Error(`Invalid terrain page extent: ${page.url}`);
      if (
        page.width !== page.interiorWidth + page.gutter * 2 ||
        page.height !== page.interiorHeight + page.gutter * 2 ||
        page.gutter < 2
      )
        throw new Error(`Invalid padded terrain manifest: ${page.url}`);
      this.pageIndex.set(record, region(page));
    }
    for (const tile of initial) {
      this.cells.set(tile.id, tile);
      this.tileIndex.set(tile, bounds(tile));
    }
  }
  /** Called only on game/terrain publication. Each dirty page is rebuilt once. */
  plan(current: readonly TerrainCell[]) {
    this.current = current;
    const live = new Set<string>();
    for (const tile of current) {
      live.add(tile.id);
      const previous = this.cells.get(tile.id);
      if (previous && same(previous, tile)) continue;
      if (previous) {
        for (const page of this.pageIndex.query(bounds(previous))) this.dirty.add(page);
        this.tileIndex.delete(previous);
      }
      for (const page of this.pageIndex.query(bounds(tile))) this.dirty.add(page);
      this.cells.set(tile.id, tile);
      this.tileIndex.set(tile, bounds(tile));
    }
    for (const [id, tile] of this.cells)
      if (!live.has(id)) {
        for (const page of this.pageIndex.query(bounds(tile))) this.dirty.add(page);
        this.cells.delete(id);
        this.tileIndex.delete(tile);
      }
    const urls = new Set(
      this.pages
        .filter((record) => !this.retiredSources.has(record.page.url))
        .map((record) => record.page.url),
    );
    for (const page of this.dirty)
      for (const tile of this.tileIndex.query(region(page.page))) {
        if (!tile.terrainFrame) throw new Error(`Missing terrain frame: ${tile.id}`);
        urls.add(tile.terrainFrame.url);
      }
    return urls;
  }
  private makeSource(page: TerrainPage) {
    const source = this.scene.textures.get(page.url).getSourceImage() as HTMLImageElement;
    if (source.naturalWidth !== page.width || source.naturalHeight !== page.height)
      throw new Error(`Invalid natural terrain page dimensions: ${page.url}`);
    return this.scene.make
      .image({ key: page.url }, false)
      .setOrigin(0)
      .setPosition((page.x - page.gutter) / scale, (page.y - page.gutter) / scale)
      .setDisplaySize(page.width / scale, page.height / scale);
  }
  private rebuild(record: PageRecord) {
    const page = record.page;
    let surface: Phaser.GameObjects.RenderTexture;
    if (record.unlit instanceof Phaser.GameObjects.RenderTexture) surface = record.unlit;
    else {
      surface = new Phaser.GameObjects.RenderTexture(
        this.scene,
        (page.x - page.gutter) / scale,
        (page.y - page.gutter) / scale,
        page.width,
        page.height,
        false,
      )
        .setOrigin(0)
        .setDisplaySize(page.width / scale, page.height / scale);
      record.unlit?.destroy();
      this.retiredSources.add(page.url);
      record.unlit = surface;
    }
    surface.camera
      .setOrigin(0, 0)
      .setZoom(scale)
      .setScroll((page.x - page.gutter) / scale, (page.y - page.gutter) / scale);
    const images: Phaser.GameObjects.Image[] = [];
    try {
      const tiles = this.tileIndex
        .query(region(page))
        .sort(
          (a, b) => (a.terrainFrame?.z ?? 0) - (b.terrainFrame?.z ?? 0) || a.y - b.y || a.x - b.x,
        );
      for (const tile of tiles)
        images.push(
          this.scene.make
            .image({ key: tile.terrainFrame!.url }, false)
            .setOrigin(0)
            .setPosition(tile.spriteX, tile.spriteY)
            .setDisplaySize(tile.spriteWidth, tile.spriteHeight)
            .setPipeline("BattlefieldLighting", {
              unlit: true,
              brightness: TERRAIN[tile.terrain].role === "forest" ? 0.93 : 1,
            }),
        );
      surface.clear().draw(images);
      this.clipOuter(surface, page);
      record.dirty = true;
      this.resources.set(
        `terrain-edit:${page.url}`,
        "edited unlit terrain",
        0,
        page.width * page.height * 4,
      );
    } finally {
      for (const image of images) image.destroy();
    }
  }
  private clipOuter(target: Phaser.GameObjects.RenderTexture, page: TerrainPage) {
    const texture = target.texture as Phaser.Textures.DynamicTexture;
    const left = Math.max(0, page.gutter - page.x),
      top = Math.max(0, page.gutter - page.y),
      right = Math.max(0, page.x - page.gutter + page.width - this.manifest.width),
      bottom = Math.max(0, page.y - page.gutter + page.height - this.manifest.height);
    // Phaser marks the whole texture clean even for a partial clear. Preserve
    // its real content state so every guard and the next full clear execute.
    const clear = (x: number, y: number, width: number, height: number) => {
      texture.dirty = true;
      texture.clear(x, y, width, height);
    };
    if (left) clear(0, 0, left, page.height);
    if (top) clear(0, 0, page.width, top);
    if (right) clear(page.width - right, 0, right, page.height);
    if (bottom) clear(0, page.height - bottom, page.width, bottom);
    texture.dirty = true;
  }
  update(lighting: BattlefieldLighting) {
    const signature = lighting.time === "day" ? "day" : lightingKey(lighting);
    if (signature === this.signature && !this.dirty.size && this.pages.every((page) => page.unlit))
      return;
    this.signature = signature;
    const before = this.lighting;
    const global = before?.time !== lighting.time;
    const changed = new Set<PageRecord>();
    if (!before || global) for (const page of this.pages) changed.add(page);
    else {
      for (const light of changedLights(before.lights ?? [], lighting.lights ?? [])) {
        const p = lightPosition(light),
          r = Math.max(light.radius, 25);
        for (const page of this.pageIndex.query({
          left: (p.x - r) * scale,
          right: (p.x + r) * scale,
          top: (p.y - r) * scale,
          bottom: (p.y + r) * scale,
        }))
          changed.add(page);
      }
    }
    this.lighting = lighting;
    this.lightIndex.clear();
    for (const [index, light] of (lighting.time === "day"
      ? []
      : (lighting.lights ?? [])
    ).entries()) {
      const p = lightPosition(light),
        r = Math.max(light.radius, 25);
      this.lightIndex.set(index, { left: p.x - r, right: p.x + r, top: p.y - r, bottom: p.y + r });
    }
    for (const record of this.pages) {
      if (!record.unlit) record.unlit = this.makeSource(record.page);
      if (this.dirty.has(record)) {
        measureTrace("Phaser terrain page rebuild", () => this.rebuild(record));
        changed.add(record);
      }
      if (changed.has(record))
        measureTrace("Phaser terrain page lighting", () => this.shade(record, lighting));
    }
    this.dirty.clear();
    if (lighting.time === "day") this.pipeline.release();
    this.resources.publish();
  }
  private shade(record: PageRecord, lighting: BattlefieldLighting) {
    this.version++;
    const page = record.page,
      source = record.unlit!;
    record.display?.destroy();
    record.display = undefined;
    if (lighting.time === "day") {
      record.lit?.destroy();
      record.lit = undefined;
      this.resources.remove(`terrain-lit:${page.url}`);
      source.setPipeline("BattlefieldLighting", { unlit: true });
      const display = new Phaser.GameObjects.Image(this.scene, 0, 0, source.texture)
        .setOrigin(0)
        .setPosition(source.x, source.y)
        .setDisplaySize(page.width / scale, page.height / scale)
        .setCrop(page.gutter, page.gutter, page.interiorWidth, page.interiorHeight);
      record.display = display;
      return;
    }
    if (!record.lit)
      record.lit = new Phaser.GameObjects.RenderTexture(
        this.scene,
        source.x,
        source.y,
        page.width,
        page.height,
        false,
      )
        .setOrigin(0)
        .setDisplaySize(page.width / scale, page.height / scale);
    const target = record.lit;
    target.camera.setOrigin(0, 0).setZoom(scale).setScroll(source.x, source.y);
    const localBounds = region(page),
      indices = this.lightIndex
        .query({
          left: localBounds.left / scale,
          top: localBounds.top / scale,
          right: localBounds.right / scale,
          bottom: localBounds.bottom / scale,
        })
        .sort((a, b) => a - b);
    const lights = lighting.lights ?? [],
      local = { ...lighting, lights: indices.map((index) => lights[index]!) };
    this.pipeline.configure(local, page.width / scale, page.height / scale, source.x, source.y);
    source.setPipeline("BattlefieldLighting", { lightingCamera: target.camera });
    const entries: Phaser.GameObjects.GameObject[] = [],
      temporary: Phaser.GameObjects.GameObject[] = [];
    for (const cell of this.pipeline.cells) {
      const piece = new Phaser.GameObjects.Image(this.scene, source.x, source.y, source.texture)
        .setOrigin(0)
        .setDisplaySize(source.displayWidth, source.displayHeight)
        .setCrop(cell.x, cell.y, cell.width, cell.height)
        .setPipeline("BattlefieldLighting", { lightingCamera: target.camera, lightCell: cell });
      entries.push(piece);
      temporary.push(piece);
    }
    try {
      for (const light of local.lights) {
        const { x, y } = lightPosition(light),
          fixture = light.fixture !== false;
        if (fixture) {
          const brush = this.scene.make.graphics({ x: 0, y: 0 });
          temporary.push(brush);
          entries.push(brush);
          brush.fillStyle(0x49361f).fillRect(x - 3, y - 9, 6, 12);
          brush.fillStyle(0xf6aa46).fillRect(x - 2, y - 7, 4, 5);
          brush.fillStyle(0xfff0b0).fillRect(x - 1, y - 6, 2, 3);
          brush.fillStyle(0x50402e).fillRect(x - 1, y + 3, 2, 8);
        }
        const radius = fixture ? 20 : 25,
          halo = this.scene.make
            .image({ key: "battlefield-white", x, y: y - (fixture ? 5 : 0) }, false)
            .setDisplaySize(radius * 2, radius * 2)
            .setPipeline("BattlefieldHalo", { fixture, color: light.color ?? [255, 170, 70] })
            .setBlendMode(Phaser.BlendModes.SCREEN);
        temporary.push(halo);
        entries.push(halo);
      }
      // One native capture and one whole-page blit, preserving every source,
      // NORMAL fixture and SCREEN halo in authored order.
      target.clear().beginDraw();
      try {
        target.batchDraw(entries);
      } finally {
        target.endDraw();
      }
    } finally {
      for (const object of temporary) object.destroy();
    }
    this.clipOuter(target, page);
    record.display = new Phaser.GameObjects.Image(this.scene, 0, 0, target.texture)
      .setOrigin(0)
      .setPosition(source.x, source.y)
      .setDisplaySize(page.width / scale, page.height / scale)
      .setCrop(page.gutter, page.gutter, page.interiorWidth, page.interiorHeight);
    this.resources.set(
      `terrain-lit:${page.url}`,
      "lit terrain finals",
      0,
      page.width * page.height * 4,
    );
  }
  visible(bounds: PaintBounds) {
    const signature = `${bounds.left}:${bounds.top}:${bounds.right}:${bounds.bottom}:${this.version}`;
    if (signature === this.visibilitySignature) return;
    this.visibilitySignature = signature;
    const records = this.pageIndex.query({
      left: bounds.left * scale,
      top: bounds.top * scale,
      right: bounds.right * scale,
      bottom: bounds.bottom * scale,
    });
    const visible = new Set(records);
    for (const record of this.active)
      if (!visible.has(record) && record.display?.parentContainer)
        detachRoot(this.root, record.display);
    for (const record of records)
      if (record.display && !record.display.parentContainer) this.root.add(record.display);
    this.active = visible;
  }
  destroy() {
    for (const record of this.pages) {
      record.unlit?.destroy();
      record.lit?.destroy();
      record.display?.destroy();
      this.resources.remove(`terrain-edit:${record.page.url}`);
      this.resources.remove(`terrain-lit:${record.page.url}`);
    }
    this.root.destroy();
  }
}
