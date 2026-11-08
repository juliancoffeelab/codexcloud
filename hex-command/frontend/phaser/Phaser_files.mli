type failure = { message : string; cause : Foreign_error.cause option }

val describe : failure -> string
val image : Phaser.loader -> string -> (Phaser.file -> unit) option -> Phaser.file

val audio :
  Phaser.loader -> string -> string -> Phaser.audio_context -> (failure -> unit) -> Phaser.file

external image_data : Phaser.file -> Phaser.image_element = "data" [@@mel.get]
external audio_length : Phaser.audio_buffer -> int = "length" [@@mel.get]
external audio_channels : Phaser.audio_buffer -> int = "numberOfChannels" [@@mel.get]
val image_failure : Phaser.file -> failure option
val admit : Phaser.file -> unit
val cancel : Phaser.loader -> Phaser.file -> unit

val settle : Phaser.loader -> Phaser.file -> unit
(** Settle a completed file without exposing the loader's private queue or state constants. *)

external on_key : Phaser.loader -> string -> ((string -> unit)[@u]) -> unit = "on" [@@mel.send]
external off_key : Phaser.loader -> string -> ((string -> unit)[@u]) -> unit = "off" [@@mel.send]

external on_error : Phaser.loader -> string -> ((Phaser.file -> unit)[@u]) -> unit = "on"
[@@mel.send]

external off_error : Phaser.loader -> string -> ((Phaser.file -> unit)[@u]) -> unit = "off"
[@@mel.send]
