(* Offline authoring client. Rust resolves conditions; the game's selectors,
   complete-frame painter, fog and lighting own the rendered presentation. *)
module J = struct
  let object_ value =
    match Js.Json.decodeObject value with
    | Some v -> v
    | None -> invalid_arg {js|Expected object|js}

  let field value key =
    match Js.Json.decodeObject value with
    | None -> Js.Json.null
    | Some v -> Option.value ~default:Js.Json.null (Js.Dict.get v key)

  let has value key = field value key <> Js.Json.null

  let string value =
    match Js.Json.decodeString value with
    | Some v -> v
    | None -> invalid_arg {js|Expected string|js}

  let text value key = string (field value key)

  let number value =
    match Js.Json.decodeNumber value with
    | Some v when Float.is_finite v -> v
    | _ -> invalid_arg {js|Expected finite number|js}

  let n value key = number (field value key)

  let array value =
    match Js.Json.decodeArray value with Some v -> v | None -> invalid_arg {js|Expected array|js}

  let list value key = if has value key then array (field value key) else [||]
  let str value = Js.Json.string value
  let set value key v = Js.Dict.set (object_ value) key v
  let default value key fallback = if has value key then field value key else fallback
  let text_default value key fallback = if has value key then text value key else fallback
  let num_default value key fallback = if has value key then n value key else fallback
  let bool value key = Js.Json.decodeBoolean (field value key) = Some true
  let obj values = Js.Json.object_ (Js.Dict.fromArray (Array.of_list values))
end

module Node = struct
  type buffer
  type file_info = < mtimeMs : float ; ctimeMs : float ; size : float ; ino : float > Js.t

  external paths : string -> < recursive : bool > Js.t -> string array = "readdirSync"
  [@@mel.module "node:fs"]

  external read_text : string -> string -> string = "readFileSync" [@@mel.module "node:fs"]
  external write : string -> buffer -> unit = "writeFileSync" [@@mel.module "node:fs"]

  external write_text : string -> string -> < flag : string > Js.t -> unit = "writeFileSync"
  [@@mel.module "node:fs"]

  external mkdir : string -> < recursive : bool > Js.t -> unit = "mkdirSync"
  [@@mel.module "node:fs"]

  external absolute : string -> string = "resolve" [@@mel.module "node:path"]
  external resolve : string -> string -> string = "resolve" [@@mel.module "node:path"]
  external dirname : string -> string = "dirname" [@@mel.module "node:path"]

  external realpath : string -> string = "native"
  [@@mel.module "node:fs"] [@@mel.scope "realpathSync"]

  external argv : string array = "argv" [@@mel.module "node:process"]
  external stat : string -> file_info = "statSync" [@@mel.module "node:fs"]

  let read path = Js.Json.parseExn (read_text path {js|utf8|js})
  let absolute path = absolute path
  let resolve base path = resolve base path
  let dirname path = dirname path
end

let used_files = Hashtbl.create 32

let rec watch_existing_parent path =
  try Hashtbl.replace used_files (Node.realpath path) ()
  with _ ->
    let parent = Node.dirname path in
    if parent <> path then watch_existing_parent parent

let physical_path path =
  try Node.realpath path
  with error ->
    (* A missing file cannot be canonicalized yet. Watch its physical ancestor
       so creating it (or a missing parent directory) retries the render. *)
    watch_existing_parent (Node.dirname path);
    raise error

let read_json path =
  Hashtbl.replace used_files (Node.absolute path) ();
  let path = physical_path path in
  Hashtbl.replace used_files path ();
  Node.read path

let dependencies () = Array.of_seq (Hashtbl.to_seq_keys used_files)

module C = struct
  type canvas = Terrain_plan.image
  type context = Terrain_plan.context
  type pixels = < data : Lighting.pixels > Js.t

  external create : int -> int -> canvas = "createCanvas" [@@mel.module "@napi-rs/canvas"]
  external load : string -> canvas Js.Promise.t = "loadImage" [@@mel.module "@napi-rs/canvas"]
  external context : canvas -> string -> context = "getContext" [@@mel.send]
  external png : canvas -> string -> Node.buffer = "toBuffer" [@@mel.send]
  external width : canvas -> int = "width" [@@mel.get]
  external height : canvas -> int = "height" [@@mel.get]
  external draw : context -> canvas -> float -> float -> unit = "drawImage" [@@mel.send]
  external image_data : context -> int -> int -> int -> int -> pixels = "getImageData" [@@mel.send]
  external put : context -> pixels -> int -> int -> unit = "putImageData" [@@mel.send]
  external data : pixels -> Lighting.pixels = "data" [@@mel.get]
  external alpha : context -> float -> unit = "globalAlpha" [@@mel.set]
  external fill : context -> string -> unit = "fillStyle" [@@mel.set]
  external stroke_color : context -> string -> unit = "strokeStyle" [@@mel.set]
  external line_width : context -> float -> unit = "lineWidth" [@@mel.set]
  external font : context -> string -> unit = "font" [@@mel.set]
  external align : context -> string -> unit = "textAlign" [@@mel.set]
  external text : context -> string -> float -> float -> unit = "fillText" [@@mel.send]
  external stroke_text : context -> string -> float -> float -> unit = "strokeText" [@@mel.send]

  external rect : context -> float -> float -> float -> float -> float -> unit = "roundRect"
  [@@mel.send]

  external ellipse : context -> float -> float -> float -> float -> float -> float -> float -> unit
    = "ellipse"
  [@@mel.send]

  external arc : context -> float -> float -> float -> float -> float -> unit = "arc" [@@mel.send]
  external dash : context -> float array -> unit = "setLineDash" [@@mel.send]
  external stroke : context -> unit = "stroke" [@@mel.send]
  external translate : context -> float -> float -> unit = "translate" [@@mel.send]
  external save : context -> unit = "save" [@@mel.send]
  external restore : context -> unit = "restore" [@@mel.send]
  external fill_rect : context -> float -> float -> float -> float -> unit = "fillRect" [@@mel.send]
  external begin_path : context -> unit = "beginPath" [@@mel.send]
  external move_to : context -> float -> float -> unit = "moveTo" [@@mel.send]
  external line_to : context -> float -> float -> unit = "lineTo" [@@mel.send]
  external fill_path : context -> unit = "fill" [@@mel.send]

  external draw_scaled : context -> canvas -> float -> float -> float -> float -> unit = "drawImage"
  [@@mel.send]

  external get : Lighting.pixels -> int -> float = "" [@@mel.get_index]
  external set : Lighting.pixels -> int -> float -> unit = "" [@@mel.set_index]

  let text c value x y = text c value x y
