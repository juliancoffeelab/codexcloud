type sprite = { url : string; width : float; height : float;
                display_width : float [@mel.as "displayWidth"];
                display_height : float [@mel.as "displayHeight"] }
type authored = { sprite : sprite; map_id : string; cols : float; rows : float;
                  shapes : string Js.Dict.t }
type scene = { plan : Terrain_plan.t; source : Domain.tile array;
               index : Terrain_plan.tile Js.Dict.t; shapes : string Js.Dict.t;
               originals : Domain.tile Js.Dict.t; artwork : Art.t }
type preview = { sprite : sprite; changed : string array; changed_index : bool Js.Dict.t }
type canvas
type bitmap = { scene : scene; canvas : canvas; dimensions : Terrain_plan.dimensions }
type request = { mutable cancelled : bool; mutable cancel : (unit -> unit) list }
type entry = { scene : scene; request : request; promise : bitmap option Js.Promise.t }
type t = { exact : authored Js.Dict.t; maps : authored array;
           mutable prepared : scene option; mutable active : scene option;
           mutable entry : entry option;
           mutable warm : (string * Terrain_plan.image) option }
external finite : float -> bool = "isFinite" [@@mel.scope "Number"]
external integer : float -> bool = "isSafeInteger" [@@mel.scope "Number"]
external freeze : 'a -> 'a = "freeze" [@@mel.scope "Object"]
external new_image : unit -> Terrain_plan.image = "Image" [@@mel.new]
external set_source : Terrain_plan.image -> string -> unit = "src" [@@mel.set]
external set_load : Terrain_plan.image -> (unit -> unit) Js.Nullable.t -> unit = "onload" [@@mel.set]
external set_error : Terrain_plan.image -> (unit -> unit) Js.Nullable.t -> unit = "onerror" [@@mel.set]
external remove_source : Terrain_plan.image -> string -> unit = "removeAttribute" [@@mel.send]
external make_canvas : string -> canvas = "createElement" [@@mel.scope "document"]
external set_width : canvas -> float -> unit = "width" [@@mel.set]
external set_height : canvas -> float -> unit = "height" [@@mel.set]
external set_class : canvas -> string -> unit = "className" [@@mel.set]
external get_context : canvas -> string -> Terrain_plan.context Js.Nullable.t = "getContext" [@@mel.send]
external replace_canvas : Dom.element -> canvas -> unit = "replaceChildren" [@@mel.send]
external empty_children : Dom.element -> unit = "replaceChildren" [@@mel.send]
type timer
external timeout : (unit -> unit) -> int -> timer = "setTimeout"
external clear_timeout : timer -> unit = "clearTimeout"
let stop_image image =
  set_load image Js.Nullable.null; set_error image Js.Nullable.null;
  (try remove_source image "src" with _ -> ())
let cancel request =
  if not request.cancelled then (
    request.cancelled <- true;
    let callbacks = request.cancel in request.cancel <- [];
    List.iter (fun callback -> callback ()) callbacks)
