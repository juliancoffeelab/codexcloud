let number = Js.Float.toString

let key (lighting : Domain.battlefield_lighting) =
  let optional encode = Option.fold ~none:Js.Json.null ~some:encode in
  Js.Json.stringify
    (Js.Json.array
       [|
         Js.Json.string
           (match lighting.time with
           | Domain.Time_of_day.Day -> "day"
           | Domain.Time_of_day.Night -> "night");
         Js.Json.array
           (Array.map
              (fun (light : Domain.battlefield_light) ->
                Js.Json.array
                  [|
                    Js.Json.number (Int32.to_float light.col);
                    Js.Json.number (Int32.to_float light.row);
                    Js.Json.number (Option.value ~default:0. light.dx);
                    Js.Json.number (Option.value ~default:0. light.dy);
                    Js.Json.number light.radius;
                    Js.Json.number light.power;
                    optional
                      (fun colors ->
                        Js.Json.array
                          (Array.map (fun color -> Js.Json.number (float_of_int color)) colors))
                      light.color;
                    optional Js.Json.boolean light.fixture;
                  |])
              lighting.lights);
       |])

let position (light : Domain.battlefield_light) =
  let point = Geometry.center ~col:(Int32.to_int light.col) ~row:(Int32.to_int light.row) in
  (point.x +. Option.value ~default:0. light.dx, point.y +. Option.value ~default:0. light.dy)

let contribution x y (light : Domain.battlefield_light) =
  let lx, ly = position light in
  let distance = hypot (x -. lx) ((y -. ly) *. 1.12) /. light.radius in
  if distance >= 1. then 0. else light.power *. ((1. -. (distance *. distance)) ** 2.)

let strength x y (lighting : Domain.battlefield_lighting) =
  min 1. (Array.fold_left (fun total light -> total +. contribution x y light) 0. lighting.lights)
  *. 0.88

let warm = [| 1.02; 0.78; 0.49 |]
and offset = [| 22.; 14.; 7. |]

let palette x y (lighting : Domain.battlefield_lighting) =
  let total = ref 0. and colors = Array.make 3 0. and offsets = Array.make 3 0. in
  Array.iter
    (fun (light : Domain.battlefield_light) ->
      let amount = contribution x y light in
      total := !total +. amount;
      for channel = 0 to 2 do
        colors.(channel) <-
          (colors.(channel)
          +. amount
             *.
             match light.color with
             | None -> warm.(channel)
             | Some color -> float_of_int color.(channel) /. 255. *. 1.06);
        offsets.(channel) <-
          (offsets.(channel)
          +. amount
             *.
             match light.color with
             | None -> offset.(channel)
             | Some color -> float_of_int color.(channel) /. 255. *. 30.)
      done)
    lighting.lights;
  if !total = 0. then (warm, offset)
  else
    ( Array.map (fun value -> value /. !total) colors,
      Array.map (fun value -> value /. !total) offsets )

let colored (lighting : Domain.battlefield_lighting) =
  Array.exists (fun (light : Domain.battlefield_light) -> light.color <> None) lighting.lights

let matrix x y (lighting : Domain.battlefield_lighting) =
  if lighting.time = Domain.Time_of_day.Day then
    [| 1.; 0.; 0.; 0.; 0.; 0.; 1.; 0.; 0.; 0.; 0.; 0.; 1.; 0.; 0.; 0.; 0.; 0.; 1.; 0. |]
  else
    let mix = strength x y lighting in
    let luminance = [| 0.2126; 0.7152; 0.0722 |]
    and night = [| 0.28; 0.41; 0.63 |]
    and night_offset = [| 4.; 8.; 17. |] in
    let warm, offset = if colored lighting then palette x y lighting else (warm, offset) in
    Array.init 20 (fun index ->
        let channel = index / 5 and source = index mod 5 in
        if channel = 3 then if source = 3 then 1. else 0.
        else if source = 3 then 0.
        else if source = 4 then
          ((night_offset.(channel) *. (1. -. mix)) +. (offset.(channel) *. mix)) /. 255.
        else
          ((if source = channel then 0.8 else 0.) +. (0.2 *. luminance.(source)))
          *. night.(channel) *. (1. -. mix)
          +. if source = channel then warm.(channel) *. mix else 0.)