end

type image_stamp = { modified : float; changed : float; size : float; inode : float }
type decoded_image = { stamp : image_stamp; image : C.canvas }
type terrain_layer = { key : string; canvas : C.canvas }

type draft_actor = {
  tile : Domain.hex;
  palette : string;
  frame_key : string;
  health_bar : bool;
  order : int;
}

let decoded_images : (string, decoded_image) Hashtbl.t = Hashtbl.create 32
let terrain_layer : terrain_layer option ref = ref None
let sprite_bundle = ref None
let sprite_sources = Hashtbl.create 32
let used_images = Hashtbl.create 32

let frame_path root (frame : Art.frame) =
  if
    Js.String.startsWith ~prefix:{js|/assets/|js} frame.url
    || Js.String.startsWith ~prefix:{js|assets/|js} frame.url
  then (
    match Hashtbl.find_opt sprite_sources frame.url with
    | Some source -> source
    | None ->
        let sprites =
          match !sprite_bundle with
          | Some sprites -> sprites
          | None ->
              let sprites =
                Node.read (Node.resolve root {js|_build/default/artwork/sprites.json|js})
              in
              sprite_bundle := Some sprites;
              sprites
        in
        let source =
          let js_text_part_1 = J.text sprites (Filename.basename frame.url) in
          {j|data:image/png;base64,$(js_text_part_1)|j}
        in
        Hashtbl.replace sprite_sources frame.url source;
        source)
  else Node.absolute frame.url

let help =
  {js|Usage: node scripts/render-map.mjs [--watch] --map map.json --roster roster.json --scenario scenario.json [--overlay overlay.json] --output preview.png\nOptions: --generate SEED [--write-map draft.json]  Resample natural terrain on the input map.\n         SEED is a decimal unsigned 32-bit integer. Export creates a new file and requires a one-shot render.\n         --grid [--grid-color COLOR] --artwork frames.json --content content.json\n         --draw-order native|reverse-within-z|shuffle-within-z|reverse-all|shuffle-all\n         --order-seed 17 --order-terrain castle-wall,castle-bell\nSupply standalone current-format map, roster and scenario documents; no catalogue registration is needed.\nStatic draft only: Rust resolves perception; encounters and gameplay are not executed.|js}

let options () =
  let flags = Hashtbl.create 12 in
  let args = Node.argv in
  let rec read i =
    if i < Array.length args then
      let name = args.(i) in
      if List.mem name [ {js|--grid|js}; {js|--help|js} ] then (
        Hashtbl.replace flags name {js|true|js};
        read (i + 1))
      else if
        List.mem name
          [
            {js|--map|js};
            {js|--roster|js};
            {js|--scenario|js};
            {js|--overlay|js};
            {js|--output|js};
            {js|--generate|js};
            {js|--write-map|js};
            {js|--grid-color|js};
            {js|--artwork|js};
            {js|--content|js};
            {js|--draw-order|js};
            {js|--order-seed|js};
            {js|--order-terrain|js};
          ]
      then (
        if i + 1 >= Array.length args || Js.String.startsWith ~prefix:{js|--|js} args.(i + 1) then
          invalid_arg {j|Missing value for $(name)|j};
        Hashtbl.replace flags name args.(i + 1);
        read (i + 2))
      else invalid_arg {j|Unknown option: $(name)|j}
  in
  read 2;
  flags

let cli_error error = Js.Exn.raiseError (Foreign_error.exception_ error)

let ( let* ) promise f =
  Js.Promise.then_ (fun value -> try f value with error -> cli_error error) promise

let return = Js.Promise.resolve

