type command = Build of string | Cancel | Query of string | Validate | Shutdown

type event =
  | Ready
  | Progress of int
  | Indexed
  | Cancelled
  | Search_error of string
  | Query_error of string
  | Worker_failed of string
  | Clear
  | Match of string
  | Queried
  | Validation of bool
  | Closed

val start : (event -> unit) -> unit
(** Start the worker and receive typed events. Raises Failure if already started. *)

val send : command -> unit
(** Send a command. Raises Failure if the worker is closed or its payload is too large.
*)

val listen : (command -> unit) -> unit
(** Inside the worker, receive validated commands. Unknown or malformed messages are
    ignored. *)

val emit : event -> unit
(** Inside the worker, send an event to its parent. *)

val close : unit -> unit
(** Inside the worker, close the worker global scope. *)
