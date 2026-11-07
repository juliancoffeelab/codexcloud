import Phaser from "phaser";
import { BattlefieldVertexPipeline } from "./phaser-vertices";

// Standard Phaser sprite attributes, including tint modes and its texture ID.
// Keeping the full layout active also lets Phaser release every shader attribute.
const vertex = `
precision highp float;
uniform mat4 uProjectionMatrix;
attribute vec2 inPosition;
attribute vec2 inTexCoord;
attribute float inTexId;
attribute float inTintEffect;
attribute vec4 inTint;
varying vec2 uv;
varying vec2 mode;
varying vec4 tint;
void main(){
 gl_Position=uProjectionMatrix*vec4(inPosition,1.0,1.0);
 uv=inTexCoord;mode=vec2(inTexId,inTintEffect);tint=inTint;
}`;
const varyings = `
precision highp float;
uniform sampler2D uMainSampler;
varying vec2 uv;
varying vec2 mode;
varying vec4 tint;
`;
const finish = `
 if(mode.x!=0.0)discard;
 color*=vec4(tint.bgr*tint.a,tint.a);
 if(mode.y==2.0)color=vec4(tint.bgr*tint.a,tint.a);
 gl_FragColor=color;
`;

/** Authored lamp halos, sampled into Phaser's native background render texture. */
export class BattlefieldHaloPipeline extends Phaser.Renderer.WebGL.Pipelines.SinglePipeline {
  private binding = "";
  constructor(game: Phaser.Game) {
    super({
      game,
      vertShader: vertex,
      fragShader:
        varyings +
        `
 uniform float uFixture;
 uniform vec3 uGlow;
 void main(){
  float distance=length(uv*2.0-1.0);
  if(distance>=1.0)discard;
  vec3 rgb;float alpha;
  if(uFixture>.5){
   float t=min(distance/.25,1.0);
   rgb=mix(vec3(255.0,204.0,110.0),vec3(255.0,169.0,65.0),t)/255.0;
   alpha=distance<.25?mix(.65,.24,t):.24*(1.0-distance)/.75;
  }else{
   rgb=uGlow;
   alpha=distance<.25?mix(.95,.45,distance/.25):.45*(1.0-distance)/.75;
  }
  vec4 color=vec4(rgb*alpha,alpha);
 ` +
        finish +
        `}`,
    });
  }
  onBind(object?: Phaser.GameObjects.GameObject) {
    const data = (object as Phaser.GameObjects.Image).pipelineData as {
      fixture: boolean;
      color: readonly number[];
    };
    const binding = JSON.stringify(data);
    if (binding !== this.binding) {
      this.flush();
      this.binding = binding;
    }
    this.set1f("uFixture", data.fixture ? 1 : 0);
    this.set3f("uGlow", data.color[0]! / 255, data.color[1]! / 255, data.color[2]! / 255);
  }
}

