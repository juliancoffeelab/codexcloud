include Phaser_dom

let modal () = not (Js.Nullable.isNullable (query "dialog[open]"))

let position (pointer : Phaser.pointer) : Geometry.point =
  let found = ref None in
  if pointer##wasTouch then
    Option.iter
      (fun touches ->
        for i = 0 to touch_count touches - 1 do
          let touch = touch_at touches i in
          if touch.identifier = pointer##identifier && Option.is_none !found then
            found := Some { Geometry.x = touch.client_x; y = touch.client_y }
        done)
      (Js.Undefined.toOption (changed_touches pointer##event));
  match !found with
  | Some point -> point
  | None -> { x = client_x pointer##event; y = client_y pointer##event }
