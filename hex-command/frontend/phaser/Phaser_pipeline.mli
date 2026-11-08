type t

external native : t -> Phaser.pipeline = "native" [@@mel.get]
val set_values : t -> float array -> unit

val create :
  ?multi:bool ->
  ?attributes:Phaser.Pipeline.attribute array ->
  game:Phaser.game ->
  vertex:string ->
  fragment:string ->
  bind:(Phaser.pipeline -> float array ref -> Phaser.game_object option -> unit) ->
  ?dispose:(unit -> unit) ->
  unit ->
  t

val attributes : Phaser.Pipeline.attribute array -> Phaser.Pipeline.attribute array
val shader_count : string -> int -> string
val source_texture : Phaser.texture -> Phaser.gl_texture option
val bind_texture : Phaser.pipeline -> Phaser.gl_texture option -> int -> unit
val restore_texture : Phaser.pipeline -> unit

module Data (Data : sig
  type t
end) : sig
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
