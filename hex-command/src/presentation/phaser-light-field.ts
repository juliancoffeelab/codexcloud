import Phaser from "phaser";
import { battlefieldSpriteSize } from "../ui/geometry";
import {
  lightPosition,
  type BattlefieldLighting,
  type BattlefieldLight,
} from "./battlefield-lighting";
import { BattlefieldResources } from "./phaser-resources";
import { measureTrace, traceInstant } from "./performance-trace";

const scale = 64 / battlefieldSpriteSize;
const cellSize = 128;
const defaultWarm = [1.02, 0.78, 0.49] as const,
  defaultOffset = [22, 14, 7] as const;
export type LightCell = {
  x: number;
  y: number;
  width: number;
  height: number;
  lights: BattlefieldLight[];
  packedX?: number;
  packedY?: number;
};
type FieldData = { cell: LightCell; originX: number; originY: number; pass: number; kind: number };
const vertex = `precision highp float;
uniform mat4 uProjectionMatrix;
attribute vec2 inPosition;attribute vec2 inTexCoord;attribute vec4 inTint;
attribute float inTexId;attribute float inTintEffect;
varying vec2 uv;varying vec4 ink;varying vec2 mode;
void main(){gl_Position=uProjectionMatrix*vec4(inPosition,1.,1.);uv=inTexCoord;ink=inTint;mode=vec2(inTexId,inTintEffect);}`;

/** Bounded native RGBA8 targets. Each scalar occupies one opaque RGB24 texel;
 * alpha is always one, so Phaser's ordinary premultiplied blending preserves data.
 * Four horizontal texels hold total strength/weighted RGB or weighted offsets.
 * No float-target extension, CPU pixel loop, readback or external GL resource. */
