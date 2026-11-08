type command = Build(string) | Cancel | Query(string) | Validate | Shutdown

type event =
  | Ready
  | Progress(int)
  | Indexed
  | Cancelled
  | Search_error(string)
  | Query_error(string)
  | Worker_failed(string)
  | Clear
  | Match(string)
  | Queried
  | Validation(bool)
  | Closed

/** Start the worker and receive typed events. Raises Failure if already started. */
let start: (event => unit) => unit

/** Send a command. Raises Failure if the worker is closed or its payload is too large. */
let send: command => unit

/** Inside the worker, receive validated commands. Unknown or malformed messages are ignored. */
let listen: (command => unit) => unit

/** Inside the worker, send an event to its parent. */
let emit: event => unit

/** Inside the worker, close the worker global scope. */
let close: unit => unit
