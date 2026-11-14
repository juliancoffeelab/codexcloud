type view = {
  game : Native.view;
  catalog : Catalog.t;
  artwork : Art.t;
  selected : string option;
  target : string option;
  inspected_tile : string option;
  recruitment : bool;
  session_version : int;
  sound : bool;
  follow_ai : bool;
  ai_focus : string array;
  action : Action_presentation.token option;
  combat_playback : Combat_playback.frame option;
}

type actions = { choose : string -> unit; clear_selection : bool -> unit }

type progress = {
  identity : string;
  phase : string;
  loaded : int;
  total : int;
  audio_loaded : int;
  audio_total : int;
  error : string option;
}

type mount_options = {
  parent : Dom.element;
  actions : unit -> actions;
  describe : string -> unit;
  previews : Terrain_preview.t;
  progress : progress -> unit;
  failed : string -> unit;
}

module Commander : module type of Browser_dom.Event (struct
  type t = int Js.Nullable.t
end)
