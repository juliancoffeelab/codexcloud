type t
type cancellation
type line = { character : string; speaker : string; text : string }

type readiness = {
  scope : string;
  ready : bool;
  completed : int;
  total : int;
  error : string Js.Nullable.t;
}

external create_raw : ((string -> Js.Json.t -> unit)[@u]) -> t = "create"
[@@mel.module "@presentation/voice/player.mjs"]

external enabled : t -> bool = "enabled" [@@mel.module "@presentation/voice/player.mjs"]

external set_enabled : t -> bool -> unit = "setEnabled"
[@@mel.module "@presentation/voice/player.mjs"]

external close : t -> unit = "close" [@@mel.module "@presentation/voice/player.mjs"]

external prepare_raw : t -> string -> line array -> unit = "prepare"
[@@mel.module "@presentation/voice/player.mjs"]

external needs_preparation : t -> string -> bool = "needsPreparation"
[@@mel.module "@presentation/voice/player.mjs"]

external speak_raw : t -> string -> line array -> int -> (string -> unit) -> cancellation = "speak"
[@@mel.module "@presentation/voice/player.mjs"]

external cancel : cancellation -> unit = "call" [@@mel.send]

external watch_opening_raw : t -> string -> line array -> int -> (readiness -> unit) -> cancellation
  = "watchOpening"
[@@mel.module "@presentation/voice/player.mjs"]

external retry : t -> unit = "retry" [@@mel.module "@presentation/voice/player.mjs"]

let create () = create_raw (fun[@u] name args -> Performance_trace.instant ~args name)

let line (value : Domain.dialogue) =
  {
    character = Utf8.to_js (Option.value ~default:"" value.character);
    speaker = Utf8.to_js value.speaker;
    text = Utf8.to_js value.text;
  }

let speak t scope lines index reveal =
  let cancellation = speak_raw t scope (Array.map line lines) index reveal in
  fun () -> cancel cancellation

let watch_opening t scope lines index receive =
  let cancellation = watch_opening_raw t scope (Array.map line lines) index receive in
  fun () -> cancel cancellation

let prepare_mission t scope catalog view =
  let rec speech nodes =
    Array.to_list nodes
    |> List.concat_map (function
      | Domain.Speech.Line value ->
          [
            {
              character = Utf8.to_js (Option.value ~default:"" value.character);
              speaker = Utf8.to_js (Option.value ~default:"" value.speaker);
              text = Utf8.to_js value.text;
            };
          ]
      | Domain.Speech.Branch branch -> speech branch.dialogue @ speech branch.alternative)
  in
  let rec actions values =
    Array.to_list values
    |> List.concat_map (function
      | Domain.Event_action.Say nodes -> speech nodes
      | Domain.Event_action.For_each_unit loop -> actions loop.actions
      | Domain.Event_action.For_each_mark loop -> actions loop.actions
      | _ -> [])
  in
  let lines =
    match Native.campaign view with
    | None -> []
    | Some progress ->
        let campaign = Catalog.campaign catalog progress.id in
        let mission = campaign.missions.(int_of_float progress.mission) in
        speech mission.intro
        @ (Array.to_list mission.scripted_events
          |> List.concat_map (fun (event : Domain.script_event) -> actions event.actions))
        @ speech mission.victory
        @
        if int_of_float progress.mission = Array.length campaign.missions - 1 then
          speech campaign.ending
        else []
  in
  prepare_raw t scope (Array.of_list lines)

let prepare t scope catalog view =
  if needs_preparation t scope then prepare_mission t scope catalog view
