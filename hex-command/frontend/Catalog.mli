(** Indexed, immutable metadata from the Rust-validated catalog. No rules are
    reconstructed in the browser; unknown authored identifiers fail explicitly. *)
type t

(** Build indexes once for one factory's resolved catalog. *)
val of_view : Domain.catalog_view -> t
val units : t -> Domain.unit_rule array
val terrain : t -> Domain.terrain_definition array
val races : t -> Domain.race_definition array
val maps : t -> Domain.map_definition array
val campaigns : t -> Domain.campaign array
val unit : t -> string -> Domain.unit_rule
val terrain_type : t -> string -> Domain.terrain_definition
val race : t -> string -> Domain.race_definition
val map : t -> string -> Domain.map_definition
val campaign : t -> string -> Domain.campaign

(** Visible scenario choices, excluding campaign-only layouts. Hidden developer
    scenarios remain valid for commands but do not appear in this menu. *)
val battle_maps : t -> Domain.map_definition array
(** All command-addressable scenarios, including hidden developer layouts.
    Campaign-only maps remain outside this collection. *)
val scenario_maps : t -> Domain.map_definition array

(** Resolve a chapter's map metadata. Its roster remains the mission layout's
    own [units], rather than the general scenario's starting roster. *)
val mission_map : t -> Domain.mission -> Domain.map_definition

(** The factory-owned immutable resolved catalog, for validated save migration. *)
val raw : t -> Domain.catalog_view