export class BattlefieldLightField extends Phaser.Renderer.WebGL.Pipelines.SinglePipeline {
  private targets: Phaser.GameObjects.RenderTexture[][] = [];
  private stamp?: Phaser.GameObjects.Image;
  private positions: Float32Array;
  private warm: Float32Array;
  private offsets: Float32Array;
  private fieldWidth = 0;
  private fieldHeight = 0;
  cells: LightCell[] = [];
  get packedSize() {
    return [this.fieldWidth * 4, this.fieldHeight] as const;
  }
  private previous?: Phaser.GameObjects.RenderTexture[];
  private normalization = 1;
  readonly batchSize: number;
  private readonly sceneOwner: Phaser.Scene;
  constructor(
    scene: Phaser.Scene,
    private resources: BattlefieldResources,
  ) {
    const renderer = scene.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    const batchSize = Math.max(
      1,
      Math.min(
        32,
        Math.floor((renderer.gl.getParameter(renderer.gl.MAX_FRAGMENT_UNIFORM_VECTORS) - 20) / 3),
      ),
    );
    super({
      game: scene.game,
      vertShader: vertex,
      fragShader: `precision highp float;
      uniform sampler2D uMainSampler;uniform sampler2D uPrevious;
      uniform vec2 uSize;uniform vec4 uCell;uniform vec2 uOrigin;uniform vec2 uAtlas;
      uniform float uCount;uniform float uKind;uniform float uHasPrevious;uniform float uNormalization;
      uniform vec4 uLights[${batchSize}];uniform vec3 uWarm[${batchSize}];uniform vec3 uOffset[${batchSize}];
      varying vec2 uv;varying vec4 ink;varying vec2 mode;
      float unpackValue(vec4 v){return dot(v.rgb,vec3(65536.,256.,1.))*255./16777215.;}
      vec4 packValue(float v){float n=min(16777215.,floor(clamp(v,0.,1.)*16777215.+.5));return vec4(floor(mod(n/vec3(65536.,256.,1.),256.))/255.,1.);}
      void main(){
        if(mode.x!=0.||mode.y!=0.||ink.a==0.)discard;
        vec2 local=floor(uv*vec2(uCell.z*4.,uCell.w));
        float slot=mod(local.x,4.);vec2 localPixel=vec2(floor(local.x/4.),local.y);vec2 pixel=localPixel+uCell.xy;vec2 packedPixel=localPixel+uAtlas;
        float value=0.;
        if(uHasPrevious>.5)value=unpackValue(texture2D(uPrevious,vec2((packedPixel.x*4.+slot+.5)/(uSize.x*4.),(packedPixel.y+.5)/uSize.y)));
        vec2 point=pixel/${scale.toPrecision(17)}+uOrigin;
        for(int i=0;i<${batchSize};i++){
          if(float(i)>=uCount)break;
          vec2 delta=(point-uLights[i].xy)*vec2(1.,1.12);
          float d2=dot(delta,delta)/(uLights[i].z*uLights[i].z);
          if(d2>=1.)continue;
          float amount=uLights[i].w*(1.-d2)*(1.-d2)/uNormalization;
          if(uKind<.5){
            if(slot<.5)value+=amount;
            else if(slot<1.5)value+=amount*uWarm[i].r/1.06;
            else if(slot<2.5)value+=amount*uWarm[i].g/1.06;
            else value+=amount*uWarm[i].b/1.06;
          }else{
            if(slot<.5)value+=amount*uOffset[i].r/30.;
            else if(slot<1.5)value+=amount*uOffset[i].g/30.;
            else if(slot<2.5)value+=amount*uOffset[i].b/30.;
          }
        }
        gl_FragColor=packValue(value);
      }`,
    });
    this.batchSize = batchSize;
    this.positions = new Float32Array(batchSize * 4);
    this.warm = new Float32Array(batchSize * 3);
    this.offsets = new Float32Array(batchSize * 3);
    this.sceneOwner = scene;
  }
  build(
    lighting: BattlefieldLighting,
    width: number,
    height: number,
    originX: number,
    originY: number,
  ) {
    const cols = Math.round(width * scale),
      rows = Math.round(height * scale);

    const lights = lighting.lights ?? [];
    this.normalization = Math.max(
      1,
      lights.reduce((sum, light) => sum + light.power, 0),
    );
    const cells: LightCell[] = [];
    for (let y = 0; y < rows; y += cellSize)
      for (let x = 0; x < cols; x += cellSize) {
        const width = Math.min(cellSize, cols - x),
          height = Math.min(cellSize, rows - y);
        const local = lights.filter((light) => {
          const p = lightPosition(light),
            r = light.radius;
          return (
            (p.x + r - originX) * scale > x &&
            (p.x - r - originX) * scale < x + width &&
            (p.y + r / 1.12 - originY) * scale > y &&
            (p.y - r / 1.12 - originY) * scale < y + height
          );
        });
        cells.push({ x, y, width, height, lights: local });
      }
    this.cells = cells;
    const crowded = cells.filter((cell) => cell.lights.length > this.batchSize);
    traceInstant("Battlefield GPU lighting workload", {
      lights: lights.length,
      cells: cells.length,
      crowdedCells: crowded.length,
      lightCellReferences: cells.reduce((sum, cell) => sum + cell.lights.length, 0),
      batchSize: this.batchSize,
    });
    if (!crowded.length)
      return { factors: undefined, offsets: undefined, normalization: this.normalization };
    const across = Math.min(
      Math.ceil(Math.sqrt(crowded.length)),
      Math.max(1, Math.floor(this.resources.target / (cellSize * 4))),
    );
    const packedWidth = across * cellSize,
      packedHeight = Math.ceil(crowded.length / across) * cellSize;
    this.resources.check("packed GPU lighting scratch", packedWidth * 4, packedHeight, true);
    for (const [index, cell] of crowded.entries()) {
      cell.packedX = (index % across) * cellSize;
      cell.packedY = Math.floor(index / across) * cellSize;
    }
    if (packedWidth !== this.fieldWidth || packedHeight !== this.fieldHeight) {
      this.release();
      this.fieldWidth = packedWidth;
      this.fieldHeight = packedHeight;
      this.targets = Array.from({ length: 2 }, () =>
        Array.from({ length: 2 }, () => {
          const target = new Phaser.GameObjects.RenderTexture(
            this.sceneOwner,
            0,
            0,
            packedWidth * 4,
            packedHeight,
            false,
          ).setOrigin(0);
          target.camera.setOrigin(0, 0).setScroll(0, 0);
          target.texture.setFilter(Phaser.Textures.FilterMode.NEAREST);
          return target;
        }),
      );
      this.resources.set(
        "light-gpu-scratch",
        "GPU lighting scratch",
        0,
        packedWidth * packedHeight * 64,
      );
    }
    if (!this.stamp) {
      this.stamp = new Phaser.GameObjects.Image(this.sceneOwner, 0, 0, "battlefield-white")
        .setOrigin(0)
        .setPipeline(
          "BattlefieldLightField",
          { cell: crowded[0]!, originX, originY, pass: 0, kind: 0 } satisfies FieldData,
          false,
        );
    }
    const data = this.stamp.pipelineData as FieldData;
    data.originX = originX;
    data.originY = originY;
    const passes = Math.max(
      1,
      ...crowded.map((cell) => Math.ceil(cell.lights.length / this.batchSize)),
    );
    let output = this.targets[0]!;
    measureTrace("Phaser GPU lighting fields", () => {
      for (let pass = 0; pass < passes; pass++) {
        output = this.targets[pass % 2]!;
        this.previous = pass ? this.targets[(pass - 1) % 2] : undefined;
        for (let kind = 0; kind < 2; kind++) {
          const target = output[kind]!;
          try {
            target.clear().beginDraw();
            data.pass = pass;
            data.kind = kind;
            for (const cell of crowded) {
              data.cell = cell;
              this.stamp!.setPosition(cell.packedX! * 4, cell.packedY!).setDisplaySize(
                cell.width * 4,
                cell.height,
              );
              target.batchDraw(this.stamp!);
            }
          } finally {
            target.endDraw();
          }
        }
      }
    });
    this.previous = undefined;
    return {
      factors: output[0]!.texture,
      offsets: output[1]!.texture,
      normalization: this.normalization,
    };
  }
  onBind(object?: Phaser.GameObjects.GameObject) {
    if (!object) return;
    this.flush();
    const { cell, originX, originY, pass, kind } = (object as Phaser.GameObjects.Image)
      .pipelineData as FieldData;
    const start = pass * this.batchSize;
    const count = Math.max(0, Math.min(this.batchSize, cell.lights.length - start));
    const positions = this.positions,
      warm = this.warm,
      offset = this.offsets;
    for (let i = 0; i < count; i++) {
      const light = cell.lights[start + i]!,
        p = lightPosition(light);
      positions[i * 4] = p.x;
      positions[i * 4 + 1] = p.y;
      positions[i * 4 + 2] = light.radius;
      positions[i * 4 + 3] = light.power;
      for (let channel = 0; channel < 3; channel++) {
        warm[i * 3 + channel] = light.color
          ? (light.color[channel]! / 255) * 1.06
          : defaultWarm[channel]!;
        offset[i * 3 + channel] = light.color
          ? (light.color[channel]! / 255) * 30
          : defaultOffset[channel]!;
      }
    }
    this.set2f("uSize", this.fieldWidth, this.fieldHeight);
    this.set4f("uCell", cell.x, cell.y, cell.width, cell.height);
    this.set2f("uOrigin", originX, originY);
    this.set2f("uAtlas", cell.packedX!, cell.packedY!);
    this.set1f("uCount", count);
    this.set1f("uKind", kind);
    this.set1f("uNormalization", this.normalization);
    this.set1f("uHasPrevious", this.previous ? 1 : 0);
    this.set4fv("uLights", positions);
    this.set3fv("uWarm", warm);
    this.set3fv("uOffset", offset);
    this.set1i("uPrevious", 1);
    this.bindTexture(
      this.previous?.[kind]?.texture.source[0]?.glTexture ?? this.renderer.whiteTexture,
      1,
    );
    this.bindTexture(this.activeTextures[0] ?? this.renderer.whiteTexture, 0);
  }
  release() {
    this.stamp?.destroy();
    this.stamp = undefined;
    for (const pair of this.targets) for (const target of pair) target.destroy();
    this.targets = [];
    this.fieldWidth = this.fieldHeight = 0;
    this.resources.remove("light-gpu-scratch");
  }
}
