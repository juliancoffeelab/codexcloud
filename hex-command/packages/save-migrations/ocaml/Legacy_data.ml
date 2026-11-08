(* Typed, migration-owned projections of the frozen browser-era catalog. *)
open Migration_json
module T = Historical_types

type placement = { type_ : string; side : T.V2.side; name : string option; role : T.V2.role option }

type terrain = {
  col : float;
  row : float;
  terrain : string;
  owner : T.fragments_tiles11_item_owner;
}

type map = {
  id : string;
  revision : float;
  cols : float;
  rows : float;
  terrain : terrain array;
  units : placement array;
}

type wave = {
  id : string;
  legacy_index : float option;
  previously_deployed_before_revision : float option;
}

type goal =
  | Seal of string array
  | Hold of { tile : string; turns : float; start_event : string option }
  | Vigil of { tile : string; turns : float }
  | Escort
  | Other

type dialogue_patch = { intro : T.V2.dialogue array; victory : T.V2.dialogue array }
type event = { id : string; dialogue : T.V2.dialogue array; war_dialogue : T.V2.dialogue array }
type refuge = { escalation : T.V2.dialogue array; war_victory : T.V2.dialogue array }

type mission = {
  map : map;
  waves : wave array;
  goals : goal array;
  events : event array;
  refuge : refuge option;
  required_companions : string array;
  intro : T.V2.dialogue array;
  victory : T.V2.dialogue array;
  legacy_dialogue : T.V2.dialogue array;
  without_dain : dialogue_patch option;
  companion_losses : dialogue_patch array;
}

type campaign = {
  id : string;
  leader : string;
  missions : mission array;
  ending : T.V2.dialogue array;
}

type rule = {
  id : string;
  hp : float;
  experience_needed : float;
  family : string;
  costs : float option array;
}

type catalog = {
  maps : map array;
  campaigns : campaign array;
  rules : rule array;
  terrain_order : string array;
}

let optional decode value key = Option.map decode (get value key)

let optional_array decode value key =
  Option.value (optional (fun v -> Array.map decode (array v)) value key) ~default:[||]

let strings value = Array.map string (array value)
let side = function "player" -> `Player | "enemy" -> `Enemy | _ -> invalid "Invalid frozen side"

let role = function
  | "leader" -> `Leader
  | "courier" -> `Courier
  | "captive" -> `Captive
  | _ -> invalid "Invalid frozen role"

let owner value = if value = Js.Json.null then `Null else `String (string value)

let dialogue value =
  Historical_codec.read_fragments_native_state38_campaign_member1_dialogue_item value

let lines value key = optional_array dialogue value key
let patch value = { intro = lines value "intro"; victory = lines value "victory" }

let placement value =
  {
    type_ = text value "type";
    side = side (text value "side");
    name = optional string value "name";
    role = optional (fun v -> role (string v)) value "role";
  }

let terrain value =
  {
    col = num value "col";
    row = num value "row";
    terrain = text value "terrain";
    owner = owner (default value "owner" Js.Json.null);
  }

let map value =
  {
    id = text value "id";
    revision = Option.value (optional number value "revision") ~default:1.;
    cols = num value "cols";
    rows = num value "rows";
    terrain = Array.map terrain (array (field value "terrain"));
    units = optional_array placement value "units";
  }

let wave value =
  {
    id = text value "id";
    legacy_index = optional number value "legacyIndex";
    previously_deployed_before_revision = optional number value "previouslyDeployedBeforeRevision";
  }

let goal value =
  match text value "kind" with
  | "seal" -> Seal (strings (field value "tiles"))
  | "hold" ->
      Hold
        {
          tile = text value "tile";
          turns = num value "turns";
          start_event = Option.map string (present value "startEvent");
        }
  | "vigil" -> Vigil { tile = text value "tile"; turns = num value "turns" }
  | "escort" -> Escort
  | _ -> Other

let event value =
  {
    id = text value "id";
    dialogue = lines value "dialogue";
    war_dialogue = lines value "warDialogue";
  }

let mission ?layout value =
  let map =
    match layout with
    | None -> map (field value "map")
    | Some layout -> { layout with units = optional_array placement (field value "map") "units" }
  in
  let historical = get (Lazy.force Frozen_data.history_stages) map.id in
  let historical_waves =
    Option.fold ~none:[||] ~some:(fun value -> optional_array wave value "waves") historical
  in
  let waves =
    Array.map
      (fun (source : wave) ->
        match Array.find_opt (fun (old : wave) -> old.id = source.id) historical_waves with
        | None -> source
        | Some old ->
            {
              source with
              legacy_index = old.legacy_index;
              previously_deployed_before_revision =
                (match old.previously_deployed_before_revision with
                | None -> source.previously_deployed_before_revision
                | Some _ -> old.previously_deployed_before_revision);
            })
      (optional_array wave value "waves")
  in
  {
    map;
    waves;
    goals = optional_array goal value "goals";
    events = optional_array event value "events";
    refuge =
      optional
        (fun value ->
          { escalation = lines value "escalation"; war_victory = lines value "warVictory" })
        value "refuge";
    required_companions = Option.value (optional strings value "requiredCompanions") ~default:[||];
    intro = lines value "intro";
    victory = lines value "victory";
    legacy_dialogue = Option.fold ~none:[||] ~some:(fun value -> lines value "dialogue") historical;
    without_dain = optional patch value "withoutDain";
    companion_losses =
      Option.fold ~none:[||]
        ~some:(fun value -> Array.map (fun (_, value) -> patch value) (entries value))
        (get value "companionLosses");
  }

let campaign value =
  {
    id = text value "id";
    leader = text value "leader";
    missions = Array.map (fun value -> mission value) (array (field value "missions"));
    ending = lines value "ending";
  }

let rule value =
  {
    id = text value "id";
    hp = num value "hp";
    experience_needed =
      (match get value "experienceNeeded" with
      | None -> 0.
      | Some v when v = Js.Json.null -> 0.
      | Some v -> number v);
    family = text value "family";
    costs =
      Array.map
        (fun v -> if v = Js.Json.null then None else Some (number v))
        (array (field value "costs"));
  }

let catalog =
  lazy
    (let value = Save_history.catalog in
     {
       maps = Array.map map (array (field value "maps"));
       campaigns = Array.map campaign (array (field value "campaigns"));
       rules = Array.map rule (array (field value "units"));
       terrain_order = Array.map (fun value -> text value "id") (array (field value "terrain"));
     })

let ashen_refuge =
  lazy
    (let value = Lazy.force Frozen_data.history_ashen_refuge_v1 in
     {
       id = "ashen-refuge";
       revision = 1.;
       cols = num value "cols";
       rows = num value "rows";
       terrain = Array.map terrain (array (field value "tiles"));
       units = [||];
     })

(* The actual published Ferry preview. Upstream currently selects the enlarged
   catalog board, rejecting a valid original browser save before conversion. *)
let ferry_source = lazy (map (field (Lazy.force Frozen_data.history_lantern_ferry_v5) "source"))

let ferry_mission =
  lazy
    (mission ~layout:(Lazy.force ferry_source)
       (field (Lazy.force Frozen_data.history_lantern_ferry_v5) "mission"))

let ferry_leader_name = lazy (text (Lazy.force Frozen_data.history_lantern_ferry_v5) "leaderName")

let leader_projection kind =
  Option.map string
    (get (field (Lazy.force Frozen_data.history_lantern_ferry_v5) "leaderProjection") kind)
