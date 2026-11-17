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

  external read_text : string -> string -> string = "readFileSync" [@@mel.module "node:fs"]
  external write : string -> buffer -> unit = "writeFileSync" [@@mel.module "node:fs"]

  external mkdir : string -> < recursive : bool > Js.t -> unit = "mkdirSync"
  [@@mel.module "node:fs"]

  external absolute : string -> string = "resolve" [@@mel.module "node:path"]
  external resolve : string -> string -> string = "resolve" [@@mel.module "node:path"]
  external dirname : string -> string = "dirname" [@@mel.module "node:path"]
  external argv : string array = "argv" [@@mel.module "node:process"]
  external stat : string -> file_info = "statSync" [@@mel.module "node:fs"]

  let read path = Js.Json.parseExn (read_text path {js|utf8|js})
  let absolute path = absolute path
  let resolve base path = resolve base path
  let dirname path = dirname path
end

let used_files = Hashtbl.create 32

let read_json path =
  Hashtbl.replace used_files (Node.absolute path) ();
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
  external clip : context -> unit = "clip" [@@mel.send]
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

let decoded_images : (string, decoded_image) Hashtbl.t = Hashtbl.create 32
let terrain_layer : terrain_layer option ref = ref None
let cached_roles : Domain.Terrain_role.t Js.Dict.t option ref = ref None

let terrain_roles () =
  match !cached_roles with
  | Some roles -> roles
  | None ->
      let factory =
        match Domain.NativeEngineFactory.create () with
        | Ok factory -> factory
        | Error error -> failwith (Native.message (Native.Engine error))
      in
      let roles =
        Fun.protect
          ~finally:(fun () -> Domain.NativeEngineFactory.dispose factory)
          (fun () ->
            Catalog.terrain_roles (Catalog.of_view (Domain.NativeEngineFactory.catalog factory)))
      in
      cached_roles := Some roles;
      roles

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
  {js|Usage: node scripts/render-map.mjs [--watch] --map map.json [--overlay overlay.json] --output preview.png\nOptions: --grid [--grid-color COLOR] --artwork candidate-frames.json\n         --draw-order native|reverse-within-z|shuffle-within-z|reverse-all|shuffle-all\n         --order-seed 17 --order-terrain castle-wall,castle-bell\nStatic draft only: Rust resolves fog/lights; encounters and gameplay are not executed.|js}

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
            {js|--overlay|js};
            {js|--output|js};
            {js|--grid-color|js};
            {js|--artwork|js};
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

