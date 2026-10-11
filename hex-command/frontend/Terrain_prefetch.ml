type cancel = unit -> unit
type defer = (unit -> unit) -> cancel
type load = string -> unit Js.Promise.t
type scheduled = { generation : int; mutable cancel : cancel option; mutable fired : bool }
type t = { defer : defer; load : load; loaded : bool Js.Dict.t;
           loading : bool Js.Dict.t; mutable active : int; mutable epoch : int;
           mutable queue : string array; mutable offset : int;
           mutable scheduled : scheduled option; mutable draining : bool }
type frame
type idle
type timer
type image
external request_frame : (float -> unit) -> frame = "requestAnimationFrame"
external cancel_frame : frame -> unit = "cancelAnimationFrame"
external request_idle : (unit -> unit) -> idle = "requestIdleCallback"
external cancel_idle : idle -> unit = "cancelIdleCallback"
external timeout : (unit -> unit) -> int -> timer = "setTimeout"
external clear_timeout : timer -> unit = "clearTimeout"
external new_image : unit -> image = "Image" [@@mel.new]
external set_priority : image -> string -> unit = "fetchPriority" [@@mel.set]
external set_decoding : image -> string -> unit = "decoding" [@@mel.set]
external set_source : image -> string -> unit = "src" [@@mel.set]
external decode : image -> unit Js.Promise.t = "decode" [@@mel.send]
external delete : 'a Js.Dict.t -> string -> bool = "deleteProperty" [@@mel.scope "Reflect"]
let has_idle : unit -> bool = [%mel.raw {|function () { return typeof requestIdleCallback === "function" && typeof cancelIdleCallback === "function"; }|}]
let after_paint task =
  let cancelled = ref false and cancel_later = ref None in
  let run () = if not !cancelled then task () in
  let frame = request_frame (fun _ -> if not !cancelled then (
      if has_idle () then (
        let idle = request_idle run in cancel_later := Some (fun () -> cancel_idle idle))
      else (
        let timer = timeout run 0 in cancel_later := Some (fun () -> clear_timeout timer)))) in
  fun () -> if not !cancelled then (
      cancelled := true; cancel_frame frame; Option.iter (fun cancel -> cancel ()) !cancel_later)
let load_image url =
  let image = new_image () in
  set_priority image "low"; set_decoding image "async"; set_source image url; decode image
let create ?(defer=after_paint) ?(load=load_image) () =
  {defer; load; loaded = Js.Dict.empty (); loading = Js.Dict.empty ();
   active = 0; epoch = 0; queue = [||]; offset = 0; scheduled = None; draining = false}
let close owner =
  owner.epoch <- owner.epoch + 1; owner.queue <- [||]; owner.offset <- 0;
  let pending = owner.scheduled in owner.scheduled <- None;
  Option.iter (fun scheduled -> Option.iter (fun cancel -> cancel ()) scheduled.cancel) pending
let rec schedule owner =
  if not owner.draining && owner.scheduled = None && owner.offset < Array.length owner.queue && owner.active < 2 then (
    let ticket = {generation = owner.epoch; cancel = None; fired = false} in
    owner.scheduled <- Some ticket;
    try
      let cancel = owner.defer (fun () ->
          ticket.fired <- true;
          if owner.epoch = ticket.generation && Option.fold ~none:false ~some:(fun current -> current == ticket) owner.scheduled then (
            owner.scheduled <- None; owner.draining <- true;
            let attempts = ref 0 in
            while owner.offset < Array.length owner.queue && owner.active < 2 && !attempts < 2 && owner.epoch = ticket.generation do
              let url = owner.queue.(owner.offset) in owner.offset <- owner.offset + 1; incr attempts;
              owner.active <- owner.active + 1; Js.Dict.set owner.loading url true;
              let finish success =
                if success then Js.Dict.set owner.loaded url true;
                ignore (delete owner.loading url); owner.active <- owner.active - 1;
                schedule owner in
              match (try Some (owner.load url) with _ -> None) with
              | None -> finish false
              | Some pending ->
                let observed = Js.Promise.then_ (fun () -> Js.Promise.resolve true) pending
                               |> Js.Promise.catch (fun _ -> Js.Promise.resolve false) in
                ignore (Js.Promise.then_ (fun success -> finish success; Js.Promise.resolve ()) observed)
            done; owner.draining <- false;
            schedule owner)) in
      ticket.cancel <- Some cancel;
      if owner.epoch <> ticket.generation && not ticket.fired then cancel ()
    with _ ->
      if Option.fold ~none:false ~some:(fun current -> current == ticket) owner.scheduled then owner.scheduled <- None)
let start owner scene =
  close owner;
  let unique = Js.Dict.empty () in
  owner.queue <- Terrain.tiles scene |> Array.to_list |> List.filter_map (fun (tile : Terrain_plan.tile) ->
      match tile.frame with
      | Some frame when Js.Dict.get unique frame.url = None ->
        Js.Dict.set unique frame.url true;
        if Js.Dict.get owner.loaded frame.url = None && Js.Dict.get owner.loading frame.url = None then Some frame.url else None
      | _ -> None) |> Array.of_list;
  let epoch = owner.epoch in schedule owner;
  fun () -> if owner.epoch = epoch then close owner