let positive name value =
  let n = J.number value in
  if floor n <> n || n <= 0. || n > 2147483647. then
    invalid_arg {j|Expected positive integer $(name)|j};
  int_of_float n

let generate_map seed map roster =
  let seed =
    match Int64.of_string_opt seed with
    | Some value
      when Js.Re.test ~str:seed (Js.Re.fromString {js|^[0-9]+$|js})
           && value >= 0L && value <= 4294967295L ->
        Int64.to_int32 value
    | _ -> invalid_arg {js|--generate requires a decimal seed between 0 and 4294967295|js}
  in
  let cols = positive {js|cols|js} (J.field map {js|cols|js})
  and rows = positive {js|rows|js} (J.field map {js|rows|js}) in
  let terrain_at = Draft_noise.terrain ~seed ~cols ~rows in
  let point value = (J.n value {js|col|js}, J.n value {js|row|js}) in
  let occupied = Hashtbl.create 16 in
  Array.iter
    (fun army ->
      Array.iter
        (fun unit -> Hashtbl.replace occupied (point (J.field unit {js|tile|js})) ())
        (J.list army {js|units|js}))
    (J.list roster {js|factions|js});
  Array.iter
    (fun tile ->
      let col, row = point tile in
      let terrain = J.text tile {js|terrain|js} in
      if
        List.mem terrain
          [
            {js|plains|js};
            {js|forest|js};
            {js|hill|js};
            {js|mountain|js};
            {js|water|js};
            {js|marsh|js};
          ]
        && (not (Hashtbl.mem occupied (col, row)))
        && (not
              (List.exists (J.has tile)
                 [ {js|id|js}; {js|owner|js}; {js|connections|js}; {js|deadEnds|js}; {js|walls|js} ]))
        && Array.length (J.list tile {js|environmentEffects|js}) = 0
      then
        let generated, floor = terrain_at ~col:(int_of_float col) ~row:(int_of_float row) in
        if generated <> terrain then (
          J.set tile {js|terrain|js} (J.str generated);
          J.set tile {js|floor|js} (J.str floor)))
    (J.array (J.field map {js|tiles|js}))

let write_generated_map path preview map =
  let parent = Node.dirname path in
  Node.mkdir parent [%mel.obj { recursive = true }];
  let physical_destination path =
    Node.resolve (Node.realpath (Node.dirname path)) (Filename.basename path)
  in
  if physical_destination path = physical_destination preview then
    invalid_arg {js|--write-map and --output require different paths|js};
  Node.write_text path
    (Js.Json.stringifyWithSpace map 2 ^ {js|\n|js})
    [%mel.obj { flag = {js|wx|js} }]

(* Keep content definitions, but load only the supplied scenario in this temporary
   bundle. The on-disk catalogue is untouched and cannot shadow a draft identity. *)
let draft_factory root map_path map roster_path roster scenario_path content_path =
  let data_root = Node.resolve root {js|rust/data|js} in
  let scenario = read_json scenario_path in
  let key path = {js|__draft__|js} ^ Node.absolute path in
  let reference field supplied =
    let relative = J.text scenario field in
    if Js.String.startsWith ~prefix:{js|/|js} relative then
      invalid_arg {j|Draft scenario $(field) reference must be relative|j};
    let referenced = physical_path (Node.dirname scenario_path ^ {js|/|js} ^ relative) in
    if referenced <> supplied then
      invalid_arg {j|Draft scenario $(field) reference does not match --$(field)|j};
    ()
  in
  reference {js|map|js} map_path;
  reference {js|roster|js} roster_path;
  let scenario_key = {js|__draft__/scenario.json|js}
  and map_key = {js|__draft__/map.json|js}
  and roster_key = {js|__draft__/roster.json|js} in
  J.set scenario {js|map|js} (J.str {js|map.json|js});
  J.set scenario {js|roster|js} (J.str {js|roster.json|js});
  let documents =
    Array.fold_left
      (fun documents path ->
        if not (Js.String.endsWith ~suffix:{js|.json|js} path) then documents
        else
          let source_path = Node.resolve data_root path in
          Hashtbl.replace used_files source_path ();
          Domain.Text_map.add path (Node.read_text source_path {js|utf8|js}) documents)
      Domain.Text_map.empty
      (Node.paths data_root [%mel.obj { recursive = true }])
  in
  let manifest =
    match Domain.Text_map.find_opt {js|manifest.json|js} documents with
    | Some text -> Js.Json.parseExn text
    | None -> invalid_arg {js|Missing catalogue manifest.json|js}
  in
  let documents =
    Option.fold ~none:documents
      ~some:(fun path ->
        let content = read_json path in
        let registries =
          [
            {js|units|js};
            {js|races|js};
            {js|items|js};
            {js|abilities|js};
            {js|environmentEffects|js};
          ]
        in
        Array.fold_left
          (fun documents (registry, paths) ->
            if not (List.mem registry registries) then
              invalid_arg {j|Unsupported draft content registry: $(registry)|j};
            let ids = Hashtbl.create 8 in
            let local, documents =
              Array.fold_left
                (fun (local, documents) relative ->
                  let source = Node.dirname (Node.realpath path) ^ {js|/|js} ^ J.string relative in
                  let value = read_json source in
                  let source = Node.realpath source in
                  let id = J.text value {js|id|js} in
                  if Hashtbl.mem ids id then invalid_arg {j|Duplicate draft content ID: $(id)|j};
                  Hashtbl.add ids id ();
                  let document_key = key source in
                  if Domain.Text_map.mem document_key documents then
                    invalid_arg {j|Duplicate draft document: $(source)|j};
                  ( J.str document_key :: local,
                    Domain.Text_map.add document_key (Js.Json.stringify value) documents ))
                ([], documents) (J.array paths)
            in
            let retained =
              Array.to_list (J.array (J.field manifest registry))
              |> List.filter (fun path ->
                  let document = Domain.Text_map.find (J.string path) documents in
                  not (Hashtbl.mem ids (J.text (Js.Json.parseExn document) {js|id|js})))
            in
            J.set manifest registry (Js.Json.array (Array.of_list (retained @ List.rev local)));
            documents)
          documents
          (Js.Dict.entries (J.object_ content)))
      content_path
  in
  J.set manifest {js|scenarios|js} (Js.Json.array [| J.str scenario_key |]);
  J.set manifest {js|campaigns|js} (Js.Json.array [||]);
  let documents =
    Domain.Text_map.add {js|manifest.json|js} (Js.Json.stringify manifest) documents
  in
  let documents =
    List.fold_left
      (fun documents (key, value) ->
        if Domain.Text_map.mem key documents then
          invalid_arg {j|Duplicate draft document path: $(key)|j};
        Domain.Text_map.add key (Js.Json.stringify value) documents)
      documents
      [ (scenario_key, scenario); (map_key, map); (roster_key, roster) ]
  in
  match Domain.NativeEngineFactory.with_documents documents with
  | Ok factory -> factory
  | Error error -> failwith (Native.message (Native.Engine error))

