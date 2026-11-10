module Command = Domain.Command

type recording = {
  checkpoint : string;
  setup : Setup.t;
  mutable commands : Domain.Command.t list;
  mutable completed : bool;
}

type t = recording list ref

let create () = ref []
let clear turns = turns := []

let begin_turn turns checkpoint setup =
  let recording = { checkpoint; setup; commands = []; completed = false } in
  turns :=
    List.filteri (fun index _ -> index >= max 0 (List.length !turns - 4)) !turns @ [ recording ];
  recording

let record turn command = turn.commands <- command :: turn.commands
let complete turn = turn.completed <- true
let obj fields = Js.Json.object_ (Js.Dict.fromArray (Array.of_list fields))
let text value = Js.Json.string (Utf8.to_js value)
let number = Js.Json.number
let boolean = Js.Json.boolean
let optional f = Option.fold ~none:Js.Json.null ~some:f

let command value =
  let open Domain in
  let kind name fields = obj (("kind", text name) :: fields) in
  match value with
  | Command.Move { unit; tile } ->
      kind "move"
        [
          ("unit", number unit);
          ( "tile",
            obj
              [
                ("col", number (Int32.to_float tile.col)); ("row", number (Int32.to_float tile.row));
              ] );
        ]
  | Command.Attack { unit; target; weapon } ->
      kind "attack" [ ("unit", number unit); ("target", number target); ("weapon", text weapon) ]
  | Command.Recruit { unit; kind = type_; tile } ->
      kind "recruit"
        [
          ("unit", number unit);
          ("type", text type_);
          ( "tile",
            obj
              [
                ("col", number (Int32.to_float tile.col)); ("row", number (Int32.to_float tile.row));
              ] );
        ]
  | Command.Advance { unit; kind = type_ } ->
      kind "advance" [ ("unit", number unit); ("type", text type_) ]
  | Command.Drop_item { unit; item } ->
      kind "dropItem" [ ("unit", number unit); ("item", text item) ]
  | Command.End_turn -> kind "endTurn" []
  | Command.Dismiss_dialogue -> kind "dismissDialogue" []
  | Command.Recall_mission { units } ->
      kind "recallMission" [ ("units", Js.Json.array (Array.map number units)) ]
  | Command.Retry_mission -> kind "retryMission" []
  | Command.Previous_dialogue -> kind "previousDialogue" []
  | Command.Skip_dialogue -> kind "skipDialogue" []
  | Command.Set_unit_intent { unit; retreating; rebuilding; attacked; record_retreat } ->
      kind "setUnitIntent"
        [
          ("unit", number unit);
          ("retreating", optional boolean retreating);
          ("rebuilding", optional boolean rebuilding);
          ("attacked", boolean attacked);
          ("record_retreat", boolean record_retreat);
        ]

let download_file = Browser_download.json

let download ?browser_saves turns checkpoint setup (state : Domain.state) =
  let report =
    obj
      [
        ("format", text "hex-command-debug");
        ("version", number 1.);
        ("createdAt", Js.Json.string (Js.Date.toISOString (Js.Date.make ())));
        ("setup", Setup.to_json setup);
        ("checkpoint", Js.Json.parseExn checkpoint);
        ("browserSaves", Option.value ~default:Js.Json.null browser_saves);
        ("bootProcess", Boot_process.snapshot ());
        ( "enemyTurns",
          Js.Json.array
            (Array.of_list
               (List.map
                  (fun turn ->
                    obj
                      [
                        ("checkpoint", Js.Json.parseExn turn.checkpoint);
                        ("setup", Setup.to_json turn.setup);
                        ( "commands",
                          Js.Json.array (Array.of_list (List.map command (List.rev turn.commands)))
                        );
                        ("completed", boolean turn.completed);
                      ])
                  !turns)) );
      ]
  in
  download_file
    (Utf8.to_js
       ("hex-command-debug-" ^ state.map_id ^ "-turn-" ^ Js.Float.toString state.round ^ ".json"))
    (Js.Json.stringify report)
