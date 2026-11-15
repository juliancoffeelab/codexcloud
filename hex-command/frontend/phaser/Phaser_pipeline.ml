(* The only subclass seam here is Phaser's native pipeline. Application callbacks
   are OCaml closures; sprites/textures remain their original Phaser objects. *)
type t = { native : Phaser.pipeline; values : float array ref }

external native : t -> Phaser.pipeline = "native" [@@mel.get]

let set_values pipeline values = pipeline.values := values

type definition =
  < initialize : (Phaser.pipeline -> Phaser.Pipeline.config -> unit[@mel.this])
  ; onBind : (Phaser.pipeline -> Phaser.game_object Js.Undefined.t -> unit[@mel.this])
  ; destroy : (Phaser.pipeline -> unit[@mel.this])
  ; batchVert :
      (Phaser.pipeline -> float -> float -> float -> float -> int -> float -> int -> unit
      [@mel.this])
  ; batchQuad :
      (Phaser.pipeline ->
       Phaser.game_object Js.Nullable.t ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       int ->
       int ->
       int ->
       int ->
       float ->
       Phaser.gl_texture Js.Nullable.t ->
       int Js.Undefined.t ->
       bool
      [@mel.this])
  ; batchTri :
      (Phaser.pipeline ->
       Phaser.game_object Js.Nullable.t ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       float ->
       int ->
       int ->
       int ->
       float ->
       Phaser.gl_texture Js.Nullable.t ->
       int Js.Undefined.t ->
       bool
      [@mel.this]) >
  Js.t

external subclass : definition -> (Phaser.Pipeline.config, Phaser.pipeline) Phaser.constructor
  = "Class"
[@@mel.module "phaser"] [@@mel.new]

type single_definition =
  < initialize : (Phaser.pipeline -> Phaser.Pipeline.config -> unit[@mel.this])
  ; onBind : (Phaser.pipeline -> Phaser.game_object Js.Undefined.t -> unit[@mel.this])
  ; destroy : (Phaser.pipeline -> unit[@mel.this]) >
  Js.t

external single_subclass :
  single_definition -> (Phaser.Pipeline.config, Phaser.pipeline) Phaser.constructor = "Class"
[@@mel.module "phaser"] [@@mel.new]

external single_extends :
  single_definition -> (Phaser.Pipeline.config, Phaser.pipeline) Phaser.constructor -> unit
  = "Extends"
[@@mel.set]

external extends :
  definition -> (Phaser.Pipeline.config, Phaser.pipeline) Phaser.constructor -> unit = "Extends"
[@@mel.set]

type prototype =
  < destroy : < call : Phaser.pipeline -> unit [@mel.meth] > Js.t
  ; batchVert :
      < call : Phaser.pipeline -> float -> float -> float -> float -> int -> float -> int -> unit
            [@mel.meth] >
      Js.t >
  Js.t

external prototype : (Phaser.Pipeline.config, Phaser.pipeline) Phaser.constructor -> prototype
  = "prototype"
[@@mel.get]

external value_at : float array -> int -> float = "" [@@mel.get_index]

let write_values (pipeline : Phaser.pipeline) values start count =
  let stride = pipeline##currentShader##vertexComponentCount in
  for vertex = start to start + count - 1 do
    let offset = (vertex * stride) + 7 in
    for i = 0 to Array.length values - 1 do
      Webgl.set pipeline##vertexViewF32 (offset + i) (value_at values i)
    done
  done

