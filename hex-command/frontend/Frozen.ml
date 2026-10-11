external values : 'a -> Js.Json.t array = "values" [@@mel.scope "Object"]
external freeze : 'a -> 'a = "freeze" [@@mel.scope "Object"]
let object_ : 'a -> bool = [%mel.raw {|value => value !== null && typeof value === "object"|}]
let rec own : 'a. 'a -> 'a = fun value ->
  if object_ value then (
    Array.iter (fun child -> ignore (own child)) (values value);
    ignore (freeze value));
  value
