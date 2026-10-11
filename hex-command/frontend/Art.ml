type style = Normal | Low_poly | Pixel
type frame = { url : string; width : float; height : float; pixel : bool }
type t = { style : style; frames : frame Js.Dict.t }
type source = { filename : string; width : float; height : float; revision : string }
type sheet = { source : source; frames : float array Js.Dict.t }
type definition = { source_url : string; crop : float array; pixel : bool }
type image
type canvas
type context
external pixel_sources : source Js.Dict.t = "pixelSources" [@@mel.module "@assets/pixel-art"]
external pixel_frames : float array Js.Dict.t = "pixelFrames" [@@mel.module "@assets/pixel-art"]
external pixel_models : string Js.Dict.t = "pixelUnitModels" [@@mel.module "@assets/pixel-art"]
external pixel_colors : string Js.Dict.t = "pixelTerrainColors" [@@mel.module "@assets/pixel-art"]
external styled_sheets : sheet Js.Dict.t = "styledSheets" [@@mel.module "@assets/styled-art"]
external styled_terrain : sheet Js.Dict.t = "styledTerrainSheets" [@@mel.module "@assets/styled-art"]
external styled_colors : string Js.Dict.t Js.Dict.t = "styledTerrainColors" [@@mel.module "@assets/styled-art"]
external storage_get : string -> string Js.Nullable.t = "getItem" [@@mel.scope "localStorage"]
external storage_set : string -> string -> unit = "setItem" [@@mel.scope "localStorage"]
external delete : 'a Js.Dict.t -> string -> unit = "deleteProperty" [@@mel.scope "Reflect"]
external freeze : 'a -> 'a = "freeze" [@@mel.scope "Object"]
external new_image : unit -> image = "Image" [@@mel.new]
external on_load : image -> (unit -> unit) -> unit = "onload" [@@mel.set]
external on_error : image -> (unit -> unit) -> unit = "onerror" [@@mel.set]
external set_src : image -> string -> unit = "src" [@@mel.set]
external create_canvas : string -> canvas = "createElement" [@@mel.scope "document"]
external set_width : canvas -> float -> unit = "width" [@@mel.set]
external set_height : canvas -> float -> unit = "height" [@@mel.set]
external get_width : canvas -> float = "width" [@@mel.get]
external get_height : canvas -> float = "height" [@@mel.get]
external get_context : canvas -> string -> context Js.Nullable.t = "getContext" [@@mel.send]
external smoothing : context -> bool -> unit = "imageSmoothingEnabled" [@@mel.set]
external draw : context -> image -> float -> float -> float -> float -> float -> float -> float -> float -> unit = "drawImage" [@@mel.send]
external data_url : canvas -> string -> string = "toDataURL" [@@mel.send]
let base_url : string = [%mel.raw "import.meta.env?.BASE_URL ?? '/' "]
let styles = [|Normal, "Normal"; Low_poly, "Low-poly"; Pixel, "Pixel"|]
let style_name = function Normal -> "normal" | Low_poly -> "low-poly" | Pixel -> "pixel"
let style_of_string = function "normal" -> Some Normal | "low-poly" -> Some Low_poly | "pixel" -> Some Pixel | _ -> None
let saved_style () = try
    match Js.Nullable.toOption (storage_get "hex-command-art-style") with
    | Some value -> Option.value (style_of_string value) ~default:Normal | None -> Normal
  with _ -> Normal
let save_style style = try storage_set "hex-command-art-style" (style_name style) with _ -> ()
let published = Hashtbl.create 3
let artwork style = match Hashtbl.find_opt published style with
  | Some artwork -> artwork
  | None -> let artwork = freeze { style; frames = freeze (Js.Dict.empty ()) } in
    Hashtbl.add published style artwork; artwork
