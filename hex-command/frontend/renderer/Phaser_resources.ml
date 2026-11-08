open Phaser_dom
module Map = Host_collections.Map

type limits = {
  texture : int;
  target : int;
  viewport : int array;
  attributes : int;
  varyings : int;
  textures : int;
}

type allocation = { kind : string; cpu : float; gpu : float }
type group = { mutable cpu : float; mutable gpu : float; mutable count : int }
type amount = { cpu : float; gpu : float }
type snapshot = { groups : group Js.Dict.t; totals : amount; peaks : amount }

external safe_integer : float -> bool = "isSafeInteger" [@@mel.scope "Number"]

let limits_json (limits : limits) =
  Performance_trace.obj
    [
      ("texture", Performance_trace.int limits.texture);
      ("target", Performance_trace.int limits.target);
      ("viewport", Js.Json.array (Array.map Performance_trace.int limits.viewport));
      ("attributes", Performance_trace.int limits.attributes);
      ("varyings", Performance_trace.int limits.varyings);
      ("textures", Performance_trace.int limits.textures);
    ]

let snapshot_json (snapshot : snapshot) =
  let amount (amount : amount) =
    Performance_trace.obj [ ("cpu", Js.Json.number amount.cpu); ("gpu", Js.Json.number amount.gpu) ]
  in
  let groups =
    Array.map
      (fun (kind, (group : group)) ->
        ( kind,
          Performance_trace.obj
            [
              ("cpu", Js.Json.number group.cpu);
              ("gpu", Js.Json.number group.gpu);
              ("count", Performance_trace.int group.count);
            ] ))
      (Js.Dict.entries snapshot.groups)
  in
  Performance_trace.obj
    [
      ("groups", Js.Json.object_ (Js.Dict.fromArray groups));
      ("totals", amount snapshot.totals);
      ("peaks", amount snapshot.peaks);
    ]

class resources (game : Phaser.game) =
  let renderer =
    match Js.Nullable.toOption game##renderer with
    | Some renderer -> renderer
    | None -> failwith "Battlefield WebGL renderer is unavailable"
  in
  let gl = renderer##gl in
  let viewport = Webgl.viewport_array (Webgl.viewport_parameter gl 0x0d3a) in
  let texture = Webgl.parameter gl 0x0d33 in
  let target = Array.fold_left min (min texture (Webgl.parameter gl 0x84e8)) viewport in
  let limits =
    {
      texture;
      target;
      viewport;
      attributes = Webgl.parameter gl 0x8869;
      varyings = Webgl.parameter gl 0x8dfc;
      textures = Webgl.parameter gl 0x8872;
    }
  in
  object (self)
    val owned : (string, allocation) Map.t = Map.create ()
    val mutable cpu = 0.
    val mutable gpu = 0.
    val mutable peak_cpu = 0.
    val mutable peak_gpu = 0.

    initializer
      if limits.attributes < 7 || limits.varyings < 6 || limits.textures < 3 then
        failwith
          (Printf.sprintf
             "Unsupported battlefield WebGL limits: attributes=%d, varyings=%d, textures=%d"
             limits.attributes limits.varyings limits.textures)

    initializer Performance_trace.instant "Battlefield device limits" ~args:(limits_json limits)
    method texture = texture
    method target = target
    method limits = limits

    method area label width height =
      if
        not
          (safe_integer width && safe_integer height && width > 0. && height > 0.
          && safe_integer (width *. height *. 28.))
      then failwith (Printf.sprintf "Invalid battlefield dimensions %s: %g×%g" label width height);
      width *. height

    method check ?(target = false) label width height =
      let area = self#area label width height in
      let limit = if target then limits.target else texture in
      if width > float_of_int limit || height > float_of_int limit then
        failwith
          (Printf.sprintf "Unsupported battlefield allocation %s: %g×%g; device limit %d" label
             width height limit);
      area

    method image key image =
      let area =
        self#check key (float_of_int (natural_width image)) (float_of_int (natural_height image))
      in
      self#set key "decoded image + source texture" (area *. 4.) (area *. 4.)

    method set key kind next_cpu next_gpu =
      let old_cpu, old_gpu =
        match Map.get owned key with None -> (0., 0.) | Some old -> (old.cpu, old.gpu)
      in
      cpu <- cpu +. next_cpu -. old_cpu;
      gpu <- gpu +. next_gpu -. old_gpu;
      peak_cpu <- max peak_cpu cpu;
      peak_gpu <- max peak_gpu gpu;
      let _ = Map.set owned key { kind; cpu = next_cpu; gpu = next_gpu } in
      ()

    method remove key =
      Option.iter
        (fun (old : allocation) ->
          cpu <- cpu -. old.cpu;
          gpu <- gpu -. old.gpu)
        (Map.get owned key);
      let _ = Map.delete owned key in
      ()

    method snapshot =
      Option.iter
        (fun (canvas : Phaser.canvas) ->
          self#set "main-color" "main drawing buffer" 0.
            (float_of_int canvas##width *. float_of_int canvas##height *. 4.))
        (Js.Nullable.toOption game##canvas);
      Option.iter
        (fun (capture : Phaser.render_target) ->
          self#set "native-capture" "native Phaser capture target" 0.
            (float_of_int capture##width
            *. float_of_int capture##height
            *. if capture##hasDepthBuffer then 8. else 4.))
        (Js.Nullable.toOption renderer##renderTarget);
      let groups = Js.Dict.empty () in
      Map.each owned (fun[@u] value _ ->
          let group =
            match Js.Dict.get groups value.kind with
            | Some group -> group
            | None ->
                let group = { cpu = 0.; gpu = 0.; count = 0 } in
                Js.Dict.set groups value.kind group;
                group
          in
          group.cpu <- group.cpu +. value.cpu;
          group.gpu <- group.gpu +. value.gpu;
          group.count <- group.count + 1);
      { groups; totals = { cpu; gpu }; peaks = { cpu = peak_cpu; gpu = peak_gpu } }

    method publish =
      Performance_trace.instant "Battlefield owned resource estimates"
        ~args:(snapshot_json self#snapshot)
  end
