type error =
  | Engine of Domain.Engine_error.t
  | Initialization of Domain.Wasm.Load_error.t
  | Module_failed of Domain.Wasm.Failure.t
  | Disposed
  | Invalid_request of string
  | Publication of Foreign_error.cause

exception Rejected of error
(** Lifecycle guards on otherwise total owner queries raise this typed exception. *)

val message : error -> string

type factory
(** Direct owner of Rust/WASM resources. Rule queries for rendering use a detached publication and
    cannot touch a mutable battle. *)

type battle
type view

val create_factory :
  ?documents:string Domain.Text_map.t -> unit -> (factory, error) result Js.Promise.t
(** Initialize the generated WASM module and validate its embedded catalog. Initialization is shared
    and idempotent across factories; failed initial loads may be retried. A failed catalog read
    releases any factory already constructed. A terminal module trap requires a page/module reload.
*)

val catalog : factory -> Catalog.t

val dispose_factory : factory -> unit
(** Release every battle and the factory, exactly once, even if cleanup fails. *)

val create : factory -> Domain.Start.t -> float -> (battle, error) result
(** Create or restore an exclusively owned handle. Publication failure releases that handle before
    propagating the error. Seeds must be unsigned 32-bit. *)

val restore : factory -> Domain.saved_game -> (battle, error) result
val dispose : battle -> unit
val view : battle -> view

val on_failure : battle -> (error -> unit) -> unit
(** Register the owner's terminal failure handler. An accepted mutation whose publication fails
    releases the handle, then reports the original error before returning Error. Retained detached
    views remain readable; live operations on released handles raise. A terminal module trap
    invalidates every factory and battle handle before invoking handlers, including reentrant
    cleanup. Sessions then publish terminal recovery from their handlers. The handler is released on
    disposal. *)

val state : view -> Domain.state

val combat : view -> Domain.combat_animation option
(** Transient resolved exchange, detached with this publication. *)

val presentation : view -> Domain.battlefield_presentation
val inspection : view -> Domain.battle_inspection
val revision : view -> float

val dispatch : battle -> Domain.Command.t -> (view, error) result
(** Return the accepted detached publication at the current revision, replacing it only when state
    and inspection agree. Rejected commands return Error and preserve the current publication.
    Native failures retain their typed error; accepted commands with failed inspection release the
    handle and notify its owner. *)

val choose_ai_command : battle -> Domain.ai_options -> (Domain.ai_decision, error) result
(** Read-only AI planning. Decision revisions must still match before dispatch. *)

val prepare_enemy_step : battle -> Domain.ai_options -> (unit, error) result
(** Prepare a read-only decision for this revision. It is reused by enemy_step and invalidated by
    dispatch, replacement, changed options or disposal. *)

val enemy_step : battle -> Domain.ai_options -> (view, error) result
val save : battle -> (Domain.saved_game, error) result

(** Serialize a trusted Rust checkpoint once, without a native-value round trip. *)

val save_json : battle -> (string, error) result
(** Serialized checkpoint as a host JavaScript string (not native UTF-8 bytes). *)

val state_json : view -> string
val actions_json : view -> string -> string option

val replace : battle -> Domain.saved_game -> (view, error) result
(** Transactionally restore a checkpoint under a fresh handle. The old handle stays usable if
    restore or initial publication fails. *)

val tile : view -> string -> Domain.tile option

val hex : view -> string -> Domain.hex
(** Constant-time entity lookups and display-only inspection queries. Returned data is detached from
    Rust and remains available after later commands. Collections use ordinary OCaml semantics;
    unchanged records can be shared between publications. *)

val unit : view -> string -> Domain.public_unit option
val at : view -> string -> Domain.public_unit option
val actions : view -> string -> Domain.unit_actions option

val neighbors : view -> string -> Domain.tile array
(** Fresh array containing shared detached tiles; replacing array entries does not alter the view.
*)

val reachable : view -> string -> float Domain.Text_map.t
val targets : view -> string -> Domain.public_unit array
val attack_options : view -> string -> string -> Domain.attack_details array
val best_attack : view -> string -> string -> Domain.attack_details option
val attack_details : view -> string -> string -> string -> Domain.attack_details option
val promotion_options : view -> string -> string array
val recruitment_options : view -> string -> string array
val recruit_tiles : view -> string -> string -> Domain.tile array

val movement_cost : view -> string -> string -> float
(** Movement cost by authored unit type; impassable terrain returns infinity, matching the native
    inspection. Static costs are shared within their factory. *)

val income : view -> string -> float

val unit_id : string -> float
(** Convert the stable public identity [uN] to the native unsigned integer. *)

val faction : view -> string -> Domain.unit_faction option
val controllable : view -> string -> bool
val visible : view -> Domain.public_unit -> bool
val ground_items : view -> Domain.ground_item array
val fogged : view -> string -> bool
val gold : Domain.gold -> string -> float
val unit_name : view -> string -> string option
val recruit_price : view -> string -> string -> float
val campaign : view -> Domain.campaign_state option
val enemy_step_recorded : battle -> Domain.ai_options -> (view * Domain.Command.t, error) result
val validate_undo : factory -> Js.Json.t -> (unit, error) result

val project_units : view -> Domain.public_unit array -> view
(** Display-only projection of unit health; never dispatched to the engine. *)

val units : view -> Domain.public_unit array
(** Current visible unit publication, shared across presentation queries. *)