let normalize factory map overlay =
  let cols = positive {js|cols|js} (J.field map {js|cols|js})
  and rows = positive {js|rows|js} (J.field map {js|rows|js}) in
  let map_id = J.text map {js|mapId|js} in
  let coordinate value =
    let col = J.n value {js|col|js} and row = J.n value {js|row|js} in
    if
      floor col <> col
      || floor row <> row
      || col < 0. || row < 0.
      || col >= float_of_int cols
      || row >= float_of_int rows
    then invalid_arg {js|Position outside draft map|js};
    (int_of_float col, int_of_float row)
  in
  let catalog = Domain.NativeEngineFactory.catalog factory in
  let prepared =
    match
      Array.find_opt
        (fun (m : Domain.browser_map) -> m.id = map_id && m.revision = J.n map {js|mapRevision|js})
        catalog.maps
    with
    | Some map -> map.state
    | None -> invalid_arg {js|Draft map was not loaded|js}
  in
  let state_json = Js.Json.parseExn (Domain.state_to_json prepared) in
  if J.has overlay {js|units|js} then J.set state_json {js|units|js} (J.field overlay {js|units|js});
  if J.has overlay {js|round|js} then J.set state_json {js|round|js} (J.field overlay {js|round|js});
  let observer = J.default overlay {js|observer|js} (J.str {js|player|js}) in
  (if J.has overlay {js|memory|js} then
     let faction =
       match
         Array.find_opt
           (fun value -> J.field value {js|id|js} = observer)
           (J.array (J.field state_json {js|factions|js}))
       with
       | Some value -> value
       | None -> invalid_arg {js|Unknown draft observer faction|js}
     in
     let group_id = J.field faction {js|visionGroup|js} in
     Array.iter
       (fun group ->
         if J.field group {js|id|js} = group_id then
           J.set group {js|explored|js} (J.field overlay {js|memory|js}))
       (J.array (J.field state_json {js|visionGroups|js})));
  let input =
    J.obj
      [
        ({js|observer|js}, observer);
        ({js|state|js}, state_json);
        ({js|pending|js}, J.default overlay {js|pending|js} (Js.Json.array [||]));
        ({js|marks|js}, J.default overlay {js|marks|js} (Js.Json.array [||]));
        ({js|choices|js}, J.default overlay {js|choices|js} (Js.Json.array [||]));
      ]
  in
  (cols, rows, map_id, Map_draft_json.of_presentation_input input, coordinate)

