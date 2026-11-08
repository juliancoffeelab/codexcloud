(* Explicit handlers for fixed press-activated buttons. Ordinary and scrolling
   buttons use native onClick instead. No document listeners or synthetic clicks. *)
type button

external pointer_button : React.Event.Pointer.t -> button = "currentTarget" [@@mel.get]
external click_button : React.Event.Mouse.t -> button = "currentTarget" [@@mel.get]
external disabled : button -> bool = "disabled" [@@mel.get]
external attribute : button -> string -> string Js.Nullable.t = "getAttribute" [@@mel.send]
external focus : button -> < preventScroll : bool > Js.t -> unit = "focus" [@@mel.send]
external detail : React.Event.Mouse.t -> int = "detail" [@@mel.get]

let enabled button =
  (not (disabled button)) && Js.Nullable.toOption (attribute button "aria-disabled") <> Some "true"

let pointer_down action event =
  let button = pointer_button event in
  if React.Event.Pointer.isPrimary event && React.Event.Pointer.button event = 0 && enabled button
  then (
    React.Event.Pointer.preventDefault event;
    focus button [%mel.obj { preventScroll = true }];
    action ())

let click action event =
  (* Keyboard, assistive technology and programmatic activation have detail=0.
     Pointer clicks were already handled by this button's onPointerDown. *)
  if detail event = 0 && enabled (click_button event) then action ()
