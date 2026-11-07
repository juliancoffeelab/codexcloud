import Phaser from "phaser";
import { BattlefieldResources } from "./phaser-resources";
import { measureTrace } from "./performance-trace";
import { BattlefieldLightField, type LightCell } from "./phaser-light-field";
import { battlefieldSpriteSize } from "../ui/geometry";
import {
  lightPosition,
  lightingKey,
  lightingMatrix,
  type BattlefieldLighting,
} from "./battlefield-lighting";

const vertex = `
precision highp float;
uniform mat4 uProjectionMatrix;
uniform vec2 uWorldView;
uniform vec2 uZoom;
attribute vec2 inPosition;
attribute vec2 inTexCoord;
attribute vec4 inTint;
attribute float inTexId;
attribute float inTintEffect;
varying vec2 uv;
varying vec2 worldPoint;
varying vec4 tint;
varying vec2 mode;
void main() {
  gl_Position = uProjectionMatrix * vec4(inPosition, 1.0, 1.0);
  uv = inTexCoord;
  worldPoint = inPosition / uZoom + uWorldView;
  tint = inTint;
  mode = vec2(inTexId, inTintEffect);
}`;
const fragment = `
precision highp float;
uniform sampler2D uMainSampler;
uniform sampler2D uFactors;
uniform sampler2D uOffsets;
uniform vec2 uMapSize;
uniform vec2 uFieldOrigin;
uniform vec2 uFieldSize;
uniform float uNativeScale;
uniform float uNormalization;
uniform vec2 uPackedSize;uniform vec2 uPackedShift;
uniform float uDirect;uniform float uCount;
uniform vec4 uLights[%count%];uniform vec3 uWarm[%count%];uniform vec3 uOffsetsDirect[%count%];
uniform float uNight;
uniform float uBrightness;
uniform float uGround;
uniform vec3 uRowR;
uniform vec3 uRowG;
uniform vec3 uRowB;
uniform vec3 uOffset;
varying vec2 uv;
varying vec2 worldPoint;
varying vec4 tint;
varying vec2 mode;
void main() {
  // Consume Phaser's full SinglePipeline vertex layout and preserve its tint modes.
  // This pipeline owns one artwork sampler; every submitted texture ID is zero.
  if (mode.x != 0.0) discard;
  vec4 texture = texture2D(uMainSampler, uv);
  vec4 ink = vec4(tint.bgr * tint.a, tint.a);
  vec4 source = texture * ink;
  if (mode.y == 1.0) source.rgb = mix(texture.rgb, ink.rgb, texture.a);
  else if (mode.y == 2.0) source = ink;
  source.rgb *= uBrightness;
  if (source.a <= 0.0) { gl_FragColor = source; return; }
  if (uNight < .5) { gl_FragColor = source; return; }
  if (uGround > .5) {
    vec3 rgb = source.rgb / source.a;
    rgb = vec3(dot(rgb,uRowR),dot(rgb,uRowG),dot(rgb,uRowB)) + uOffset;
    gl_FragColor = vec4(clamp(rgb,0.0,1.0)*source.a,source.a);
    return;
  }
  vec2 cell = clamp(floor((worldPoint-uFieldOrigin)*uNativeScale),vec2(0.0),vec2(uFieldSize.x*.25-1.0,uFieldSize.y-1.0));
  float values[7];for(int i=0;i<7;i++)values[i]=0.;
  if(uDirect>.5){
    vec2 point=cell/uNativeScale+uFieldOrigin;
    for(int i=0;i<%count%;i++){
      if(float(i)>=uCount)break;
      vec2 delta=(point-uLights[i].xy)*vec2(1.,1.12);
      float d2=dot(delta,delta)/(uLights[i].z*uLights[i].z);
      if(d2>=1.)continue;
      float a=uLights[i].w*(1.-d2)*(1.-d2);
      values[0]+=a;
      values[1]+=a*uWarm[i].r/1.06;values[2]+=a*uWarm[i].g/1.06;values[3]+=a*uWarm[i].b/1.06;
      values[4]+=a*uOffsetsDirect[i].r;values[5]+=a*uOffsetsDirect[i].g;values[6]+=a*uOffsetsDirect[i].b;
    }
  }else{
  for(int i=0;i<4;i++){
    vec2 p=vec2((cell.x+uPackedShift.x)*4.+float(i)+.5,cell.y+uPackedShift.y+.5)/uPackedSize;
    values[i]=dot(texture2D(uFactors,p).rgb,vec3(65536.,256.,1.))*255./16777215.*uNormalization;
  }
  for(int i=0;i<3;i++){
    vec2 p=vec2((cell.x+uPackedShift.x)*4.+float(i)+.5,cell.y+uPackedShift.y+.5)/uPackedSize;
    values[4+i]=dot(texture2D(uOffsets,p).rgb,vec3(65536.,256.,1.))*255./16777215.*uNormalization*30.;
  }
  }
  float total=values[0];float mixAmount=min(1.,total)*.88;
  vec3 warm=total>0.?vec3(values[1],values[2],values[3])*1.06/total:vec3(1.02,.78,.49);
  vec3 warmOffset=total>0.?vec3(values[4],values[5],values[6])/total:vec3(22.,14.,7.);
  vec4 factor=vec4((.8*vec3(.28,.41,.63)*(1.-mixAmount)+warm*mixAmount)/2.,1.-mixAmount);
  vec3 offset=(vec3(4.,8.,17.)*(1.-mixAmount)+warmOffset*mixAmount)/255.;
  vec3 rgb = source.rgb / source.a;
  float lum = dot(rgb, vec3(.2126, .7152, .0722));
  rgb = rgb * factor.rgb * 2.0 + lum * vec3(.28, .41, .63) * .2 * factor.a + offset;
  gl_FragColor = vec4(clamp(rgb, 0.0, 1.0) * source.a, source.a);
}`;

