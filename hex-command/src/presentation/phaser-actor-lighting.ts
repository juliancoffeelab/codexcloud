import Phaser from "phaser";
import { BattlefieldVertexPipeline } from "./phaser-vertices";
import {
  changedLights,
  lightPosition,
  lightingKey,
  lightingMatrix,
  type BattlefieldLighting,
} from "./battlefield-lighting";

const identity = [1, 1, 1, 0, 0, 0, 0] as const;
type Anchor = { x: number; y: number };
type Sample = { x: number; y: number; epoch: number; values: readonly number[] };

/** Ground-anchored actor coefficients travel with each vertex, so different
 * anchors and artwork textures share Phaser's normal multi-texture batches. */
export class BattlefieldActorLightingPipeline extends BattlefieldVertexPipeline {
  private lighting!: BattlefieldLighting;
  private signature = "";
  private epoch = 0;
  private samples = new Map<Anchor, Sample>();
  private owners = new Map<Anchor, Set<Phaser.GameObjects.GameObject>>();
  private anchors = new WeakMap<Phaser.GameObjects.GameObject, Anchor>();
  private pending = new Set<Anchor>();
  private registered = new WeakSet<Phaser.GameObjects.GameObject>();
  private buckets = new Map<string, number[]>();
  constructor(game: Phaser.Game) {
    super({
      game,
      attributes: [
        { name: "inPosition", size: 2, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTexCoord", size: 2, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTexId", size: 1, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTintEffect", size: 1, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTint", size: 4, type: Phaser.Renderer.WebGL.UNSIGNED_BYTE, normalized: true },
        { name: "inFactor", size: 4, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inOffset", size: 3, type: Phaser.Renderer.WebGL.FLOAT },
      ],
      vertShader: `precision highp float;
      uniform mat4 uProjectionMatrix;
      attribute vec2 inPosition;attribute vec2 inTexCoord;
      attribute float inTexId;attribute float inTintEffect;attribute vec4 inTint;
      attribute vec4 inFactor;attribute vec3 inOffset;
      varying vec2 outTexCoord;varying float outTexId;varying float outTintEffect;
      varying vec4 outTint;varying vec4 factor;varying vec3 offset;
      void main(){gl_Position=uProjectionMatrix*vec4(inPosition,1.,1.);
      outTexCoord=inTexCoord;outTexId=inTexId;outTintEffect=inTintEffect;outTint=inTint;
      factor=inFactor;offset=inOffset;}`,
      fragShader: `precision highp float;
      uniform sampler2D uMainSampler[%count%];
      varying vec2 outTexCoord;varying float outTexId;varying float outTintEffect;
      varying vec4 outTint;varying vec4 factor;varying vec3 offset;
      void main(){vec4 texture;%forloop%
      vec4 ink=vec4(outTint.bgr*outTint.a,outTint.a);vec4 source=texture*ink;
      if(outTintEffect==1.)source.rgb=mix(texture.rgb,ink.rgb,texture.a);
      else if(outTintEffect==2.)source=ink;
      if(source.a>0.){vec3 rgb=source.rgb/source.a;
      float lum=dot(rgb,vec3(.2126,.7152,.0722));
      rgb=rgb*factor.rgb+lum*vec3(.28,.41,.63)*.2*factor.a+offset;
      source.rgb=clamp(rgb,0.,1.)*source.a;}gl_FragColor=source;}`,
    });
    this.values = identity;
  }
  configure(lighting: BattlefieldLighting) {
    if (lighting.time === "day") {
      if (this.lighting?.time !== "day") {
        this.epoch++;
        this.samples.clear();
        this.pending.clear();
        this.buckets.clear();
      }
      this.signature = "day";
      this.lighting = lighting;
      return;
    }
    const signature = lightingKey(lighting);
    if (signature === this.signature) return;
    this.signature = signature;
    const previous = this.lighting;
    this.lighting = lighting;
    this.epoch++;
    const bounds: { left: number; right: number; top: number; bottom: number }[] = [];
    for (const light of changedLights(previous?.lights ?? [], lighting.lights ?? [])) {
      const p = lightPosition(light),
        r = light.radius;
      bounds.push({ left: p.x - r, right: p.x + r, top: p.y - r / 1.12, bottom: p.y + r / 1.12 });
    }
    for (const [anchor, sample] of this.samples) {
      if (
        previous?.time !== lighting.time ||
        bounds.some(
          (b) =>
            sample.x >= b.left && sample.x <= b.right && sample.y >= b.top && sample.y <= b.bottom,
        )
      ) {
        this.samples.delete(anchor);
        this.pending.add(anchor);
      } else sample.epoch = this.epoch;
    }
    this.buckets.clear();
    for (const [index, light] of (lighting.lights ?? []).entries()) {
      const p = lightPosition(light),
        r = light.radius;
      for (let y = Math.floor((p.y - r / 1.12) / 128); y <= Math.floor((p.y + r / 1.12) / 128); y++)
        for (let x = Math.floor((p.x - r) / 128); x <= Math.floor((p.x + r) / 128); x++) {
          const key = `${x}:${y}`,
            bucket = this.buckets.get(key) ?? [];
          bucket.push(index);
          this.buckets.set(key, bucket);
        }
    }
  }
  prepare(object: Phaser.GameObjects.GameObject) {
    this.coefficients(object);
  }
  preparePending() {
    for (const anchor of this.pending) {
      const object = this.owners.get(anchor)?.values().next().value;
      if (object) this.coefficients(object);
    }
    this.pending.clear();
  }
  onBind(object?: Phaser.GameObjects.GameObject) {
    if (object) this.values = this.coefficients(object);
  }
  private coefficients(object: Phaser.GameObjects.GameObject): readonly number[] {
    const data = (object as Phaser.GameObjects.Image | undefined)?.pipelineData as
      | {
          unlit?: boolean;
          lightAnchor?: { x: number; y: number };
          lightPoint?: { x: number; y: number };
        }
      | undefined;
    const point = data?.lightAnchor ?? data?.lightPoint;
    if (!object || data?.unlit || !point || this.lighting.time === "day") {
      return identity;
    }
    if (this.anchors.get(object) !== point) {
      const previous = this.anchors.get(object),
        oldOwners = previous && this.owners.get(previous);
      oldOwners?.delete(object);
      if (previous && !oldOwners?.size) {
        this.owners.delete(previous);
        this.samples.delete(previous);
        this.pending.delete(previous);
      }
      const owners = this.owners.get(point) ?? new Set<Phaser.GameObjects.GameObject>();
      owners.add(object);
      this.owners.set(point, owners);
      this.anchors.set(object, point);
    }
    if (!this.registered.has(object)) {
      this.registered.add(object);
      object.once(Phaser.GameObjects.Events.DESTROY, () => {
        const anchor = this.anchors.get(object);
        if (!anchor) return;
        const owners = this.owners.get(anchor);
        owners?.delete(object);
        if (!owners?.size) {
          this.owners.delete(anchor);
          this.samples.delete(anchor);
          this.pending.delete(anchor);
        }
      });
    }
    const old = this.samples.get(point);
    if (old && old.x === point.x && old.y === point.y && old.epoch === this.epoch) {
      return old.values;
    }
    const local =
      this.buckets.get(`${Math.floor(point.x / 128)}:${Math.floor(point.y / 128)}`) ?? [];
    const lights = this.lighting.lights ?? [];
    const lighting = { ...this.lighting, lights: local.map((index) => lights[index]!) };
    const matrix = lightingMatrix(point.x, point.y, lighting);
    const nightMix = matrix[1]! / (0.2 * 0.7152 * 0.28);
    const values = [
      matrix[0]! - 0.2 * 0.2126 * 0.28 * nightMix,
      matrix[6]! - 0.2 * 0.7152 * 0.41 * nightMix,
      matrix[12]! - 0.2 * 0.0722 * 0.63 * nightMix,
      nightMix,
      matrix[4]!,
      matrix[9]!,
      matrix[14]!,
    ];
    this.samples.set(point, { x: point.x, y: point.y, epoch: this.epoch, values });
    this.pending.delete(point);
    return values;
  }
}
