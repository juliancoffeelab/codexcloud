type context = { catalog : Catalog.t; artwork : Art.t; game : Native.view }

(* One resolved presentation owns both pane visibility and its content. Playback
   remains independent: an enemy animation need not replace the chosen inspection. *)
type content =
  | Terrain
  | Unit of Presentation.unit_details
  | Attack of Domain.public_unit * Domain.public_unit * string
  | Combat_report of Battle_details.report
  | Outcome of string * string

type contents = {
  content : content;
  terrain : Presentation.terrain_panel option;
  ground_items : Domain.item array;
}

type t = Closed | Open of contents

let resolve catalog game details revision =
  let focus = Battle_details.focus details game in
  let terminal = (Native.state game).winner <> None in
  (* Completed reports yield to the authoritative outcome, never the pre-combat
     snapshot retained by a report for actor names and factions. *)
  let report =
    match details with
    | Battle_details.Combat_report { report; _ } when (not report.complete) || not terminal ->
        Some report
    | _ -> None
  in
  let terrain =
    Presentation.terrain_panel catalog game ~selected:focus.selected ~inspected:focus.inspected
  in
  let ground_items =
    Native.ground_items game
    |> Array_ext.filter_map (fun (ground : Domain.ground_item) ->
        if Some (Hex.key ground.tile) = focus.inspected then Some ground.item else None)
  in
  let content =
    match report with
    | Some report -> Some (Combat_report report)
    | None -> (
        match
          Presentation.unit_panel catalog game ~selected:focus.selected ~target:focus.target
            ~revision
        with
        | Presentation.Hidden ->
            if terminal || (terrain = None && Array.length ground_items = 0) then None
            else Some Terrain
        | Presentation.Outcome (title, description) -> Some (Outcome (title, description))
        | Presentation.Details details -> Some (Unit details)
        | Presentation.Combat (attacker, defender, key) -> Some (Attack (attacker, defender, key)))
  in
  match content with None -> Closed | Some content -> Open { content; terrain; ground_items }

let closed = Closed
let is_open = function Closed -> false | Open _ -> true
let report = function Open { content = Combat_report report; _ } -> Some report | _ -> None

let panel = function
  | Closed | Open { content = Terrain; _ } -> Presentation.Hidden
  | Open { content = Unit details; _ } -> Presentation.Details details
  | Open { content = Attack (a, b, key); _ } -> Presentation.Combat (a, b, key)
  | Open { content = Combat_report report; _ } ->
      Presentation.Combat (report.combat.units.(0), report.combat.units.(1), "report")
  | Open { content = Outcome (title, description); _ } -> Presentation.Outcome (title, description)

let terrain = function Closed -> None | Open contents -> contents.terrain
let ground_items = function Closed -> [||] | Open contents -> contents.ground_items

let reveal_key = function
  | Closed -> "closed"
  | Open { content = Terrain; terrain; _ } ->
      "terrain:"
      ^ Option.fold ~none:""
          ~some:(fun (panel : Presentation.terrain_panel) -> panel.tile.id)
          terrain
  | Open { content = Unit details; _ } -> "unit:" ^ details.unit.id
  | Open { content = Attack (a, b, _); _ } -> "attack:" ^ a.id ^ ":" ^ b.id
  | Open { content = Combat_report report; _ } ->
      "report:" ^ report.combat.units.(0).id ^ ":" ^ report.combat.units.(1).id
  | Open { content = Outcome _; _ } -> "outcome"
