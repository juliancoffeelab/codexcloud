type chapter = {
  source : Domain.mission;
  briefing : string;
  debrief : string option;
  hints : string list;
}

(** Campaign display policy. Resolves authored chapter overrides without mutating catalog data, and
    formats objective progress from one detached state. *)

val mission : Catalog.t -> Native.view -> Domain.campaign_state -> chapter
val objectives : Native.view -> Domain.campaign_objectives option
val goal_statuses : Native.view -> Domain.objective_status list
val optional_statuses : Native.view -> Domain.objective_status list
val objective_labels : Catalog.t -> Native.view -> string list Js.Dict.t
val dialogue_pending : Domain.campaign_state option -> bool
val returns_with_story : Catalog.t -> Domain.campaign_state -> Domain.public_unit -> bool
