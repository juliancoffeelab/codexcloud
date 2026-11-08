open Migration_json

(* This frozen schema is the complete historical wire contract, including retired fields. *)
let document = Lazy.force Frozen_data.formats_readers
let shapes = array (field document "shapes")

let rec matches value index =
  let shape = shapes.(index) in
  let at k = int_of_float (num shape k) in
  match text shape "kind" with
  | "unknown" -> true
  | "never" -> false
  | "undefined" -> value = undefined
  | "null" -> value = Js.Json.null
  | "literal" -> value = field shape "value"
  | "string" -> Option.is_some (Js.Json.decodeString value)
  | "number" -> Option.fold ~none:false ~some:finite (Js.Json.decodeNumber value)
  | "boolean" -> Option.is_some (Js.Json.decodeBoolean value)
  | "union" -> List.exists (fun i -> matches value (int_of_float (number i))) (list shape "members")
  | "tuple" ->
      Option.fold ~none:false
        ~some:(fun xs ->
          let types = array (field shape "items") in
          Array.length xs = Array.length types
          && Array.for_all
               (fun (i, t) -> matches xs.(i) (int_of_float (number t)))
               (Array.mapi (fun i t -> (i, t)) types))
        (Js.Json.decodeArray value)
  | "array" ->
      Option.fold ~none:false
        ~some:(Array.for_all (fun v -> matches v (at "element")))
        (Js.Json.decodeArray value)
  | "object" -> (
      is_record value
      && Array.for_all
           (fun (k, p) ->
             match get value k with
             | Some v -> matches v (int_of_float (num p "type"))
             | None -> boolean (field p "optional"))
           (entries (field shape "properties"))
      &&
      match get shape "index" with
      | None -> true
      | Some i -> Array.for_all (fun (_, v) -> matches v (int_of_float (number i))) (entries value))
  | _ -> invalid "Unknown frozen reader kind"

let read value name =
  require
    (matches value (int_of_float (num (field document "entries") name)))
    ("Invalid historical " ^ name);
  value

let save value version = read value ("Save" ^ string_of_int version)
let history value version = read value ("History" ^ string_of_int version)

let fragment value name =
  let fragments = shapes.(int_of_float (num (field document "entries") "Fragments")) in
  require
    (matches value (int_of_float (num (field (field fragments "properties") name) "type")))
    ("Invalid frozen fragment " ^ name);
  value