/** Same separable Gaussian kernel and half-native mask as the upstream scene. */
export class BattlefieldFogBlurPipeline extends Phaser.Renderer.WebGL.Pipelines.SinglePipeline {
  private binding = "";
  private weights: number[];
  private offsets: number[];
  constructor(game: Phaser.Game) {
    super({
      game,
      vertShader: vertex,
      fragShader:
        varyings +
        `
 uniform vec2 uStep;
 uniform vec4 uRegion;uniform vec2 uExtent;uniform vec2 uCapture;
 bool inside(vec2 p){vec2 global=uRegion.xy+p*uRegion.zw;return all(greaterThanEqual(global,vec2(-.0001)))&&all(lessThanEqual(global,uExtent+vec2(.0001)));}
 float sampleMask(vec2 p){vec2 global=clamp(uRegion.xy+p*uRegion.zw,vec2(.5),uExtent-vec2(.5));return texture2D(uMainSampler,(global-uRegion.xy)/uRegion.zw).r;}
 uniform float uWeights[7];uniform float uOffsets[7];
 void main(){
  // Preserve the legacy odd-extent camera translation and its top raster edge.
  vec2 outputPoint=uRegion.xy+uv*uRegion.zw+uCapture;
  if((uCapture.y>.25&&outputPoint.y<1.)||any(lessThan(outputPoint,vec2(0.)))||any(greaterThan(outputPoint,uExtent))){gl_FragColor=vec4(0.,0.,0.,1.);return;}
  float value=sampleMask(uv)*uWeights[0];
  for(int i=1;i<7;i++){
   vec2 delta=uStep*uOffsets[i];vec2 a=uv-delta,b=uv+delta;
   if(inside(a)&&all(greaterThanEqual(a,vec2(0.0)))&&all(lessThanEqual(a,vec2(1.0))))value+=sampleMask(a)*uWeights[i];
   if(inside(b)&&all(greaterThanEqual(b,vec2(0.0)))&&all(lessThanEqual(b,vec2(1.0))))value+=sampleMask(b)*uWeights[i];
  }
  vec4 color=vec4(value,0.0,0.0,1.0);
 ` +
        finish +
        `}`,
    });
    const kernel = Array.from({ length: 13 }, (_, i) => Math.exp((-i * i) / 32));
    const sum = kernel[0]! + 2 * kernel.slice(1).reduce((a, b) => a + b, 0);
    this.weights = [kernel[0]! / sum];
    this.offsets = [0];
    for (let i = 1; i < 13; i += 2) {
      const weight = kernel[i]! + kernel[i + 1]!;
      this.weights.push(weight / sum);
      this.offsets.push(i + kernel[i + 1]! / weight);
    }
  }
  onBind(object?: Phaser.GameObjects.GameObject) {
    const data = (object as Phaser.GameObjects.Image).pipelineData as {
      x: number;
      y: number;
      region: readonly number[];
      extent: readonly number[];
      capture?: readonly number[];
    };
    const binding = JSON.stringify(data);
    if (binding !== this.binding) {
      this.flush();
      this.binding = binding;
    }
    this.set2f("uStep", data.x, data.y);
    this.set4f("uRegion", data.region[0]!, data.region[1]!, data.region[2]!, data.region[3]!);
    this.set2f("uExtent", data.extent[0]!, data.extent[1]!);
    this.set2f("uCapture", data.capture?.[0] ?? 0, data.capture?.[1] ?? 0);
    this.set1fv("uWeights", this.weights);
    this.set1fv("uOffsets", this.offsets);
  }
}

export class BattlefieldFogCompositePipeline
  extends Phaser.Renderer.WebGL.Pipelines.SinglePipeline
{
  constructor(game: Phaser.Game) {
    super({
      game,
      vertShader: vertex,
      fragShader:
        varyings +
        `
 void main(){float cover=texture2D(uMainSampler,uv).r;
 vec4 color=vec4(vec3(12.0,19.0,32.0)/255.0*cover,cover);
 ` +
        finish +
        `}`,
    });
  }
}

