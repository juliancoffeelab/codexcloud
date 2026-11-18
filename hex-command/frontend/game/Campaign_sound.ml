let mission (mission : Domain.browser_chapter) ending =
  let found = Host_collections.Set.create () in
  let add cue =
    let _ = Host_collections.Set.add found cue in
    ()
  in
  let rec speech nodes =
    Array.iter
      (function
        | Domain.Speech.Line line -> Option.iter add line.sound
        | Domain.Speech.Branch branch ->
            speech branch.dialogue;
            speech branch.alternative)
      nodes
  in
  let rec actions values =
    Array.iter
      (function
        | Domain.Event_action.Sound sound -> add sound
        | Domain.Event_action.Say lines -> speech lines
        | Domain.Event_action.For_each_unit loop -> actions loop.actions
        | Domain.Event_action.For_each_mark loop -> actions loop.actions
        | _ -> ())
      values
  in
  speech mission.intro;
  speech mission.victory;
  speech ending;
  Array.iter (fun (event : Domain.script_event) -> actions event.actions) mission.scripted_events;
  Host_collections.Set.values_array found
