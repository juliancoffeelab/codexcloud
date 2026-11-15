let missing value = Js.Types.test value Js.Types.Undefined

(** JSON at the versioned browser-storage boundary. Game state remains typed in Rust/Domain. *)
let obj values = Js.Json.object_ (Js.Dict.fromArray (Array.of_list values))

let arr values = Js.Json.array (Array.of_list values)
let str value = Js.Json.string value
let num = Js.Json.number
let bool = Js.Json.boolean

let dict value =
  match Js.Json.decodeObject value with
  | Some value -> value
  | None -> invalid_arg {js|Expected save object|js}

let get value key =
  match Js.Json.decodeObject value with
  | None -> None
  | Some d ->
      Option.bind (Js.Dict.get d key) (fun value -> if missing value then None else Some value)

let field value key = Option.value ~default:Js.Json.null (get value key)
let host_string value = Option.value ~default:{js||js} (Js.Json.decodeString value)
let string value = Option.value ~default:{js||js} (Js.Json.decodeString value)
let text value key = string (field value key)
let number value = Option.value ~default:0. (Js.Json.decodeNumber value)
let n value key = number (field value key)
let boolean value = Js.Json.decodeBoolean value = Some true
let b value key = boolean (field value key)
let array value = Option.value ~default:[||] (Js.Json.decodeArray value)
let entries value = Js.Dict.entries (dict value)
let copy value = obj (Array.to_list (entries value))
let set value key entry = Js.Dict.set (dict value) key entry

external remove : Js.Json.t -> string -> bool = "deleteProperty" [@@mel.scope "Reflect"]

let without value keys =
  obj (Array.to_list (entries value) |> List.filter (fun (key, _) -> not (List.mem key keys)))

let detail = Save_error.detail
let reject = Save_error.reject_error
let attempt = Save_error.attempt
let ( let* ) promise f = Js.Promise.then_ f promise
let return = Js.Promise.resolve
let all values = Js.Promise.all (Array.of_list values)
