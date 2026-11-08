(* Frozen recipes used by the explicit v18 -> v29 conversions. These readers
   describe the recipe declarations in the corresponding upstream migrations;
   game-state fragments are decoded through the version-specific codecs. *)
open Migration_json

type 'a nullable = Missing | Null | Value of 'a

let optional decode value key = Option.map decode (get value key)

let nullable decode value key =
  match get value key with
  | None -> Missing
  | Some v when v = Js.Json.null -> Null
  | Some v -> Value (decode v)

let strings value = Array.map string (array value)
let dictionary decode value = Array.map (fun (k, v) -> (k, decode v)) (entries value)

type leader = { name : string; faction : string; ai_group : string }

type ('delta, 'faction) economy = {
  map_id : string;
  from_revision : float;
  to_revision : float;
  tiles : 'delta array;
  faction : 'faction;
  leader : leader;
}

let economy decode_tiles decode_factions =
  let value = Lazy.force Frozen_data.history_faction_economy_v20 in
  let leader = field value "leader" in
  {
    map_id = text value "mapId";
    from_revision = num value "fromRevision";
    to_revision = num value "toRevision";
    tiles = decode_tiles (field value "tiles");
    faction = (decode_factions (a [ field value "faction" ])).(0);
    leader =
      {
        name = text leader "name";
        faction = text leader "faction";
        ai_group = text leader "aiGroup";
      };
  }

type 'light light_entry = { tile : string; legacy_key : string; light : 'light }
type 'light lighting = { revision : float; lights : 'light light_entry array }

let lighting decode_light =
  dictionary
    (fun value ->
      {
        revision = num value "revision";
        lights =
          Array.map
            (fun entry ->
              {
                tile = text entry "tile";
                legacy_key = text entry "legacyKey";
                light = decode_light (field entry "light");
              })
            (array (field value "lights"));
      })
    (Lazy.force Frozen_data.history_tile_lights_v22)

type character = { id : string; name : string }
type registry = { characters : character array; aliases : (string * string) array }
type characters = { campaigns : (string * registry) array; maps : (string * string) array }

let characters =
  lazy
    (let value = Lazy.force Frozen_data.history_characters_v25 in
     {
       campaigns =
         dictionary
           (fun campaign ->
             {
               characters =
                 Array.map
                   (fun entry -> { id = text entry "id"; name = text entry "name" })
                   (array (field campaign "characters"));
               aliases = dictionary string (field campaign "aliases");
             })
           (field value "campaigns");
       maps = dictionary string (field value "maps");
     })

let boards = lazy (dictionary (dictionary string) (Lazy.force Frozen_data.history_tiles_v26))
let origins = lazy (dictionary (dictionary number) (Lazy.force Frozen_data.history_flags_v28))

type historical_member = {
  wire : Historical_types.wire;
  old_id : string;
  unit_type : string;
  character : string option;
  role : string option;
  items : string array;
}

type spawn = {
  event : string;
  area : string array;
  faction : string;
  ai_group : string nullable;
  units : historical_member array;
}

type hold = { event : string; tile : string; predicate : float }
type recruit = { unit : string; cost : float option }

type ritual = {
  id : string;
  sources : string array;
  radius : float;
  (* Upstream deliberately leaves this payload unknown until readFragment(light28). *)
  light : Js.Json.t;
  units : historical_member array;
  ai_group : string;
}

type events = {
  event_ids : string array;
  old_event_ids : string array;
  aliases : (string * string) array;
  choice_flags : (string * string) array;
  spawns : spawn array;
  waves : string array;
  holds : hold array;
  ritual : ritual option;
  recruitment_grants : (string * (string * recruit array) array) array;
  removed_transfers : string array;
}

let historical_member value =
  {
    wire =
      { Historical_types.empty_wire with order = Array.to_list (Array.map fst (entries value)) };
    old_id = text value "oldId";
    unit_type = text value "unitType";
    character = optional string value "character";
    role = optional string value "role";
    items = strings (field value "items");
  }

let events =
  lazy
    (dictionary
       (fun value ->
         {
           event_ids = strings (field value "eventIds");
           old_event_ids = strings (field value "oldEventIds");
           aliases = dictionary string (field value "aliases");
           choice_flags = dictionary string (field value "choiceFlags");
           spawns =
             Array.map
               (fun group ->
                 {
                   event = text group "event";
                   area = strings (field group "area");
                   faction = text group "faction";
                   ai_group = nullable string group "aiGroup";
                   units = Array.map historical_member (array (field group "units"));
                 })
               (array (field value "spawns"));
           waves = strings (field value "waves");
           holds =
             Array.map
               (fun hold ->
                 {
                   event = text hold "event";
                   tile = text hold "tile";
                   predicate = num hold "predicate";
                 })
               (array (field value "holds"));
           ritual =
             Option.map
               (fun ritual ->
                 {
                   id = text ritual "id";
                   sources = strings (field ritual "sources");
                   radius = num ritual "radius";
                   light = field ritual "light";
                   units = Array.map historical_member (array (field ritual "units"));
                   ai_group = text ritual "aiGroup";
                 })
               (present value "ritual");
           recruitment_grants =
             dictionary
               (dictionary (fun units ->
                    Array.map
                      (fun unit -> { unit = text unit "unit"; cost = optional number unit "cost" })
                      (array units)))
               (field value "recruitmentGrants");
           removed_transfers = strings (field value "removedTransfers");
         })
       (Lazy.force Frozen_data.history_events_v27))
