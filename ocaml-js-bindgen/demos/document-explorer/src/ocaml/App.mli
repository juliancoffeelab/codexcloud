val start :
  (Bindings.Blob.t ->
  ((Bindings.Document.t, Bindings.Document_error.t) result -> unit) ->
  (unit -> unit) ->
  unit) ->
  unit
(** Install the application UI. The supplied loader invokes its success or failure
    callback when loading finishes. *)
