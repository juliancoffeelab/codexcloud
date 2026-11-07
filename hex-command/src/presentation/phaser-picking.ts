import Phaser from "phaser";
import type { PhaserBattlefieldFrame } from "./phaser-battlefield";
import { battlefieldSpriteSize, hexColumnSpacing, hexPoints, tileCenter } from "../ui/geometry";

type Cell = { id: string; x: number; y: number; order: number };

/** Rust board topology is fixed for a session/map revision. Phaser's staggered
 * hex conversion narrows picking to a constant-size neighborhood. The original
 * polygon test and tile order retain edge/vertex selection semantics. */
export class BattlefieldPicking {
  private identity = "";
  private rows = new Map<number, Map<number, Cell>>();
  private coordinate = new Phaser.Math.Vector2();
  private layer = new Phaser.Tilemaps.LayerData({
    baseTileWidth: hexColumnSpacing,
    baseTileHeight: battlefieldSpriteSize,
  });
  private origin = tileCenter({ col: 0, row: 0 });
  private polygon = new Phaser.Geom.Polygon(
    hexPoints(0, 0)
      .split(" ")
      .map((point) => {
        const [x, y] = point.split(",").map(Number);
        return { x: x!, y: y! };
      }),
  );
  private offsets = this.polygon.points.map(({ x, y }) => ({ x, y }));

  constructor(private camera: Phaser.Cameras.Scene2D.Camera) {
    this.layer.staggerAxis = "y";
    this.layer.staggerIndex = "odd";
  }

  prepare(frame: PhaserBattlefieldFrame) {
    const { game, sessionVersion } = frame.view;
    const identity = `${sessionVersion}:${game.mapId}:${game.mapRevision}:${game.cols}:${game.rows}:${frame.tiles.length}`;
    if (identity === this.identity) return;
    this.clear();
    for (const [order, tile] of frame.tiles.entries()) {
      let row = this.rows.get(tile.row);
      if (!row) this.rows.set(tile.row, (row = new Map()));
      const old = row.get(tile.col);
      if (!old || order < old.order)
        row.set(tile.col, { id: tile.id, x: tile.x, y: tile.y, order });
    }
    this.identity = identity;
  }
  pick(frame: PhaserBattlefieldFrame, x: number, y: number): string | undefined {
    this.prepare(frame);
    Phaser.Tilemaps.Components.HexagonalWorldToTileXY(
      x - this.origin.x + hexColumnSpacing / 2,
      y - this.origin.y + battlefieldSpriteSize / 2,
      true,
      this.coordinate,
      this.camera,
      this.layer,
    );
    let best: Cell | undefined;
    for (let dy = -1; dy <= 1; dy++)
      for (let dx = -1; dx <= 1; dx++) {
        const cell = this.rows.get(this.coordinate.y + dy)?.get(this.coordinate.x + dx);
        if (!cell || (best && best.order < cell.order)) continue;
        for (let i = 0; i < this.offsets.length; i++) {
          this.polygon.points[i]!.x = cell.x + this.offsets[i]!.x;
          this.polygon.points[i]!.y = cell.y + this.offsets[i]!.y;
        }
        if (Phaser.Geom.Polygon.Contains(this.polygon, x, y)) best = cell;
      }
    return best?.id;
  }
  clear() {
    this.rows.clear();
    this.identity = "";
  }
}
