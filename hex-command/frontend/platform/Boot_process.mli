val progress : step:int -> label:string -> value:float -> detail:string -> unit
(** Page-local startup diagnostics exposed as [window.BOOT_PROCESS]. The first completed or failed
    startup is retained across later battlefield changes. Resource events are bounded; stage timings
    and the initial event are preserved. All timestamps are navigation-relative milliseconds. *)

val fail : string -> unit

val resource : key:string -> url:string -> phase:string -> unit
(** Cheap lifecycle timestamp during the first startup; no-op once startup finishes. *)

val response :
  url:string -> status:int -> source:string -> get_header:(string -> string Js.Nullable.t) -> unit
(** Select cache/CDN response headers from an existing fetch/XHR response. Bounded to the first 64
    responses; calls no getter after startup completes. Header-read failures never fail loading. *)

val snapshot : unit -> Js.Json.t
(** Detached JSON for the debug report, including elapsed time for a stalled startup and native
    browser navigation/resource timings collected only at export. *)
