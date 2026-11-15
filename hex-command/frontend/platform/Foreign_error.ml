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
  with _ -> ( try stringify value with _ -> {js|Unprintable foreign error|js})

let message_raw value =
  match string_field message_value value with Some text -> text | None -> printable value

let detail_raw value =
  let message = message_raw value in
  match string_field stack_value value with
  | Some stack when stack <> {js||js} ->
      if message = {js||js} || Js.String.includes ~search:message stack then stack
      else {j|$(message)\n$(stack)|j}
  | _ -> message

let next_cause value =
  Option.bind (read cause_value value) (fun cause -> if truthy cause then Some cause else None)

let rec cause_details seen value =
  if List.exists (fun previous -> previous == value) seen then {js|[Circular error cause]|js}
  else
    let js_text_part_0 = detail_raw value in
    let js_text_part_1 =
      Option.fold ~none:{js||js}
        ~some:(fun cause ->
          let js_text_part_1 = cause_details (value :: seen) cause in
          {j|\nCaused by: $(js_text_part_1)|j})
        (next_cause value)
    in
    {j|$(js_text_part_0)$(js_text_part_1)|j}

let describe_raw value =
  let js_text_part_0 = detail_raw value in
  let js_text_part_1 =
    Option.fold ~none:{js||js}
      ~some:(fun cause ->
        let js_text_part_1 = cause_details [] cause in
        {j|\nCaused by: $(js_text_part_1)|j})
      (next_cause value)
  in
  {j|$(js_text_part_0)$(js_text_part_1)|j}

let describe value = describe_raw value

let exception_ value =
  match Js.Exn.asJsExn value with
  | Some original -> describe_raw original
  | None -> (
      match string_field stack_value value with
      | Some _ -> describe_raw value
      | None -> Fault.describe value)

let message value = message_raw value

let cause_detail = function
  | Rejection value -> describe value
  | Exception value -> exception_ value

let cause = function
  | Rejection value -> message value
  | Exception value -> (
      match Js.Exn.asJsExn value with
      | Some original -> message_raw original
      | None -> exception_ value)
