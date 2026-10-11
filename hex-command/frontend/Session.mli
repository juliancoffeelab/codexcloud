(** Owns one battle, coherent immutable publications, undo checkpoints and deferred
    command lifetimes. Rendering receives data and a stable write capability only. *)
type t
type model = private {
  game : Native.view; catalog : Catalog.t; artwork : Art.t;
  selected : string option; target : string option; inspected : string option;
  focused : string; revision : int; status : string; sound : bool;
  can_undo : bool; failure : string option;
}
type actions = {
  choose : string -> unit; focus : string -> unit; clear : bool -> unit;
  attack : string -> unit; recruit : string -> string -> unit;
  advance : string -> unit; end_turn : unit -> unit Js.Promise.t;
  undo : unit -> unit; reset : string -> unit; start_campaign : string -> unit;
  next_mission : string array -> unit; retry : unit -> unit;
  dialogue : bool -> unit; previous_dialogue : unit -> unit;
  set_sound : bool -> unit Js.Promise.t;
}
type clock = float -> unit Js.Promise.t
(** Factory ownership stays with the embedding; failed session construction
    releases any acquired battle before propagating the error. *)
val create : ?clock:clock -> Native.factory -> t
(** Terminal publication failures retain the last coherent view and original
    saved bytes, cancel deferred work, and expose [failure] for reload UI. Further
    commands reject; retained models and subscriptions remain readable. *)
val view : t -> model
val commands : t -> actions
(** Immediate initial delivery and ordered reentrant publications. Initial callback
    failure rolls back registration; later failures cannot suppress other listeners. *)
val subscribe : t -> (model -> unit) -> (unit -> unit)
(** Only an acknowledgement of the current revision may release DOM/audio effects. *)
val rendered : t -> int -> unit
val initialize : t -> unit
(** Idempotent; invalidates pending work and releases the owned battle. *)
val close : t -> unit
val message : t -> string -> unit
val error : t -> exn -> unit
(** Unlock optional audio only from a trusted user gesture. *)
val gesture : t -> bool -> unit
val pause : clock
