(* Phaser owns shader compilation. Daylight matches the shared Lighting module. *)

let halo_vertex =
  {glsl|
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
}|glsl}

let halo_fragment =
  {glsl|
precision highp float;
uniform sampler2D uMainSampler;
varying vec2 uv;
varying vec2 mode;
varying vec4 tint;

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
 
 if(mode.x!=0.0)discard;
 color*=vec4(tint.bgr*tint.a,tint.a);
 if(mode.y==2.0)color=vec4(tint.bgr*tint.a,tint.a);
 gl_FragColor=color;
}|glsl}

let fog_blur_vertex =
  {glsl|
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
}|glsl}

let fog_blur_fragment =
  {glsl|
precision highp float;
uniform sampler2D uMainSampler;
varying vec2 uv;
varying vec2 mode;
varying vec4 tint;

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
 
 if(mode.x!=0.0)discard;
 color*=vec4(tint.bgr*tint.a,tint.a);
 if(mode.y==2.0)color=vec4(tint.bgr*tint.a,tint.a);
 gl_FragColor=color;
}|glsl}

let fog_composite_vertex =
  {glsl|
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
}|glsl}

let fog_composite_fragment =
  {glsl|
precision highp float;
uniform sampler2D uMainSampler;
varying vec2 uv;
varying vec2 mode;
varying vec4 tint;

 void main(){float cover=texture2D(uMainSampler,uv).r;
 vec4 color=vec4(vec3(12.0,19.0,32.0)/255.0*cover,cover);
 
 if(mode.x!=0.0)discard;
 color*=vec4(tint.bgr*tint.a,tint.a);
 if(mode.y==2.0)color=vec4(tint.bgr*tint.a,tint.a);
 gl_FragColor=color;
}|glsl}

let fog_mask_vertex =
  {glsl|
precision highp float;
uniform vec2 uWorldSize;uniform vec2 uWorldOrigin;varying vec2 world;
attribute vec2 inPosition;
attribute vec2 inTexCoord;
attribute float inTexId;
attribute float inTintEffect;
attribute vec4 inTint;
varying vec2 uv;
varying vec2 mode;
varying vec4 tint;
void main(){
 world=inPosition;gl_Position=vec4((inPosition-uWorldOrigin)/uWorldSize*vec2(2.0,-2.0)+vec2(-1.0,1.0),0.0,1.0);
 uv=inTexCoord;mode=vec2(inTexId,inTintEffect);tint=inTint;
}|glsl}

let fog_mask_fragment =
  {glsl|
precision highp float;
uniform sampler2D uMainSampler;
varying vec2 uv;
varying vec2 mode;
varying vec4 tint;

 uniform vec2 uExtent;varying vec2 world;
 void main(){if(any(lessThan(world,vec2(0.)))||any(greaterThan(world,uExtent)))discard;
 vec4 color=texture2D(uMainSampler,uv);
 
 if(mode.x!=0.0)discard;
 color*=vec4(tint.bgr*tint.a,tint.a);
 if(mode.y==2.0)color=vec4(tint.bgr*tint.a,tint.a);
 gl_FragColor=color;
}|glsl}

let lighting_vertex =
  {glsl|
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
}|glsl}

let lighting_fragment =
  {glsl|
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
uniform float uDay;
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
  if (uNight < .5) {
    if (uDay > .5) {
      // Sample native world pixels so panning, zooming and page boundaries do
      // not move the cloud shade relative to the landscape.
      vec2 p = floor(worldPoint * uNativeScale) / uNativeScale;
      float wave = .5 + .25*sin(p.x*.010+p.y*.006+.9*sin(p.y*.009))
                       + .25*sin(p.x*.004-p.y*.011+1.7);
      float shade = smoothstep(.10,.85,wave)*.52;
      vec3 factor = vec3(1.04,1.,.94)-vec3(.42,.24,.05)*shade;
      vec3 offset = vec3(.018*(1.-shade),.012,.006+.012*shade);
      vec3 rgb = source.rgb / source.a;
      float lum = dot(rgb,vec3(.2126,.7152,.0722));
      rgb = mix(rgb,vec3(lum),.14)*factor+offset;
      source.rgb = clamp(rgb,0.,1.)*source.a;
    }
    gl_FragColor = source; return;
  }
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
}|glsl}

