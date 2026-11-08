import Phaser from "phaser";
import { battlefieldSpriteSize, hexPoints } from "../ui/geometry";

const points = hexPoints(0, 0)
  .split(" ")
  .map((point) => {
    const [x, y] = point.split(",").map(Number);
    return new Phaser.Geom.Point(x!, y!);
  });
const edges = points.map((a, i) => {
  const b = points[(i + 1) % points.length]!;
  return { a, b, length: Phaser.Math.Distance.BetweenPoints(a, b) };
});
const perimeter = edges.reduce((sum, edge) => sum + edge.length, 0);

export const idleHexLineWidth = 0.7;
const idleTextureKey = "battlefield-idle-hex";
const nativeScale = 64 / battlefieldSpriteSize;
const idleTextureSize = 72;

/** One shared native-resolution outline, created and uploaded by Phaser. */
function idleTexture(scene: Phaser.Scene) {
  if (!scene.textures.exists(idleTextureKey)) {
    const graphics = scene.make.graphics({ x: 0, y: 0 }, false);
    try {
      graphics.lineStyle(idleHexLineWidth * nativeScale, 0xffffff, 1).strokePoints(
        points.map(({ x, y }) => ({
          x: x * nativeScale + idleTextureSize / 2,
          y: y * nativeScale + idleTextureSize / 2,
        })),
        true,
      );
      graphics.generateTexture(idleTextureKey, idleTextureSize, idleTextureSize);
      scene.textures.get(idleTextureKey).setFilter(Phaser.Textures.FilterMode.NEAREST);
    } finally {
      graphics.destroy();
    }
  }
  return idleTextureKey;
}

/** A baked idle grid with vector selection and action highlights. */
export class BattlefieldHexOutline {
  readonly root: Phaser.GameObjects.Container;
  private idle: Phaser.GameObjects.Image;
  private graphics: Phaser.GameObjects.Graphics;
  private underlay?: Phaser.GameObjects.Graphics;
  private style = "";
  constructor(private scene: Phaser.Scene) {
    this.root = scene.add.container(0, 0);
    this.idle = scene.add
      .image(0, 0, idleTexture(scene))
      .setDisplaySize(idleTextureSize / nativeScale, idleTextureSize / nativeScale);
    this.graphics = scene.add.graphics();
    this.root.add([this.idle, this.graphics]);
  }
  update(
    x: number,
    y: number,
    color: number,
    width: number,
    alpha: number,
    highlighted: boolean,
    dash?: readonly [number, number],
  ) {
    this.root.setPosition(x, y);
    const idle = !highlighted;
    this.idle.setVisible(idle).setTint(color).setAlpha(alpha);
    this.graphics.setVisible(!idle).setAlpha(alpha);
    this.underlay?.setVisible(!idle && !!dash);
    if (idle) return;
    const signature = `${color}:${width}:${dash?.join(":")}`;
    if (signature === this.style) return;
    this.style = signature;
    this.graphics.clear();
    if (dash) {
      if (!this.underlay) {
        this.underlay = this.scene.add.graphics();
        this.root.addAt(this.underlay, 0);
      }
      this.underlay.clear();
      this.line(this.underlay, 7, 0x172015, dash);
    }
    this.underlay?.setVisible(!!dash);
    this.line(this.graphics, width, color, dash);
  }
  private line(
    graphics: Phaser.GameObjects.Graphics,
    width: number,
    color: number,
    dash?: readonly [number, number],
  ) {
    graphics.lineStyle(width, color, 1);
    if (!dash) {
      graphics.strokePoints(points, true);
      return;
    }
    for (let start = 0; start < perimeter; start += dash[0] + dash[1]) {
      const end = Math.min(start + dash[0], perimeter);
      let offset = 0,
        begun = false;
      graphics.beginPath();
      for (const { a, b, length } of edges) {
        const from = Math.max(start, offset),
          to = Math.min(end, offset + length);
        if (to > from) {
          const x = (at: number) => a.x + ((b.x - a.x) * (at - offset)) / length;
          const y = (at: number) => a.y + ((b.y - a.y) * (at - offset)) / length;
          if (!begun) graphics.moveTo(x(from), y(from));
          graphics.lineTo(x(to), y(to));
          begun = true;
        }
        offset += length;
      }
      graphics.strokePath();
    }
  }
}
