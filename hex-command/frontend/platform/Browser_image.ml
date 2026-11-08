type t

external image : unit -> t = "Image" [@@mel.new]
external source : t -> string -> unit = "src" [@@mel.set]
external class_name : t -> string -> unit = "className" [@@mel.set]
external alt : t -> string -> unit = "alt" [@@mel.set]
external width : t -> int -> unit = "width" [@@mel.set]
external height : t -> int -> unit = "height" [@@mel.set]
external replace : Dom.element -> t -> unit = "replaceChildren" [@@mel.send]
external clear : Dom.element -> unit = "replaceChildren" [@@mel.send]
