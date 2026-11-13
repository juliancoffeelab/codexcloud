type error = Native of Native.error | Rejected of string | Foreign of Foreign_error.cause

exception Failed of error

val error_message : error -> string

type t
(** Owns one battle, cached detached publications, undo checkpoints and one cancellable enemy task
    per phase. Accepted native mutations are immediately adopted into [model]; display reads never
    consult the live owner. Rendering receives data and a stable write capability only. *)

type model = private {
  game : Native.view;
  interaction_game : Native.view;
  session_version : int;
  action : Action_presentation.token option;
  combat_playback : Combat_playback.frame option;
  settling : bool;
  catalog : Catalog.t;
  artwork : Art.t;
  details : Battle_details.t;
  map_focus : Battle_details.focus;
  hud : Hud_details.t;
  treasury : string;
  enemy_combat_details : bool;
  focused : string;
  revision : int;
  status : string;
  sound : bool;
  can_undo : bool;
  failure : string option;
  setup : Setup.t;
  completed_turns : float;
  persistence : string;
  save_failure : Save_error.t option;
  load_problem : Js.Json.t option;
  awaiting_new_game : bool;
  active_run : string option;
}

type actions = {
  choose : string -> unit;
  focus : string -> unit;
  clear : bool -> unit;
  drop : string -> string -> unit;
  attack : string -> unit;
  recruit : string -> string -> unit;
  advance : string -> unit;
  end_turn : unit -> (unit, error) result Js.Promise.t;
  undo : unit -> unit;
  reset : ?setup:Setup.t -> ?armies:string * string -> string -> unit;
  start_campaign : ?setup:Setup.t -> string -> unit;
  next_mission : string list -> unit;
  retry : unit -> unit;
  dialogue : bool -> unit;
  previous_dialogue : unit -> unit;
  set_sound : bool -> unit Js.Promise.t;
  download_debug_report : unit -> unit;
}

type clock = float -> unit Js.Promise.t

(** Factory ownership stays with the embedding; failed session construction releases any acquired
    battle before propagating the error. *)

val create_async : ?clock:clock -> Native.factory -> t Js.Promise.t
val create_with_audio : ?clock:clock -> Phaser_sound.t -> Native.factory -> t Js.Promise.t

val view : t -> model
(** Terminal publication failures retain the last coherent view and original saved bytes, cancel
    deferred work, and expose [failure] for reload UI. Further commands reject; retained models and
    subscriptions remain readable. *)

val commands : t -> actions

val subscribe : t -> (model -> unit) -> unit -> unit
(** Immediate initial delivery and ordered reentrant publications. Initial callback failure rolls
    back registration; later failures cannot suppress other listeners. *)

val rendered : t -> int -> unit
(** Only an acknowledgement of the current revision may release DOM/audio effects. *)

val initialize : t -> unit
(** Idempotent initialization reuses a running opening phase. Internal phase requests share its
    completion; public [end_turn] still requires the active player turn. Replacement, failure and
    close cancel ownership centrally. *)

val close : t -> unit
(** Idempotent; invalidates pending work and releases the owned battle. *)

val message : t -> string -> unit
val error : t -> exn -> unit

val gesture : t -> bool -> unit
(** Unlock optional audio only from a trusted user gesture. *)

val pause : clock

val move_unit : t -> string -> string -> unit
(** Explicit command identities for integrations; rejected commands raise. *)

val attack_unit : t -> string -> string -> string option -> unit
val recruit_unit : t -> string -> string -> string -> unit
val advance_unit : t -> string -> string -> unit
val scene_audio : t -> unit
val narrate : t -> Native.view -> unit
val saved : t -> unit Js.Promise.t
val saved_progress : t -> Js.Json.t array Js.Promise.t
val saved_flash : t -> Js.Json.t option Js.Promise.t
val save_game : t -> unit Js.Promise.t
val saved_summary : t -> string -> Js.Json.t Js.Promise.t
val restore_progress : t -> string -> unit Js.Promise.t
val delete_saved_progress : t -> string -> unit Js.Promise.t
val delete_saved_run : t -> string -> unit Js.Promise.t
val repair_inventory : t -> Js.Json.t Js.Promise.t
val repair_saved_data : t -> Js.Json.t -> unit Js.Promise.t
val retry_saved_progress : t -> Js.Json.t option option -> bool Js.Promise.t
val prepare_audio : t -> unit Js.Promise.t
val menu_cue : ?cue:Phaser_sound.hud_cue -> t -> bool Js.Promise.t
val renderer_failed : t -> string -> unit
val prepare_reload : t -> string -> unit Js.Promise.t
val trace_context : t -> Js.Json.t
val set_enemy_combat_details : t -> bool -> unit
