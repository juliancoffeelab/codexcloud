let dictionary : unit -> 'a Js.Dict.t = [%mel.raw {js|() => Object.create(null)|js}]

type t = {
  source : Domain.catalog_view;
  item_index : Domain.wiki_item Js.Dict.t;
  unit_index : Domain.wiki_unit Js.Dict.t;
  terrain_index : Domain.wiki_terrain Js.Dict.t;
  race_index : Domain.wiki_race Js.Dict.t;
  map_index : Domain.browser_map Js.Dict.t;
  campaign_index : Domain.browser_campaign Js.Dict.t;
  resolved_maps : Domain.browser_map list;
  scenarios : Domain.browser_map list;
}

let index key values =
  let result = dictionary () in
  Array.iter
    (fun value ->
      let id = key value in
      if Js.Dict.get result id <> None then invalid_arg {j|Duplicate catalog identifier: $(id)|j};
      Js.Dict.set result id value)
    values;
  result

(* Keep the highest authored revision while preserving first occurrence order. *)
let map_projection (maps : Domain.browser_map array) =
  let result = dictionary () in
  let identifiers = ref [] in
  Array.iter
    (fun (map : Domain.browser_map) ->
      if Js.Dict.get result map.id = None then identifiers := map.id :: !identifiers;
      match Js.Dict.get result map.id with
      | Some (previous : Domain.browser_map) when previous.Domain.revision > map.revision -> ()
      | _ -> Js.Dict.set result map.id map)
    maps;
  let resolved = List.rev_map (fun id -> Option.get (Js.Dict.get result id)) !identifiers in
  (result, resolved)

let of_view (source : Domain.catalog_view) =
  let map_index, resolved_maps = map_projection source.maps in
  let campaign_maps = dictionary () in
  Array.iter
    (fun (campaign : Domain.browser_campaign) ->
      Array.iter
        (fun (mission : Domain.browser_chapter) ->
          Js.Dict.set campaign_maps mission.Domain.map.id true)
        campaign.Domain.missions)
    source.Domain.campaigns;
  let scenarios =
    List.filter
      (fun (map : Domain.browser_map) -> Js.Dict.get campaign_maps map.id = None)
      resolved_maps
  in
  {
    source;
    item_index = index (fun (item : Domain.wiki_item) -> item.id) source.Domain.items;
    unit_index = index (fun (unit : Domain.wiki_unit) -> unit.id) source.Domain.units;
    terrain_index =
      index
        (fun (terrain : Domain.wiki_terrain) -> Enum.string_of_terrain terrain.id)
        source.Domain.terrain;
    race_index =
      index (fun (race : Domain.wiki_race) -> Enum.string_of_race race.id) source.Domain.races;
    map_index;
    resolved_maps;
    campaign_index =
      index (fun (campaign : Domain.browser_campaign) -> campaign.id) source.Domain.campaigns;
    scenarios;
  }

let require index id =
  match Js.Dict.get index id with
  | Some value -> value
  | None -> invalid_arg {j|Unknown catalog identifier: $(id)|j}

let units t = Array.to_list t.source.Domain.units
let terrain t = Array.to_list t.source.Domain.terrain
let races t = Array.to_list t.source.Domain.races
let maps t = t.resolved_maps
let campaigns t = Array.to_list t.source.Domain.campaigns
let item t id = require t.item_index id
let unit t id = require t.unit_index id
let terrain_type t id = require t.terrain_index id
let terrain_role t id = (terrain_type t id).Domain.role
let race t id = require t.race_index id
let map t id = require t.map_index id
let campaign t id = require t.campaign_index id
let scenario_maps t = t.scenarios
let mission_map t (mission : Domain.browser_chapter) = map t mission.Domain.map.id
let raw t = t.source

let terrain_roles t =
  let roles = dictionary () in
  List.iter
    (fun (terrain : Domain.wiki_terrain) ->
      Js.Dict.set roles (Enum.string_of_terrain terrain.id) terrain.role)
    (terrain t);
  roles
