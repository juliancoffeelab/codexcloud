type t
(** Indexed, detached metadata from the Rust-validated catalog. No rules are reconstructed in the
    browser; unknown authored identifiers fail explicitly. *)

val of_view : Domain.catalog_view -> t
(** Build indexes once from validated, detached metadata. Collections retain their ordinary OCaml
    representations and records are shared with the catalog. *)

val units : t -> Domain.wiki_unit list
val terrain : t -> Domain.wiki_terrain list
val races : t -> Domain.wiki_race list

val maps : t -> Domain.browser_map list
(** One map per identifier, taking the highest authored revision while preserving first occurrence
    order. Historical revisions remain available through [raw]. *)

val campaigns : t -> Domain.browser_campaign list
val unit : t -> string -> Domain.wiki_unit
val terrain_type : t -> string -> Domain.wiki_terrain
val terrain_role : t -> string -> Domain.Terrain_role.t
val race : t -> string -> Domain.wiki_race
val map : t -> string -> Domain.browser_map
val campaign : t -> string -> Domain.browser_campaign

val scenario_maps : t -> Domain.browser_map list
(** All command-addressable scenarios, including non-human layouts. Campaign-only maps remain
    outside this collection. *)

val mission_map : t -> Domain.browser_chapter -> Domain.browser_map
(** Resolve a chapter's map metadata. Its roster remains the mission layout's own [units], rather
    than the general scenario's starting roster. *)

val raw : t -> Domain.catalog_view
(** Complete runtime metadata, including non-human content and historical map revisions. *)

val terrain_roles : t -> Domain.Terrain_role.t Js.Dict.t
val item : t -> string -> Domain.wiki_item
