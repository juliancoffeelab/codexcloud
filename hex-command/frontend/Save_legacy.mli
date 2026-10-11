(** Private compatibility boundary for the historical browser snapshot. It reads
    authored JSON history and the already validated native catalog, and returns
    an owned checkpoint state. The original 6x20 developer browser layout stays
    revision one until the outer sequential migration expands it. Unknown
    historical ownership IDs are rejected instead of guessing a layout.
    This module never executes commands or touches storage. *)
val import : Domain.catalog_view -> Js.Json.t -> Js.Json.t

(** Convert the old positional deployment marker using the wave's historical ID. *)
val wave_marker : string -> Js.Json.t array -> string
