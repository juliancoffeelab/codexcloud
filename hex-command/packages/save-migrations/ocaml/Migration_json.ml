(* Persistence JSON stays at this boundary; application state uses native types. *)
let is_undefined value = Js.Types.test value Js.Types.Undefined
let invalid message = invalid_arg message
let require condition message = if not condition then invalid message

let object_ value =
  match Js.Json.decodeObject value with Some v -> v | None -> invalid "Expected object"

let array value =
  match Js.Json.decodeArray value with Some v -> v | None -> invalid "Expected array"

let legacy_string value =
  match Js.Json.decodeString value with
  | Some v -> Migration_utf8.of_js v
  | None -> invalid "Expected string"

let js_string value =
  match Js.Json.decodeString value with Some value -> value | None -> invalid "Expected string"

let string = legacy_string

let number value =
  match Js.Json.decodeNumber value with Some v -> v | None -> invalid "Expected number"

let boolean value =
  match Js.Json.decodeBoolean value with Some v -> v | None -> invalid "Expected boolean"

external has_own : 'a -> string -> bool = "hasOwn" [@@mel.scope "Object"]
external safe_integer : float -> bool = "isSafeInteger" [@@mel.scope "Number"]
external finite : float -> bool = "isFinite" [@@mel.scope "Number"]
external delete : Js.Json.t Js.Dict.t -> string -> unit = "deleteProperty" [@@mel.scope "Reflect"]
external undefined : Js.Json.t = "undefined"
external imul : int -> int -> int = "imul" [@@mel.scope "Math"]

external object_from_entries : (string * Js.Json.t) array -> Js.Json.t = "fromEntries"
[@@mel.scope "Object"]

let obj fields =
  object_from_entries
    (Array.of_list (List.map (fun (key, value) -> (Migration_utf8.to_js key, value)) fields))

let entries v = Array.map (fun (k, v) -> (Migration_utf8.of_js k, v)) (Js.Dict.entries (object_ v))

let get v k =
  let k = Migration_utf8.to_js k in
  let d = object_ v in
  if has_own d k then Js.Dict.get d k else None

let field v k = match get v k with Some v -> v | None -> invalid ("Missing " ^ k)
let set v k x = Js.Dict.set (object_ v) (Migration_utf8.to_js k) x
let remove v k = delete (object_ v) (Migration_utf8.to_js k)
let copy v = obj (Array.to_list (entries v))
let clone v = Js.Json.parseExn (Js.Json.stringify v)
let s v = Js.Json.string (Migration_utf8.to_js v)
let n = Js.Json.number
let b = Js.Json.boolean
let a v = Js.Json.array (Array.of_list v)
let text v k = string (field v k)
let num v k = number (field v k)
let list v k = Array.to_list (array (field v k))

let optional_list v k =
  match get v k with
  | None -> []
  | Some x when x = Js.Json.null || x = undefined -> []
  | Some x -> Array.to_list (array x)

let present v k =
  match get v k with Some x when x <> Js.Json.null && x <> undefined -> Some x | _ -> None

let default v k fallback = match present v k with Some x -> x | None -> fallback
let is_record v = Option.is_some (Js.Json.decodeObject v)

let update v fields =
  let v = copy v in
  List.iter (fun (k, x) -> set v k x) fields;
  v

let map_field v k f = match present v k with None -> () | Some x -> set v k (f x)
let map_list v k f = map_field v k (fun x -> Js.Json.array (Array.map f (array x)))
let unique xs = List.fold_left (fun acc x -> if List.mem x acc then acc else acc @ [ x ]) [] xs
let keys v = Array.to_list (Array.map fst (entries v))

let rename_keys v f =
  let result = obj [] in
  Array.iter
    (fun (k, x) ->
      let k = f k in
      require (get result k = None) "Ambiguous historical reference rename";
      set result k x)
    (entries v);
  result

let starts prefix text = String.starts_with ~prefix text

let suffix prefix text =
  String.sub text (String.length prefix) (String.length text - String.length prefix)

(* Current-format consumers receive host text, including dictionary keys. *)
module Host = struct
  let string = js_string
  let s = Js.Json.string
  let obj fields = object_from_entries (Array.of_list fields)
  let entries value = Js.Dict.entries (object_ value)

  let get value key =
    let fields = object_ value in
    if has_own fields key then Js.Dict.get fields key else None

  let field value key =
    match get value key with Some value -> value | None -> invalid_arg {j|Missing $(key)|j}

  let set value key item = Js.Dict.set (object_ value) key item
  let remove value key = delete (object_ value) key
  let text value key = string (field value key)
  let list value key = Array.to_list (array (field value key))
  let copy value = obj (Array.to_list (entries value))

  let present value key =
    match get value key with
    | Some value when value <> Js.Json.null && value <> undefined -> Some value
    | _ -> None

  let default value key fallback = Option.value ~default:fallback (present value key)

  let update value fields =
    let value = copy value in
    List.iter (fun (key, item) -> set value key item) fields;
    value
end

(* Historical conversion failures also cross the public text boundary. *)
let host_errors work input =
  try work input with Invalid_argument message -> invalid_arg (Migration_utf8.to_js message)