let actor_lighting_vertex =
  {glsl|precision highp float;
      uniform mat4 uProjectionMatrix;
      attribute vec2 inPosition;attribute vec2 inTexCoord;
      attribute float inTexId;attribute float inTintEffect;attribute vec4 inTint;
      attribute vec4 inFactor;attribute vec3 inOffset;
      varying vec2 outTexCoord;varying float outTexId;varying float outTintEffect;
      varying vec4 outTint;varying vec4 factor;varying vec3 offset;
      void main(){gl_Position=uProjectionMatrix*vec4(inPosition,1.,1.);
      outTexCoord=inTexCoord;outTexId=inTexId;outTintEffect=inTintEffect;outTint=inTint;
      factor=inFactor;offset=inOffset;}|glsl}

let actor_lighting_fragment =
  {glsl|precision highp float;
      uniform sampler2D uMainSampler[%count%];
      varying vec2 outTexCoord;varying float outTexId;varying float outTintEffect;
      varying vec4 outTint;varying vec4 factor;varying vec3 offset;
      void main(){vec4 texture;%forloop%
      vec4 ink=vec4(outTint.bgr*outTint.a,outTint.a);vec4 source=texture*ink;
      if(outTintEffect==1.)source.rgb=mix(texture.rgb,ink.rgb,texture.a);
      else if(outTintEffect==2.)source=ink;
      if(source.a>0.){vec3 rgb=source.rgb/source.a;
      float lum=dot(rgb,vec3(.2126,.7152,.0722));
      if(factor.a<-.5)rgb=mix(rgb,vec3(lum),.14);
      rgb=rgb*factor.rgb+lum*vec3(.28,.41,.63)*.2*max(factor.a,0.)+offset;
      source.rgb=clamp(rgb,0.,1.)*source.a;}gl_FragColor=source;}|glsl}

let light_field_vertex =
  {glsl|precision highp float;
uniform mat4 uProjectionMatrix;
attribute vec2 inPosition;attribute vec2 inTexCoord;attribute vec4 inTint;
attribute float inTexId;attribute float inTintEffect;
varying vec2 uv;varying vec4 ink;varying vec2 mode;
void main(){gl_Position=uProjectionMatrix*vec4(inPosition,1.,1.);uv=inTexCoord;ink=inTint;mode=vec2(inTexId,inTintEffect);}|glsl}

let light_field_fragment =
  {glsl|precision highp float;
      uniform sampler2D uMainSampler;uniform sampler2D uPrevious;
      uniform vec2 uSize;uniform vec4 uCell;uniform vec2 uOrigin;uniform vec2 uAtlas;
      uniform float uCount;uniform float uKind;uniform float uHasPrevious;uniform float uNormalization;
      uniform vec4 uLights[%count%];uniform vec3 uWarm[%count%];uniform vec3 uOffset[%count%];
      varying vec2 uv;varying vec4 ink;varying vec2 mode;
      float unpackValue(vec4 v){return dot(v.rgb,vec3(65536.,256.,1.))*255./16777215.;}
      vec4 packValue(float v){float n=min(16777215.,floor(clamp(v,0.,1.)*16777215.+.5));return vec4(floor(mod(n/vec3(65536.,256.,1.),256.))/255.,1.);}
      void main(){
        if(mode.x!=0.||mode.y!=0.||ink.a==0.)discard;
        vec2 local=floor(uv*vec2(uCell.z*4.,uCell.w));
        float slot=mod(local.x,4.);vec2 localPixel=vec2(floor(local.x/4.),local.y);vec2 pixel=localPixel+uCell.xy;vec2 packedPixel=localPixel+uAtlas;
        float value=0.;
        if(uHasPrevious>.5)value=unpackValue(texture2D(uPrevious,vec2((packedPixel.x*4.+slot+.5)/(uSize.x*4.),(packedPixel.y+.5)/uSize.y)));
        vec2 point=pixel/0.91428571428571426+uOrigin;
        for(int i=0;i<%count%;i++){
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
      }|glsl}
