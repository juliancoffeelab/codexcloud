val to_js : string -> string
(** Decode a native UTF-8 byte string for DOM/JSON, preserving a leading BOM. Invalid UTF-8 raises;
    no borrowed byte buffer escapes. *)

val of_js : string -> string
(** Encode a host string as native UTF-8 bytes. Lone host surrogates follow the standard TextEncoder
    replacement behavior. *)
