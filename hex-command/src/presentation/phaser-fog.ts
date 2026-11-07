import Phaser from "phaser";
import { detachRoot } from "./phaser-visibility";
import type { BattlefieldTileView } from "../ui/battlefield-view";
import type { TerrainMapSprite } from "./terrain-map";
import { battlefieldSpriteSize } from "../ui/geometry";
import { BattlefieldResources } from "./phaser-resources";
import { SpatialIndex, type PaintBounds } from "./spatial-index";
import { measureTrace } from "./performance-trace";

const interior = 510,
  guard = 14; // Full kernel reach, bilinear footprint and legacy odd-extent capture offset.
type Page = {
  x: number;
  y: number;
  width: number;
  height: number;
  target?: Phaser.GameObjects.RenderTexture;
  display?: Phaser.GameObjects.Image;
};
/** Reuse two bounded scratch targets. Final fog pages stay resident. Both passes
 * sample the same global half-native lattice, including the zero outer boundary. */
export class BattlefieldFog {
  private visibilitySignature = "";
  private version = 0;
  readonly root: Phaser.GameObjects.Container;
  private pages: Page[] = [];
  private active = new Set<Page>();
  private pagesIndex = new SpatialIndex<Page>(512);
  private tilesIndex = new SpatialIndex<BattlefieldTileView>(128);
  private fog = new Set<string>();
  private dirty = new Set<Page>();
  private mask?: Phaser.GameObjects.RenderTexture;
  private scratch?: Phaser.GameObjects.RenderTexture;
  private width: number;
  private height: number;
  private scratchWidth: number;
  private scratchHeight: number;
  private pixelX: number;
  private pixelY: number;
  private initialized = false;
  constructor(
    private scene: Phaser.Scene,
    private manifest: TerrainMapSprite,
    tiles: readonly BattlefieldTileView[],
    private resources: BattlefieldResources,
  ) {
    this.root = scene.add.container(0, 0).setDepth(9500);
    const scale = 64 / battlefieldSpriteSize;
    this.width = Math.ceil(manifest.width / 2);
    this.height = Math.ceil(manifest.height / 2);
    this.pixelX = manifest.width / scale / this.width;
    this.pixelY = manifest.height / scale / this.height;
    this.scratchWidth = Math.min(interior, this.width) + guard * 2;
    this.scratchHeight = Math.min(interior, this.height) + guard * 2;
    resources.check("padded half-native fog", this.scratchWidth, this.scratchHeight, true);
    for (let y = 0; y < this.height; y += interior)
      for (let x = 0; x < this.width; x += interior) {
        const page = {
          x,
          y,
          width: Math.min(interior, this.width - x),
          height: Math.min(interior, this.height - y),
        };
        this.pages.push(page);
        this.pagesIndex.set(page, {
          left: x - guard,
          top: y - guard,
          right: x + page.width + guard,
          bottom: y + page.height + guard,
        });
      }
    for (const tile of tiles)
      this.tilesIndex.set(tile, {
        left: tile.x - 36,
        top: tile.y - 35,
        right: tile.x + 36,
        bottom: tile.y + 35,
      });
  }
  update(ids: readonly string[], tiles: readonly BattlefieldTileView[]) {
    const fog = new Set(ids),
      changed = new Set<Page>();
    if (!this.initialized) {
      for (const page of this.pages) changed.add(page);
      this.initialized = true;
    } else {
      const byId = new Map(tiles.map((tile) => [tile.id, tile]));
      for (const id of new Set([...fog, ...this.fog]))
        if (fog.has(id) !== this.fog.has(id)) {
          const tile = byId.get(id);
          if (!tile) continue;
          for (const page of this.pagesIndex.query({
            left: (tile.x - 36) / this.pixelX,
            top: (tile.y - 35) / this.pixelY,
            right: (tile.x + 36) / this.pixelX,
            bottom: (tile.y + 35) / this.pixelY,
          }))
            changed.add(page);
        }
    }
    for (const page of changed) this.dirty.add(page);
    this.fog = fog;
    this.root.setVisible(!!fog.size);
    if (!this.dirty.size || !fog.size) return;
    if (!this.mask) {
      this.mask = new Phaser.GameObjects.RenderTexture(
        this.scene,
        0,
        0,
        this.scratchWidth,
        this.scratchHeight,
        false,
      ).setOrigin(0);
      this.scratch = new Phaser.GameObjects.RenderTexture(
        this.scene,
        0,
        0,
        this.scratchWidth,
        this.scratchHeight,
        false,
      ).setOrigin(0);
      this.mask.texture.setFilter(Phaser.Textures.FilterMode.LINEAR);
      this.scratch.texture.setFilter(Phaser.Textures.FilterMode.LINEAR);
      // Phaser 3.90 Camera.preRender rounds its default half-viewport origin.
      // Preserve the existing whole-mask odd-extent half-pixel translation in
      // both blur captures, independently of this bounded target's dimensions.
      this.scratch.camera.setOrigin(0, 0).setScroll(-(this.width % 2) / 2, -(this.height % 2) / 2);
      this.resources.set(
        "fog-scratch",
        "fog scratch",
        0,
        this.scratchWidth * this.scratchHeight * 8,
      );
    }
    for (const page of this.dirty)
      measureTrace("Phaser fog page rebuild", () => this.rebuild(page));
    this.dirty.clear();
  }
  private rebuild(page: Page) {
    this.version++;
    const x = page.x - guard,
      y = page.y - guard,
      world: PaintBounds = {
        left: x * this.pixelX,
        top: y * this.pixelY,
        right: (x + this.scratchWidth) * this.pixelX,
        bottom: (y + this.scratchHeight) * this.pixelY,
      };
    const brush = this.scene.make
      .graphics({ x: 0, y: 0 })
      .fillStyle(0xffffff)
      .setPipeline("BattlefieldFogMask", {
        width: this.scratchWidth * this.pixelX,
        height: this.scratchHeight * this.pixelY,
        originX: world.left,
        originY: world.top,
        extent: [this.manifest.displayWidth, this.manifest.displayHeight],
      });
    try {
      for (const tile of this.tilesIndex.query(world))
        if (this.fog.has(tile.id))
          brush.fillPoints(
            tile.points.split(" ").map((p) => {
              const [x, y] = p.split(",").map(Number);
              return new Phaser.Math.Vector2(x!, y!);
            }),
            true,
          );
      this.mask!.camera.setOrigin(0, 0).setZoom(1);
      this.mask!.clear().draw(brush);
    } finally {
      brush.destroy();
    }
    const region = [x, y, this.scratchWidth, this.scratchHeight],
      extent = [this.width, this.height],
      capture = [(this.width % 2) / 2, (this.height % 2) / 2];
    this.mask!.setPipeline("BattlefieldFogBlur", {
      x: 1 / this.scratchWidth,
      y: 0,
      region,
      extent,
      capture,
    });
    this.scratch!.clear().draw(this.mask!);
    this.scratch!.setPipeline("BattlefieldFogBlur", {
      x: 0,
      y: 1 / this.scratchHeight,
      region,
      extent,
      capture,
    });
    if (!page.target) {
      page.target = new Phaser.GameObjects.RenderTexture(
        this.scene,
        0,
        0,
        this.scratchWidth,
        this.scratchHeight,
        false,
      ).setOrigin(0);
      page.target.texture.setFilter(Phaser.Textures.FilterMode.LINEAR);
      page.target.camera.setOrigin(0, 0).setScroll(-(this.width % 2) / 2, -(this.height % 2) / 2);
      page.display = new Phaser.GameObjects.Image(
        this.scene,
        world.left,
        world.top,
        page.target.texture,
      )
        .setOrigin(0)
        .setDisplaySize(this.scratchWidth * this.pixelX, this.scratchHeight * this.pixelY)
        .setCrop(guard, guard, page.width, page.height)
        .setPipeline("BattlefieldFogComposite");
      this.resources.set(
        `fog:${page.x}:${page.y}`,
        "fog finals",
        0,
        this.scratchWidth * this.scratchHeight * 4,
      );
    }
    page.target.clear().draw(this.scratch!);
  }
  visible(bounds: PaintBounds) {
    const signature = `${bounds.left}:${bounds.top}:${bounds.right}:${bounds.bottom}:${this.version}`;
    if (signature === this.visibilitySignature) return;
    this.visibilitySignature = signature;
    const records = this.pagesIndex.query({
      left: bounds.left / this.pixelX,
      top: bounds.top / this.pixelY,
      right: bounds.right / this.pixelX,
      bottom: bounds.bottom / this.pixelY,
    });
    const visible = new Set(records);
    for (const page of this.active)
      if (!visible.has(page) && page.display?.parentContainer) detachRoot(this.root, page.display);
    for (const page of records)
      if (page.display && !page.display.parentContainer) this.root.add(page.display);
    this.active = visible;
  }
  destroy() {
    this.mask?.destroy();
    this.scratch?.destroy();
    this.resources.remove("fog-scratch");
    for (const page of this.pages) {
      page.display?.destroy();
      page.target?.destroy();
      this.resources.remove(`fog:${page.x}:${page.y}`);
    }
    this.root.destroy();
  }
}
