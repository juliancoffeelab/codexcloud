type t

(** Versioned persistence owner. Reads distinguish empty storage from blocked or damaged data. *)

type envelope = { checkpoint : Domain.saved_game; pending_enemy : bool; setup : Setup.t option }

type 'game loaded = {
  game : 'game;
  pending_enemy : bool;
  notice : string option;
  saves : t;
  setup : Setup.t;
  undo : Save_undo.entry list;
  entry : Js.Json.t option;
}

val version : int
val key : string
val rejected_key : string
val create : unit -> t
val resume : t -> unit
val close : t -> unit
val invalidate : t -> unit
val new_run : t -> unit
val subscribe : t -> (unit -> unit) -> unit
val set_undo : t -> Save_undo.entry list -> unit
val snapshot : ?source:Domain.state -> t -> string -> Js.Json.t
val status : t -> string * Save_error.t option * Js.Json.t option * bool * string
val problem : t -> Js.Json.t option
val awaiting : t -> bool
val active_run : t -> string option
val failed : t -> 'a -> unit
val saved : t -> unit Js.Promise.t
val queue : t -> (unit -> 'a Js.Promise.t) -> 'a Js.Promise.t
val migrate : Js.Json.t -> envelope * bool

val load :
  restore:(Domain.saved_game -> 'game) ->
  validate:(Js.Json.t -> unit) ->
  fresh:(unit -> 'game) ->
  dispose:('game -> unit) ->
  'game loaded Js.Promise.t

val restore_text :
  restore:(Domain.saved_game -> 'game) ->
  validate:(Js.Json.t -> unit) ->
  string ->
  string ->
  'game * Js.Json.t * Save_undo.entry list * bool

val write_json : ?setup:Setup.t -> ?source:Domain.state -> t -> string -> pending_enemy:bool -> unit
val write : ?setup:Setup.t -> t -> Domain.saved_game -> pending_enemy:bool -> unit
val list : t -> Js.Json.t array Js.Promise.t
val flash : t -> Js.Json.t option Js.Promise.t
val manual : t -> Setup.t -> string -> unit Js.Promise.t
val delete_progress : t -> string -> unit Js.Promise.t
val delete_run : t -> string -> unit Js.Promise.t
val prune : ?run:string -> unit -> unit Js.Promise.t
val inspect : t -> (Js.Json.t -> unit) -> Js.Json.t Js.Promise.t
val repair : t -> Js.Json.t -> unit Js.Promise.t

type retry_policy = Before_restore | After_cleanup

val retry_pending : retry_policy -> t -> (unit, Save_error.t) result Js.Promise.t
val mark_idle : t -> unit
val chapter_start : t -> unit
val capture : unit -> Js.Json.t Js.Promise.t
val position : t -> string -> float -> unit
val owns_position : t -> Js.Json.t -> bool

val restore_progress :
  t ->
  current:(unit -> unit) ->
  restore:(Domain.saved_game -> 'game) ->
  validate:(Js.Json.t -> unit) ->
  dispose:('game -> unit) ->
  install:('game -> Setup.t -> unit) ->
  string ->
  unit Js.Promise.t
(** Validate and durably fork a historical checkpoint before adopting it. Epoch checks fence
    asynchronous steps; failed work releases the restored battle. *)

val reload_target : t -> string -> (string option, string) result Js.Promise.t

val inspect_progress :
  t ->
  current:(unit -> unit) ->
  restore:(Domain.saved_game -> 'game) ->
  validate:(Js.Json.t -> unit) ->
  dispose:('game -> unit) ->
  inspect:('game -> 'a) ->
  string ->
  'a Js.Promise.t
