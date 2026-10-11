(** Immutable native 64×64 image metadata. Creating or reading a snapshot never
    fetches images, allocates canvases, or performs atlas extraction. *)
type frame = { url : string; width : float; height : float }
type t
val current : unit -> t
(** Zero-copy foreign dictionary, frozen together with each frame at publication.
    Consumers may retain its identity; mutation is prohibited at runtime. *)
val frames : t -> frame Js.Dict.t
val find : t -> string -> frame option
val frame_key : kind:string -> name:string -> ?side:string -> ?floor:string -> unit -> string
val frame : t -> kind:string -> name:string -> ?side:string -> ?floor:string -> unit -> frame option
val unit_frame : t -> side:string -> string -> frame option
val terrain_frame : t -> ?floor:string -> string -> frame option
val portrait_frame : t -> string -> frame option
val icon_markup : string -> string
