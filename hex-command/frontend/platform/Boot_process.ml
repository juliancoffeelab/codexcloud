type stage = {
  step : int;
  label : string;
  startedAtMs : float;
  mutable durationMs : float;
  mutable finishedAtMs : float Js.Nullable.t;
}

type event = {
  atMs : float;
  step : int;
  label : string;
  value : float;
  detail : string;
  status : string;
}

type resource_event = { atMs : float; key : string; url : string; phase : string }

type response_headers = {
  atMs : float;
  url : string;
  status : int;
  source : string;
  headers : Js.Json.t;
  readError : bool;
}

type t = {
  version : int;
  startedAt : string;
  timeOrigin : float;
  startedAtMs : float;
  mutable elapsedMs : float;
  mutable status : string;
  mutable step : int;
  mutable label : string;
  mutable value : float;
  mutable detail : string;
  mutable error : string Js.Nullable.t;
  mutable droppedEvents : int;
  stages : stage array;
  events : event array;
  resourceEvents : resource_event array;
  mutable droppedResourceEvents : int;
  resourceTimingBufferFull : bool;
  mutable browserTiming : Js.Json.t;
  responseHeaders : response_headers array;
  mutable droppedResponseHeaders : int;
}

(* Keep this record plain JSON and available even in catalog-free startup. The
   initial HTML installs the same shape before any application module downloads. *)
let get : unit -> t Js.Undefined.t =
  [%mel.raw {|() => (globalThis.window ?? globalThis).BOOT_PROCESS|}]

let set : t -> unit =
  [%mel.raw {|value => { (globalThis.window ?? globalThis).BOOT_PROCESS = value; }|}]

external push_stage : stage array -> stage -> unit = "push" [@@mel.send]
external push_event : event array -> event -> unit = "push" [@@mel.send]
external discard_event : event array -> int -> int -> unit = "splice" [@@mel.send]
external push_resource : resource_event array -> resource_event -> unit = "push" [@@mel.send]
external discard_resource : resource_event array -> int -> int -> unit = "splice" [@@mel.send]
external push_response : response_headers array -> response_headers -> unit = "push" [@@mel.send]
external as_json : t -> Js.Json.t = "%identity"

let ensure () =
  match Js.Undefined.toOption (get ()) with
  | Some process -> process
  | None ->
      let atMs = Browser_time.now () in
      let label = Utf8.to_js "Loading game engine…" in
      let process =
        {
          version = 1;
          startedAt = Utf8.of_js (Js.Date.toISOString (Js.Date.make ()));
          timeOrigin = Browser_time.origin;
          startedAtMs = atMs;
          elapsedMs = 0.;
          status = "loading";
          step = 1;
          label;
          value = 0.;
          detail = "";
          error = Js.Nullable.null;
          droppedEvents = 0;
          resourceEvents = [||];
          droppedResourceEvents = 0;
          resourceTimingBufferFull = false;
          browserTiming = Js.Json.null;
          responseHeaders = [||];
          droppedResponseHeaders = 0;
          stages =
            [|
              {
                step = 1;
                label;
                startedAtMs = atMs;
                durationMs = 0.;
                finishedAtMs = Js.Nullable.null;
              };
            |];
          events = [| { atMs; step = 1; label; value = 0.; detail = ""; status = "loading" } |];
        }
      in
      set process;
      process

let update process ~status ~step ~label ~value ~detail =
  let atMs = Browser_time.now () in
  let previous = process.stages.(Array.length process.stages - 1) in
  previous.durationMs <- atMs -. previous.startedAtMs;
  if previous.step <> step then (
    previous.finishedAtMs <- Js.Nullable.return atMs;
    push_stage process.stages
      { step; label; startedAtMs = atMs; durationMs = 0.; finishedAtMs = Js.Nullable.null });
  let stage = process.stages.(Array.length process.stages - 1) in
  if status <> "loading" then stage.finishedAtMs <- Js.Nullable.return atMs;
  process.elapsedMs <- atMs -. process.startedAtMs;
  process.status <- status;
  process.step <- step;
  process.label <- label;
  process.value <- value;
  process.detail <- detail;
  push_event process.events { atMs; step; label; value; detail; status };
  (* Preserve the initial event and stage timings while bounding resource updates. *)
  if Array.length process.events > 512 then (
    discard_event process.events 1 1;
    process.droppedEvents <- process.droppedEvents + 1)

let progress ~step ~label ~value ~detail =
  let process = ensure () in
  if process.status = "loading" then
    update process
      ~status:(if value >= 5. then "ready" else "loading")
      ~step ~label:(Utf8.to_js label) ~value ~detail:(Utf8.to_js detail)

let fail message =
  let process = ensure () in
  if process.status = "loading" then (
    let message = Utf8.to_js message in
    process.error <- Js.Nullable.return message;
    update process ~status:"failed" ~step:process.step ~label:process.label ~value:process.value
      ~detail:message)

let resource ~key ~url ~phase =
  (* Never create a record for later battles; after boot this is only a global
     lookup and status check. No scans, serialization, observers or layout reads. *)
  match Js.Undefined.toOption (get ()) with
  | Some process when process.status = "loading" ->
      push_resource process.resourceEvents { atMs = Browser_time.now (); key; url; phase };
      if Array.length process.resourceEvents > 512 then (
        discard_resource process.resourceEvents 0 1;
        process.droppedResourceEvents <- process.droppedResourceEvents + 1)
  | _ -> ()

let response ~url ~status ~source ~get_header =
  match Js.Undefined.toOption (get ()) with
  | Some process when process.status = "loading" ->
      if Array.length process.responseHeaders >= 64 then
        process.droppedResponseHeaders <- process.droppedResponseHeaders + 1
      else
        let headers = Js.Dict.empty () in
        let readError = ref false in
        (try
           Array.iter
             (fun name ->
               Option.iter
                 (fun value -> Js.Dict.set headers name (Js.Json.string value))
                 (Js.Nullable.toOption (get_header name)))
             [|
               "cache-control";
               "age";
               "etag";
               "last-modified";
               "expires";
               "date";
               "vary";
               "cf-cache-status";
               "cf-ray";
               "server-timing";
               "cdn-cache-control";
               "surrogate-control";
               "x-cache";
               "via";
               "server";
             |]
         with _ -> readError := true);
        push_response process.responseHeaders
          {
            atMs = Browser_time.now ();
            url;
            status;
            source;
            headers = Js.Json.object_ headers;
            readError = !readError;
          }
  | _ -> ()

let snapshot () =
  let process = ensure () in
  (* Refresh the duration of a stalled stage when exporting, without inventing
     another progress event or modifying a completed startup. *)
  if process.status = "loading" then (
    let atMs = Browser_time.now () in
    process.elapsedMs <- atMs -. process.startedAtMs;
    let stage = process.stages.(Array.length process.stages - 1) in
    stage.durationMs <- atMs -. stage.startedAtMs);
  process.browserTiming <-
    Browser_timing.snapshot ~recorder_started_at_ms:process.startedAtMs
      ~until_ms:(process.startedAtMs +. process.elapsedMs);
  Js.Json.parseExn (Js.Json.stringify (as_json process))
