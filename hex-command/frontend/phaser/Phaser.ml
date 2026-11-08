(* Phaser 3.90 objects stay JavaScript objects. Method signatures preserve
   receivers and fluent return values; these declarations allocate no wrappers. *)
type ('argument, 'instance) constructor
type child_destroyed
type sound_unlock
type target
type raf
type uints
type image_element = Browser_image.t
type audio_context
type audio_buffer
type pipeline_data
type xhr

type game_object =
  < scene : scene Js.Undefined.t
  ; parentContainer : container Js.Nullable.t
  ; removeFromDisplayList : unit -> unit [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> game_object [@mel.meth]
  ; off : string -> child_destroyed -> container -> game_object [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and style = < visibility : string ; width : string ; height : string ; cursor : string > Js.t

and canvas =
  < width : int
  ; height : int
  ; parentElement : Dom.element Js.Nullable.t
  ; style : style
  ; setAttribute : string -> string -> unit [@mel.meth]
  ; remove : unit -> unit [@mel.meth] >
  Js.t

and image =
  < x : float
  ; y : float
  ; width : float
  ; height : float
  ; alpha : float
  ; depth : float
  ; visible : bool
  ; scene : scene Js.Undefined.t
  ; parentContainer : container Js.Nullable.t
  ; removeFromDisplayList : unit -> unit [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> game_object [@mel.meth]
  ; off : string -> child_destroyed -> container -> game_object [@mel.meth]
  ; destroy : unit -> unit [@mel.meth]
  ; clearTint : unit -> image [@mel.meth]
  ; setPosition : float -> float -> image [@mel.meth]
  ; setDepth : float -> image [@mel.meth]
  ; setOrigin : float -> float -> image [@mel.meth]
  ; setScale : float -> float -> image [@mel.meth]
  ; setAlpha : float -> image [@mel.meth]
  ; setVisible : bool -> image [@mel.meth]
  ; setDisplaySize : float -> float -> image [@mel.meth]
  ; setScrollFactor : float -> float -> image [@mel.meth]
  ; texture : texture
  ; frame : texture_frame
  ; scaleX : float
  ; scaleY : float
  ; pipelineData : pipeline_data
  ; setTexture : string -> image [@mel.meth]
  ; setTint : int -> image [@mel.meth]
  ; setBlendMode : int -> image [@mel.meth]
  ; setCrop : float -> float -> float -> float -> image [@mel.meth] >
  Js.t

and container =
  < x : float
  ; y : float
  ; width : float
  ; height : float
  ; alpha : float
  ; depth : float
  ; visible : bool
  ; scene : scene Js.Undefined.t
  ; parentContainer : container Js.Nullable.t
  ; removeFromDisplayList : unit -> unit [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> game_object [@mel.meth]
  ; off : string -> child_destroyed -> container -> game_object [@mel.meth]
  ; destroy : unit -> unit [@mel.meth]
  ; setY : float -> container [@mel.meth]
  ; setPosition : float -> float -> container [@mel.meth]
  ; setDepth : float -> container [@mel.meth]
  ; setOrigin : float -> float -> container [@mel.meth]
  ; setScale : float -> float -> container [@mel.meth]
  ; setAlpha : float -> container [@mel.meth]
  ; setVisible : bool -> container [@mel.meth]
  ; setDisplaySize : float -> float -> container [@mel.meth]
  ; setScrollFactor : float -> float -> container [@mel.meth]
  ; list : game_object array
  ; onChildDestroyed : child_destroyed
  ; addAt : game_object -> int -> container [@mel.meth]
  ; add : game_object -> container [@mel.meth]
  ; remove : game_object -> bool -> container [@mel.meth]
  ; removeAll : bool -> container [@mel.meth]
  ; moveTo : game_object -> int -> container [@mel.meth] >
  Js.t

and ellipse =
  < x : float
  ; y : float
  ; width : float
  ; height : float
  ; alpha : float
  ; depth : float
  ; visible : bool
  ; scene : scene Js.Undefined.t
  ; parentContainer : container Js.Nullable.t
  ; removeFromDisplayList : unit -> unit [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> game_object [@mel.meth]
  ; off : string -> child_destroyed -> container -> game_object [@mel.meth]
  ; destroy : unit -> unit [@mel.meth]
  ; setPosition : float -> float -> ellipse [@mel.meth]
  ; setDepth : float -> ellipse [@mel.meth]
  ; setOrigin : float -> float -> ellipse [@mel.meth]
  ; setScale : float -> float -> ellipse [@mel.meth]
  ; setAlpha : float -> ellipse [@mel.meth]
  ; setVisible : bool -> ellipse [@mel.meth]
  ; setDisplaySize : float -> float -> ellipse [@mel.meth]
  ; setScrollFactor : float -> float -> ellipse [@mel.meth]
  ; fillColor : int
  ; scaleX : float
  ; setSmoothness : int -> ellipse [@mel.meth]
  ; setFillStyle : int -> float -> ellipse [@mel.meth] >
  Js.t

and graphics =
  < x : float
  ; y : float
  ; width : float
  ; height : float
  ; alpha : float
  ; depth : float
  ; visible : bool
  ; scene : scene Js.Undefined.t
  ; parentContainer : container Js.Nullable.t
  ; removeFromDisplayList : unit -> unit [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> game_object [@mel.meth]
  ; off : string -> child_destroyed -> container -> game_object [@mel.meth]
  ; destroy : unit -> unit [@mel.meth]
  ; setPosition : float -> float -> graphics [@mel.meth]
  ; setDepth : float -> graphics [@mel.meth]
  ; setOrigin : float -> float -> graphics [@mel.meth]
  ; setScale : float -> float -> graphics [@mel.meth]
  ; setAlpha : float -> graphics [@mel.meth]
  ; setVisible : bool -> graphics [@mel.meth]
  ; setDisplaySize : float -> float -> graphics [@mel.meth]
  ; setScrollFactor : float -> float -> graphics [@mel.meth]
  ; pipelineData : pipeline_data
  ; clear : unit -> graphics [@mel.meth]
  ; fillStyle : int -> float -> graphics [@mel.meth]
  ; lineStyle : float -> int -> float -> graphics [@mel.meth]
  ; fillCircle : float -> float -> float -> graphics [@mel.meth]
  ; strokeCircle : float -> float -> float -> graphics [@mel.meth]
  ; fillRect : float -> float -> float -> float -> graphics [@mel.meth]
  ; strokeRect : float -> float -> float -> float -> graphics [@mel.meth]
  ; fillPoints : vector array -> bool -> bool -> graphics [@mel.meth]
  ; strokePoints : vector array -> bool -> bool -> graphics [@mel.meth]
  ; beginPath : unit -> graphics [@mel.meth]
  ; moveTo : float -> float -> graphics [@mel.meth]
  ; lineTo : float -> float -> graphics [@mel.meth]
  ; closePath : unit -> graphics [@mel.meth]
  ; strokePath : unit -> graphics [@mel.meth]
  ; generateTexture : string -> int -> int -> graphics [@mel.meth]
  ; fillPath : unit -> graphics [@mel.meth] >
  Js.t

and render_texture =
  < camera : camera
  ; x : float
  ; y : float
  ; width : float
  ; height : float
  ; alpha : float
  ; depth : float
  ; visible : bool
  ; scene : scene Js.Undefined.t
  ; parentContainer : container Js.Nullable.t
  ; removeFromDisplayList : unit -> unit [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> game_object [@mel.meth]
  ; off : string -> child_destroyed -> container -> game_object [@mel.meth]
  ; destroy : unit -> unit [@mel.meth]
  ; setPosition : float -> float -> render_texture [@mel.meth]
  ; setDepth : float -> render_texture [@mel.meth]
  ; setOrigin : float -> float -> render_texture [@mel.meth]
  ; setScale : float -> float -> render_texture [@mel.meth]
  ; setAlpha : float -> render_texture [@mel.meth]
  ; setVisible : bool -> render_texture [@mel.meth]
  ; setDisplaySize : float -> float -> render_texture [@mel.meth]
  ; setScrollFactor : float -> float -> render_texture [@mel.meth]
  ; texture : texture
  ; frame : texture_frame
  ; clear : unit -> render_texture [@mel.meth]
  ; fill : int -> float -> render_texture [@mel.meth]
  ; beginDraw : unit -> render_texture [@mel.meth]
  ; endDraw : unit -> render_texture [@mel.meth]
  ; batchDraw : game_object -> render_texture [@mel.meth]
  ; draw : game_object -> float -> float -> render_texture [@mel.meth]
  ; saveTexture : string -> render_texture [@mel.meth]
  ; setSize : int -> int -> render_texture [@mel.meth] >
  Js.t

and vector = < x : float ; y : float > Js.t
and polygon = < points : vector array > Js.t
and layer_data = < baseTileWidth : float ; baseTileHeight : float > Js.t

and factory =
  < image : float -> float -> string -> image [@mel.meth]
  ; container : float -> float -> container [@mel.meth]
  ; ellipse : float -> float -> float -> float -> int -> ellipse [@mel.meth]
  ; graphics : unit -> graphics [@mel.meth]
  ; renderTexture : float -> float -> int -> int -> render_texture [@mel.meth] >
  Js.t

and camera =
  < zoomX : float
  ; zoomY : float
  ; setOrigin : float -> float -> camera [@mel.meth]
  ; scrollX : float
  ; scrollY : float
  ; zoom : float
  ; width : float
  ; height : float
  ; worldView :
      < x : float ; y : float ; width : float ; height : float ; right : float ; bottom : float >
      Js.t
  ; getWorldPoint : float -> float -> vector -> vector [@mel.meth]
  ; setZoom : float -> camera [@mel.meth]
  ; setScroll : float -> float -> camera [@mel.meth]
  ; setBounds : float -> float -> float -> float -> camera [@mel.meth]
  ; setViewport : float -> float -> float -> float -> camera [@mel.meth]
  ; setVisible : bool -> camera [@mel.meth] >
  Js.t

and cameras = < main : camera > Js.t
and creator = < graphics : < x : float ; y : float > Js.t -> bool -> graphics [@mel.meth] > Js.t

and scene =
  < make : creator
  ; add : factory
  ; cameras : cameras
  ; game : game
  ; events : events
  ; scale : scale
  ; load : loader
  ; textures : textures
  ; cache : cache
  ; input : scene_input
  ; tweens : tweens
  ; sys : systems >
  Js.t

and systems = < displayList : display_list > Js.t
and display_list = < list : game_object array > Js.t
and scene_input = < enabled : bool ; activePointer : pointer ; manager : input > Js.t

and pointer =
  < id : int
  ; identifier : int
  ; button : int
  ; isDown : bool
  ; wasTouch : bool
  ; wasCanceled : bool
  ; leftButtonDown : unit -> bool [@mel.meth]
  ; middleButtonDown : unit -> bool [@mel.meth]
  ; getDistance : unit -> float [@mel.meth]
  ; x : float
  ; y : float
  ; worldX : float
  ; worldY : float
  ; wasTouch : bool
  ; identifier : int
  ; id : int
  ; isDown : bool
  ; downX : float
  ; downY : float
  ; downTime : float
  ; event : Dom.event
  ; position : vector >
  Js.t

and tweens =
  < killTweensOf : game_object -> unit [@mel.meth] ; killAll : unit -> unit [@mel.meth] > Js.t

and events =
  < on : string -> ((unit -> unit)[@u]) -> events [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> events [@mel.meth]
  ; off : string -> ((unit -> unit)[@u]) -> events [@mel.meth]
  ; removeAllListeners : unit -> events [@mel.meth] >
  Js.t

and manager = < game : game Js.Nullable.t ; destroy : unit -> unit [@mel.meth] > Js.t

and sound_manager =
  < game : game Js.Nullable.t
  ; unlock : sound_unlock
  ; context : audio_context Js.Nullable.t
  ; locked : bool
  ; add : string -> sound [@mel.meth]
  ; get : string -> sound Js.Nullable.t [@mel.meth]
  ; removeAll : unit -> unit [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and sound =
  < duration : float
  ; isPlaying : bool
  ; key : string
  ; play : < volume : float ; rate : float ; delay : float > Js.t -> bool [@mel.meth]
  ; stop : unit -> bool [@mel.meth]
  ; destroy : unit -> unit [@mel.meth]
  ; once : string -> ((unit -> unit)[@u]) -> sound [@mel.meth] >
  Js.t

and device = < target : target Js.Nullable.t > Js.t

and input =
  < pointers : pointer array
  ; game : game Js.Nullable.t
  ; keyboard : device Js.Nullable.t
  ; mouse : device Js.Nullable.t
  ; touch : device Js.Nullable.t
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and textures =
  < game : game Js.Nullable.t
  ; stamp : image Js.Nullable.t
  ; _tempCanvas : canvas Js.Nullable.t
  ; getTextureKeys : unit -> string array [@mel.meth]
  ; exists : string -> bool [@mel.meth]
  ; get : string -> texture [@mel.meth]
  ; addImage : string -> image_element -> texture [@mel.meth]
  ; remove : string -> textures [@mel.meth]
  ; removeAllListeners : unit -> textures [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and texture =
  < key : string
  ; source : texture_source array
  ; get : unit -> texture_frame [@mel.meth]
  ; setFilter : int -> texture [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and texture_source =
  < glTexture : gl_texture Js.Nullable.t
  ; width : int
  ; height : int
  ; image : image_element
  ; setFilter : int -> unit [@mel.meth] >
  Js.t

and texture_frame =
  < source : texture_source
  ; width : int
  ; height : int
  ; cutX : int
  ; cutY : int
  ; u0 : float
  ; v0 : float
  ; u1 : float
  ; v1 : float >
  Js.t

and gl_texture = < width : int ; height : int > Js.t

and render_target =
  < width : int ; height : int ; hasDepthBuffer : bool ; texture : gl_texture > Js.t

and pipeline =
  < name : string
  ; game : game
  ; renderer : renderer
  ; activeTextures : gl_texture array
  ; vertexCount : int
  ; vertexViewF32 : Webgl.floats
  ; vertexViewU32 : uints
  ; currentUnit : int
  ; currentBatch : batch Js.Nullable.t
  ; currentShader : shader
  ; flush : unit -> unit [@mel.meth]
  ; shouldFlush : int -> bool [@mel.meth]
  ; setTexture2D : gl_texture Js.Nullable.t -> int [@mel.meth]
  ; onBatch : game_object Js.Undefined.t -> unit [@mel.meth]
  ; bindTexture : gl_texture -> int -> pipeline [@mel.meth]
  ; set1i : string -> int -> pipeline [@mel.meth]
  ; set1f : string -> float -> pipeline [@mel.meth]
  ; set2f : string -> float -> float -> pipeline [@mel.meth]
  ; set3f : string -> float -> float -> float -> pipeline [@mel.meth]
  ; set4f : string -> float -> float -> float -> float -> pipeline [@mel.meth]
  ; set1fv : string -> float array -> pipeline [@mel.meth]
  ; set3fv : string -> Webgl.floats -> pipeline [@mel.meth]
  ; set4fv : string -> Webgl.floats -> pipeline [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and batch = < start : int ; count : int > Js.t
and shader = < vertexComponentCount : int > Js.t

and pipeline_manager =
  < flush : unit -> unit [@mel.meth]
  ; remove : string -> bool -> unit [@mel.meth]
  ; add : string -> pipeline -> pipeline [@mel.meth] >
  Js.t

and renderer =
  < gl : Webgl.t
  ; pipelines : pipeline_manager
  ; whiteTexture : gl_texture
  ; renderTarget : render_target Js.Nullable.t
  ; preRender : unit -> unit [@mel.meth]
  ; postRender : unit -> unit [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and scenes =
  < systemScene : scene Js.Nullable.t
  ; update : float -> float -> unit [@mel.meth]
  ; render : renderer -> unit [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and loop =
  < raf : raf Js.Nullable.t ; stop : unit -> unit [@mel.meth] ; destroy : unit -> unit [@mel.meth] >
  Js.t

and scale =
  < game : game Js.Nullable.t
  ; on : string -> ((unit -> unit)[@u]) -> scale [@mel.meth]
  ; resize : int -> int -> scale [@mel.meth]
  ; updateBounds : unit -> scale [@mel.meth]
  ; destroy : unit -> unit [@mel.meth] >
  Js.t

and cache = < audio : audio_cache > Js.t

and audio_cache =
  < exists : string -> bool [@mel.meth] ; get : string -> audio_buffer [@mel.meth] > Js.t

and loader =
  < maxParallelDownloads : int
  ; scene : scene
  ; audioContext : audio_context
  ; isLoading : unit -> bool [@mel.meth]
  ; start : unit -> loader [@mel.meth]
  ; addFile : file -> loader [@mel.meth]
  ; nextFile : file -> bool -> unit [@mel.meth]
  ; fileProcessComplete : file -> unit [@mel.meth]
  ; list : file_set
  ; inflight : file_set
  ; queue : file_set
  ; on : string -> ((unit -> unit)[@u]) -> loader [@mel.meth]
  ; off : string -> ((unit -> unit)[@u]) -> loader [@mel.meth] >
  Js.t

and file_set =
  < size : int ; delete : file -> file_set [@mel.meth] ; contains : file -> bool [@mel.meth] > Js.t

and file =
  < key : string
  ; src : string
  ; type_ : string
  ; state : int
  ; loader : loader Js.Nullable.t
  ; xhrLoader : xhr Js.Nullable.t
  ; destroy : unit -> unit [@mel.meth]
  ; resetXHR : unit -> unit [@mel.meth]
  ; onProcessError : unit -> unit [@mel.meth]
  ; onProcessComplete : unit -> unit [@mel.meth]
  ; load : unit -> unit [@mel.meth] >
  Js.t

and game =
  < cache : cache
  ; events : events
  ; scale : scale
  ; scene : scenes
  ; loop : loop
  ; input : input
  ; sound : sound_manager
  ; anims : manager
  ; plugins : manager
  ; textures : textures
  ; renderer : renderer Js.Nullable.t
  ; canvas : canvas Js.Nullable.t
  ; domContainer : canvas Js.Nullable.t
  ; pendingDestroy : bool
  ; isPaused : bool
  ; destroy : bool -> bool -> unit [@mel.meth] >
  Js.t

type game_config =
  < parent : Dom.element
  ; width : int
  ; height : int
  ; backgroundColor : string
  ; pixelArt : bool
  ; antialias : bool
  ; transparent : bool
  ; scene : scene array
  ; canvasStyle : string
  ; roundPixels : bool
  ; antialiasGL : bool
  ; banner : bool
  ; callbacks : < preBoot : (game -> unit[@u]) > Js.t
  ; input : < touch : < capture : bool > Js.t ; mouse : < preventDefaultWheel : bool > Js.t > Js.t
  ; loader :
      < maxParallelDownloads : int ; timeout : int ; maxRetries : int ; imageLoadType : string >
      Js.t
  ; scale : < mode : int ; width : int ; height : int > Js.t >
  Js.t

type game_definition =
  < boot : (game -> unit[@mel.this])
  ; texturesReady : (game -> unit[@mel.this])
  ; start : (game -> unit[@mel.this])
  ; step : (game -> float -> float -> unit[@mel.this])
  ; destroy : (game -> bool -> bool -> unit[@mel.this]) >
  Js.t

type game_prototype =
  < boot : < call : game -> unit [@mel.meth] > Js.t
  ; texturesReady : < call : game -> unit [@mel.meth] > Js.t
  ; start : < call : game -> unit [@mel.meth] > Js.t
  ; destroy : < call : game -> bool -> unit [@mel.meth] > Js.t
  ; runDestroy : < call : game -> unit [@mel.meth] > Js.t >
  Js.t

external construct : ('argument, 'instance) constructor -> 'argument array -> 'instance
  = "construct"
[@@mel.scope "Reflect"]

module Game = struct
  type t = game
  type config = game_config

  external constructor : (config, t) constructor = "Game" [@@mel.module "phaser"]
  external prototype : (config, t) constructor -> game_prototype = "prototype" [@@mel.get]
  external set_type : config -> int -> unit = "type" [@@mel.set]
  external webgl : int = "WEBGL" [@@mel.module "phaser"]
  external scale_none : int = "NONE" [@@mel.module "phaser"] [@@mel.scope "Scale"]

  external off_unlock : events -> string -> sound_unlock -> sound_manager -> events = "off"
  [@@mel.send]

  let config ~parent ~width ~height ~scene =
    let value =
      [%mel.obj
        {
          parent;
          width;
          height;
          backgroundColor = "#19231c";
          pixelArt = false;
          antialias = false;
          transparent = false;
          scene;
          canvasStyle = "visibility: hidden;";
          roundPixels = false;
          antialiasGL = false;
          banner = false;
          callbacks =
            [%mel.obj
              {
                preBoot =
                  (fun[@u] owner ->
                    let manager = owner##sound in
                    let _ = off_unlock owner##events "boot" manager##unlock manager in
                    ());
              }];
          input =
            [%mel.obj
              {
                touch = [%mel.obj { capture = true }];
                mouse = [%mel.obj { preventDefaultWheel = true }];
              }];
          loader =
            [%mel.obj
              {
                maxParallelDownloads = 6;
                timeout = 30000;
                maxRetries = 0;
                imageLoadType = "HTMLImageElement";
              }];
          scale = [%mel.obj { mode = scale_none; width; height }];
        }]
    in
    set_type value webgl;
    value
end

module Scene = struct
  type t = scene

  external constructor : (string, t) constructor = "Scene" [@@mel.module "phaser"]
  external set_create : t -> ((t -> unit)[@mel.this]) -> unit = "create" [@@mel.set]

  external set_update : t -> ((t -> float -> float -> unit)[@mel.this]) -> unit = "update"
  [@@mel.set]

  let create key = construct constructor [| key |]
end

module Image = struct
  type t = image
end

module Camera = struct
  type t = camera
end

module Class = struct
  external game : game_definition -> (game_config, game) constructor = "Class"
  [@@mel.module "phaser"] [@@mel.new]

  external extends_game : game_definition -> (game_config, game) constructor -> unit = "Extends"
  [@@mel.set]
end

module Events = struct
  external emit : events -> string -> unit = "emit" [@@mel.send]
  external emit_step : events -> string -> float -> float -> unit = "emit" [@@mel.send]

  external emit_render : events -> string -> renderer -> float -> float -> unit = "emit"
  [@@mel.send]
end

module Internal = struct
  (* These are pinned 3.90 lifecycle seams omitted from Phaser's public types. *)
  external processing : scenes -> bool -> unit = "isProcessing" [@@mel.set]
  external drop_keyboard : input -> device Js.Nullable.t -> unit = "keyboard" [@@mel.set]
  external drop_mouse : input -> device Js.Nullable.t -> unit = "mouse" [@@mel.set]
  external drop_touch : input -> device Js.Nullable.t -> unit = "touch" [@@mel.set]

  external remove_canvas : canvas -> unit = "remove"
  [@@mel.module "phaser"] [@@mel.scope "Display", "Canvas", "CanvasPool"]
end

module Graphics = struct
  external stroke_points : graphics -> vector array -> bool -> graphics = "strokePoints"
  [@@mel.send]
end

module Textures = struct
  external nearest : int = "NEAREST" [@@mel.module "phaser"] [@@mel.scope "Textures", "FilterMode"]
end

module Geom = struct
  module Point = struct
    external create : float -> float -> vector = "Point"
    [@@mel.module "phaser"] [@@mel.scope "Geom"] [@@mel.new]
  end

  module Vector2 = struct
    external constructor : (unit, vector) constructor = "Vector2"
    [@@mel.module "phaser"] [@@mel.scope "Math"]

    external create : unit -> vector = "Vector2"
    [@@mel.module "phaser"] [@@mel.scope "Math"] [@@mel.new]

    external set_x : vector -> float -> unit = "x" [@@mel.set]
    external set_y : vector -> float -> unit = "y" [@@mel.set]
  end

  module Polygon = struct
    external create : vector array -> polygon = "Polygon"
    [@@mel.module "phaser"] [@@mel.scope "Geom"] [@@mel.new]

    external contains : polygon -> float -> float -> bool = "Contains"
    [@@mel.module "phaser"] [@@mel.scope "Geom", "Polygon"]
  end
end

module Tilemaps = struct
  external layer : < baseTileWidth : float ; baseTileHeight : float > Js.t -> layer_data
    = "LayerData"
  [@@mel.module "phaser"] [@@mel.scope "Tilemaps"] [@@mel.new]

  external stagger_axis : layer_data -> string -> unit = "staggerAxis" [@@mel.set]
  external stagger_index : layer_data -> string -> unit = "staggerIndex" [@@mel.set]

  external world_to_tile : float -> float -> bool -> vector -> camera -> layer_data -> vector
    = "HexagonalWorldToTileXY"
  [@@mel.module "phaser"] [@@mel.scope "Tilemaps", "Components"]
end

module Pipeline = struct
  type attribute_type

  external float_type : attribute_type = "FLOAT"
  [@@mel.module "phaser"] [@@mel.scope "Renderer", "WebGL"]

  external unsigned_byte : attribute_type = "UNSIGNED_BYTE"
  [@@mel.module "phaser"] [@@mel.scope "Renderer", "WebGL"]

  type attribute = {
    name : string;
    size : int;
    kind : attribute_type; [@mel.as "type"]
    normalized : bool;
  }

  type config =
    < game : game
    ; vertShader : string
    ; fragShader : string
    ; attributes : attribute array Js.Undefined.t >
    Js.t

  external config :
    game:game ->
    vertShader:string ->
    fragShader:string ->
    ?attributes:attribute array ->
    unit ->
    config = ""
  [@@mel.obj]

  external single : (config, pipeline) constructor = "SinglePipeline"
  [@@mel.module "phaser"] [@@mel.scope "Renderer", "WebGL", "Pipelines"]

  external multi : (config, pipeline) constructor = "MultiPipeline"
  [@@mel.module "phaser"] [@@mel.scope "Renderer", "WebGL", "Pipelines"]

  external initialize : (config, pipeline) constructor -> pipeline -> config -> unit = "call"
  [@@mel.send]

  external set_vertex_count : pipeline -> int -> unit = "vertexCount" [@@mel.set]
  external set_batch_count : batch -> int -> unit = "count" [@@mel.set]
  external set_uint : uints -> int -> int -> unit = "" [@@mel.set_index]
end

module GameObjects = struct
  external image : scene -> float -> float -> string -> image = "Image"
  [@@mel.module "phaser"] [@@mel.scope "GameObjects"] [@@mel.new]

  external graphics : scene -> graphics = "Graphics"
  [@@mel.module "phaser"] [@@mel.scope "GameObjects"] [@@mel.new]

  external render_texture : scene -> float -> float -> int -> int -> bool -> render_texture
    = "RenderTexture"
  [@@mel.module "phaser"] [@@mel.scope "GameObjects"] [@@mel.new]
end

module Upcast = struct
  external canvas_element : canvas -> Dom.element = "%identity"

  (* Js.t is invariant: these one-way FFI coercions encode Phaser's documented
     inheritance without wrapping objects or permitting arbitrary downcasts. *)
  external image : image -> game_object = "%identity"
  external graphics : graphics -> game_object = "%identity"
  external container : container -> game_object = "%identity"
  external ellipse : ellipse -> game_object = "%identity"
  external render_texture : render_texture -> game_object = "%identity"
  external container_point : container -> vector = "%identity"
end

module TextureImage = struct
  external create : scene -> float -> float -> texture -> image = "Image"
  [@@mel.module "phaser"] [@@mel.scope "GameObjects"] [@@mel.new]
end

module DynamicTexture = struct
  external dirty : texture -> bool -> unit = "dirty" [@@mel.set]
  external clear : texture -> int -> int -> int -> int -> unit = "clear" [@@mel.send]
end

module RenderTexture = struct
  external draw_many : render_texture -> game_object array -> unit = "draw" [@@mel.send]
  external batch_many : render_texture -> game_object array -> unit = "batchDraw" [@@mel.send]
end

module Texture = struct
  external source_image : texture -> image_element = "getSourceImage" [@@mel.send]
end

module CameraMethods = struct
  external zoom : camera -> float -> float -> unit = "setZoom" [@@mel.send]
  external pre_render : camera -> unit = "preRender" [@@mel.send]
  external remove_bounds : camera -> unit = "removeBounds" [@@mel.send]
end

module ScaleMethods = struct
  type size

  external display_size : scale -> size = "displaySize" [@@mel.get]
  external set_size : size -> float -> float -> unit = "setSize" [@@mel.send]
  external display_scale : scale -> vector = "displayScale" [@@mel.get]
  external set_vector : vector -> float -> float -> unit = "set" [@@mel.send]
end

module SceneMethods = struct
  type bytes

  external bytes : int array -> bytes = "Uint8Array" [@@mel.new]

  external white_texture : textures -> string -> bytes -> int -> int -> texture = "addUint8Array"
  [@@mel.send]

  external input_enabled : scene_input -> bool -> unit = "enabled" [@@mel.set]
  external prevent_wheel : device -> bool -> unit = "preventDefaultWheel" [@@mel.set]

  external color : string -> < color : int > Js.t = "HexStringToColor"
  [@@mel.module "phaser"] [@@mel.scope "Display", "Color"]

  external add_many : container -> game_object array -> unit = "add" [@@mel.send]
end
