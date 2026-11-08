type rect = { left : float; right : float; top : float; bottom : float }
(** Inspection visibility policy. The UI supplies committed native elements and tile identifiers;
    this module owns measurement, bounds and callback lifetime. *)

type extent = {
  width : float;
  height : float;
  scroll_width : float;
  scroll_height : float;
  scroll_left : float;
  scroll_top : float;
}

type scroll = { left : float; top : float }

val position : area:rect -> viewport:extent -> rect array -> scroll
(** Keep the whole inspection union visible with 12px margins when it fits. Otherwise prioritize the
    final bound on each axis. Clamp to native scroll limits; hidden viewports and invalid bounds do
    not trigger movement. *)

val reveal_map : scroller:Dom.element -> board:Dom.element -> ids:string array -> unit -> unit
(** Reveal after the committed layout settles and again on viewport resize. Duplicate IDs retain
    their final occurrence so inspected-tile priority wins. Cleanup disconnects observation and
    fences queued or late callbacks. *)

val reveal_attack : panel:Dom.element -> unit -> unit
(** After committed layout, reveal mobile attack choices with nearest scrolling. Missing native
    media/scroll capabilities are harmless; cleanup fences the task. *)