/** Phaser owns compilation, texture units, buffers and batching for this pipeline. */
export class BattlefieldLightingPipeline extends Phaser.Renderer.WebGL.Pipelines.SinglePipeline {
  private signature = "";
  private factors?: Phaser.Textures.Texture;
  private offsets?: Phaser.Textures.Texture;
  private mapWidth = 1;
  private mapHeight = 1;
  private night = false;
  private originX = 0;
  private originY = 0;
  private lighting!: BattlefieldLighting;
  private binding = "";
  private field: BattlefieldLightField;
  private normalization = 1;
  private boundCell?: LightCell;
  get cells() {
    return this.field.cells;
  }

  constructor(
    game: Phaser.Game,
    private camera: Phaser.Cameras.Scene2D.Camera,
    resources: BattlefieldResources,
    scene: Phaser.Scene,
  ) {
    const renderer = game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    const count = Math.max(
      1,
      Math.min(
        32,
        Math.floor((renderer.gl.getParameter(renderer.gl.MAX_FRAGMENT_UNIFORM_VECTORS) - 20) / 3),
      ),
    );
    super({ game, vertShader: vertex, fragShader: fragment.replaceAll("%count%", String(count)) });
    this.field = new BattlefieldLightField(scene, resources);
    (game.renderer as Phaser.Renderer.WebGL.WebGLRenderer).pipelines.add(
      "BattlefieldLightField",
      this.field,
    );
  }

  configure(
    lighting: BattlefieldLighting,
    width: number,
    height: number,
    originX = 0,
    originY = 0,
  ) {
    const signature = `${originX}:${originY}:${width}:${height}:${lightingKey(lighting)}`;
    if (signature === this.signature) return;
    this.signature = signature;
    this.mapWidth = width;
    this.mapHeight = height;
    this.originX = originX;
    this.originY = originY;
    this.night = lighting.time !== "day";
    this.lighting = lighting;
    this.binding = "";
    this.flush();
    this.factors = this.offsets = undefined;
    if (!this.night) {
      this.field.release();
      return;
    }
    const field = measureTrace("Phaser lighting field", () =>
      this.field.build(lighting, width, height, originX, originY),
    );
    this.factors = field.factors;
    this.offsets = field.offsets;
    this.normalization = field.normalization;
  }