let drawing_order flags source =
  let ordered = Array.copy source in
  let z (t : Terrain_plan.tile) = (Option.get t.frame).z in
  Array.stable_sort
    (fun a b ->
      let c = Float.compare (z a) (z b) in
      if c <> 0 then c
      else
        let c = Float.compare a.y b.y in
        if c <> 0 then c else Float.compare a.x b.x)
    ordered;
  let order = Option.value ~default:{js|native|js} (Hashtbl.find_opt flags {js|--draw-order|js}) in
  if
    not
      (List.mem order
         [
           {js|native|js};
           {js|reverse-within-z|js};
           {js|shuffle-within-z|js};
           {js|reverse-all|js};
           {js|shuffle-all|js};
         ])
  then invalid_arg {js|Unsupported diagnostic draw order|js};
  let kinds =
    Option.map
      (fun text -> Array.to_list (Js.String.split ~sep:{js|,|js} text))
      (Hashtbl.find_opt flags {js|--order-terrain|js})
  in
  Option.iter
    (List.iter (fun kind ->
         if not (Array.exists (fun (tile : Terrain_plan.tile) -> tile.terrain = kind) source) then
           invalid_arg {j|Order terrain absent from draft: $(kind)|j}))
    kinds;
  let selected (tile : Terrain_plan.tile) =
    Option.fold ~none:true ~some:(List.mem tile.terrain) kinds
  in
  if order = {js|native|js} then ordered
  else
    let seed =
      ref
        (Int64.of_string
           (Option.value ~default:{js|1|js} (Hashtbl.find_opt flags {js|--order-seed|js})))
    in
    if !seed <= 0L || !seed > 4294967295L then
      invalid_arg {js|order-seed must be a positive 32-bit integer|js};
    let state = ref (Int64.to_int32 !seed) in
    let shuffle group =
      if Js.String.startsWith ~prefix:{js|reverse|js} order then
        Array.init (Array.length group) (fun i -> group.(Array.length group - 1 - i))
      else
        let group = Array.copy group in
        for i = Array.length group - 1 downto 1 do
          let x = Int32.logxor !state (Int32.shift_left !state 13) in
          let x = Int32.logxor x (Int32.shift_right_logical x 17) in
          state := Int32.logxor x (Int32.shift_left x 5);
          let unsigned = Int64.logand (Int64.of_int32 !state) 4294967295L in
          let j = Int64.to_int (Int64.rem unsigned (Int64.of_int (i + 1))) in
          let tmp = group.(i) in
          group.(i) <- group.(j);
          group.(j) <- tmp
        done;
        group
    in
    let targets = Array_ext.filter selected ordered in
    let groups =
      if Js.String.endsWith ~suffix:{js|all|js} order then [| targets |]
      else
        let groups = ref [] in
        Array.iter
          (fun tile ->
            match !groups with
            | (depth, items) :: tail when depth = z tile -> groups := (depth, tile :: items) :: tail
            | _ -> groups := (z tile, [ tile ]) :: !groups)
          targets;
        Array.of_list
          (List.map (fun (_, items) -> Array.of_list (List.rev items)) (List.rev !groups))
    in
    let reordered = Array.concat (Array.to_list (Array.map shuffle groups)) in
    let cursor = ref 0 in
    Array.map
      (fun tile ->
        if selected tile then (
          let result = reordered.(!cursor) in
          incr cursor;
          result)
        else tile)
      ordered