type pixels

external get : pixels -> int -> float = "" [@@mel.get_index]
external set : pixels -> int -> float -> unit = "" [@@mel.set_index]

type source = {
  x : float;
  y : float;
  radius : float;
  power : float;
  r : float;
  g : float;
  b : float;
  ro : float;
  go : float;
  bo : float;
}

external source_at : source array -> int -> source = "" [@@mel.get_index]

let shade data width start finish pixel_scale (lighting : Domain.battlefield_lighting) =
  if lighting.time <> Domain.Time_of_day.Day then
    let colored = colored lighting in
    let sources =
      Array.map
        (fun (light : Domain.battlefield_light) ->
          let x, y = position light in
          let r, g, b, ro, go, bo =
            match light.color with
            | Some [| r; g; b |] ->
                let r = float_of_int r and g = float_of_int g and b = float_of_int b in
                ( r /. 255. *. 1.06,
                  g /. 255. *. 1.06,
                  b /. 255. *. 1.06,
                  r /. 255. *. 30.,
                  g /. 255. *. 30.,
                  b /. 255. *. 30. )
            | _ -> (1.02, 0.78, 0.49, 22., 14., 7.)
          in
          { x; y; radius = light.radius; power = light.power; r; g; b; ro; go; bo })
        lighting.lights
    in
    let count = Array.length sources in
    for y = start to finish - 1 do
      let py = float_of_int y /. pixel_scale in
      for x = 0 to width - 1 do
        let index = ((y * width) + x) * 4 in
        if get data (index + 3) <> 0. then (
          let r = get data index and g = get data (index + 1) and b = get data (index + 2) in
          let lum = (0.2126 *. r) +. (0.7152 *. g) +. (0.0722 *. b) in
          if count = 0 then (
            set data index ((((r *. 0.8) +. (lum *. 0.2)) *. 0.28) +. 4.);
            set data (index + 1) ((((g *. 0.8) +. (lum *. 0.2)) *. 0.41) +. 8.);
            set data (index + 2) ((((b *. 0.8) +. (lum *. 0.2)) *. 0.63) +. 17.))
          else
            let px = float_of_int x /. pixel_scale in
            let total = ref 0.
            and wr = ref 0.
            and wg = ref 0.
            and wb = ref 0.
            and ro = ref 0.
            and go = ref 0.
            and bo = ref 0. in
            for index = 0 to count - 1 do
              let source = source_at sources index in
              let distance = hypot (px -. source.x) ((py -. source.y) *. 1.12) /. source.radius in
              if distance < 1. then (
                let amount = source.power *. ((1. -. (distance *. distance)) ** 2.) in
                total := !total +. amount;
                if colored then (
                  wr := !wr +. (amount *. source.r);
                  wg := !wg +. (amount *. source.g);
                  wb := !wb +. (amount *. source.b);
                  ro := !ro +. (amount *. source.ro);
                  go := !go +. (amount *. source.go);
                  bo := !bo +. (amount *. source.bo)))
            done;
            let mix = min 1. !total *. 0.88 in
            let dark = 1. -. mix in
            let use_palette = colored && !total <> 0. in
            let wr = if use_palette then !wr /. !total else 1.02 in
            let wg = if use_palette then !wg /. !total else 0.78 in
            let wb = if use_palette then !wb /. !total else 0.49 in
            let ro = if use_palette then !ro /. !total else 22. in
            let go = if use_palette then !go /. !total else 14. in
            let bo = if use_palette then !bo /. !total else 7. in
            set data index
              ((((r *. 0.8) +. (lum *. 0.2)) *. 0.28 *. dark)
              +. (4. *. dark)
              +. (((r *. wr) +. ro) *. mix));
            set data (index + 1)
              ((((g *. 0.8) +. (lum *. 0.2)) *. 0.41 *. dark)
              +. (8. *. dark)
              +. (((g *. wg) +. go) *. mix));
            set data (index + 2)
              ((((b *. 0.8) +. (lum *. 0.2)) *. 0.63 *. dark)
              +. (17. *. dark)
              +. (((b *. wb) +. bo) *. mix)))
      done
    done

