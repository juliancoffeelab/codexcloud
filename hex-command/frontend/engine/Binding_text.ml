(* The SDK loader and its diagnostics still use native OCaml byte strings.
   Game data uses Text/js_value; keep these conversions at this SDK boundary. *)
(* Native OCaml strings hold UTF-8 bytes, while DOM/JSON hold UTF-16. Keep this
   small adapter independent of the lazily loaded WASM runtime: the loading shell
   and the build-time terrain renderer both use it before Rust/WASM is initialized. *)
type buffer
type decoder
type encoder

external create_buffer : int -> buffer = "Uint8Array" [@@mel.new]
external set : buffer -> int -> int -> unit = "" [@@mel.set_index]
external get : buffer -> int -> int = "" [@@mel.get_index]
external length : buffer -> int = "length" [@@mel.get]

external create_decoder : string -> < fatal : bool ; ignoreBOM : bool > Js.t -> decoder
  = "TextDecoder"
[@@mel.new]

external create_encoder : unit -> encoder = "TextEncoder" [@@mel.new]
external decode : decoder -> buffer -> string = "decode" [@@mel.send]
external encode : encoder -> string -> buffer = "encode" [@@mel.send]
external code_at : string -> int -> int = "charCodeAt" [@@mel.send]

let decoder = create_decoder {js|utf-8|js} [%mel.obj { fatal = true; ignoreBOM = true }]
let encoder = create_encoder ()

let ascii value =
  let rec loop index =
    index = String.length value || (code_at value index < 128 && loop (index + 1))
  in
  loop 0

let to_js value =
  if ascii value then value
  else
    let size = String.length value in
    let bytes = create_buffer size in
    for index = 0 to size - 1 do
      set bytes index (code_at value index)
    done;
    decode decoder bytes

let of_js value =
  if ascii value then value
  else
    let bytes = encode encoder value in
    String.init (length bytes) (fun index -> Char.chr (get bytes index))
