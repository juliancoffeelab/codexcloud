type t = { mutable suspended : bool }
type envelope = { checkpoint : Domain.saved_game; pending_enemy : bool }
type 'game loaded = { game : 'game; pending_enemy : bool; notice : string option; saves : t }

let version = 4
let key = "hex-command-battle"
let rejected_key = key ^ "-unrestored"
let create () = { suspended = false }
let resume owner = owner.suspended <- false

(* Identity bindings are confined to the persistence decoder. A checkpoint is
   passed to Rust unchanged; no object-to-JSON round trip occurs at the FFI. *)
external checkpoint_json : Domain.saved_game -> Js.Json.t = "%identity"
external checkpoint_native : Js.Json.t -> Domain.saved_game = "%identity"
external catalog_json : Domain.catalog_view -> Js.Json.t = "%identity"
external storage_get : string -> string Js.Nullable.t = "getItem" [@@mel.scope "localStorage"]
external storage_set : string -> string -> unit = "setItem" [@@mel.scope "localStorage"]
external safe_integer : float -> bool = "isSafeInteger" [@@mel.scope "Number"]
external has_own : 'a -> string -> bool = "hasOwn" [@@mel.scope "Object"]
external from_entries : (string * Js.Json.t) array -> Js.Json.t = "fromEntries" [@@mel.scope "Object"]

let invalid () = invalid_arg "Invalid saved battle"
let require condition = if not condition then invalid ()
let object_ value = match Js.Json.decodeObject value with Some dict -> dict | None -> invalid ()
let get value name = let dict = object_ value in if has_own dict name then Js.Dict.get dict name else None
let field value name = match get value name with Some v -> v | None -> invalid ()
let number value = match Js.Json.decodeNumber value with Some v -> v | None -> invalid ()
let string value = match Js.Json.decodeString value with Some v -> v | None -> invalid ()
let boolean value = match Js.Json.decodeBoolean value with Some v -> v | None -> invalid ()
let array value = match Js.Json.decodeArray value with Some v -> v | None -> invalid ()
let null value = Js.Json.classify value = Js.Json.JSONNull
let obj fields = Js.Json.object_ (Js.Dict.fromArray (Array.of_list fields))
let copy value = from_entries (Js.Dict.entries (object_ value))
let set value name entry = Js.Dict.set (object_ value) name entry

let find_map catalog state =
  let maps = array (field (catalog_json catalog) "maps") in
  match Array.find_opt (fun map -> field map "id" = field state "mapId") maps with
  | Some map -> map | None -> invalid ()

(* Both published revision-one developer layouts grow without replacing owned
   cells. Validate geometry before indexing so malformed tiles cannot disappear
   during expansion and become an apparently valid native checkpoint. *)
let expand_developer catalog value =
  let checkpoint = field value "checkpoint" in
  let state = field checkpoint "state" in ignore (object_ state);
  let value = copy value in set value "version" (Js.Json.number 4.);
  if get state "mapId" = Some (Js.Json.string "developer") &&
     get state "mapRevision" = Some (Js.Json.number 1.) then (
    let cols = number (field state "cols") and rows = number (field state "rows") in
    require ((cols = 6. && rows = 20.) || (cols = 13. && rows = 23.));
    let tiles = array (field state "tiles") in
    require (Array.length tiles = int_of_float (cols *. rows));
    let old = Js.Dict.empty () in
    let id col row = string_of_int (int_of_float col) ^ "," ^ string_of_int (int_of_float row) in
    Array.iter (fun tile ->
        let col = number (field tile "col") and row = number (field tile "row") in
        require (safe_integer col && safe_integer row && col >= 0. && col < cols && row >= 0. && row < rows);
        let tile_id = id col row in
        require (get tile "id" = Some (Js.Json.string tile_id) && Js.Dict.get old tile_id = None);
        Js.Dict.set old tile_id tile) tiles;
    let map = find_map catalog state in
    let terrain = Js.Dict.empty () in Array.iter (fun tile ->
        Js.Dict.set terrain (id (number (field tile "col")) (number (field tile "row"))) tile) (array (field map "terrain"));
    let width = int_of_float (number (field map "cols")) and height = int_of_float (number (field map "rows")) in
    let tiles = Array.init (width * height) (fun index ->
        let col = float_of_int (index mod width) and row = float_of_int (index / width) in
        let tile_id = id col row in
        match Js.Dict.get old tile_id with
        | Some tile -> tile
        | None -> match Js.Dict.get terrain tile_id with
          | Some tile -> copy tile
          | None -> obj ["id", Js.Json.string tile_id; "col", Js.Json.number col;
                         "row", Js.Json.number row; "terrain", Js.Json.string "plains"; "owner", Js.Json.null]) in
    let state = copy state in set state "mapRevision" (field map "revision");
    set state "cols" (field map "cols"); set state "rows" (field map "rows");
    set state "tiles" (Js.Json.array tiles);
    let checkpoint = copy checkpoint in set checkpoint "state" state;
    set value "checkpoint" checkpoint);
  value