open Browser_canvas

let paint context (lighting : Domain.battlefield_lighting) =
  if lighting.time <> Domain.Time_of_day.Day then (
    save context;
    Array.iter
      (fun (light : Domain.battlefield_light) ->
        let x, y = position light in
        if light.fixture = Some false then (
          let color = Option.value ~default:[| 255; 170; 70 |] light.color in
          let rgba alpha =
            "rgba("
            ^ String.concat ","
                (Array.to_list (Array.map (fun value -> number (float_of_int value)) color))
            ^ "," ^ alpha ^ ")"
          in
          composite context "screen";
          let halo = gradient context x y 0. x y 25. in
          stop halo 0. (rgba ".95");
          stop halo 0.25 (rgba ".45");
          stop halo 1. (rgba "0");
          fill_gradient context halo;
          fill_rect context (x -. 25.) (y -. 25.) 50. 50.;
          composite context "source-over")
        else (
          List.iter
            (fun (color, dx, dy, w, h) ->
              fill context color;
              fill_rect context (x +. dx) (y +. dy) w h)
            [
              ("#49361f", -3., -9., 6., 12.);
              ("#f6aa46", -2., -7., 4., 5.);
              ("#fff0b0", -1., -6., 2., 3.);
              ("#50402e", -1., 3., 2., 8.);
            ];
          composite context "screen";
          let halo = gradient context x (y -. 5.) 0. x (y -. 5.) 20. in
          stop halo 0. "rgba(255,204,110,.65)";
          stop halo 0.25 "rgba(255,169,65,.24)";
          stop halo 1. "rgba(255,145,44,0)";
          fill_gradient context halo;
          fill_rect context (x -. 20.) (y -. 25.) 40. 40.;
          composite context "source-over"))
      lighting.lights;
    restore context)

let fog context plan tiles =
  save context;
  filter context "blur(8px)";
  fill context "#0c1320";
  begin_path context;
  Array.iter
    (fun id ->
      match Array.find_opt (fun (tile : Terrain_plan.tile) -> tile.id = id) plan with
      | None -> invalid_arg "Unknown fog tile"
      | Some tile ->
          String.split_on_char ' ' tile.points
          |> List.iteri (fun index point ->
              match String.split_on_char ',' point with
              | [ x; y ] ->
                  (if index = 0 then move_to else line_to)
                    context (float_of_string x) (float_of_string y)
              | _ -> invalid_arg "Invalid fog polygon");
          close_path context)
    tiles;
  fill_path context;
  restore context

(* Reordering an unchanged multiset still changes fixture/halo paint order. *)
let changed_lights before after =
  let module Map = Host_collections.Map in
  let light_key (light : Domain.battlefield_light) =
    key
      {
        Domain.time = Domain.Time_of_day.Day;
        lights = [| { light with fixture = Some (Option.value ~default:true light.fixture) } |];
      }
  in
  let old = Array.map (fun light -> (light_key light, light)) before
  and next = Array.map (fun light -> (light_key light, light)) after in
  let remaining = Map.create () in
  Array.iter
    (fun (key, _) ->
      let _ = Map.set remaining key (Option.value ~default:0 (Map.get remaining key) + 1) in
      ())
    old;
  let added = [||] and removed = [||] in
  Array.iter
    (fun (key, light) ->
      let count = Option.value ~default:0 (Map.get remaining key) in
      if count > 0 then
        let _ = Map.set remaining key (count - 1) in
        ()
      else
        let _ = Array_ext.append_value added light in
        ())
    next;
  Array.iter
    (fun (key, light) ->
      let count = Option.value ~default:0 (Map.get remaining key) in
      if count > 0 then
        let _ = Array_ext.append_value removed light in
        let _ = Map.set remaining key (count - 1) in
        ())
    old;
  let reordered = Map.create () in
  if Array.length removed = 0 && Array.length added = 0 then
    Array.iteri
      (fun i (old_key, old_light) ->
        let next_key, next_light = next.(i) in
        if old_key <> next_key then
          let _ = Map.set reordered old_key old_light in
          let _ = Map.set reordered next_key next_light in
          ())
      old;
  Array.concat [ removed; added; Map.values_array reordered ]
