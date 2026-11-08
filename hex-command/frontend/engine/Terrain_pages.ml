(* Offline page metadata only; scenario definitions always come from Rust. *)
type page = {
  url : string;
  width : int;
  height : int;
  x : int;
  y : int;
  interior_width : int; [@mel.as "interiorWidth"]
  interior_height : int; [@mel.as "interiorHeight"]
  gutter : int;
}

type sprite = {
  width : int;
  height : int;
  display_width : float; [@mel.as "displayWidth"]
  display_height : float; [@mel.as "displayHeight"]
  pages : page array;
}

let interior = 1020
let gutter = 2

external sprites : sprite Js.Dict.t = "default" [@@mel.module "virtual:terrain-prebuilt"]

let asset_key map_id revision = Terrain_plan.asset_key (map_id ^ ":" ^ Js.Float.toString revision)

let frame (page : page) : Art.frame =
  {
    url = page.url;
    width = float_of_int page.width;
    height = float_of_int page.height;
    anchor_x = 0.;
    anchor_y = 0.;
    z = 0.;
  }

let additional : sprite Js.Dict.t = Js.Dict.empty ()
let register entries = Array.iter (fun (key, value) -> Js.Dict.set additional key value) entries

let find key =
  match Js.Dict.get additional key with Some _ as value -> value | None -> Js.Dict.get sprites key
