external freeze : 'a -> 'a = "freeze" [@@mel.scope "Object"]

type t = {
  source : Domain.catalog_view;
  unit_index : Domain.unit_rule Js.Dict.t;
  terrain_index : Domain.terrain_definition Js.Dict.t;
  race_index : Domain.race_definition Js.Dict.t;
  map_index : Domain.map_definition Js.Dict.t;
  campaign_index : Domain.campaign Js.Dict.t;
  scenarios : Domain.map_definition array;
  visible : Domain.map_definition array;
}

let index key values =
  let result = Js.Dict.empty () in
  Array.iter (fun value ->
      let id = key value in
      if Js.Dict.get result id <> None then invalid_arg ("Duplicate catalog identifier: " ^ id);
      Js.Dict.set result id value) values;
  result

let of_view (source : Domain.catalog_view) =
  let campaign_maps = Js.Dict.empty () in
  Array.iter (fun (campaign : Domain.campaign) ->
      Array.iter (fun (mission : Domain.mission) -> Js.Dict.set campaign_maps (mission.Domain.map.id) true)
        ((campaign.Domain.missions))) ((source.Domain.campaigns));
  let scenarios = Array.of_list (Array.to_list ((source.Domain.maps))
                                 |> List.filter (fun (map : Domain.map_definition) -> Js.Dict.get campaign_maps ((map.Domain.id)) = None)) in
  let visible = scenarios in
  { source;
    unit_index = index (fun (unit : Domain.unit_rule) -> unit.id) ((source.Domain.units));
    terrain_index = index (fun (terrain : Domain.terrain_definition) -> (terrain.id :> string)) ((source.Domain.terrain));
    race_index = index (fun (race : Domain.race_definition) -> (race.id :> string)) ((source.Domain.races));
    map_index = index (fun (map : Domain.map_definition) -> map.id) ((source.Domain.maps));
    campaign_index = index (fun (campaign : Domain.campaign) -> campaign.id) ((source.Domain.campaigns));
    scenarios = freeze scenarios; visible = freeze visible }

let require index id = match Js.Dict.get index id with
  | Some value -> value
  | None -> invalid_arg ("Unknown catalog identifier: " ^ id)
let units t = (t.source.Domain.units)
let terrain t = (t.source.Domain.terrain)
let races t = (t.source.Domain.races)
let maps t = (t.source.Domain.maps)
let campaigns t = (t.source.Domain.campaigns)
let unit t id = require t.unit_index id
let terrain_type t id = require t.terrain_index id
let race t id = require t.race_index id
let map t id = require t.map_index id
let campaign t id = require t.campaign_index id
let battle_maps t = t.visible
let scenario_maps t = t.scenarios
let mission_map t (mission : Domain.mission) = map t (mission.Domain.map.id)

let raw t = t.source
