import Phaser from "phaser";

/** Phaser 3.90 vertex-layout seam. Native MultiPipeline retains transforms,
 * texture slots, tint packing, capacity checks and onBatch bookkeeping. */
export class BattlefieldVertexPipeline extends Phaser.Renderer.WebGL.Pipelines.MultiPipeline {
  protected values: readonly number[] = [];
  batchVert(...args: Parameters<Phaser.Renderer.WebGL.WebGLPipeline["batchVert"]>) {
    const start = this.vertexCount;
    super.batchVert(...args);
    this.writeValues(start, 1);
  }
  private writeValues(start: number, count: number) {
    const stride = this.currentShader.vertexComponentCount;
    for (let vertex = start; vertex < start + count; vertex++) {
      let offset = vertex * stride + 7;
      for (const value of this.values) this.vertexViewF32[offset++] = value;
    }
  }
  batchQuad(...args: Parameters<Phaser.Renderer.WebGL.WebGLPipeline["batchQuad"]>) {
    let [
      gameObject,
      x0,
      y0,
      x1,
      y1,
      x2,
      y2,
      x3,
      y3,
      u0,
      v0,
      u1,
      v1,
      tintTL,
      tintTR,
      tintBL,
      tintBR,
      tintEffect,
      texture,
      unit,
    ] = args;

    if (unit === undefined) {
      unit = this.currentUnit;
    }

    let hasFlushed = false;

    if (this.shouldFlush(6)) {
      this.flush();

      hasFlushed = true;
    }

    if (!this.currentBatch) {
      unit = this.setTexture2D(texture);
    }

    let vertexViewF32 = this.vertexViewF32;
    let vertexViewU32 = this.vertexViewU32;

    let vertexOffset = this.vertexCount * this.currentShader.vertexComponentCount - 1;

    vertexViewF32[++vertexOffset] = x0;
    vertexViewF32[++vertexOffset] = y0;
    vertexViewF32[++vertexOffset] = u0;
    vertexViewF32[++vertexOffset] = v0;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintTL;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    vertexViewF32[++vertexOffset] = x1;
    vertexViewF32[++vertexOffset] = y1;
    vertexViewF32[++vertexOffset] = u0;
    vertexViewF32[++vertexOffset] = v1;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintBL;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    vertexViewF32[++vertexOffset] = x2;
    vertexViewF32[++vertexOffset] = y2;
    vertexViewF32[++vertexOffset] = u1;
    vertexViewF32[++vertexOffset] = v1;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintBR;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    vertexViewF32[++vertexOffset] = x0;
    vertexViewF32[++vertexOffset] = y0;
    vertexViewF32[++vertexOffset] = u0;
    vertexViewF32[++vertexOffset] = v0;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintTL;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    vertexViewF32[++vertexOffset] = x2;
    vertexViewF32[++vertexOffset] = y2;
    vertexViewF32[++vertexOffset] = u1;
    vertexViewF32[++vertexOffset] = v1;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintBR;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    vertexViewF32[++vertexOffset] = x3;
    vertexViewF32[++vertexOffset] = y3;
    vertexViewF32[++vertexOffset] = u1;
    vertexViewF32[++vertexOffset] = v0;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintTR;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    this.vertexCount += 6;

    this.currentBatch!.count = this.vertexCount - this.currentBatch!.start;

    this.onBatch(gameObject ?? undefined);

    return hasFlushed;
  }
  batchTri(...args: Parameters<Phaser.Renderer.WebGL.WebGLPipeline["batchTri"]>) {
    let [
      gameObject,
      x0,
      y0,
      x1,
      y1,
      x2,
      y2,
      u0,
      v0,
      u1,
      v1,
      tintTL,
      tintTR,
      tintBL,
      tintEffect,
      texture,
      unit,
    ] = args;

    if (unit === undefined) {
      unit = this.currentUnit;
    }

    let hasFlushed = false;

    if (this.shouldFlush(3)) {
      this.flush();

      hasFlushed = true;
    }

    if (!this.currentBatch) {
      unit = this.setTexture2D(texture);
    }

    let vertexViewF32 = this.vertexViewF32;
    let vertexViewU32 = this.vertexViewU32;

    let vertexOffset = this.vertexCount * this.currentShader.vertexComponentCount - 1;

    vertexViewF32[++vertexOffset] = x0;
    vertexViewF32[++vertexOffset] = y0;
    vertexViewF32[++vertexOffset] = u0;
    vertexViewF32[++vertexOffset] = v0;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintTL;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    vertexViewF32[++vertexOffset] = x1;
    vertexViewF32[++vertexOffset] = y1;
    vertexViewF32[++vertexOffset] = u0;
    vertexViewF32[++vertexOffset] = v1;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintTR;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    vertexViewF32[++vertexOffset] = x2;
    vertexViewF32[++vertexOffset] = y2;
    vertexViewF32[++vertexOffset] = u1;
    vertexViewF32[++vertexOffset] = v1;
    vertexViewF32[++vertexOffset] = unit;
    vertexViewF32[++vertexOffset] = Number(tintEffect);
    vertexViewU32[++vertexOffset] = tintBL;
    for (const value of this.values) vertexViewF32[++vertexOffset] = value;

    this.vertexCount += 3;

    this.currentBatch!.count = this.vertexCount - this.currentBatch!.start;

    this.onBatch(gameObject ?? undefined);

    return hasFlushed;
  }
}
