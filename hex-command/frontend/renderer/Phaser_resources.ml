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
      ({js|texture|js}, Performance_trace.int limits.texture);
      ({js|target|js}, Performance_trace.int limits.target);
      ({js|viewport|js}, Js.Json.array (Array.map Performance_trace.int limits.viewport));
      ({js|attributes|js}, Performance_trace.int limits.attributes);
      ({js|varyings|js}, Performance_trace.int limits.varyings);
      ({js|textures|js}, Performance_trace.int limits.textures);
    ]

let snapshot_json (snapshot : snapshot) =
  let amount (amount : amount) =
    Performance_trace.obj
      [ ({js|cpu|js}, Js.Json.number amount.cpu); ({js|gpu|js}, Js.Json.number amount.gpu) ]
  in
  let groups =
    Array.map
      (fun (kind, (group : group)) ->
        ( kind,
          Performance_trace.obj
            [
              ({js|cpu|js}, Js.Json.number group.cpu);
              ({js|gpu|js}, Js.Json.number group.gpu);
              ({js|count|js}, Performance_trace.int group.count);
            ] ))
      (Js.Dict.entries snapshot.groups)
  in
  Performance_trace.obj
    [
      ({js|groups|js}, Js.Json.object_ (Js.Dict.fromArray groups));
      ({js|totals|js}, amount snapshot.totals);
      ({js|peaks|js}, amount snapshot.peaks);
    ]

class resources (game : Phaser.game) =
  let renderer =
    match Js.Nullable.toOption game##renderer with
    | Some renderer -> renderer
    | None -> failwith {js|Battlefield WebGL renderer is unavailable|js}
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
      if limits.attributes < 8 || limits.varyings < 7 || limits.textures < 4 then
        failwith
          (let attributes = string_of_int limits.attributes in
           let varyings = string_of_int limits.varyings in
           let textures = string_of_int limits.textures in
           {j|Unsupported battlefield WebGL limits: attributes=$attributes, varyings=$varyings, textures=$textures|j})

    initializer
      Performance_trace.instant {js|Battlefield device limits|js} ~args:(limits_json limits)

    method texture = texture
    method target = target
    method limits = limits

    method area label width height =
      if
        not
          (safe_integer width && safe_integer height && width > 0. && height > 0.
          && safe_integer (width *. height *. 28.))
      then
        failwith
          (let width = Js.String.make width and height = Js.String.make height in
           {j|Invalid battlefield dimensions $label: $width×$height|j});
      width *. height

    method check ?(target = false) label width height =
      let area = self#area label width height in
      let limit = if target then limits.target else texture in
      if width > float_of_int limit || height > float_of_int limit then
        failwith
          (let width = Js.String.make width and height = Js.String.make height in
           let limit = string_of_int limit in
           {j|Unsupported battlefield allocation $label: $width×$height; device limit $limit|j});
      area

    method image key image =
      let area =
        self#check key (float_of_int (natural_width image)) (float_of_int (natural_height image))
      in
      self#set key {js|decoded image + source texture|js} (area *. 4.) (area *. 4.)

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
          self#set {js|main-color|js} {js|main drawing buffer|js} 0.
            (float_of_int canvas##width *. float_of_int canvas##height *. 4.))
        (Js.Nullable.toOption game##canvas);
      Option.iter
        (fun (capture : Phaser.render_target) ->
          self#set {js|native-capture|js} {js|native Phaser capture target|js} 0.
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
      Performance_trace.instant {js|Battlefield owned resource estimates|js}
        ~args:(snapshot_json self#snapshot)
  end
