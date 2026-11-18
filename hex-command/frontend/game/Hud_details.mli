type t
(** Resolved inspection content. Only [resolve] constructs an open pane, so pane visibility and
    panel contents share the same outcome/report policy. *)

type context = { catalog : Catalog.t; artwork : Art.t; game : Native.view }

val closed : t
val resolve : Catalog.t -> Native.view -> Battle_details.t -> float -> t
val is_open : t -> bool
val report : t -> Battle_details.report option
val panel : t -> Presentation.panel
val terrain : t -> Presentation.terrain_panel option
val ground_items : t -> Domain.wiki_item array
val reveal_key : t -> string
