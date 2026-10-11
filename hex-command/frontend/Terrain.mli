(** One battlefield's static scene, prebuilt preview and latest fallback bake.
    Changed sessions cancel decoder handlers and queued drawing before publishing. *)
type sprite = { url : string; width : float; height : float;
                display_width : float [@mel.as "displayWidth"];
                display_height : float [@mel.as "displayHeight"] }
type t
type scene
type preview
type bitmap
(** Invalid asset keys or dimensions are ignored, leaving the tile fallback usable. *)
val create : sprite Js.Dict.t -> t
(** Derive and cache a scene without starting or cancelling resources. Reuses
    physical identity through unit, ownership and unused-art changes; an abandoned
    render cannot replace the committed scene or cancel its bake. *)
val prepare : t -> Domain.state -> Art.t -> scene
(** Commit a prepared scene. Cancels work for the previous scene even when the
    new scene uses an exact authored PNG. Call from the UI layout effect. *)
val activate : t -> scene -> unit
val key : scene -> string
val tiles : scene -> Terrain_plan.tile array
(** One default-floor canonical image per terrain type, using the current full
    artwork snapshot even when the battlefield geometry has been reused. *)
val inspection_urls : scene -> Art.t -> string array
val get_tile : scene -> string -> Terrain_plan.tile option
val size : scene -> Geometry.dimensions
(** Authored PNGs are valid only for the compiled artwork used to build them.
    Changing a used terrain image bypasses both exact and overlay previews. *)
val preview : t -> scene -> preview option
val sprite : preview -> sprite
val changed : preview -> string array
val changed_contains : preview -> string -> bool
(** Only the activated scene may bake; stale effects return no bitmap.
    Exact authored sprites need no runtime bake. Fallbacks decode at most four
    images together and yield between 64-cell drawing batches. Failures are retryable. *)
val bake : t -> scene -> bitmap option Js.Promise.t
val valid_bitmap : scene -> bitmap -> bool
val bitmap_size : bitmap -> Terrain_plan.dimensions
(** Cancel all pending work and release decoded images/bitmaps. Idempotent and
    reusable so React effect cleanup/setup replay can retry the same scene. *)
val close : t -> unit
(** Attach the completed native canvas directly, without encoding or decoding it. *)
val mount : Dom.element -> bitmap -> unit
val clear : Dom.element -> unit
(** Warm only the latest authored PNG before a battle transition. A completed
    warm retains its URL, releases the native image/handlers, and can be retried
    after failure. Late callbacks cannot replace a newer selected map. *)
val preload : t -> string -> unit

(** Shared bounded warm owner for map selection and pre-transition callers. *)
val preload_manifest : sprite Js.Dict.t -> string -> unit
