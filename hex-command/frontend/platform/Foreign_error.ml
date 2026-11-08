type t = Js.Promise.error

(* Arbitrary thrown/rejected values remain opaque. Only guarded diagnostic
    operations inspect them; property getters and string conversion can throw. *)
type cause = Rejection of t | Exception of exn

external stack_value : 'a -> t = "stack" [@@mel.get]
external message_value : 'a -> t = "message" [@@mel.get]
external cause_value : 'a -> t = "cause" [@@mel.get]
external stringify : 'a -> string = "String"
external truthy : t -> bool = "Boolean"
external json_string : 'a -> string Js.Undefined.t = "stringify" [@@mel.scope "JSON"]

let read getter value = try Some (getter value) with _ -> None

let string_field getter value =
  Option.bind (read getter value) (fun value ->
      match Js.Types.classify value with Js.Types.JSString value -> Some value | _ -> None)

let printable value =
  try
    match Js.Types.classify value with
    | Js.Types.JSObject _ -> (
        match Js.Undefined.toOption (json_string value) with
        | Some text -> text
        | None -> stringify value)
    | _ -> stringify value
  with _ -> ( try stringify value with _ -> "Unprintable foreign error")

let message_raw value =
  match string_field message_value value with Some text -> text | None -> printable value

let detail_raw value =
  match string_field stack_value value with Some text -> text | None -> message_raw value

let next_cause value =
  Option.bind (read cause_value value) (fun cause -> if truthy cause then Some cause else None)

let rec cause_details seen value =
  if List.exists (fun previous -> previous == value) seen then "[Circular error cause]"
  else
    detail_raw value
    ^ Option.fold ~none:""
        ~some:(fun cause -> "\nCaused by: " ^ cause_details (value :: seen) cause)
        (next_cause value)

let describe_raw value =
  detail_raw value
  ^ Option.fold ~none:""
      ~some:(fun cause -> "\nCaused by: " ^ cause_details [] cause)
      (next_cause value)

let describe value = Utf8.of_js (describe_raw value)

let exception_ value =
  match Js.Exn.asJsExn value with
  | Some original -> Utf8.of_js (describe_raw original)
  | None -> (
      match string_field stack_value value with
      | Some _ -> Utf8.of_js (describe_raw value)
      | None -> Fault.describe value)

let message value = Utf8.of_js (message_raw value)

let cause_detail = function
  | Rejection value -> describe value
  | Exception value -> exception_ value

let cause = function
  | Rejection value -> message value
  | Exception value -> (
      match Js.Exn.asJsExn value with
      | Some original -> Utf8.of_js (message_raw original)
      | None -> exception_ value)