let run root =
  let root = root in
  Hashtbl.clear used_files;
  Hashtbl.clear used_images;
  Hashtbl.clear sprite_sources;
  sprite_bundle := None;
  let flags = options () in
  if Hashtbl.mem flags {js|--help|js} then (
    Js.log help;
    return ())
  else
    let required name =
      match Hashtbl.find_opt flags name with Some v -> v | None -> invalid_arg help
    in
    let map_path = physical_path (required {js|--map|js})
    and roster_path = physical_path (required {js|--roster|js})
    and scenario_path = physical_path (required {js|--scenario|js}) in
    let map = read_json map_path in
    let roster = read_json roster_path in
    let export = Hashtbl.find_opt flags {js|--write-map|js} in
    (match Hashtbl.find_opt flags {js|--generate|js} with
    | Some seed -> generate_map seed map roster
    | None -> if Option.is_some export then invalid_arg {js|--write-map requires --generate|js});
    let overlay =
      Option.fold ~none:(J.obj []) ~some:read_json (Hashtbl.find_opt flags {js|--overlay|js})
    in
    let factory =
      draft_factory root map_path map roster_path roster scenario_path
        (Hashtbl.find_opt flags {js|--content|js})
    in
    let cols, rows, map_id, input, coordinate, resolved, roles =
      let prepare () =
        let cols, rows, map_id, input, coordinate = normalize factory map overlay in
        ( cols,
          rows,
          map_id,
          input,
          coordinate,
          Domain.NativeEngineFactory.preview_presentation factory input,
          Catalog.terrain_roles (Catalog.of_view (Domain.NativeEngineFactory.catalog factory)) )
      in
      (* Fun.protect's exception path requires an unavailable Melange backtrace primitive. *)
      match prepare () with
      | result ->
          Domain.NativeEngineFactory.dispose factory;
          result
      | exception error ->
          (* A failed boundary call may have already poisoned/released its handle. *)
          (try Domain.NativeEngineFactory.dispose factory with _ -> ());
          raise error
    in
    let base = Art.current () in
    let overrides =
      Option.fold ~none:[||]
        ~some:(fun path ->
          let entries = Js.Dict.entries (J.object_ (read_json path)) in
          Array.map
            (fun (key, value) ->
              let frame =
                {
                  Art.url =
                    Node.resolve (Node.dirname (Node.absolute path)) (J.text value {js|url|js});
                  width = J.n value {js|width|js};
                  height = J.n value {js|height|js};
                  anchor_x = J.n value {js|anchorX|js};
                  anchor_y = J.n value {js|anchorY|js};
                  z = J.n value {js|z|js};
                }
              in
              (match Art.find base key with
              | Some original ->
                  if
                    original.width <> frame.width || original.height <> frame.height
                    || original.anchor_x <> frame.anchor_x
                    || original.anchor_y <> frame.anchor_y
                    || floor frame.z <> frame.z
                  then invalid_arg {j|Candidate must preserve existing frame geometry: $(key)|j}
              | None ->
                  if
                    frame.width <> 64. || frame.height <> 64. || frame.anchor_x <> 32.
                    || frame.anchor_y <> 32.
                    || (not (Float.is_finite frame.z))
                    || floor frame.z <> frame.z
                  then invalid_arg {j|New actor frame must use native 64x64 geometry: $(key)|j});
              (key, frame))
            entries)
        (Hashtbl.find_opt flags {js|--artwork|js})
    in
    let artwork = Art.with_overrides base overrides in
    let plan =
      Terrain_plan.prepare ~roles ~map_id ~cols:(float_of_int cols) ~rows:(float_of_int rows)
        input.state.tiles artwork
    in
    let images = Js.Dict.empty () in
    let load_frame (frame : Art.frame) =
      if Js.Dict.get images frame.url <> None then return ()
      else
        let path = frame_path root frame in
        Hashtbl.replace used_images path ();
        let dependency =
          if Js.String.startsWith ~prefix:{js|data:image/png;base64,|js} path then
            Node.resolve root {js|_build/default/artwork/sprites.json|js}
          else path
        in
        Hashtbl.replace used_files dependency ();
        let info = Node.stat dependency in
        let stamp =
          {
            modified = info##mtimeMs;
            changed = info##ctimeMs;
            size = info##size;
            inode = info##ino;
          }
        in
        let* image =
          match Hashtbl.find_opt decoded_images path with
          | Some cached when cached.stamp = stamp -> return cached.image
          | _ ->
              let* image = C.load path in
              Hashtbl.replace decoded_images path { stamp; image };
              return image
        in
        Js.Dict.set images frame.url image;
        return ()
    in
    let rec load i =
      if i = Array.length plan.tiles then return ()
      else
        let tile = plan.tiles.(i) in
        let frame =
          match tile.frame with
          | Some f -> f
          | None ->
              invalid_arg
                (let js_text_part_1 = tile.id in
                 {j|Missing terrain frame: $(js_text_part_1)|j})
        in
        let* () = load_frame frame in
        load (i + 1)
    in
    let* () = load 0 in
    let scale = Geometry.native_pixel_scale in
    let size = Terrain_plan.dimensions plan in
    let width = int_of_float size.width and height = int_of_float size.height in
    if width <= 0 || height <= 0 then invalid_arg {js|Draft map has invalid rendered dimensions|js};
    let ordered = drawing_order flags plan.tiles in
    let key =
      Js.Json.stringify
        (Js.Json.array
           [|
             Js.Json.string plan.key;
             Js.Json.string (Lighting.key resolved.lighting);
             Js.Json.array
               (Array.map
                  (fun (tile : Terrain_plan.tile) ->
                    let frame = Option.get tile.frame in
                    let cached = Hashtbl.find decoded_images (frame_path root frame) in
                    Js.Json.array
                      [|
                        J.str tile.id;
                        J.str frame.url;
                        Js.Json.number frame.z;
                        Js.Json.number cached.stamp.modified;
                        Js.Json.number cached.stamp.changed;
                        Js.Json.number cached.stamp.size;
                        Js.Json.number cached.stamp.inode;
                      |])
                  ordered);
           |])
    in
    let terrain =
      match !terrain_layer with
      | Some cached when cached.key = key -> cached.canvas
      | _ ->
          let canvas = C.create width height in
          let tc = C.context canvas {js|2d|js} in
          Terrain_plan.configure tc scale;
          Array.iter (fun tile -> Terrain_plan.paint tc [| tile |] images) ordered;
          let pixels = C.image_data tc 0 0 width height in
          Lighting.shade (C.data pixels) width 0 height scale resolved.lighting;
          C.put tc pixels 0 0;
          Lighting.paint tc resolved.lighting;
          terrain_layer := Some { key; canvas };
          canvas
    in
    let notes = J.list overlay {js|notes|js} in
    let margin = 30. and header = 74. in
    let canvas = C.create (width + 60) (height + 74 + 30 + (22 * Array.length notes)) in
    let ctx = C.context canvas {js|2d|js} in
    C.fill ctx {js|#101827|js};
    C.fill_rect ctx 0. 0. (float_of_int (C.width canvas)) (float_of_int (C.height canvas));
    C.fill ctx {js|#ede8d6|js};
    C.font ctx {js|bold 24px sans-serif|js};
    C.text ctx
      (let js_text_part_0 =
         J.text_default overlay {js|title|js} (J.text_default map {js|name|js} map_id)
       in
       {j|$(js_text_part_0) · Map draft|j})
      margin 31.;
    C.fill ctx {js|#aab8c5|js};
    C.font ctx {js|14px sans-serif|js};
    C.text ctx
      {js|Static design preview · placements and arrows do not execute scenario mechanics|js} margin
      55.;
    C.draw ctx terrain margin header;
    C.save ctx;
    C.translate ctx margin header;
    Terrain_plan.configure ctx scale;
    Lighting.fog ctx plan.tiles (Array.map Hex.key resolved.fog);
    let fog = Hashtbl.create 16 in
    Array.iter (fun hex -> Hashtbl.replace fog (Hex.key hex) ()) resolved.fog;
    let point value =
      let col, row = coordinate value in
      Geometry.center ~col ~row
    in
    let placements = Hashtbl.create (Array.length input.state.units) in
    Array.iter
      (fun (unit : Domain.public_unit) -> Hashtbl.replace placements (Hex.key unit.tile) unit)
      input.state.units;
    let palette faction_id =
      match Array.find_opt (fun (f : Domain.faction) -> f.id = faction_id) input.state.factions with
      | Some faction -> faction.palette
      | None -> invalid_arg {j|Draft actor requires a declared faction: $(faction_id)|j}
    in
    let tile_order = Hashtbl.create (Array.length input.state.tiles) in
    Array.iteri (fun i (tile : Domain.tile) -> Hashtbl.add tile_order tile.id i) input.state.tiles;
    let known_units = Hashtbl.create (Array.length resolved.visible_units) in
    Array.iter (fun id -> Hashtbl.replace known_units id ()) resolved.visible_units;
    let native =
      Array_ext.filter_map
        (fun (tile : Domain.tile) ->
          match Hashtbl.find_opt placements tile.id with
          | Some unit when Hashtbl.mem known_units unit.id ->
              let palette = palette unit.faction in
              Some
                {
                  tile = unit.tile;
                  palette;
                  frame_key = Art.frame_key ~kind:{js|units|js} ~name:unit.type_ ~side:palette ();
                  health_bar = true;
                  order = Hashtbl.find tile_order tile.id;
                }
          | _ -> None)
        input.state.tiles
    in
    let figures =
      Array_ext.filter_map
        (fun figure ->
          Array.iter
            (fun (key, _) ->
              if
                not
                  (List.mem key
                     [
                       {js|col|js}; {js|row|js}; {js|frame|js}; {js|faction|js}; {js|visibility|js};
                     ])
              then invalid_arg {j|Unknown draft figure field: $(key)|j})
            (Js.Dict.entries (J.object_ figure));
          let col, row = coordinate figure in
          let tile : Domain.hex = { col = Int32.of_int col; row = Int32.of_int row } in
          let id = Hex.key tile in
          let order =
            match Hashtbl.find_opt tile_order id with
            | Some i -> i
            | None -> invalid_arg {j|Draft figure requires an authored tile: $(id)|j}
          in
          let actor =
            {
              tile;
              palette = palette (J.text figure {js|faction|js});
              frame_key = J.text figure {js|frame|js};
              health_bar = false;
              order;
            }
          in
          let visible =
            match J.text figure {js|visibility|js} with
            | "shown" -> true
            | "hidden" -> false
            | "terrain-visible" -> not (Hashtbl.mem fog id)
            | value -> invalid_arg {j|Unknown draft figure visibility: $(value)|j}
          in
          if visible then Some actor else None)
        (J.list overlay {js|figures|js})
    in
    let visible = Array.append native figures in
    Array.stable_sort (fun a b -> Int.compare a.order b.order) visible;
    let rec paint_unit i =
      if i = Array.length visible then return ()
      else
        let unit = visible.(i) in
        let p =
          Geometry.center ~col:(Int32.to_int unit.tile.col) ~row:(Int32.to_int unit.tile.row)
        in
        let palette = unit.palette in
        let color =
          if palette = {js|passive|js} then {js|#efecdc|js}
          else if palette = {js|green|js} then {js|#83b765|js}
          else if palette = {js|player|js} then {js|#6fdec0|js}
          else {js|#ff9590|js}
        in
        let frame_key = unit.frame_key in
        let frame =
          match Art.find artwork frame_key with
          | Some f -> f
          | None -> invalid_arg {j|Missing actor frame: $(frame_key)|j}
        in
        let* () = load_frame frame in
        C.begin_path ctx;
        C.ellipse ctx p.x (p.y +. 19.) 19. 7. 0. 0. (Float.pi *. 2.);
        C.fill ctx color;
        C.alpha ctx 0.5;
        C.fill_path ctx;
        C.alpha ctx 1.;
        C.stroke_color ctx color;
        C.line_width ctx 1.8;
        C.stroke ctx;
        let sprite = C.create 64 64 in
        let sc = C.context sprite {js|2d|js} in
        let image = Option.get (Js.Dict.get images frame.url) in
        if C.width image <> 64 || C.height image <> 64 then
          invalid_arg {j|Actor image must be native 64x64: $(frame_key)|j};
        C.draw sc image 0. 0.;
        let data = C.image_data sc 0 0 64 64 in
        let pixels = C.data data in
        let matrix = Lighting.matrix p.x p.y resolved.lighting in
        for pixel = 0 to 4095 do
          let start = pixel * 4 in
          if C.get pixels (start + 3) <> 0. then
            let rgb = Array.init 3 (fun channel -> C.get pixels (start + channel)) in
            for channel = 0 to 2 do
              let value = ref (matrix.((channel * 5) + 4) *. 255.) in
              for source = 0 to 2 do
                value := !value +. (rgb.(source) *. matrix.((channel * 5) + source))
              done;
              C.set pixels (start + channel) !value
            done
        done;
        C.put sc data 0 0;
        C.draw_scaled ctx sprite
          (p.x -. (Geometry.sprite_size /. 2.))
          (p.y +. 20. -. Geometry.sprite_size)
          Geometry.sprite_size Geometry.sprite_size;
        paint_unit (i + 1)
    in
    let* () = paint_unit 0 in
    Array.iter
      (fun unit ->
        if unit.health_bar then (
          let p =
            Geometry.center ~col:(Int32.to_int unit.tile.col) ~row:(Int32.to_int unit.tile.row)
          in
          C.begin_path ctx;
          C.rect ctx (p.x -. 19.) (p.y +. 27.) 38. 4. 2.;
          C.fill ctx {js|#a9d0a5|js};
          C.fill_path ctx))
      visible;
    if Hashtbl.mem flags {js|--grid|js} then (
      C.save ctx;
      C.stroke_color ctx
        (Option.value ~default:{js|#ff4040|js} (Hashtbl.find_opt flags {js|--grid-color|js}));
      C.alpha ctx 0.9;
      C.line_width ctx (1. /. scale);
      C.begin_path ctx;
      let edges = Hashtbl.create (cols * rows * 4) in
      Array.iter
        (fun (tile : Terrain_plan.tile) ->
          let points =
            Array.of_list
              (List.map
                 (fun item ->
                   match Array.to_list (Js.String.split ~sep:{js|,|js} item) with
                   | [ x; y ] -> (float_of_string x, float_of_string y)
                   | _ -> assert false)
                 (Array.to_list (Js.String.split ~sep:{js| |js} tile.points)))
          in
          Array.iteri
            (fun i a ->
              let b = points.((i + 1) mod 6) in
              let key (x, y) =
                let x = Js.Float.toFixed ~digits:6 x and y = Js.Float.toFixed ~digits:6 y in
                {j|$x,$y|j}
              in
              let ak = key a and bk = key b in
              let edge = if ak < bk then {j|$(ak)|$(bk)|j} else {j|$(bk)|$(ak)|j} in
              if not (Hashtbl.mem edges edge) then (
                Hashtbl.add edges edge ();
                C.move_to ctx (fst a) (snd a);
                C.line_to ctx (fst b) (snd b)))
            points)
        plan.tiles;
      C.stroke ctx;
      C.restore ctx);
    Array.iter
      (fun arrow ->
        let a = point (J.field arrow {js|from|js}) and b = point (J.field arrow {js|to|js}) in
        let angle = atan2 (b.y -. a.y) (b.x -. a.x) in
        C.stroke_color ctx (J.text_default arrow {js|color|js} {js|#dfb5bb|js});
        C.line_width ctx 2.;
        C.begin_path ctx;
        C.move_to ctx a.x a.y;
        C.line_to ctx b.x b.y;
        List.iter
          (fun offset ->
            C.move_to ctx b.x b.y;
            C.line_to ctx
              (b.x -. (12. *. cos (angle +. offset)))
              (b.y -. (12. *. sin (angle +. offset))))
          [ -0.5; 0.5 ];
        C.stroke ctx)
      (J.list overlay {js|arrows|js});
    Array.iter
      (fun mark ->
        let p = point mark and color = J.text_default mark {js|color|js} {js|#c3cbd3|js} in
        C.stroke_color ctx color;
        C.line_width ctx 2.;
        if J.bool mark {js|ring|js} then (
          C.dash ctx [| 5.; 5. |];
          C.begin_path ctx;
          C.arc ctx p.x p.y 32. 0. (Float.pi *. 2.);
          C.stroke ctx;
          C.dash ctx [||]);
        if J.has mark {js|text|js} then (
          let text = J.text mark {js|text|js} and y = p.y +. J.num_default mark {js|dy|js} (-40.) in
          C.font ctx {js|bold 14px sans-serif|js};
          C.align ctx {js|center|js};
          C.line_width ctx 4.;
          C.stroke_color ctx {js|#101827|js};
          C.stroke_text ctx text p.x y;
          C.fill ctx color;
          C.text ctx text p.x y))
      (J.list overlay {js|annotations|js});
    C.restore ctx;
    C.fill ctx {js|#c0cbd4|js};
    C.font ctx {js|14px sans-serif|js};
    Array.iteri
      (fun i note ->
        C.text ctx (J.string note) margin
          (header +. float_of_int height +. 24. +. float_of_int (i * 22)))
      notes;
    let output = Node.absolute (required {js|--output|js}) in
    Node.mkdir (Node.dirname output) [%mel.obj { recursive = true }];
    let preview = C.png canvas {js|image/png|js} in
    Option.iter (fun path -> write_generated_map (Node.absolute path) output map) export;
    Node.write output preview;
    Hashtbl.filter_map_inplace
      (fun path image -> if Hashtbl.mem used_images path then Some image else None)
      decoded_images;
    return ()

let main root = try run root with error -> cli_error error
