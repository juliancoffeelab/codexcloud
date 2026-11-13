module Side = Domain.Turn_phase
open Domain

let dictionary : unit -> 'a Js.Dict.t = [%mel.raw "() => Object.create(null)"]

type chapter = { source : mission; briefing : string; debrief : string option; hints : string list }

let mission catalog view (state : campaign_state) =
  let source =
    Array.get (Catalog.campaign catalog state.id).missions (int_of_float state.mission)
  in
  let story = (Native.inspection view).story in
  {
    source;
    briefing = Option.fold ~none:"" ~some:(fun (s : resolved_story) -> s.briefing) story;
    debrief = Option.bind story (fun (s : resolved_story) -> s.debrief);
    hints = Option.fold ~none:[] ~some:(fun (s : resolved_story) -> Array.to_list s.hints) story;
  }

let objectives view = (Native.inspection view).objectives

let goal_statuses view =
  Option.fold ~none:[]
    ~some:(fun (objectives : campaign_objectives) ->
      Array_ext.filter
        (fun (status : objective_status) -> status.visible)
        (Array.append objectives.goals objectives.survival)
      |> Array.to_list)
    (objectives view)

let optional_statuses view =
  Option.fold ~none:[]
    ~some:(fun (objectives : campaign_objectives) ->
      Array_ext.filter (fun (status : objective_status) -> status.visible) objectives.optional
      |> Array.to_list)
    (objectives view)

let objective_labels _catalog view =
  let labels = dictionary () in
  Option.iter
    (fun (objectives : campaign_objectives) ->
      Array.iter
        (fun (status : objective_status) ->
          if status.visible then
            Array.iter
              (fun tile ->
                Js.Dict.set labels tile
                  (Option.value ~default:[] (Js.Dict.get labels tile) @ [ status.progress.label ]))
              (Array.map Hex.key status.active_tiles))
        (Array.append objectives.goals objectives.optional))
    (objectives view);
  labels

let dialogue_pending =
  Option.fold ~none:false ~some:(fun (state : campaign_state) ->
      state.dialogue_index < float_of_int (Array.length state.dialogue))

let returns_with_story catalog (state : campaign_state) (unit : public_unit) =
  let chapters = (Catalog.campaign catalog state.id).missions in
  let next = int_of_float state.mission + 1 in
  next < Array.length chapters
  && Option.fold ~none:false
       ~some:(fun character ->
         let mission = Array.get chapters next in
         Array.exists
           (fun (arrival : companion_arrival) -> arrival.character = character)
           mission.arrivals
         || Array.exists
              (fun (incoming : starting_unit) ->
                incoming.side = Side.Player && incoming.character = Some character)
              mission.map.units)
       unit.character