let normalize map overlay =
  let cols = positive {js|cols|js} (J.field map {js|cols|js})
  and rows = positive {js|rows|js} (J.field map {js|rows|js}) in
  let map_id = J.text map {js|mapId|js} in
  if map_id = {js||js} then invalid_arg {js|Expected mapId|js};
  let positions = Hashtbl.create (cols * rows) and locations = Hashtbl.create 16 in
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
  let address (col, row) =
    let js_text_part_0 = string_of_int col in
    let js_text_part_2 = string_of_int row in
    {j|$(js_text_part_0),$(js_text_part_2)|j}
  in
  let tiles = J.array (J.field map {js|tiles|js}) in
  Array.iter
    (fun tile ->
      let key = address (coordinate tile) in
      if Hashtbl.mem positions key then invalid_arg {j|Duplicate draft tile: $(key)|j};
      let id = J.text_default tile {js|id|js} key in
      if id <> key && J.has tile {js|location|js} && J.text tile {js|location|js} <> id then
        invalid_arg {js|Conflicting authored tile identifiers|js};
      let location =
        if id <> key then Some id
        else if J.has tile {js|location|js} then Some (J.text tile {js|location|js})
        else None
      in
      Option.iter
        (fun name ->
          if
            (not (Js.Re.test ~str:name (Js.Re.fromString {js|^[a-z0-9-]+$|js})))
            || Hashtbl.mem locations name
          then invalid_arg {j|Invalid or duplicate location: $(name)|j};
          Hashtbl.add locations name ();
          J.set tile {js|location|js} (J.str name))
        location;
      J.set tile {js|id|js} (J.str key);
      Hashtbl.add positions key ())
    tiles;
  if Array.length tiles <> cols * rows then invalid_arg {js|Supply the complete draft tile grid|js};
  let units =
    if J.has overlay {js|units|js} then J.array (J.field overlay {js|units|js})
    else J.list map {js|units|js}
  in
  Array.iter
    (fun unit ->
      let point = if J.has unit {js|tile|js} then J.field unit {js|tile|js} else unit in
      let col, row = coordinate point in
      J.set unit {js|tile|js}
        (J.obj
           [
             ({js|col|js}, Js.Json.number (float_of_int col));
             ({js|row|js}, Js.Json.number (float_of_int row));
           ]))
    units;
  let input =
    J.obj
      [
        ({js|observer|js}, J.default overlay {js|observer|js} (J.str {js|player|js}));
        ({js|tiles|js}, Js.Json.array tiles);
        ({js|units|js}, Js.Json.array units);
        ( {js|lighting|js},
          J.default overlay {js|lighting|js}
            (J.default map {js|lighting|js} (J.obj [ ({js|time|js}, J.str {js|day|js}) ])) );
        ({js|factions|js}, J.default map {js|factions|js} (Js.Json.array [||]));
        ( {js|visionGroups|js},
          J.default overlay {js|visionGroups|js}
            (J.default map {js|visionGroups|js} (Js.Json.array [||])) );
        ({js|pending|js}, Js.Json.array [||]);
        ({js|marks|js}, J.default overlay {js|marks|js} (Js.Json.array [||]));
        ({js|choices|js}, J.default overlay {js|choices|js} (Js.Json.array [||]));
        ({js|memory|js}, J.default overlay {js|memory|js} (Js.Json.array [||]));
        ({js|round|js}, J.default overlay {js|round|js} (Js.Json.number 1.));
      ]
  in
  (cols, rows, map_id, units, Map_draft_json.of_presentation_input input, coordinate)

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
    let map = read_json (required {js|--map|js}) in
    let overlay =
      Option.fold ~none:(J.obj []) ~some:read_json (Hashtbl.find_opt flags {js|--overlay|js})
    in
    let cols, rows, map_id, units, input, coordinate = normalize map overlay in
    let resolved = Domain.preview_presentation input in
    let roles = terrain_roles () in
    let base = Art.current () in
    let overrides =
      Option.fold ~none:[||]
        ~some:(fun path ->
          let entries = Js.Dict.entries (J.object_ (read_json path)) in
          Array.map
            (fun (key, value) ->
              let original =
                match Art.find base key with
                | Some f -> f
                | None -> invalid_arg {j|Unknown candidate frame: $(key)|j}
              in
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
              if
                original.width <> frame.width || original.height <> frame.height
                || original.anchor_x <> frame.anchor_x
                || original.anchor_y <> frame.anchor_y
                || floor frame.z <> frame.z
              then invalid_arg {j|Candidate must preserve existing frame geometry: $(key)|j};
              (key, frame))
            entries)
        (Hashtbl.find_opt flags {js|--artwork|js})
    in
    let artwork = Art.with_overrides base overrides in
    let plan =
      Terrain_plan.prepare ~roles ~map_id ~cols:(float_of_int cols) ~rows:(float_of_int rows)
        input.tiles artwork
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
    (* Author-supplied visual states, not a second perception evaluator. Snapshot
       transforms affect terrain only; fog is painted over all proposals. *)
    let remembered = Hashtbl.create 16 in
    Array.iter
      (fun cell ->
        let col, row = coordinate cell in
        let key = Hex.key { Domain.col = Int32.of_int col; row = Int32.of_int row } in
        Hashtbl.replace remembered key ())
      (J.list overlay {js|rememberedPreview|js});
    let paint_state field transform =
      let cells = J.list overlay field in
      if Array.length cells > 0 then (
        let snapshot = C.create width height in
        let sc = C.context snapshot {js|2d|js} in
        let data = C.image_data (C.context terrain {js|2d|js}) 0 0 width height in
        let pixels = C.data data in
        for i = 0 to (width * height) - 1 do
          if C.get pixels ((i * 4) + 3) > 0. then (
            let r = C.get pixels (i * 4)
            and g = C.get pixels ((i * 4) + 1)
            and b = C.get pixels ((i * 4) + 2) in
            let r, g, b = transform (i mod width) (i / width) r g b in
            C.set pixels (i * 4) r;
            C.set pixels ((i * 4) + 1) g;
            C.set pixels ((i * 4) + 2) b)
        done;
        C.put sc data 0 0;
        Array.iter
          (fun cell ->
            let col, row = coordinate cell in
            let p = Geometry.center ~col ~row in
            let points = Js.String.split ~sep:{js| |js} (Geometry.hex_points p.x p.y) in
            C.save ctx;
            C.begin_path ctx;
            Array.iteri
              (fun i item ->
                let xy = Js.String.split ~sep:{js|,|js} item in
                let x = float_of_string xy.(0) and y = float_of_string xy.(1) in
                if i = 0 then C.move_to ctx x y else C.line_to ctx x y)
              points;
            C.clip ctx;
            C.draw_scaled ctx snapshot 0. 0.
              (float_of_int width /. scale)
              (float_of_int height /. scale);
            C.restore ctx)
          cells)
    in
    paint_state {js|rememberedPreview|js} (fun _ _ r g b ->
        ((r *. 0.68) +. 8., (g *. 0.68) +. 7., (b *. 0.72) +. 20.));
    let darkvision = J.bool overlay {js|darkvisionPreview|js} in
    paint_state {js|darknessPreview|js} (fun x y r g b ->
        let wisp = (sin ((float_of_int x *. 0.035) +. sin (float_of_int y *. 0.041)) +. 1.) /. 2. in
        if darkvision then
          ((r *. 0.57) +. 9. +. (wisp *. 4.), (g *. 0.32) +. 3., (b *. 0.82) +. 23. +. (wisp *. 7.))
        else
          ( (r *. 0.09) +. 8. +. (wisp *. 7.),
            (g *. 0.035) +. 3.,
            (b *. 0.18) +. 17. +. (wisp *. 12.) ));
    Lighting.fog ctx plan.tiles (Array.map Hex.key resolved.fog);
    let fog = Hashtbl.create 16 in
    Array.iter (fun hex -> Hashtbl.replace fog (Hex.key hex) ()) resolved.fog;
    let point value =
      let col, row = coordinate value in
      Geometry.center ~col ~row
    in
    let placements = Hashtbl.create (Array.length units) in
    Array.iteri
      (fun i (unit : Domain.starting_unit) ->
        Hashtbl.replace placements (Hex.key unit.tile) (unit, units.(i)))
      input.units;
    let faction (unit : Domain.starting_unit) =
      match Array.find_opt (fun (f : Domain.faction) -> f.id = unit.faction) input.factions with
      | Some f -> f
      | None -> invalid_arg {js|Draft unit requires a declared faction|js}
    in
    let hostile unit =
      let f = faction unit in
      let relation = Domain.Text_map.find_opt {js|player|js} f.relations in
      relation = Some Domain.Relation.Hostile
    in
    let visible =
      Array_ext.filter_map
        (fun (tile : Domain.tile) ->
          match Hashtbl.find_opt placements tile.id with
          | Some (unit, raw)
            when (not (Hashtbl.mem remembered tile.id))
                 && ((not (hostile unit)) || not (Hashtbl.mem fog tile.id)) ->
              Some (unit, raw)
          | _ -> None)
        input.tiles
    in
    let rec paint_unit i =
      if i = Array.length visible then return ()
      else
        let unit, _raw = visible.(i) in
        let p =
          Geometry.center ~col:(Int32.to_int unit.tile.col) ~row:(Int32.to_int unit.tile.row)
        in
        let palette = (faction unit).palette in
        let color =
          if palette = {js|passive|js} then {js|#efecdc|js}
          else if palette = {js|green|js} then {js|#83b765|js}
          else if palette = {js|player|js} then {js|#6fdec0|js}
          else {js|#ff9590|js}
        in
        let frame =
          match Art.unit_frame artwork ~side:palette unit.type_ with
          | Some f -> f
          | None ->
              invalid_arg
                (let js_text_part_1 = unit.type_ in
                 {j|Missing unit frame: $(js_text_part_1)/$(palette)|j})
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
        C.draw sc (Option.get (Js.Dict.get images frame.url)) 0. 0.;
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
      (fun ((unit : Domain.starting_unit), _) ->
        let p =
          Geometry.center ~col:(Int32.to_int unit.tile.col) ~row:(Int32.to_int unit.tile.row)
        in
        C.begin_path ctx;
        C.rect ctx (p.x -. 19.) (p.y +. 27.) 38. 4. 2.;
        C.fill ctx {js|#a9d0a5|js};
        C.fill_path ctx)
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
    Node.write output (C.png canvas {js|image/png|js});
    Hashtbl.filter_map_inplace
      (fun path image -> if Hashtbl.mem used_images path then Some image else None)
      decoded_images;
    return ()

let main root = try run root with error -> cli_error error
