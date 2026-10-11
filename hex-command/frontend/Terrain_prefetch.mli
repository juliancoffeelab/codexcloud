(** Background warming for one canonical default inspection image per terrain
    type present in the displayed scene. One owner spans map changes and
    retains at most two in-flight requests, including requests from old maps. *)
type t
type cancel = unit -> unit
type defer = (unit -> unit) -> cancel
type load = string -> unit Js.Promise.t
(** Defaults wait for a paint, then idle time (or a zero-delay timer), and decode
    native images with low fetch priority. Dependencies can be injected without
    exposing queues, URL caches or generation bookkeeping. *)
val create : ?defer:defer -> ?load:load -> unit -> t
(** Call only after this physical scene's backdrop has loaded or been mounted.
    Its cancellation drops queued work; stale cleanup cannot stop a newer map.
    Failures remain silent and never mark a URL as successfully warmed. *)
val start : t -> Terrain.scene -> Art.t -> cancel
(** Cancel queued/deferred work. Already-started requests retain their shared
    slots until completion; successful URL reuse and effect replay remain safe. *)
val close : t -> unit