/** Analytic world-space hex strokes, rendered as ordinary Phaser Images. */
export class BattlefieldHexPipeline extends BattlefieldVertexPipeline {
  private styles = new WeakMap<
    Phaser.GameObjects.GameObject,
    { width: number; dash0: number; dash1: number; values: readonly number[] }
  >();
  constructor(game: Phaser.Game) {
    (game.renderer as Phaser.Renderer.WebGL.WebGLRenderer).getExtension("OES_standard_derivatives");
    super({
      game,
      vertShader: vertex.replace(
        "void main(){",
        "attribute vec3 inStyle;varying vec3 style;void main(){style=inStyle;",
      ),
      attributes: [
        { name: "inPosition", size: 2, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTexCoord", size: 2, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTexId", size: 1, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTintEffect", size: 1, type: Phaser.Renderer.WebGL.FLOAT },
        { name: "inTint", size: 4, type: Phaser.Renderer.WebGL.UNSIGNED_BYTE, normalized: true },
        { name: "inStyle", size: 3, type: Phaser.Renderer.WebGL.FLOAT },
      ],
      fragShader:
        `#extension GL_OES_standard_derivatives : enable\n` +
        varyings.replace(
          "uniform sampler2D uMainSampler;",
          "uniform sampler2D uMainSampler[%count%];",
        ) +
        `
 varying vec3 style;
 #define uHalfWidth style.x
 #define uDash style.yz
 void main(){
  vec2 point=(uv-.5)*vec2(84.0,90.0);
  float distance=-10000.0,along=0.0,walk=0.0;
  for(int i=0;i<6;i++){
   float angle=(float(i)*60.0-30.0)*.017453292519943295;
   float next=angle+1.0471975511965976;
   vec2 a=vec2(cos(angle)*35.36270398809583,sin(angle)*35.0);
   vec2 b=vec2(cos(next)*35.36270398809583,sin(next)*35.0);
   vec2 edge=b-a;float length=length(edge);vec2 tangent=edge/length;
   float d=dot(point-a,vec2(tangent.y,-tangent.x));
   if(d>distance){distance=d;along=walk+dot(point-a,tangent);}
   walk+=length;
  }
  float coverage=clamp(.5+(uHalfWidth-abs(distance))/max(fwidth(distance),.00001),0.0,1.0);
  if(uDash.x>0.0){
   float phase=mod(mod(along+walk,walk),uDash.x+uDash.y);
   float fringe=max(fwidth(along),.00001);
   coverage*=clamp(.5+phase/fringe,0.0,1.0)*clamp(.5+(uDash.x-phase)/fringe,0.0,1.0);
  }
  vec4 color=vec4(coverage);
 ` +
        finish +
        `}`,
    });
  }
  onBind(object?: Phaser.GameObjects.GameObject) {
    if (!object) return;
    const data = (object as Phaser.GameObjects.Image).pipelineData as {
      width: number;
      dash?: readonly [number, number];
    };
    const dash0 = data.dash?.[0] ?? 0,
      dash1 = data.dash?.[1] ?? 0,
      old = this.styles.get(object);
    if (old && old.width === data.width && old.dash0 === dash0 && old.dash1 === dash1) {
      this.values = old.values;
      return;
    }
    this.values = [data.width / 2, dash0, dash1];
    this.styles.set(object, { width: data.width, dash0, dash1, values: this.values });
  }
}

/** Keep mask rasterization on the original world lattice before Gaussian blur. */
export class BattlefieldFogMaskPipeline extends Phaser.Renderer.WebGL.Pipelines.SinglePipeline {
  constructor(game: Phaser.Game) {
    super({
      game,
      vertShader: vertex
        .replace(
          "uniform mat4 uProjectionMatrix;",
          "uniform vec2 uWorldSize;uniform vec2 uWorldOrigin;varying vec2 world;",
        )
        .replace(
          "gl_Position=uProjectionMatrix*vec4(inPosition,1.0,1.0);",
          "world=inPosition;gl_Position=vec4((inPosition-uWorldOrigin)/uWorldSize*vec2(2.0,-2.0)+vec2(-1.0,1.0),0.0,1.0);",
        ),
      fragShader:
        varyings +
        `
 uniform vec2 uExtent;varying vec2 world;
 void main(){if(any(lessThan(world,vec2(0.)))||any(greaterThan(world,uExtent)))discard;
 vec4 color=texture2D(uMainSampler,uv);
 ` +
        finish +
        `}`,
    });
  }
  onBind(object?: Phaser.GameObjects.GameObject) {
    if (!object) return;
    const data = (object as Phaser.GameObjects.Graphics).pipelineData as {
      width: number;
      height: number;
      originX: number;
      originY: number;
      extent: readonly number[];
    };
    this.set2f("uWorldSize", data.width, data.height);
    this.set2f("uWorldOrigin", data.originX, data.originY);
    this.set2f("uExtent", data.extent[0]!, data.extent[1]!);
  }
}
