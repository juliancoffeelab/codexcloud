type tile = {
  col : float;
  row : float;
  id : string;
  terrain : string;
  x : float;
  y : float;
  points : string;
  sprite_size : float;
  sprite_x : float;
  sprite_y : float;
  sprite_width : float;
  sprite_height : float;
  role : Domain.Terrain_role.t;
  fill : string;
  name : string;
  floor : string option;
  frame : Art.frame option;
}

type dimensions = { width : float; height : float; display_width : float; display_height : float }

external has_own : 'a Js.Dict.t -> string -> bool = "hasOwn" [@@mel.scope "Object"]

let lookup dictionary key = if has_own dictionary key then Js.Dict.get dictionary key else None

let dimensions ~width ~height ~pixel_scale =
  let width = ceil (width *. pixel_scale) and height = ceil (height *. pixel_scale) in
  { width; height; display_width = width /. pixel_scale; display_height = height /. pixel_scale }

type context = Browser_canvas.context
type image = Browser_canvas.image

open Browser_canvas

let configure context pixel_scale =
  set_smoothing context false;
  scale context pixel_scale pixel_scale

let paint context tiles images =
  let tiles = Array.copy tiles in
  let z (tile : tile) =
    Option.fold ~none:0. ~some:(fun (frame : Art.frame) -> frame.z) tile.frame
  in
  Array.stable_sort
    (fun a b ->
      let order = Float.compare (z a) (z b) in
      if order <> 0 then order
      else
        let order = Float.compare a.y b.y in
        if order <> 0 then order else Float.compare a.x b.x)
    tiles;
  Array.iter
    (fun (tile : tile) ->
      save context;
      try
        (match tile.frame with
        | Some frame -> (
            match lookup images frame.url with
            | Some image ->
                set_filter context
                  (if tile.role = Domain.Terrain_role.Forest then {js|brightness(0.93)|js}
                   else {js|none|js});
                draw context image tile.sprite_x tile.sprite_y tile.sprite_width tile.sprite_height
            | None ->
                invalid_arg
                  (let js_text_part_1 = frame.url in
                   {j|Missing decoded terrain image: $(js_text_part_1)|j}))
        | None -> invalid_arg {js|Missing complete terrain frame|js});
        set_filter context {js|none|js};
        restore context
      with error ->
        restore context;
        raise error)
    tiles