let create ?(multi = false) ?attributes ~game ~vertex ~fragment ~bind ?(dispose = fun () -> ()) () =
  let values = ref [||] in
  let base = if multi then Phaser.Pipeline.multi else Phaser.Pipeline.single in
  let config =
    Phaser.Pipeline.config ~game ~vertShader:vertex ~fragShader:fragment ?attributes ()
  in
  let definition =
    [%mel.obj
      {
        initialize = (fun[@mel.this] native config -> Phaser.Pipeline.initialize base native config);
        onBind =
          (fun[@mel.this] native object_ -> bind native values (Js.Undefined.toOption object_));
        destroy =
          (fun[@mel.this] native ->
            dispose ();
            (prototype base)##destroy##call native);
        batchVert =
          (fun[@mel.this] native x y u v unit_ tint_effect tint ->
            let start = native##vertexCount in
            (prototype base)##batchVert##call native x y u v unit_ tint_effect tint;
            write_values native !values start 1);
        batchQuad =
          (fun[@mel.this] native
            object_
            x0
            y0
            x1
            y1
            x2
            y2
            x3
            y3
            u0
            v0
            u1
            v1
            tl
            tr
            bl
            br
            tint_effect
            texture
            unit_
          ->
            let unit_ =
              ref (Option.value ~default:native##currentUnit (Js.Undefined.toOption unit_))
            in
            let flushed = native##shouldFlush 6 in
            if flushed then native##flush ();
            if Js.Nullable.isNullable native##currentBatch then
              unit_ := native##setTexture2D texture;
            let offset =
              ref ((native##vertexCount * native##currentShader##vertexComponentCount) - 1)
            in
            incr offset;
            Webgl.set native##vertexViewF32 !offset x0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset tl;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            incr offset;
            Webgl.set native##vertexViewF32 !offset x1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset bl;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            incr offset;
            Webgl.set native##vertexViewF32 !offset x2;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y2;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset br;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            incr offset;
            Webgl.set native##vertexViewF32 !offset x0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset tl;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            incr offset;
            Webgl.set native##vertexViewF32 !offset x2;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y2;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset br;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            incr offset;
            Webgl.set native##vertexViewF32 !offset x3;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y3;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset tr;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            Phaser.Pipeline.set_vertex_count native (native##vertexCount + 6);
            (match Js.Nullable.toOption native##currentBatch with
            | None -> failwith {js|Phaser batch missing after texture binding|js}
            | Some batch ->
                Phaser.Pipeline.set_batch_count batch (native##vertexCount - batch##start));
            native##onBatch (Js.Undefined.fromOption (Js.Nullable.toOption object_));
            flushed);
        batchTri =
          (fun[@mel.this] native
            object_
            x0
            y0
            x1
            y1
            x2
            y2
            u0
            v0
            u1
            v1
            tl
            tr
            bl
            tint_effect
            texture
            unit_
          ->
            let unit_ =
              ref (Option.value ~default:native##currentUnit (Js.Undefined.toOption unit_))
            in
            let flushed = native##shouldFlush 3 in
            if flushed then native##flush ();
            if Js.Nullable.isNullable native##currentBatch then
              unit_ := native##setTexture2D texture;
            let offset =
              ref ((native##vertexCount * native##currentShader##vertexComponentCount) - 1)
            in
            incr offset;
            Webgl.set native##vertexViewF32 !offset x0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset tl;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            incr offset;
            Webgl.set native##vertexViewF32 !offset x1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u0;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset bl;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            incr offset;
            Webgl.set native##vertexViewF32 !offset x2;
            incr offset;
            Webgl.set native##vertexViewF32 !offset y2;
            incr offset;
            Webgl.set native##vertexViewF32 !offset u1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset v1;
            incr offset;
            Webgl.set native##vertexViewF32 !offset (float_of_int !unit_);
            incr offset;
            Webgl.set native##vertexViewF32 !offset tint_effect;
            incr offset;
            Phaser.Pipeline.set_uint native##vertexViewU32 !offset tr;
            for value_index = 0 to Array.length !values - 1 do
              incr offset;
              Webgl.set native##vertexViewF32 !offset (value_at !values value_index)
            done;
            Phaser.Pipeline.set_vertex_count native (native##vertexCount + 3);
            (match Js.Nullable.toOption native##currentBatch with
            | None -> failwith {js|Phaser batch missing after texture binding|js}
            | Some batch ->
                Phaser.Pipeline.set_batch_count batch (native##vertexCount - batch##start));
            native##onBatch (Js.Undefined.fromOption (Js.Nullable.toOption object_));
            flushed);
      }]
  in
  extends definition base;
  let constructor =
    if multi then subclass definition
    else
      let single =
        [%mel.obj
          {
            initialize = definition##initialize;
            onBind = definition##onBind;
            destroy = definition##destroy;
          }]
      in
      single_extends single base;
      single_subclass single
  in
  { native = Phaser.construct constructor [| config |]; values }

let attributes extra =
  Array.append
    [|
      {
        Phaser.Pipeline.name = {js|inPosition|js};
        size = 2;
        kind = Phaser.Pipeline.float_type;
        normalized = false;
      };
      {
        Phaser.Pipeline.name = {js|inTexCoord|js};
        size = 2;
        kind = Phaser.Pipeline.float_type;
        normalized = false;
      };
      {
        Phaser.Pipeline.name = {js|inTexId|js};
        size = 1;
        kind = Phaser.Pipeline.float_type;
        normalized = false;
      };
      {
        Phaser.Pipeline.name = {js|inTintEffect|js};
        size = 1;
        kind = Phaser.Pipeline.float_type;
        normalized = false;
      };
      {
        Phaser.Pipeline.name = {js|inTint|js};
        size = 4;
        kind = Phaser.Pipeline.unsigned_byte;
        normalized = true;
      };
    |]
    extra

external replace_all : string -> string -> string -> string = "replaceAll" [@@mel.send]

let shader_count source count = replace_all source {js|%count%|js} (string_of_int count)

let source_texture (texture : Phaser.texture) =
  let sources = texture##source in
  if Array.length sources = 0 then None else Js.Nullable.toOption sources.(0)##glTexture

let bind_texture (native : Phaser.pipeline) texture unit_ =
  let texture =
    match texture with Some texture -> texture | None -> native##renderer##whiteTexture
  in
  let _ = native##bindTexture texture unit_ in
  ()

let restore_texture (native : Phaser.pipeline) =
  let textures = native##activeTextures in
  let texture = if Array.length textures > 0 then Some textures.(0) else None in
  bind_texture native texture 0

module Data (Data : sig
  type t
end) =
struct
  external get : Phaser.game_object -> Data.t = "pipelineData" [@@mel.get]

  external image : Phaser.image -> string -> Data.t -> bool -> Phaser.image = "setPipeline"
  [@@mel.send]

  external ellipse : Phaser.ellipse -> string -> Data.t -> bool -> Phaser.ellipse = "setPipeline"
  [@@mel.send]

  external graphics : Phaser.graphics -> string -> Data.t -> bool -> Phaser.graphics = "setPipeline"
  [@@mel.send]

  external target : Phaser.render_texture -> string -> Data.t -> bool -> Phaser.render_texture
    = "setPipeline"
  [@@mel.send]
end

external composite : Phaser.image -> string -> Phaser.image = "setPipeline" [@@mel.send]
