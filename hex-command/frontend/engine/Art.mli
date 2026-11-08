type frame = {
  url : string;
  width : float;
  height : float;
  anchor_x : float;
  anchor_y : float;
  z : float;
}
(** Native 64×64 image metadata. Creating or reading a snapshot never fetches images, allocates
    canvases, or performs atlas extraction. *)

type t

val current : unit -> t

val frames : t -> frame Js.Dict.t
(** A fresh caller-owned, mutable null-prototype dictionary. Allocates and copies one entry per
    frame; frame records are shared with the artwork. Replacing or removing dictionary entries never
    changes this artwork or other copies. Use [find] or the frame lookup functions for direct reads
    without copying the dictionary. *)

val with_overrides : t -> (string * frame) array -> t
(** Copies frame metadata and replaces selected entries, preserving the original artwork. *)

val find : t -> string -> frame option
val frame_key : kind:string -> name:string -> ?side:string -> ?floor:string -> unit -> string
val frame : t -> kind:string -> name:string -> ?side:string -> ?floor:string -> unit -> frame option
val unit_frame : t -> side:string -> string -> frame option
val terrain_frame : t -> ?floor:string -> string -> frame option
val portrait_frame : t -> string -> frame option
val icon_markup : string -> string
val icon_path : string -> string
val terrain_portrait : t -> string -> frame option

val keys : t -> string array
(** Snapshot of frame keys, without copying the metadata dictionary. *)
