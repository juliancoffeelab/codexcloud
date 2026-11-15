type t
(** Logical camera state in CSS pixels. Phaser projection and DOM measurement consume immutable
    snapshots; navigation and bounds belong to this module. *)

type pose = { left : float; top : float; zoom : float }
type size = { width : float; height : float }
type bounds = { left : float; top : float; right : float; bottom : float }
type layout = { map : size; base_width : float; capacity_width : float; viewport : size }
type snapshot = { pose : pose; layout : layout; content : size; display_scale : float }
type navigation = Pan | Fit

val create : unit -> t
val snapshot : t -> snapshot
val initialized : t -> bool

val configure : t -> layout -> unit
(** Resize without clamping the preserved pose. Capacity-width changes preserve the horizontal map
    center. *)

val set_visible_bounds : t -> bounds -> unit
(** Clipping changes navigation limits without moving the camera. *)

val move_to : t -> navigation -> left:float -> top:float -> unit
val pan_by : t -> dx:float -> dy:float -> unit

val zoom_at : t -> zoom:float -> x:float -> y:float -> bool
(** Clamp zoom to the existing limits and atomically resolve its anchored pose against the content
    dimensions at that zoom. Returns whether zoom changed. *)

val reset : t -> unit