let current value =
  require (number (field value "version") = float_of_int version);
  require (Array.for_all (fun name -> List.mem name ["version"; "checkpoint"; "pendingEnemy"]) (Js.Dict.keys (object_ value)));
  let pending_enemy = boolean (field value "pendingEnemy") in
  let checkpoint = field value "checkpoint" in
  let state = field checkpoint "state" in
  ignore (object_ state);
  require (get checkpoint "version" = None);
  require (not pending_enemy || (string (field state "turn") = "player" && null (field state "winner")));
  { checkpoint = checkpoint_native checkpoint; pending_enemy }

let migrate catalog input =
  ignore (object_ input);
  let value = if get input "version" = None && get input "engine" = Some (Js.Json.string "rust-wasm") then
      let value = copy input in set value "version" (Js.Json.number 1.); value else input in
  let original = number (field value "version") in
  require (safe_integer original && original >= 1. && original <= float_of_int version);
  let value = if original <> 1. then value else
      match get value "engine" with
      | Some engine when string engine = "rust-wasm" ->
        let saved = field value "saved" in require (number (field saved "version") = 1.);
        let checkpoint = from_entries (Array.of_list (Array.to_list (Js.Dict.entries (object_ saved)) |> List.filter (fun (key,_) -> key <> "version"))) in
        let pending = boolean (field value "pendingEnemy") in
        obj ["version", Js.Json.number 2.; "checkpoint", checkpoint; "pendingEnemy", Js.Json.boolean pending]
      | Some _ -> invalid_arg "Unknown historical engine"
      | None ->
        let state = Save_legacy.import catalog value in
        let map = find_map catalog state in
        set state "mapRevision" (field map "revision");
        obj ["version", Js.Json.number 2.; "checkpoint", obj ["seed", Js.Json.number 17.; "state", state]; "pendingEnemy", field value "pendingEnemy"] in
  let value = if number (field value "version") <> 2. then value else (
      let checkpoint = field value "checkpoint" in
      let state = field checkpoint "state" in ignore (object_ state);
      let state = copy state in
      let retired = get state "mapRevision" = Some (Js.Json.number 1.) &&
                    (get state "mapId" = Some (Js.Json.string "river") || get state "mapId" = Some (Js.Json.string "ashen-refuge")) in
      if retired then (
        let map = find_map catalog state in
        set state "mapRevision" (field map "revision"));
      let checkpoint = copy checkpoint in set checkpoint "state" state;
      let value = copy value in set value "version" (Js.Json.number 3.); set value "checkpoint" checkpoint; value) in
  let value = if number (field value "version") = 3. then expand_developer catalog value else value in
  current value, original <> float_of_int version

let write owner checkpoint ~pending_enemy =
  if not owner.suspended then (
    let value = obj ["version", Js.Json.number (float_of_int version); "checkpoint", checkpoint_json checkpoint; "pendingEnemy", Js.Json.boolean pending_enemy] in
    ignore (current value);
    let raw = Js.Json.stringify value in
    try storage_set key raw with _ -> ())

let read () = try Js.Nullable.toOption (storage_get key) with _ -> None
let preserve raw = try storage_set rejected_key raw; true with _ -> false

let load ~catalog ~restore ~fresh ~dispose ~checkpoint =
  match read () with
  | None -> { game = fresh (); pending_enemy = false; notice = None; saves = create () }
  | Some raw ->
    let restored = ref None in
    try
      let save, migrated = migrate catalog (Js.Json.parseExn raw) in
      let game = restore save.checkpoint in restored := Some game;
      let saves = create () in
      if migrated then write saves (checkpoint game) ~pending_enemy:save.pending_enemy;
      { game; pending_enemy = save.pending_enemy; notice = None; saves }
    with _ ->
      (match !restored with None -> () | Some game -> (try dispose game with _ -> ()));
      let preserved = preserve raw in
      let notice = if preserved then "The saved battle could not be restored. A new battle is ready."
        else "The saved battle could not be restored. The original is retained; start a new battle to enable saving." in
      { game = fresh (); pending_enemy = false; notice = Some notice; saves = { suspended = not preserved } }
