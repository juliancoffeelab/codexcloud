(* Presentation only: measurements remain in Loading_progress / BOOT_PROCESS.
   The browser animates the fill; no animation-frame loop or React ticking. *)
type plan = {
  from : float;
  anchor : float;
  cap : float;
  start : float;
  settle : float;
  duration : float;
}

let plan : plan option ref = ref None
let bounds = [| 0.; 0.25; 0.30; 0.55; 0.95; 1. |]
let clamp value = max 0. (min 1. value)
let interpolate first last fraction = first +. ((last -. first) *. clamp fraction)

let sample plan now =
  let elapsed = max 0. (now -. plan.start) in
  if plan.settle > 0. && elapsed < plan.settle then
    interpolate plan.from plan.anchor (elapsed /. plan.settle)
  else
    interpolate plan.anchor plan.cap
      ((elapsed -. plan.settle) /. max 1. (plan.duration -. plan.settle))

let bootstrap_elapsed : unit -> float =
  [%mel.raw
    {|() => {
    if (typeof document === "undefined") return 0;
    const animation = document.getElementById("startup-fill")?.getAnimations?.()[0];
    return typeof animation?.currentTime === "number" ? Math.min(8000, Math.max(0, animation.currentTime)) : 0;
  }|}]

let current_plan () =
  match !plan with
  | Some value -> value
  | None ->
      let elapsed = bootstrap_elapsed () in
      let from = 0.23 *. elapsed /. 8000. in
      let value =
        {
          from;
          anchor = from;
          cap = 0.23;
          start = Browser_time.now ();
          settle = 0.;
          duration = max 1. (8000. -. elapsed);
        }
      in
      plan := Some value;
      value

let value () = sample (current_plan ()) (Browser_time.now ())

let target ~step ~value =
  if value >= 5. then (1., 1.)
  else
    let index = max 0 (min 4 (step - 1)) in
    let first = bounds.(index) and last = bounds.(index + 1) in
    let measured = interpolate first last (value -. float_of_int index) in
    (measured, max measured (last -. 0.02))

let advance ~step ~value:actual =
  let now = Browser_time.now () in
  let from = sample (current_plan ()) now in
  let measured, cap = target ~step ~value:actual in
  let anchor = max from measured in
  let settle = if anchor > from then if actual >= 5. then 140. else 400. else 0. in
  plan :=
    Some
      {
        from;
        anchor;
        cap = max anchor cap;
        start = now;
        settle;
        duration = (if actual >= 5. then 140. else 8000.);
      }

let restart ~step ~value:actual =
  let anchor, cap = target ~step ~value:actual in
  plan :=
    Some { from = anchor; anchor; cap; start = Browser_time.now (); settle = 0.; duration = 8000. }

let reduced_motion : unit -> bool =
  [%mel.raw
    {|() => typeof window !== "undefined" && !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches|}]

type frame = { transform : string; offset : float }
type animation
type options = { duration : float; fill : string; easing : string }

external start_animation : Dom.element -> frame array -> options -> animation = "animate"
[@@mel.send]

external cancel_animation : animation -> unit = "cancel" [@@mel.send]

let supports_animation : Dom.element -> bool =
  [%mel.raw {|node => typeof node.animate === "function"|}]

let static_fill : Dom.element -> string -> unit =
  [%mel.raw
    {|(node, transform) => { node.style.animation = "none"; node.style.transform = transform; }|}]

let transform value =
  let js_text_part_1 = Js.Float.toString value in
  {j|scaleX($(js_text_part_1))|j}

let animate node =
  let current = current_plan () in
  let now = Browser_time.now () in
  let elapsed = max 0. (now -. current.start) in
  let first = sample current now in
  let reduced = reduced_motion () in
  if (not (supports_animation node)) || reduced then (
    static_fill node (transform current.anchor);
    fun () -> ())
  else
    let duration = max 1. (current.duration -. elapsed) in
    let frames =
      Array.of_list
        ({ transform = transform first; offset = 0. }
         ::
         (if elapsed < current.settle then
            [
              {
                transform = transform current.anchor;
                offset = (current.settle -. elapsed) /. duration;
              };
            ]
          else [])
        @ [ { transform = transform current.cap; offset = 1. } ])
    in
    let animation =
      start_animation node frames { duration; fill = {js|forwards|js}; easing = {js|linear|js} }
    in
    fun () -> cancel_animation animation
