module Command = Domain.Command

type recording = {
  checkpoint : string;
  setup : Setup.t;
  learned_model : string option;
  mutable commands : Domain.Command.t list;
  mutable completed : bool;
}

type t = recording list ref

let create () = ref []
let clear turns = turns := []

let begin_turn ?learned_model turns checkpoint setup =
  let recording = { checkpoint; setup; learned_model; commands = []; completed = false } in
  turns :=
    List.filteri (fun index _ -> index >= max 0 (List.length !turns - 4)) !turns @ [ recording ];
  recording

let record turn command = turn.commands <- command :: turn.commands
let complete turn = turn.completed <- true
let obj fields = Js.Json.object_ (Js.Dict.fromArray (Array.of_list fields))
let text value = Js.Json.string value
let number = Js.Json.number
let boolean = Js.Json.boolean
let optional f = Option.fold ~none:Js.Json.null ~some:f

let command value =
  let open Domain in
  let kind name fields = obj (({js|kind|js}, text name) :: fields) in
  match value with
  | Command.Move { unit; tile } ->
      kind {js|move|js}
        [
          ({js|unit|js}, number unit);
          ( {js|tile|js},
            obj
              [
                ({js|col|js}, number (Int32.to_float tile.col));
                ({js|row|js}, number (Int32.to_float tile.row));
              ] );
        ]
  | Command.Attack { unit; target; weapon } ->
      kind {js|attack|js}
        [
          ({js|unit|js}, number unit); ({js|target|js}, number target); ({js|weapon|js}, text weapon);
        ]
  | Command.Recruit { unit; kind = type_; tile } ->
      kind {js|recruit|js}
        [
          ({js|unit|js}, number unit);
          ({js|type|js}, text type_);
          ( {js|tile|js},
            obj
              [
                ({js|col|js}, number (Int32.to_float tile.col));
                ({js|row|js}, number (Int32.to_float tile.row));
              ] );
        ]
  | Command.Advance { unit; kind = type_ } ->
      kind {js|advance|js} [ ({js|unit|js}, number unit); ({js|type|js}, text type_) ]
  | Command.Drop_item { unit; item } ->
      kind {js|dropItem|js} [ ({js|unit|js}, number unit); ({js|item|js}, text item) ]
  | Command.End_turn -> kind {js|endTurn|js} []
  | Command.Dismiss_dialogue -> kind {js|dismissDialogue|js} []
  | Command.Recall_mission { units } ->
      kind {js|recallMission|js} [ ({js|units|js}, Js.Json.array (Array.map number units)) ]
  | Command.Retry_mission -> kind {js|retryMission|js} []
  | Command.Previous_dialogue -> kind {js|previousDialogue|js} []
  | Command.Skip_dialogue -> kind {js|skipDialogue|js} []
  | Command.Set_unit_intent { unit; retreating; rebuilding; attack_spent; record_retreat } ->
      kind {js|setUnitIntent|js}
        [
          ({js|unit|js}, number unit);
          ({js|retreating|js}, optional boolean retreating);
          ({js|rebuilding|js}, optional boolean rebuilding);
          ({js|attack_spent|js}, boolean attack_spent);
          ({js|record_retreat|js}, boolean record_retreat);
        ]

let download_file = Browser_download.json

let download ?browser_saves turns checkpoint setup (state : Domain.state) =
  let report =
    obj
      [
        ({js|format|js}, text {js|hex-command-debug|js});
        ({js|version|js}, number 1.);
        ({js|createdAt|js}, Js.Json.string (Js.Date.toISOString (Js.Date.make ())));
        ({js|setup|js}, Setup.to_json setup);
        ({js|checkpoint|js}, Js.Json.parseExn checkpoint);
        ({js|browserSaves|js}, Option.value ~default:Js.Json.null browser_saves);
        ({js|bootProcess|js}, Boot_process.snapshot ());
        ( {js|enemyTurns|js},
          Js.Json.array
            (Array.of_list
               (List.map
                  (fun turn ->
                    obj
                      [
                        ({js|checkpoint|js}, Js.Json.parseExn turn.checkpoint);
                        ({js|setup|js}, Setup.to_json turn.setup);
                        ({js|learnedModel|js}, optional text turn.learned_model);
                        ( {js|commands|js},
                          Js.Json.array (Array.of_list (List.map command (List.rev turn.commands)))
                        );
                        ({js|completed|js}, boolean turn.completed);
                      ])
                  !turns)) );
      ]
  in
  download_file
    (let js_text_part_1 = state.map_id in
     let js_text_part_3 = Js.Float.toString state.round in
     {j|hex-command-debug-$(js_text_part_1)-turn-$(js_text_part_3).json|j})
    (Js.Json.stringify report)
