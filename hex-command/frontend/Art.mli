(** Immutable, completely decoded artwork. Atlas coordinates are confined to
    loading; rendering receives standalone image URLs and dimensions. *)
type style = Normal | Low_poly | Pixel
type frame = { url : string; width : float; height : float; pixel : bool }
type t
val styles : (style * string) array
val style_name : style -> string
val style_of_string : string -> style option
val saved_style : unit -> style
val save_style : style -> unit
val artwork : style -> t
val style : t -> style
val load : style -> bool Js.Promise.t
val find : t -> string -> frame option
val unit_frame : t -> side:string -> string -> frame option
val terrain_frame : t -> string -> frame option
val portrait_frame : t -> string -> frame option
val color : t -> string -> string
val sprite : t -> kind:string -> name:string -> x:float -> y:float -> width:float -> height:float -> ?class_name:string -> ?side:string -> unit -> string
val portrait : t -> string -> x:float -> y:float -> width:float -> height:float -> string
val icon_markup : ?class_name:string -> string -> string

