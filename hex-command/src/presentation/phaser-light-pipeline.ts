import type Phaser from "phaser";
import { lightSampling } from "./map-shaders";
import type { MapRendererGpu } from "./map-renderer-gpu";
import type { BattlefieldLighting } from "./battlefield-lighting";

/** Phaser's batched sprite layout with the existing spatial lighting function. */
export function lightPipeline(
  P: typeof Phaser,
  game: Phaser.Game,
  terrain: MapRendererGpu,
  lighting: () => BattlefieldLighting,
) {
  const vertShader = `#version 300 es
precision highp float;
uniform mat4 uProjectionMatrix;
in vec2 inPosition; in vec2 inTexCoord;
in float inTexId; in float inTintEffect; in vec4 inTint;
out vec2 outTexCoord; out float outTexId; out vec4 outTint;
void main(){gl_Position=uProjectionMatrix*vec4(inPosition,1.,1.);outTexCoord=inTexCoord;outTexId=inTexId;outTint=inTint;}`;
  const fragShader = `#version 300 es
precision highp float;
uniform sampler2D uMainSampler[8];
uniform vec2 displaySize; uniform vec2 worldSize;
in vec2 outTexCoord; in float outTexId; in vec4 outTint;
${lightSampling}
out vec4 color;
void main(){
 vec4 source=vec4(0.);
 ${Array.from({ length: 8 }, (_, i) => `${i ? "else " : ""}if(outTexId<${i + 0.5})source=texture(uMainSampler[${i}],outTexCoord);`).join("\n")}
 color=source*vec4(outTint.bgr*outTint.a,outTint.a);
 vec2 point=vec2(gl_FragCoord.x,displaySize.y-gl_FragCoord.y)*worldSize/displaySize;
 if(color.a>0.)color.rgb=shadeColor(color.rgb/color.a,point)*color.a;
}`;
  return new (class extends P.Renderer.WebGL.Pipelines.MultiPipeline {
    constructor() {
      super({ game, vertShader, fragShader });
    }
    onBind() {
      const locations = this.currentShader.uniforms as unknown as Record<
        string,
        { location: { webGLUniformLocation: WebGLUniformLocation } }
      >;
      const uniforms = Object.fromEntries(
        ["night", "scale", "lights", "lightCells", "lightIndices"].map((name) => [
          name,
          locations[name]?.location.webGLUniformLocation ?? null,
        ]),
      );
      terrain.configureLighting(uniforms, lighting(), 13);
      this.set2f("displaySize", terrain.canvas.width, terrain.canvas.height);
      const size = terrain.dimensions;
      this.set2f("worldSize", size.width, size.height);
      // Raw texture binds above use reserved units, outside Phaser's 0..7 batches.
      this.gl.activeTexture(this.gl.TEXTURE0);
    }
  })();
}
