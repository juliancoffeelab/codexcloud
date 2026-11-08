(* Frozen migration-owned metadata. No engine/catalog reads occur here. *)
let snapshot = Lazy.force Frozen_data.history_catalog_v6

let leader_types =
  Lazy.force Frozen_data.history_type_leaders_v5
  |> Js.Json.decodeArray |> Option.get
  |> Array.map (fun value -> Option.get (Js.Json.decodeString value))

let obj values = Js.Json.object_ (Js.Dict.fromArray (Array.of_list values))
let object_ value = Option.get (Js.Json.decodeObject value)
let array value = Option.get (Js.Json.decodeArray value)
let field value key = Option.get (Js.Dict.get (object_ value) key)
let copy value = obj (Array.to_list (Js.Dict.entries (object_ value)))
let set value key entry = Js.Dict.set (object_ value) key entry
let str = Js.Json.string

let maps =
  Array.map
    (fun (id, source) ->
      let map = copy source in
      set map "id" (str id);
      set map "terrain"
        (Js.Json.array
           (Array.map
              (fun source ->
                let row = array source in
                let col = Option.get (Js.Json.decodeNumber row.(0))
                and line = Option.get (Js.Json.decodeNumber row.(1)) in
                let tile =
                  obj
                    [
                      ("id", str (Js.Float.toString col ^ "," ^ Js.Float.toString line));
                      ("col", row.(0));
                      ("row", row.(1));
                      ("terrain", row.(2));
                      ("owner", row.(3));
                    ]
                in
                List.iter
                  (fun (index, key) ->
                    if Js.Json.classify row.(index) <> Js.Json.JSONNull then
                      set tile key row.(index))
                  [ (4, "floor"); (5, "connections"); (6, "deadEnds") ];
                tile)
              (array (field source "terrain"))));
      map)
    (Js.Dict.entries (object_ (field snapshot "maps")))

let find_map id = Option.get (Array.find_opt (fun map -> field map "id" = id) maps)

let catalog =
  obj
    [
      ("maps", Js.Json.array maps);
      ( "units",
        Js.Json.array
          (Array.map
             (fun (id, source) ->
               let unit = copy source in
               set unit "id" (str id);
               set unit "leader" (Js.Json.boolean (Array.mem id leader_types));
               unit)
             (Js.Dict.entries (object_ (field snapshot "units")))) );
      ( "terrain",
        Js.Json.array
          (Array.map (fun id -> obj [ ("id", id) ]) (array (field snapshot "terrainOrder"))) );
      ( "campaigns",
        Js.Json.array
          (Array.map
             (fun source ->
               let campaign = copy source in
               set campaign "missions"
                 (Js.Json.array
                    (Array.map
                       (fun source ->
                         let mission = copy source in
                         let original = field source "map" in
                         let map = copy (find_map (field original "id")) in
                         set map "units" (field original "units");
                         set mission "map" map;
                         mission)
                       (array (field source "missions"))));
               campaign)
             (array (field snapshot "campaigns"))) );
    ]