let close owner =
  Option.iter (fun entry -> cancel entry.request) owner.entry;
  owner.entry <- None;
  (* Effect replay may reuse the memoized scene. Release its heavy bitmap and
     asynchronous work; the component's unreachable owner releases geometry. *)
  Option.iter (fun (_, image) -> stop_image image) owner.warm; owner.warm <- None
let json_string value = match Js.Json.decodeString value with Some value -> value | None -> invalid_arg "Terrain key string"
let json_number value = match Js.Json.decodeNumber value with Some value when finite value -> value | _ -> invalid_arg "Terrain key number"
let json_array value = match Js.Json.decodeArray value with Some value -> value | None -> invalid_arg "Terrain key array"
let require value = if not value then invalid_arg "Invalid prebuilt terrain"
let validate key sprite =
  let header = json_array (Js.Json.parseExn key) in require (Array.length header = 4);
  let map_id = json_string header.(0) and cols = json_number header.(1) and rows = json_number header.(2) in
  require (map_id <> "" && integer cols && integer rows && cols > 0. && rows > 0.);
  let geometry = Geometry.dimensions ~cols:(int_of_float cols) ~rows:(int_of_float rows) in
  let width = ceil (geometry.width *. Geometry.native_pixel_scale)
  and height = ceil (geometry.height *. Geometry.native_pixel_scale) in
  require (sprite.url <> "" && finite sprite.width && finite sprite.height &&
           finite sprite.display_width && finite sprite.display_height &&
           sprite.width = width && sprite.height = height &&
           abs_float (sprite.display_width -. width /. Geometry.native_pixel_scale) < 0.000001 &&
           abs_float (sprite.display_height -. height /. Geometry.native_pixel_scale) < 0.000001);
  let shapes = Js.Dict.empty () and coordinates = Js.Dict.empty () in
  let tiles = json_array header.(3) in require (float_of_int (Array.length tiles) = cols *. rows);
  Array.iter (fun tile ->
      let fields = json_array tile in require (Array.length fields = 7);
      let id = json_string fields.(0) and col = json_number fields.(1) and row = json_number fields.(2) in
      require (integer col && integer row && col >= 0. && col < cols && row >= 0. && row < rows);
      let coordinate = Js.Float.toString col ^ "," ^ Js.Float.toString row in
      require (id = coordinate && Js.Dict.get shapes id = None && Js.Dict.get coordinates coordinate = None);
      require (json_string fields.(3) <> "");
      (if fields.(4) <> Js.Json.null then ignore (json_string fields.(4)));
      List.iter (fun index -> if fields.(index) <> Js.Json.null then (
          let seen = Js.Dict.empty () in
          Array.iter (fun edge -> let edge = json_string edge in
                       require (List.mem edge ["e";"se";"sw";"w";"nw";"ne"] && Js.Dict.get seen edge = None);
                       Js.Dict.set seen edge true) (json_array fields.(index)))) [5;6];
      Js.Dict.set coordinates coordinate true; Js.Dict.set shapes id (Js.Json.stringify tile)) tiles;
  {sprite; map_id; cols; rows; shapes}
let create manifest =
  let exact = Js.Dict.empty () in
  let maps = Js.Dict.entries manifest |> Array.to_list |> List.filter_map (fun (key, sprite) ->
      try let authored = validate key sprite in Js.Dict.set exact key authored; Some authored with _ -> None) |> Array.of_list in
  {exact; maps; prepared = None; active = None; entry = None; warm = None}
let key (scene : scene) = scene.plan.key
let tiles (scene : scene) = scene.plan.tiles
let get_tile (scene : scene) id = Js.Dict.get scene.index id
let size (scene : scene) : Geometry.dimensions = {width = scene.plan.width; height = scene.plan.height}
let prepare owner (state : Domain.state) artwork =
  let matching = function
    | Some scene when scene.plan.map_id = state.mapId && scene.plan.cols = state.cols && scene.plan.rows = state.rows && Array.length scene.source = Array.length state.tiles ->
      (scene.source == state.tiles || Array.for_all (fun (tile : Domain.tile) ->
           Option.fold ~none:false ~some:(fun original -> Terrain_plan.same_tile original tile) (Js.Dict.get scene.originals tile.id)) state.tiles) &&
      (scene.artwork == artwork || Array.for_all (fun (tile : Terrain_plan.tile) -> Option.equal (fun a b -> a == b) tile.frame (Art.terrain_frame artwork ?floor:tile.floor tile.name)) scene.plan.tiles)
    | _ -> false in
  let scene =
    if matching owner.prepared then Option.get owner.prepared
    else if matching owner.active then Option.get owner.active
    else (
      let plan = Terrain_plan.prepare ~map_id:state.mapId ~cols:state.cols ~rows:state.rows state.tiles artwork in
      let index = Js.Dict.empty () and shapes = Js.Dict.empty () and originals = Js.Dict.empty () in
      Array.iter (fun (tile : Terrain_plan.tile) -> Js.Dict.set index tile.id tile) plan.tiles;
      Array.iter (fun (tile : Domain.tile) -> Js.Dict.set shapes tile.id (Terrain_plan.shape tile); Js.Dict.set originals tile.id tile) state.tiles;
      {plan; source = state.tiles; index; shapes; originals; artwork}) in
  (* Render may be abandoned. Only lightweight derivation is cached here;
     activation in the commit effect owns cancellation and bitmap validity. *)
  owner.prepared <- Some scene; scene
let activate owner (scene : scene) =
  if not (Option.fold ~none:false ~some:(fun current -> current == scene) owner.active) then (
    Option.iter (fun entry -> cancel entry.request) owner.entry; owner.entry <- None;
    owner.active <- Some scene)
let preview owner scene =
  let authored = match Js.Dict.get owner.exact scene.plan.key with
    | Some exact -> Some exact
    | None -> Array.find_opt (fun authored -> authored.map_id = scene.plan.map_id && authored.cols = scene.plan.cols && authored.rows = scene.plan.rows) owner.maps in
  Option.map (fun (authored : authored) ->
      let changed_index = Js.Dict.empty () in
      let changed = Array.to_list scene.source |> List.filter_map (fun (tile : Domain.tile) ->
          if Js.Dict.get authored.shapes tile.id = Js.Dict.get scene.shapes tile.id then None else (
            Js.Dict.set changed_index tile.id true; Some tile.id)) |> Array.of_list in
      {sprite = authored.sprite; changed; changed_index}) authored
let sprite (preview : preview) = preview.sprite
let changed (preview : preview) = preview.changed
let changed_contains preview id = Js.Dict.get preview.changed_index id = Some true
let valid_bitmap (scene : scene) (bitmap : bitmap) = bitmap.scene == scene
let bitmap_size (bitmap : bitmap) = bitmap.dimensions
let clear node = empty_children node
let mount node (bitmap : bitmap) = set_class bitmap.canvas "map-terrain"; replace_canvas node bitmap.canvas
let decode request source =
  Js.Promise.make (fun ~resolve ~reject:_ ->
      if request.cancelled then (resolve None [@u]) else (
        let image = new_image () and settled = ref false in
        let finish result = if not !settled then (
            settled := true; set_load image Js.Nullable.null; set_error image Js.Nullable.null;
            (resolve result [@u])) in
        request.cancel <- (fun () -> finish None; stop_image image) :: request.cancel;
        set_load image (Js.Nullable.return (fun () -> finish (if request.cancelled then None else Some image)));
        set_error image (Js.Nullable.return (fun () -> finish None));
        try set_source image source with _ -> finish None))
let yield request = Js.Promise.make (fun ~resolve ~reject:_ ->
    let settled = ref false in
    let finish () = if not !settled then (settled := true; let completed = () in (resolve completed [@u])) in
    let timer = timeout finish 0 in
    request.cancel <- (fun () -> clear_timeout timer; finish ()) :: request.cancel)
let render request (scene : scene) =
  let images = Js.Dict.empty () and unique = Js.Dict.empty () in
  Array.iter (fun (tile : Terrain_plan.tile) -> Option.iter (fun (frame : Art.frame) -> Js.Dict.set unique frame.url true) tile.frame) scene.plan.tiles;
  let sources = Js.Dict.keys unique in
  let rec load offset =
    if request.cancelled then Js.Promise.resolve false else if offset >= Array.length sources then Js.Promise.resolve true else (
      let batch = Array.sub sources offset (min 4 (Array.length sources - offset)) in
      Js.Promise.then_ (fun decoded ->
          if request.cancelled || Array.exists Option.is_none decoded then Js.Promise.resolve false else (
            Array.iteri (fun index image -> Js.Dict.set images batch.(index) (Option.get image)) decoded;
            load (offset + Array.length batch))) (Js.Promise.all (Array.map (decode request) batch))) in
  Js.Promise.then_ (fun loaded ->
      if not loaded || request.cancelled then Js.Promise.resolve None else (
        let canvas = make_canvas "canvas" and dimensions = Terrain_plan.dimensions scene.plan in
        set_width canvas dimensions.width; set_height canvas dimensions.height;
        match Js.Nullable.toOption (get_context canvas "2d") with
        | None -> Js.Promise.resolve None
        | Some context ->
          Terrain_plan.configure context scene.plan.pixel_scale;
          let rec draw offset =
            if request.cancelled then Js.Promise.resolve None else (
              let count = min 64 (Array.length scene.plan.tiles - offset) in
              Terrain_plan.paint context (Array.sub scene.plan.tiles offset count) images;
              if offset + count < Array.length scene.plan.tiles then
                Js.Promise.then_ (fun () -> draw (offset + count)) (yield request)
              else (request.cancel <- []; Js.Promise.resolve (Some (freeze {scene; canvas; dimensions})))) in
          draw 0)) (load 0)
let bake owner (scene : scene) =
  if not (Option.fold ~none:false ~some:(fun current -> current == scene) owner.active) then Js.Promise.resolve None else match owner.entry with
    | Some entry when entry.scene == scene -> entry.promise
    | _ ->
      Option.iter (fun entry -> cancel entry.request) owner.entry;
      let request = {cancelled = false; cancel = []} in
      let promise = (try render request scene with _ -> Js.Promise.resolve None)
                    |> Js.Promise.catch (fun _ -> Js.Promise.resolve None) in
      let entry = {scene; request; promise} in owner.entry <- Some entry;
      ignore (Js.Promise.then_ (fun result ->
          if result = None then (
            cancel request;
            if Option.fold ~none:false ~some:(fun current -> current == entry) owner.entry then owner.entry <- None);
          Js.Promise.resolve ()) promise);
      promise
let preload owner map_id =
  match Array.find_opt (fun authored -> authored.map_id = map_id) owner.maps with
  | None -> ()
  | Some authored when Option.fold ~none:false ~some:(fun (url, _) -> url = authored.sprite.url) owner.warm -> ()
  | Some authored ->
    Option.iter (fun (_, image) -> stop_image image) owner.warm;
    (try
       let image = new_image () in owner.warm <- Some (authored.sprite.url, image);
       set_error image (Js.Nullable.return (fun () ->
           if Option.fold ~none:false ~some:(fun (_, current) -> current == image) owner.warm then owner.warm <- None;
           stop_image image));
       set_source image authored.sprite.url
     with _ -> owner.warm <- None)
let preloading = ref None
let preload_manifest manifest map_id =
  let owner = match !preloading with
    | Some (previous, owner) when previous == manifest -> owner
    | previous -> Option.iter (fun (_, owner) -> close owner) previous;
      let owner = create manifest in preloading := Some (manifest, owner); owner in
  preload owner map_id
