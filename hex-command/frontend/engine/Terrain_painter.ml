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
                  (if tile.role = Domain.Terrain_role.Forest then "brightness(0.93)" else "none");
                draw context image tile.sprite_x tile.sprite_y tile.sprite_width tile.sprite_height
            | None -> invalid_arg ("Missing decoded terrain image: " ^ frame.url))
        | None -> invalid_arg "Missing complete terrain frame");
        set_filter context "none";
        restore context
      with error ->
        restore context;
        raise error)
    tiles
