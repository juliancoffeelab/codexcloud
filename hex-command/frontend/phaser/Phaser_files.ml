(* Pinned Phaser 3.90 File subclasses. The native File owns downloading and cache
   insertion; these callbacks preserve causes, deadlines and cancellation. *)
type timer
type failure = { message : string; cause : Foreign_error.cause option }

let describe failure =
  failure.message
  ^ Option.fold ~none:""
      ~some:(fun cause -> "\nCaused by: " ^ Foreign_error.cause_detail cause)
      failure.cause

external timeout : (unit -> unit) -> int -> timer = "setTimeout"
external clear_timeout : timer -> unit = "clearTimeout"
external set_state : Phaser.file -> int -> unit = "state" [@@mel.set]
external set_image_load : Phaser.image_element -> unit Js.Nullable.t -> unit = "onload" [@@mel.set]

external set_image_error : Phaser.image_element -> unit Js.Nullable.t -> unit = "onerror"
[@@mel.set]

external remove_source : Phaser.image_element -> string -> unit = "removeAttribute" [@@mel.send]
external set_xhr_timeout : Phaser.xhr -> unit Js.Nullable.t -> unit = "ontimeout" [@@mel.set]
external set_xhr_abort : Phaser.xhr -> unit Js.Nullable.t -> unit = "onabort" [@@mel.set]
external abort : Phaser.xhr -> unit = "abort" [@@mel.send]
external processing : int = "FILE_PROCESSING" [@@mel.module "phaser"] [@@mel.scope "Loader"]

external pending_destroy : int = "FILE_PENDING_DESTROY"
[@@mel.module "phaser"] [@@mel.scope "Loader"]

(* Native File.data is empty before load, an Image during download/decode and
   an AudioBuffer after audio processing. Keep that phase distinction narrow. *)
let loaded_image : Phaser.file -> Phaser.image_element option =
  [%mel.raw {|file => file.data instanceof HTMLImageElement ? file.data : undefined|}]

external image_data : Phaser.file -> Phaser.image_element = "data" [@@mel.get]

type array_buffer

external response : Phaser.xhr -> array_buffer = "response" [@@mel.get]

external decode : Phaser.audio_context -> array_buffer -> Phaser.audio_buffer Js.Promise.t
  = "decodeAudioData"
[@@mel.send]

external set_audio : Phaser.file -> Phaser.audio_buffer -> unit = "data" [@@mel.set]
external audio_length : Phaser.audio_buffer -> int = "length" [@@mel.get]
external audio_channels : Phaser.audio_buffer -> int = "numberOfChannels" [@@mel.get]

type image_config = { key : string; url : string }
type audio_url = { kind : string; [@mel.as "type"] url : string }
type image_base
type audio_base

external image_base : image_base = "ImageFile"
[@@mel.module "phaser"] [@@mel.scope "Loader", "FileTypes"]

external audio_base : audio_base = "AudioFile"
[@@mel.module "phaser"] [@@mel.scope "Loader", "FileTypes"]

external initialize_image : image_base -> Phaser.file -> Phaser.loader -> image_config -> unit
  = "call"
[@@mel.send]

external initialize_audio :
  audio_base ->
  Phaser.file ->
  Phaser.loader ->
  string ->
  audio_url ->
  unit Js.Undefined.t ->
  Phaser.audio_context ->
  unit = "call"
[@@mel.send]

type prototype =
  < load : < call : Phaser.file -> unit [@mel.meth] > Js.t
  ; destroy : < call : Phaser.file -> unit [@mel.meth] > Js.t
  ; onProcessComplete : < call : Phaser.file -> unit [@mel.meth] > Js.t >
  Js.t

external image_prototype : image_base -> prototype = "prototype" [@@mel.get]
external audio_prototype : audio_base -> prototype = "prototype" [@@mel.get]

let image_lifecycle = image_prototype image_base
let audio_lifecycle = audio_prototype audio_base

let detach_image image =
  set_image_load image Js.Nullable.null;
  set_image_error image Js.Nullable.null

