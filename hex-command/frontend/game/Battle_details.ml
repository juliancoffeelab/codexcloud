type report = { combat : Domain.combat_animation; shown : int; complete : bool; game : Native.view }

type t =
  | None_
  | Terrain of string
  | Unit of string
  | Attack of { attacker : string; target : string }
  | Combat_report of { report : report; selected : string option; tile : string option }

type focus = { selected : string option; target : string option; inspected : string option }

let empty_focus = { selected = None; target = None; inspected = None }

let focus details game =
  let tile id =
    Option.map (fun (unit : Domain.public_unit) -> Hex.key unit.tile) (Native.unit game id)
  in
  match details with
  | None_ -> empty_focus
  | Terrain tile -> { empty_focus with inspected = Some tile }
  | Unit unit -> { selected = Some unit; target = None; inspected = tile unit }
  | Attack { attacker; target } ->
      { selected = Some attacker; target = Some target; inspected = tile target }
  | Combat_report { selected; tile; _ } -> { selected; target = None; inspected = tile }

let report frame =
  {
    combat = frame.Combat_playback.combat;
    shown = frame.shown;
    complete = frame.complete;
    game = frame.before;
  }
