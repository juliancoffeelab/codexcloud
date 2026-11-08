val start :
  (((Bindings.Paint.t, Bindings.Image_error.t) result -> unit) -> (unit -> unit) -> unit) ->
  unit
(** Install the application UI. The supplied loader invokes its success or failure
    callback when loading finishes. *)
