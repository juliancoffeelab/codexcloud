type t
(** Owns contact matching and gesture transitions; never reads mutable Phaser pointers. *)

type view = { left : float; top : float; zoom : float; canvas_left : float; canvas_top : float }

type command =
  | Pan of float * float
  | Zoom of float * float * float
  | Tap of Phaser_pointer.scene_position
  | Coast of float * float

type phase = Idle | Pressed | Panning | Pinching | Suppressed

type release_reason =
  | Activate
  | No_press
  | Wrong_contact
  | Cancelled
  | Dragged
  | Pan_only
  | Too_far
  | Contact_suppressed

type tracking = {
  pointer_id : int;
  left : float;
  top : float;
  x : float;
  y : float;
  vx : float;
  vy : float;
  time : float;
}

val create : unit -> t
val phase : t -> phase
val tracking : t -> tracking option
val press : t -> Phaser_pointer.press -> view -> active:Phaser_pointer.sample array -> bool
val move : t -> Phaser_pointer.sample -> active:Phaser_pointer.sample array -> command list

val release :
  t -> Phaser_pointer.sample -> active:Phaser_pointer.sample array -> release_reason * command list

val cancel : t -> active:Phaser_pointer.sample array -> unit