let reset_xhr (file : Phaser.file) =
  file##resetXHR ();
  Option.iter
    (fun xhr ->
      set_xhr_timeout xhr Js.Nullable.null;
      set_xhr_abort xhr Js.Nullable.null;
      abort xhr)
    (Js.Nullable.toOption file##xhrLoader)

type load_function = < call : Phaser.file -> unit [@mel.meth] > Js.t

external load_function : Phaser.file -> load_function = "load" [@@mel.get]
external set_load : Phaser.file -> ((unit -> unit)[@u]) -> unit = "load" [@@mel.set]

external then_decode :
  Phaser.audio_buffer Js.Promise.t ->
  ((Phaser.audio_buffer -> unit)[@u]) ->
  ((Foreign_error.t -> unit)[@u]) ->
  unit = "then"
[@@mel.send]

type image_init = { loader : Phaser.loader; url : string; defer : (Phaser.file -> unit) option }

type image_owner = {
  mutable deadline : timer option;
  mutable complete : bool;
  mutable abandoned : bool;
  mutable failure : failure option;
  defer : (Phaser.file -> unit) option;
}

external owner_image : Phaser.file -> image_owner = "ocamlPreparedImage" [@@mel.get]
external set_owner_image : Phaser.file -> image_owner -> unit = "ocamlPreparedImage" [@@mel.set]

external optional_image_owner : Phaser.file -> image_owner Js.Undefined.t = "ocamlPreparedImage"
[@@mel.get]

let image_failure file =
  Option.bind (Js.Undefined.toOption (optional_image_owner file)) (fun owner -> owner.failure)

let admit (file : Phaser.file) =
  if not (owner_image file).abandoned then image_lifecycle##onProcessComplete##call file

type image_definition =
  < initialize : (Phaser.file -> image_init -> unit[@mel.this])
  ; onProcessComplete : (Phaser.file -> unit[@mel.this])
  ; destroy : (Phaser.file -> unit[@mel.this]) >
  Js.t

external image_class : image_definition -> (image_init, Phaser.file) Phaser.constructor = "Class"
[@@mel.module "phaser"] [@@mel.new]

external image_extends : image_definition -> image_base -> unit = "Extends" [@@mel.set]

let image_constructor =
  let definition =
    [%mel.obj
      {
        initialize =
          (fun[@mel.this] file (init : image_init) ->
            initialize_image image_base file init.loader { key = init.url; url = init.url };
            set_owner_image file
              {
                deadline = None;
                complete = false;
                abandoned = false;
                failure = None;
                defer = init.defer;
              };
            let native_load = load_function file in
            set_load file (fun[@u] () ->
                let owner = owner_image file in
                owner.deadline <-
                  Some
                    (timeout
                       (fun () ->
                         owner.failure <-
                           Some
                             {
                               message =
                                 "Image download/decode deadline exceeded: " ^ file##key ^ " ("
                                 ^ file##src ^ ")";
                               cause = None;
                             };
                         Option.iter
                           (fun image ->
                             detach_image image;
                             remove_source image "src")
                           (loaded_image file);
                         Option.iter
                           (fun (loader : Phaser.loader) -> loader##nextFile file false)
                           (Js.Nullable.toOption file##loader))
                       30000);
                native_load##call file));
        onProcessComplete =
          (fun[@mel.this] file ->
            let owner = owner_image file in
            Option.iter clear_timeout owner.deadline;
            owner.complete <- true;
            Option.iter detach_image (loaded_image file);
            match owner.defer with Some defer -> defer file | None -> admit file);
        destroy =
          (fun[@mel.this] file ->
            let owner = owner_image file in
            owner.abandoned <- true;
            Option.iter clear_timeout owner.deadline;
            if not owner.complete then
              Option.iter
                (fun image ->
                  detach_image image;
                  remove_source image "src")
                (loaded_image file);
            image_lifecycle##destroy##call file);
      }]
  in
  image_extends definition image_base;
  image_class definition

let image loader url defer = Phaser.construct image_constructor [| { loader; url; defer } |]

type audio_init = {
  loader : Phaser.loader;
  key : string;
  url : string;
  context : Phaser.audio_context;
  report : failure -> unit;
}

type audio_owner = {
  mutable abandoned : bool;
  mutable expired : bool;
  mutable deadline : timer option;
  context : Phaser.audio_context;
  report : failure -> unit;
}

external owner_audio : Phaser.file -> audio_owner = "ocamlPreparedAudio" [@@mel.get]
external set_owner_audio : Phaser.file -> audio_owner -> unit = "ocamlPreparedAudio" [@@mel.set]

type audio_definition =
  < initialize : (Phaser.file -> audio_init -> unit[@mel.this])
  ; onProcess : (Phaser.file -> unit[@mel.this])
  ; destroy : (Phaser.file -> unit[@mel.this]) >
  Js.t

external audio_class : audio_definition -> (audio_init, Phaser.file) Phaser.constructor = "Class"
[@@mel.module "phaser"] [@@mel.new]

external audio_extends : audio_definition -> audio_base -> unit = "Extends" [@@mel.set]

let audio_constructor =
  let definition =
    [%mel.obj
      {
        initialize =
          (fun[@mel.this] file (init : audio_init) ->
            initialize_audio audio_base file init.loader init.key { kind = "wav"; url = init.url }
              Js.Undefined.empty init.context;
            set_owner_audio file
              {
                abandoned = false;
                expired = false;
                deadline = None;
                context = init.context;
                report = init.report;
              });
        onProcess =
          (fun[@mel.this] file ->
            let owner = owner_audio file in
            set_state file processing;
            owner.deadline <-
              Some
                (timeout
                   (fun () ->
                     if not owner.abandoned then (
                       owner.expired <- true;
                       owner.report
                         {
                           message =
                             "Audio decode deadline exceeded: " ^ file##key ^ " (" ^ file##src ^ ")";
                           cause = None;
                         };
                       file##onProcessError ()))
                   30000);
            let xhr =
              match Js.Nullable.toOption file##xhrLoader with
              | Some xhr -> xhr
              | None -> failwith ("Audio response missing: " ^ file##key)
            in
            then_decode
              (decode owner.context (response xhr))
              (fun[@u] buffer ->
                if (not owner.abandoned) && not owner.expired then (
                  Option.iter clear_timeout owner.deadline;
                  set_audio file buffer;
                  file##onProcessComplete ()))
              (fun[@u] cause ->
                if (not owner.abandoned) && not owner.expired then (
                  Option.iter clear_timeout owner.deadline;
                  owner.report
                    {
                      message = "Cannot decode audio " ^ file##key ^ " (" ^ file##src ^ ")";
                      cause = Some (Foreign_error.Rejection cause);
                    };
                  file##onProcessError ())));
        destroy =
          (fun[@mel.this] file ->
            let owner = owner_audio file in
            owner.abandoned <- true;
            Option.iter clear_timeout owner.deadline;
            reset_xhr file;
            audio_lifecycle##destroy##call file);
      }]
  in
  audio_extends definition audio_base;
  audio_class definition

let audio loader key url context report =
  Phaser.construct audio_constructor [| { loader; key; url; context; report } |]

external list : Phaser.loader -> Phaser.file_set Js.Nullable.t = "list" [@@mel.get]
external inflight : Phaser.loader -> Phaser.file_set Js.Nullable.t = "inflight" [@@mel.get]
external queue : Phaser.loader -> Phaser.file_set Js.Nullable.t = "queue" [@@mel.get]
external delete_queue : Phaser.loader -> Phaser.file_set = "_deleteQueue" [@@mel.get]

external set_complete : Phaser.file -> ((unit -> unit)[@u]) -> unit = "onProcessComplete"
[@@mel.set]

external set_error : Phaser.file -> ((unit -> unit)[@u]) -> unit = "onProcessError" [@@mel.set]

let cancel loader file =
  List.iter
    (fun collection ->
      Option.iter
        (fun (collection : Phaser.file_set) ->
          let _ = collection##delete file in
          ())
        (Js.Nullable.toOption collection))
    [ list loader; inflight loader; queue loader ];
  set_complete file (fun[@u] () -> ());
  set_error file (fun[@u] () -> ());
  reset_xhr file;
  file##destroy ()

let settle loader file =
  if
    Option.fold ~none:false
      ~some:(fun current -> current == loader)
      (Js.Nullable.toOption file##loader)
    && file##state = pending_destroy
  then
    let _ = (delete_queue loader)##delete file in
    file##destroy ()

external on_key : Phaser.loader -> string -> ((string -> unit)[@u]) -> unit = "on" [@@mel.send]
external off_key : Phaser.loader -> string -> ((string -> unit)[@u]) -> unit = "off" [@@mel.send]

external on_error : Phaser.loader -> string -> ((Phaser.file -> unit)[@u]) -> unit = "on"
[@@mel.send]

external off_error : Phaser.loader -> string -> ((Phaser.file -> unit)[@u]) -> unit = "off"
[@@mel.send]