  onBind(object?: Phaser.GameObjects.GameObject) {
    if (!object) return;
    const data = ((object as Phaser.GameObjects.Image | undefined)?.pipelineData ?? {}) as {
      lightAnchor?: { x: number; y: number };
      lightPoint?: { x: number; y: number };
      unlit?: boolean;
      brightness?: number;
      lightingCamera?: Phaser.Cameras.Scene2D.BaseCamera;
      lightCell?: LightCell;
    };
    const point = data.lightAnchor ?? data.lightPoint;
    const binding = JSON.stringify([data.unlit, data.brightness, point?.x, point?.y]);
    if (binding !== this.binding || this.boundCell !== data.lightCell) {
      this.flush();
      this.binding = binding;
      this.boundCell = data.lightCell;
    }
    const direct = data.lightCell && data.lightCell.lights.length <= this.field.batchSize;
    this.set1f("uDirect", direct ? 1 : 0);
    if (data.lightCell && !direct) {
      const cell = data.lightCell;
      this.set2f("uPackedSize", ...this.field.packedSize);
      this.set2f("uPackedShift", cell.packedX! - cell.x, cell.packedY! - cell.y);
    }
    if (direct) {
      const lights = data.lightCell!.lights;
      const positions = new Float32Array(this.field.batchSize * 4);
      const warm = new Float32Array(this.field.batchSize * 3),
        offsets = new Float32Array(this.field.batchSize * 3);
      for (const [i, light] of lights.entries()) {
        const p = lightPosition(light);
        positions.set([p.x, p.y, light.radius, light.power], i * 4);
        warm.set(light.color?.map((c) => (c / 255) * 1.06) ?? [1.02, 0.78, 0.49], i * 3);
        offsets.set(light.color?.map((c) => (c / 255) * 30) ?? [22, 14, 7], i * 3);
      }
      this.set1f("uCount", lights.length);
      this.set4fv("uLights", positions);
      this.set3fv("uWarm", warm);
      this.set3fv("uOffsetsDirect", offsets);
    }
    const camera = data.lightingCamera ?? this.camera;
    const view = camera.getWorldPoint(0, 0);
    this.set2f("uWorldView", view.x, view.y);
    this.set2f("uZoom", camera.zoomX, camera.zoomY);
    this.set2f("uMapSize", this.mapWidth, this.mapHeight);
    this.set2f("uFieldOrigin", this.originX, this.originY);
    this.set2f(
      "uFieldSize",
      Math.round((this.mapWidth * 64) / battlefieldSpriteSize) * 4,
      Math.round((this.mapHeight * 64) / battlefieldSpriteSize),
    );
    this.set1f("uNativeScale", 64 / battlefieldSpriteSize);
    this.set1f("uNormalization", this.normalization);
    this.set1f("uNight", this.night && !data.unlit ? 1 : 0);
    this.set1f("uBrightness", data.brightness ?? 1);
    this.set1f("uGround", point ? 1 : 0);
    if (point && this.night) {
      const matrix = lightingMatrix(point.x, point.y, this.lighting);
      this.set3f("uRowR", matrix[0]!, matrix[1]!, matrix[2]!);
      this.set3f("uRowG", matrix[5]!, matrix[6]!, matrix[7]!);
      this.set3f("uRowB", matrix[10]!, matrix[11]!, matrix[12]!);
      this.set3f("uOffset", matrix[4]!, matrix[9]!, matrix[14]!);
    }
    this.set1i("uFactors", 1);
    this.set1i("uOffsets", 2);
    this.bindTexture(this.factors?.source[0]?.glTexture ?? this.renderer.whiteTexture, 1);
    this.bindTexture(this.offsets?.source[0]?.glTexture ?? this.renderer.whiteTexture, 2);
    // SinglePipeline flushes expect texture unit zero to remain active.
    this.bindTexture(this.activeTextures[0] ?? this.renderer.whiteTexture, 0);
  }

  destroy() {
    this.release();
    this.renderer.pipelines.remove("BattlefieldLightField");
    this.field.destroy();
    return super.destroy();
  }

  release() {
    this.signature = "";
    this.binding = "";
    this.field.release();
    this.factors = undefined;
    this.offsets = undefined;
  }
}