let style (artwork : t) = artwork.style
let find (artwork : t) key = Js.Dict.get artwork.frames key
let source_url source = base_url ^ "assets/" ^ source.filename ^ "?v=" ^ source.revision
let require dict key = match Js.Dict.get dict key with Some value -> value | None -> failwith ("Missing artwork metadata: " ^ key)
let definitions style =
  let definitions = Js.Dict.empty () in
  let add source pixel frames =
    Js.Dict.entries frames |> Array.iter (fun (key, crop) ->
        Js.Dict.set definitions key { source_url = source_url source; crop; pixel }) in
  (match style with
   | Pixel -> Js.Dict.entries pixel_frames |> Array.iter (fun (key, crop) ->
       let source_kind = if String.starts_with ~prefix:"terrain_" key then "terrain" else "units" in
       Js.Dict.set definitions key { source_url = source_url (require pixel_sources source_kind); crop; pixel = true })
   | Normal | Low_poly ->
     let key = style_name style in
     let terrain = require styled_terrain key and units = require styled_sheets key in
     add terrain.source false terrain.frames;
     add units.source false units.frames);
  definitions
let decoding : image option Js.Promise.t Js.Dict.t = Js.Dict.empty ()
let decode url = match Js.Dict.get decoding url with
  | Some promise -> promise
  | None ->
    let promise = Js.Promise.make (fun ~resolve ~reject:_ ->
        try
          let image = new_image () in
          on_load image (fun () -> (resolve (Some image) [@u]));
          on_error image (fun () -> (resolve None [@u]));
          set_src image url
        with _ -> (resolve None [@u])) in
    Js.Dict.set decoding url promise;
    ignore (Js.Promise.then_ (fun image -> delete decoding url; Js.Promise.resolve image) promise);
    promise
let extract definition image =
  let crop = definition.crop in
  if Array.length crop <> 4 || crop.(2) <= 0. || crop.(3) <= 0. then failwith "Invalid sprite crop";
  let canvas = create_canvas "canvas" in
  set_width canvas (ceil crop.(2)); set_height canvas (ceil crop.(3));
  let context = match Js.Nullable.toOption (get_context canvas "2d") with
    | Some context -> context | None -> failwith "Canvas unavailable" in
  smoothing context (not definition.pixel);
  draw context image crop.(0) crop.(1) crop.(2) crop.(3) 0. 0. (get_width canvas) (get_height canvas);
  freeze { url = data_url canvas "image/png"; width = get_width canvas; height = get_height canvas; pixel = definition.pixel }
let loading : (style, bool Js.Promise.t) Hashtbl.t = Hashtbl.create 3
let load style = match Hashtbl.find_opt loading style with
  | Some promise -> promise
  | None ->
    let promise = try
        let definitions = definitions style in
        let sources = Hashtbl.create 3 in
        Js.Dict.values definitions |> Array.iter (fun definition -> Hashtbl.replace sources definition.source_url ());
        let urls = Hashtbl.to_seq_keys sources |> Array.of_seq in
        Js.Promise.then_ (fun decoded ->
            try
              let images = Hashtbl.create (Array.length urls) in
              Array.iteri (fun index -> function
                  | Some image -> Hashtbl.add images urls.(index) image
                  | None -> failwith "Artwork unavailable") decoded;
              let frames = Js.Dict.empty () in
              Js.Dict.entries definitions |> Array.iter (fun (key, definition) ->
                  Js.Dict.set frames key (extract definition (Hashtbl.find images definition.source_url)));
              Hashtbl.replace published style (freeze { style; frames = freeze frames });
              Js.Promise.resolve true
            with _ -> Js.Promise.resolve false)
          (Js.Promise.all (Array.map decode urls))
      with _ -> Js.Promise.resolve false in
    let guarded = Js.Promise.then_ (fun ready ->
        if not ready then Hashtbl.remove loading style;
        Js.Promise.resolve ready) promise in
    Hashtbl.add loading style guarded;
    guarded
let frame_name = function "bonearcher" -> "bone_archer" | "grove" -> "sacred_grove" | name -> name
let unit_frame artwork ~side name =
  let name = if artwork.style = Pixel then Option.value (Js.Dict.get pixel_models name) ~default:name else name in
  find artwork (side ^ "_" ^ frame_name name)
