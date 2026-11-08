val to_js : string -> string
(** Decode migration UTF-8/WTF-8 bytes to the exact original JavaScript UTF-16 code units,
    preserving BOM and unpaired surrogates. Other malformed byte sequences raise. *)

val of_js : string -> string
(** Losslessly encode historical JavaScript text and dictionary keys. Ordinary Unicode uses UTF-8;
    unpaired UTF-16 surrogates use WTF-8 instead of being replaced by TextEncoder. *)
