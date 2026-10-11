(** Direct owner of Rust/WASM resources. Rule queries for rendering use a
    detached publication and cannot touch a mutable battle. *)
type factory
type battle
type view

(** Initialize the generated WASM module and validate its embedded catalog.
    A failed initialization releases any factory already constructed. *)
val create_factory : unit -> factory Js.Promise.t
val catalog : factory -> Catalog.t
(** Release every battle and the factory, exactly once, even if cleanup fails. *)
val dispose_factory : factory -> unit

(** Create or restore an exclusively owned handle. Publication failure releases
    that handle before propagating the error. Seeds must be unsigned 32-bit. *)
val create : factory -> Domain.start -> float -> battle
val restore : factory -> Domain.saved_game -> battle
val dispose : battle -> unit
val view : battle -> view
val state : view -> Domain.state
val inspection : view -> Domain.battle_inspection
val revision : view -> float

(** Dispatch at the current revision, replacing the publication only when state
    and inspection agree. Rejected commands preserve the current publication. *)
val dispatch : battle -> Domain.command -> Domain.effects
val enemy_step : battle -> Domain.ai_options -> Domain.effects
val save : battle -> Domain.saved_game
(** Transactionally restore a checkpoint under a fresh handle. The old handle
    stays usable if restore or initial publication fails. *)
val replace : battle -> Domain.saved_game -> unit

(** Constant-time entity lookups and display-only inspection queries. Returned
    data belongs to this immutable publication, including after later commands. *)
val tile : view -> string -> Domain.tile option
val unit : view -> string -> Domain.public_unit option
val at : view -> string -> Domain.public_unit option
val actions : view -> string -> Domain.unit_actions option
val neighbors : view -> string -> Domain.tile array
val reachable : view -> string -> float Js.Dict.t
val targets : view -> string -> Domain.public_unit array
val attack_options : view -> string -> string -> Domain.attack_details array
val best_attack : view -> string -> string -> Domain.attack_details option
val attack_details : view -> string -> string -> string -> Domain.attack_details option
val promotion_options : view -> string -> string array
val recruitment_options : view -> Domain.side -> string array
val recruit_tiles : view -> string -> string -> Domain.tile array
(** Movement cost by authored unit type; impassable terrain returns infinity,
    matching the native inspection. *)
val movement_cost : view -> string -> string -> float
val income : view -> Domain.side -> float
(** Convert the stable public identity [uN] to the native unsigned integer. *)
val unit_id : string -> float
