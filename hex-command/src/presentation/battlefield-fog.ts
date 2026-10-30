/** Atmospheric cover only; Rust supplies the authoritative set of fogged tiles. */
import { hexPoints, tileCenter } from "../ui/geometry";
interface FogContext {
  save(): void;
  restore(): void;
  beginPath(): void;
  closePath(): void;
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  fill(): void;
  fillStyle: string | CanvasGradient | CanvasPattern;
  filter: string;
}
export function paintFog(context: FogContext, tiles: readonly { col: number; row: number }[]) {
  context.save();
  context.filter = "blur(8px)";
  context.fillStyle = "#0c1320";
  context.beginPath();
  for (const tile of tiles) {
    const center = tileCenter(tile);
    hexPoints(center.x, center.y)
      .split(" ")
      .forEach((point, index) => {
        const [x, y] = point.split(",").map(Number);
        if (index === 0) context.moveTo(x!, y!);
        else context.lineTo(x!, y!);
      });
    context.closePath();
  }
  context.fill();
  context.restore();
}
