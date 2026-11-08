(* Migration strings retain JavaScript UTF-16 code units. Ordinary text uses UTF-8
   byte strings; unpaired surrogates use WTF-8 so accepted historical IDs and keys
   cannot collapse through TextEncoder replacement. This boundary is independent
   of the live game's Unicode/Rust string conversion. *)
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
external from_code : int -> string = "fromCharCode" [@@mel.scope "String"]
external join_host : string array -> string -> string = "join" [@@mel.send]

let decoder = create_decoder "utf-8" [%mel.obj { fatal = true; ignoreBOM = true }]
let encoder = create_encoder ()

let ascii value =
  let rec loop index =
    index = String.length value || (code_at value index < 128 && loop (index + 1))
  in
  loop 0

let decode_utf8 value =
  let size = String.length value in
  let bytes = create_buffer size in
  for index = 0 to size - 1 do
    set bytes index (code_at value index)
  done;
  decode decoder bytes

let surrogate_at value index =
  index + 2 < String.length value
  && code_at value index = 0xed
  && code_at value (index + 1) >= 0xa0
  && code_at value (index + 1) <= 0xbf
  && code_at value (index + 2) >= 0x80
  && code_at value (index + 2) <= 0xbf

let to_js value =
  if ascii value then value
  else
    let size = String.length value in
    let rec first index =
      if index >= size then None
      else if surrogate_at value index then Some index
      else first (index + 1)
    in
    match first 0 with
    | None -> decode_utf8 value
    | Some _ ->
        let chunks = ref [] in
        let push first last =
          if first < last then
            chunks := decode_utf8 (String.sub value first (last - first)) :: !chunks
        in
        let rec loop first index =
          if index >= size then push first size
          else if surrogate_at value index then (
            push first index;
            let code =
              ((code_at value index land 0x0f) lsl 12)
              lor ((code_at value (index + 1) land 0x3f) lsl 6)
              lor (code_at value (index + 2) land 0x3f)
            in
            chunks := from_code code :: !chunks;
            loop (index + 3) (index + 3))
          else loop first (index + 1)
        in
        loop 0 0;
        join_host (Array.of_list (List.rev !chunks)) ""

let high code = code >= 0xd800 && code <= 0xdbff
let low code = code >= 0xdc00 && code <= 0xdfff

let unpaired value =
  let size = String.length value in
  let rec loop index =
    if index = size then false
    else
      let code = code_at value index in
      if high code then
        if index + 1 < size && low (code_at value (index + 1)) then loop (index + 2) else true
      else low code || loop (index + 1)
  in
  loop 0

let encode_lossless value =
  let result = Buffer.create (String.length value * 3) in
  let byte code = Buffer.add_char result (Char.chr code) in
  let append code =
    if code < 0x80 then byte code
    else if code < 0x800 then (
      byte (0xc0 lor (code lsr 6));
      byte (0x80 lor (code land 0x3f)))
    else if code < 0x10000 then (
      byte (0xe0 lor (code lsr 12));
      byte (0x80 lor ((code lsr 6) land 0x3f));
      byte (0x80 lor (code land 0x3f)))
    else (
      byte (0xf0 lor (code lsr 18));
      byte (0x80 lor ((code lsr 12) land 0x3f));
      byte (0x80 lor ((code lsr 6) land 0x3f));
      byte (0x80 lor (code land 0x3f)))
  in
  let rec loop index =
    if index < String.length value then
      let code = code_at value index in
      if high code && index + 1 < String.length value && low (code_at value (index + 1)) then (
        append (0x10000 + ((code - 0xd800) lsl 10) + code_at value (index + 1) - 0xdc00);
        loop (index + 2))
      else (
        append code;
        loop (index + 1))
  in
  loop 0;
  Buffer.contents result

let of_js value =
  if ascii value then value
  else if unpaired value then encode_lossless value
  else
    let bytes = encode encoder value in
    String.init (length bytes) (fun index -> Char.chr (get bytes index))
