type t
(** Indexed, detached metadata from the Rust-validated catalog. No rules are reconstructed in the
    browser; unknown authored identifiers fail explicitly. *)

val of_view : Domain.catalog_view -> t
(** Build indexes once from validated, detached metadata. Collections retain their ordinary OCaml
    representations and records are shared with the catalog. *)

val units : t -> Domain.unit_rule list
val terrain : t -> Domain.terrain_definition list
val races : t -> Domain.race_definition list

val maps : t -> Domain.map_definition list
(** One map per identifier, taking the last authored entry while preserving first occurrence order.
    Historical revisions remain available through [raw]. *)

val campaigns : t -> Domain.campaign list
val unit : t -> string -> Domain.unit_rule
val terrain_type : t -> string -> Domain.terrain_definition
val terrain_role : t -> string -> Domain.Terrain_role.t
val race : t -> string -> Domain.race_definition
val map : t -> string -> Domain.map_definition
val campaign : t -> string -> Domain.campaign

val battle_maps : t -> Domain.map_definition list
(** Visible scenario choices, excluding campaign-only layouts. Hidden developer scenarios remain
    valid for commands but do not appear in this menu. *)

val scenario_maps : t -> Domain.map_definition list
(** All command-addressable scenarios, including hidden developer layouts. Campaign-only maps remain
    outside this collection. *)

val mission_map : t -> Domain.mission -> Domain.map_definition
(** Resolve a chapter's map metadata. Its roster remains the mission layout's own [units], rather
    than the general scenario's starting roster. *)

val raw : t -> Domain.catalog_view
(** The factory-owned resolved catalog, for validated save migration. *)

val terrain_roles : t -> Domain.Terrain_role.t Js.Dict.t
