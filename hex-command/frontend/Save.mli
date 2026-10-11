(** Browser persistence and historical compatibility. Native restoration is the
    final validation step: migrated bytes are written only after it succeeds. *)
type t
type envelope = { checkpoint : Domain.saved_game; pending_enemy : bool }
type 'game loaded = {
  game : 'game;
  pending_enemy : bool;
  notice : string option;
  saves : t;
}

val version : int
val key : string
val rejected_key : string

(** A new owner enables persistence for an explicitly created battle. A recovery
    owner returned by [load] remains suspended when rejected bytes cannot be
    backed up, so ordinary play cannot overwrite the original. *)
val create : unit -> t

(** Explicitly starting a new battle releases rejected-save protection. Ordinary
    commands never call this; a failed backup stays protected throughout play. *)
val resume : t -> unit

(** Pure migration of untrusted persistence data. It preserves native planner
    continuations and owns every historical object that it changes. Revision-one
    developer maps expand only from the two published, complete geometries;
    existing cells, units, ownership and progress remain intact. *)
val migrate : Domain.catalog_view -> Js.Json.t -> envelope * bool

(** Optional browser storage never prevents play. Rejected saves retain their
    exact original bytes; failed backup suspends writes for the recovery battle.
    [dispose] releases a restored battle if later restoration work fails. *)
val load :
  catalog:Domain.catalog_view ->
  restore:(Domain.saved_game -> 'game) ->
  fresh:(unit -> 'game) ->
  dispose:('game -> unit) ->
  checkpoint:('game -> Domain.saved_game) ->
  'game loaded

(** Write the current envelope. Compatibility logic is never run on this path;
    storage exceptions are ignored, while invalid checkpoints fail explicitly. *)
val write : t -> Domain.saved_game -> pending_enemy:bool -> unit
