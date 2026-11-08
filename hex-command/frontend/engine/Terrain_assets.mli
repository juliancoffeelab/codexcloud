type asset = { map_id : string; plan : Terrain_plan.t; key : string; asset_key : string }
(** Offline-only native preparation. No part of this module belongs in a worker. *)

val plans : unit -> asset array
(** Prepare one unlit image per terrain layout. The factory is released before the caller receives
    its owned asset array. *)