let terrain_frame artwork name = find artwork ("terrain_" ^ frame_name name)
let portrait_frame artwork = function "worker" -> find artwork "portrait_worker" | name -> unit_frame artwork ~side:"player" name
let color artwork terrain =
  let colors = if artwork.style = Pixel then pixel_colors else require styled_colors (style_name artwork.style) in
  Option.value (Js.Dict.get colors terrain) ~default:(Geometry.color terrain)
let escape value =
  let buffer = Buffer.create (String.length value) in
  String.iter (function
      | '&' -> Buffer.add_string buffer "&amp;" | '<' -> Buffer.add_string buffer "&lt;"
      | '>' -> Buffer.add_string buffer "&gt;" | '"' -> Buffer.add_string buffer "&quot;"
      | '\'' -> Buffer.add_string buffer "&#39;" | character -> Buffer.add_char buffer character) value;
  Buffer.contents buffer
let number = Js.Float.toString
let render (frame : frame option) ~x ~y ~width ~height ~class_name ~side = match frame with
  | None -> ""
  | Some frame ->
    "<svg class=\"art-sprite " ^ (if frame.pixel then "pixel-art " else "") ^ escape class_name ^
    "\" " ^ (match side with None -> "" | Some side -> "data-faction=\"" ^ escape side ^ "\" ") ^
    "x=\"" ^ number x ^ "\" y=\"" ^ number y ^ "\" width=\"" ^ number width ^ "\" height=\"" ^ number height ^
    "\" viewBox=\"0 0 " ^ number frame.width ^ " " ^ number frame.height ^
    "\" aria-hidden=\"true\" focusable=\"false\"><image href=\"" ^ escape frame.url ^
    "\" x=\"0\" y=\"0\" width=\"" ^ number frame.width ^ "\" height=\"" ^ number frame.height ^
    "\" preserveAspectRatio=\"none\"/></svg>"
let sprite artwork ~kind ~name ~x ~y ~width ~height ?(class_name="") ?(side="player") () =
  let frame, side = if kind = "units" then unit_frame artwork ~side name, Some side else terrain_frame artwork name, None in
  render frame ~x ~y ~width ~height ~class_name ~side
let portrait artwork name ~x ~y ~width ~height =
  render (portrait_frame artwork name) ~x ~y ~width ~height ~class_name:"" ~side:None
let icon_path = function
  | "move" -> {js|<path d="m5 17 4-4 3 3 7-9M14 7h5v5"/>|js}
  | "blade" -> {js|<path d="m5 19 3-3m-2-3 5 5m-3-4L17 4l3-1-1 3-10 9"/>|js}
  | "pierce" -> {js|<path d="m5 19 14-14M13 5h6v6M5 14v5h5"/>|js}
  | "impact" -> {js|<path d="m5 19 10-10M11 5l4-3 7 7-3 4z"/>|js}
  | "range" -> {js|<circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/><path d="M12 2v3m0 14v3M2 12h3m14 0h3"/>|js}
  | "shield" -> {js|<path d="M12 3 4 6v6c0 4 4 7 8 9 4-2 8-5 8-9V6z"/>|js}
  | "blocked" -> {js|<path d="M3 12h18m-4-4 4 4-4 4M8 4v16m8-16L8 20"/>|js}
  | "coin" -> {js|<circle cx="12" cy="12" r="8"/><path d="M14 8h-3a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4h-3m2-10v12"/>|js}
  | "heal" -> {js|<path d="M9 3h6v6h6v6h-6v6H9v-6H3V9h6z"/>|js}
  | "recruit" -> {js|<circle cx="9" cy="7" r="3"/><path d="M3 21v-3a6 6 0 0 1 12 0v3m4-12v8m-4-4h8"/>|js}
  | "keep" -> {js|<path d="M4 21V3h4v4h3V3h3v4h3V3h3v18zM10 21v-6h4v6"/>|js}
  | "search" -> {js|<circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/>|js}
  | _ -> ""
let icon_markup ?(class_name="") name =
  "<svg class=\"ui-icon" ^ (if class_name = "" then "" else " " ^ escape class_name) ^
  "\" viewBox=\"0 0 24 24\" aria-hidden=\"true\" focusable=\"false\">" ^ icon_path name ^ "</svg>"
