(** Campaign display policy. Resolves authored chapter overrides without mutating
    catalog data, and formats objective progress from one detached state. *)
type chapter = { source : Domain.mission; briefing : string; debrief : string option;
                 hints : string array; intro : Domain.dialogue array;
                 victory : Domain.dialogue array; recruits : string array }
type progress = { label : string; current : float; total : float;
                  done_ : bool; tiles : string array }
val mission : Catalog.t -> Domain.campaign_state -> chapter
val goal_tiles : Domain.goal -> string array
val progress : Catalog.t -> Native.view -> Domain.goal -> progress
val active_tiles : Catalog.t -> Native.view -> Domain.goal -> string array
val objective_labels : Catalog.t -> Native.view -> string array Js.Dict.t
val dialogue_pending : Domain.campaign_state option -> bool
val returns_with_story : Catalog.t -> Domain.campaign_state -> Domain.public_unit -> bool
val ends_refuge_mediation : Catalog.t -> Domain.state -> Domain.public_unit -> bool
