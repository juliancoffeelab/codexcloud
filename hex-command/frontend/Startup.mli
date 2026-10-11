(** Catalog-free startup ownership. The shell is published synchronously before
    loading is scheduled after a paint. Closing cancels queued work, invalidates
    pending completions, and releases even a late successful application. *)
type phase = Loading | Failed of string | Ready
type t
val create : show:(phase -> unit) ->
  schedule:((unit -> unit) -> (unit -> unit)) ->
  load:((unit -> bool) -> (unit -> unit) Js.Promise.t) -> t
(** Idempotent, including reentrant cleanup. *)
val close : t -> unit

(** Report a late rendering or initialization failure. Cancels acquisition,
    releases resources, and shows reload UI once. Late completions are released;
    failures after close are ignored. *)
val fail : t -> string -> unit
