open Phaser_dom
open Browser_time
module Map = Host_collections.Map
module Set = Host_collections.Set
module Frame = Battlefield_frame

external backgrounded : Dom.document -> bool = "hidden" [@@mel.get]

type transition_kind = Move | Hit | Spawn

type transition = {
  kind : transition_kind;
  owner : Phaser_sound.owner option;
  mutable t : float;
  from_x : float;
  from_y : float;
  start : float;
  end_ : float;
}

type actor = {
  mutable data : Frame.unit_actor;
  root : Phaser.container;
  image : Phaser.image;
  mutable concealment_outline : Phaser.image option;
  base : Phaser.ellipse;
  indicators : Phaser.graphics;
  mutable transition : transition option;
}

type death = {
  tile : string;
  key : string;
  actor : actor;
  mutable start : float;
  y : float;
  owner : Phaser_sound.owner option;
}

type ground_item = { root : Phaser.container; image : Phaser.image; anchor : Phaser.vector }

type outline = {
  root : Phaser.container;
  ordinal : int;
  outline : Phaser_hex_outline.outline;
  mutable tile : Frame.tile option;
}

type pulse = {
  kind : string;
  tile : string;
  mutable t : float;
  start : float;
  duration : float;
  owner : Phaser_sound.owner option;
}

type visual = {
  game : Native.view;
  tiles : Frame.tile array;
  motion : bool;
  sound : bool;
  epoch : int;
}

module Transition_tween = Phaser_dom.Tween_target (struct
  type t = transition
end)

module Pulse_tween = Phaser_dom.Tween_target (struct
  type t = pulse
end)

let color value = (Phaser.SceneMethods.color value)##color
let same_option a b = match (a, b) with None, None -> true | Some a, Some b -> a == b | _ -> false
let same_owner = same_option
let bounds left top right bottom = { Phaser_spatial.left; top; right; bottom }
let clamp value low high = max low (min high value)

let frame_url (unit : Frame.unit_actor) =
  match unit.url with Some url -> url | None -> failwith ("Missing unit artwork: " ^ unit.id)

class battlefield ~(audio : Phaser_sound.t) ~(actions : unit -> Battlefield_view.actions)
  ~(describe : string -> unit) ~(previews : Terrain_preview.t)
  ~(progress : Battlefield_view.progress -> unit) =
  let native = Phaser.Scene.create "battlefield" in
  object (self)
    val frame_cache = new Frame.cache
    val mutable runtime : Phaser_runtime.t option = None
    val mutable frame : Frame.t option = None
    val mutable resource_epoch = 0
    val mutable last_visual : visual option = None
    val mutable scene_game : Native.view option = None
    val mutable scene_artwork : Art.t option = None
    val mutable scene_data : Frame.scene_data option = None
    val mutable tiles_by_id : (string, Frame.tile) Map.t = Map.create ()
    val mutable artwork_metadata : Art.t option = None
    val mutable image_metadata : (string, Art.frame) Map.t = Map.create ()
    val mutable ready = false
    val mutable identity = ""
    val mutable loading = false
    val mutable assets : Phaser_assets.assets option = None
    val mutable resources : Phaser_resources.resources option = None
    val mutable required_images : string Set.t = Set.create ()
    val mutable required_audio : string Set.t = Set.create ()
    val mutable plan_key : Phaser_resource_plan.plan_key option = None
    val mutable planned_game : Native.view option = None
    val mutable presented = false
    val mutable previous_progress : Battlefield_view.progress option = None
    val mutable asset_error : string option = None
    val mutable refresh_queued = false
    val mutable failed = false
    val mutable lighting : Phaser_lighting.lighting option = None
    val mutable actor_lighting : Phaser_actor_lighting.actor_lighting option = None
    val mutable controls : Phaser_camera.input option = None
    val mutable terrain : Phaser_terrain.terrain option = None
    val mutable terrain_urls : string Set.t = Set.create ()
    val mutable terrain_game : Native.view option = None
    val actors : (string, actor) Map.t = Map.create ()
    val active_actors : actor Set.t = Set.create ()
    val visibility = new Phaser_visibility.visibility
    val mutable actor_roots : Phaser.container option = None
    val mutable upper_indicators : Phaser.container option = None
    val mutable action : Action_presentation.token option = None
    val mutable action_lease : Action_presentation.lease option = None
    val mutable action_finished = false
    val mutable unwatch_action : (unit -> unit) option = None
    val mutable playback : Phaser_playback.playback option = None
    val dying : (Phaser.container, death) Map.t = Map.create ()
    val mutable items : Phaser.container option = None
    val mutable atmosphere : Phaser_fog.fog option = None
    val atmosphere_pipelines : Phaser.pipeline array = [||]
    val mutable outlines : Phaser.container option = None
    val mutable owners : Phaser.container option = None
    val owner_markers : (string, Phaser.ellipse) Map.t = Map.create ()
    val ground_items : (string, ground_item) Map.t = Map.create ()
    val hexes : (string, outline) Map.t = Map.create ()
    val pulses : (string, pulse) Map.t = Map.create ()
    val mutable has_frame = false
    val mutable playback_frame : Combat_playback.frame option = None
    val world_origin = Phaser.Geom.Vector2.create ()
    val world_edge = Phaser.Geom.Vector2.create ()
    initializer Phaser.Scene.set_create native (fun[@mel.this] _ -> self#create)
    method native = native
    method attach owner = runtime <- Some owner

    method private runtime =
      match runtime with Some owner -> owner | None -> failwith "Battlefield has no boot owner"

    method private invalidate = Option.iter Phaser_runtime.invalidate runtime

    method private fail error =
      failed <- true;
      Phaser_runtime.fail self#runtime error

    method private frame = Option.get frame
    method private assets = Option.get assets
    method private resources = Option.get resources
    method private lighting = Option.get lighting
    method private actor_lighting = Option.get actor_lighting
    method private playback = Option.get playback
    method private actor_roots = Option.get actor_roots
    method private upper_indicators = Option.get upper_indicators
    method private items = Option.get items
    method private outlines = Option.get outlines
    method private owners = Option.get owners

    method private action_owner =
      match action with
      | Some action -> Action_presentation.owner action
      | None -> Phaser_sound.game_owner self#frame.view.game

    method private action_owner_option = Option.map Action_presentation.owner action

    method private acknowledge_action =
      match action with
      | Some action
        when (not action_finished) && presented && (not loading) && (not failed)
             && Option.fold ~none:false
                  ~some:(fun previous -> previous.game == self#frame.view.game)
                  last_visual ->
          let combat_complete =
            match (action.combat, self#frame.view.combat_playback) with
            | None, _ -> true
            | Some combat, Some playback -> combat == playback.combat && playback.complete
            | _ -> false
          in
          let owner = Action_presentation.owner action in
          if
            combat_complete
            && (not (Phaser_sound.pending audio owner))
            && (not
                  (Array.exists
                     (fun actor ->
                       Option.fold ~none:false
                         ~some:(fun (transition : transition) ->
                           same_owner transition.owner (Some owner))
                         actor.transition)
                     (Set.values_array active_actors)))
            && (not
                  (Array.exists
                     (fun (death : death) -> same_owner death.owner (Some owner))
                     (Map.values_array dying)))
            && not
                 (Array.exists
                    (fun (pulse : pulse) -> same_owner pulse.owner (Some owner))
                    (Map.values_array pulses))
          then (
            action_finished <- true;
            Option.iter Action_presentation.complete action_lease)
      | _ -> ()

    method private cancel_presentation =
      if Option.is_none action || action_finished then (
        self#playback#skip;
        Map.each actors (fun[@u] actor _ -> self#draw_indicators actor);
        Phaser_sound.cancel audio;
        self#invalidate)

    method private find_actor id =
      match Map.get actors id with
      | Some actor -> Some actor
      | None ->
          Option.map
            (fun (death : death) -> death.actor)
            (Array.find_opt
               (fun (death : death) -> death.actor.data.id = id)
               (Map.values_array dying))

    method private create =
      let renderer =
        match Js.Nullable.toOption native##game##renderer with
        | Some renderer -> renderer
        | None -> failwith "Battlefield WebGL renderer unavailable"
      in
      let cleaned = ref false in
      let blur = fun[@u] () -> self#cancel_presentation
      and animate = fun[@u] () -> self#animate_actors in
      let cleanup () =
        if not !cleaned then (
          cleaned := true;
          ready <- false;
          let release work =
            try work () with error -> Phaser_runtime.record_cleanup_error self#runtime error
          in
          release (fun () -> Option.iter (fun unwatch -> unwatch ()) unwatch_action);
          unwatch_action <- None;
          release (fun () -> Option.iter (fun playback -> playback#stop) playback);
          playback_frame <- None;
          release (fun () ->
              let _ = native##game##events##off "blur" blur in
              ());
          release (fun () ->
              let _ = native##game##events##off "poststep" animate in
              ());
          release (fun () -> Option.iter (fun controls -> controls#destroy) controls);
          Map.each actors (fun[@u] actor _ ->
              release (fun () -> actor.root##destroy ());
              release (fun () -> actor.indicators##destroy ()));
          Map.each dying (fun[@u] _ root -> release (fun () -> root##destroy ()));
          Map.each hexes (fun[@u] hex _ -> release (fun () -> hex.root##destroy ()));
          Map.each ground_items (fun[@u] item _ -> release (fun () -> item.root##destroy ()));
          Map.each owner_markers (fun[@u] marker _ -> release (fun () -> marker##destroy ()));
          release (fun () -> visibility#clear);
          List.iter release
            [
              (fun () -> Option.iter (fun assets -> assets#destroy) assets);
              (fun () -> native##tweens##killAll ());
              (fun () -> renderer##pipelines##flush ());
              (fun () -> Option.iter (fun lighting -> lighting#release) lighting);
              (fun () -> renderer##pipelines##remove "BattlefieldActorLighting" false);
              (fun () ->
                Option.iter (fun actor_lighting -> actor_lighting#native##destroy ()) actor_lighting);
              (fun () -> renderer##pipelines##remove "BattlefieldLighting" false);
              (fun () -> Option.iter (fun lighting -> lighting#native##destroy ()) lighting);
              (fun () -> Option.iter (fun atmosphere -> atmosphere#destroy) atmosphere);
              (fun () -> Option.iter (fun terrain -> terrain#destroy) terrain);
            ];
          Array.iter
            (fun (pipeline : Phaser.pipeline) ->
              release (fun () -> renderer##pipelines##remove pipeline##name false);
              release (fun () -> pipeline##destroy ()))
            atmosphere_pipelines)
      in
      let _ = native##events##once "shutdown" (fun[@u] () -> cleanup ()) in
      let _ = native##events##once "destroy" (fun[@u] () -> cleanup ()) in
      Phaser_runtime.on_dispose self#runtime cleanup;
      try self#initialize renderer blur animate with error -> self#fail error

    method private initialize renderer blur animate =
      ready <- true;
      let _ = native##cameras##main##setVisible false in
      let owned_resources = new Phaser_resources.resources native##game in
      resources <- Some owned_resources;
      assets <-
        Some
          (new Phaser_assets.assets
             native owned_resources previews
             ~changed:(fun resource ->
               if
                 Option.fold ~none:true
                   ~some:(fun (resource : Phaser_resource_plan.resource) ->
                     Set.has required_images resource.key
                     || ((not presented) && Set.has required_audio resource.key))
                   resource
               then self#queue_refresh)
             ~report:(fun resource error ->
               if
                 (resource.kind = Phaser_resource_plan.Image && Set.has required_images resource.key)
                 || (not presented)
                    && Option.fold ~none:false
                         ~some:(fun (frame : Frame.t) -> frame.view.sound)
                         frame
                    && Set.has required_audio resource.key
               then self#fail (Failure (Phaser_files.describe error))
               else if resource.kind = Phaser_resource_plan.Image then (
                 asset_error <- Some (Phaser_files.describe error);
                 self#report_loading))
             ~audio_readiness:(fun ready error ->
               Phaser_sound.report_readiness audio ready (Option.map Phaser_files.describe error)));
      let suspend _ = if ready && not failed then self#assets#suspend in
      let resume _ =
        if ready && (not failed) && not (backgrounded Browser_dom.document) then (
          self#assets#resume;
          self#retry_resources None)
      in
      let visibility event =
        if backgrounded Browser_dom.document then suspend event else resume event
      in
      let window_events = [ ("pagehide", suspend); ("pageshow", resume); ("online", resume) ] in
      Browser_dom.addDocumentListener Browser_dom.document "visibilitychange" visibility;
      List.iter (fun (event, handler) -> Browser_dom.listen_error event handler) window_events;
      let detach () =
        Browser_dom.removeDocumentListener Browser_dom.document "visibilitychange" visibility;
        List.iter (fun (event, handler) -> Browser_dom.remove_error event handler) window_events
      in
      let _ = native##events##once "shutdown" (fun[@u] () -> detach ()) in
      Phaser_runtime.on_dispose self#runtime detach;
      if backgrounded Browser_dom.document then self#assets#suspend;
      Phaser_sound.sound_ready audio native##game;
      Option.iter
        (fun mouse -> Phaser.SceneMethods.prevent_wheel mouse false)
        (Js.Nullable.toOption native##input##manager##mouse);
      Option.iter
        (fun canvas ->
          let lost _ =
            self#fail
              (Failure "The battlefield WebGL context was lost. Reload the game to restart cleanly.")
          in
          Phaser_input_dom.on_canvas canvas "webglcontextlost" lost;
          let _ =
            native##events##once "destroy" (fun[@u] () ->
                Phaser_input_dom.off_canvas canvas "webglcontextlost" lost)
          in
          Phaser_runtime.on_dispose self#runtime (fun () ->
              Phaser_input_dom.off_canvas canvas "webglcontextlost" lost);
          ())
        (Js.Nullable.toOption native##game##canvas);
      let lighting_pipeline =
        new Phaser_lighting.lighting native##game native##cameras##main owned_resources native
      in
      lighting <- Some lighting_pipeline;
      let _ = renderer##pipelines##add "BattlefieldLighting" lighting_pipeline#native in
      let actor_pipeline = new Phaser_actor_lighting.actor_lighting native##game in
      actor_lighting <- Some actor_pipeline;
      let _ = renderer##pipelines##add "BattlefieldActorLighting" actor_pipeline#native in
      let layer depth =
        let layer = native##add##container 0. 0. in
        let _ = layer##setDepth depth in
        layer
      in
      actor_roots <- Some (layer 20000.);
      upper_indicators <- Some (layer 30000.);
      items <- Some (layer 11000.);
      outlines <- Some (layer 10000.);
      owners <- Some (layer 40000.);
      Array.iter
        (fun (name, create) ->
          let pipeline = Phaser_pipeline.native (create native##game) in
          let _ = Array_ext.append_value atmosphere_pipelines pipeline in
          let _ = renderer##pipelines##add name pipeline in
          ())
        [|
          ("BattlefieldHalo", Phaser_atmosphere.halo);
          ("BattlefieldFogBlur", Phaser_atmosphere.fog_blur);
          ("BattlefieldFogComposite", Phaser_atmosphere.fog_composite);
          ("BattlefieldFogMask", Phaser_atmosphere.fog_mask);
        |];
      let _ =
        Phaser.SceneMethods.white_texture native##textures "battlefield-white"
          (Phaser.SceneMethods.bytes [| 255; 255; 255; 255 |])
          1 1
      in
      controls <-
        Some (new Phaser_camera.input native (fun () -> frame) actions (fun () -> self#invalidate));
      playback <-
        Some
          (new Phaser_playback.playback
             audio
             (fun () -> self#frame.view.catalog)
             ~windup:(fun id at ->
               Option.iter
                 (fun actor ->
                   self#pulse ~at ~duration:240. actor.data.tile "attack";
                   let _ = Set.add active_actors actor in
                   ())
                 (self#find_actor id))
             ~impact:(fun (combat : Domain.combat_animation) index at ->
               let strike = combat.strikes.(index) in
               Array.iteri
                 (fun i (unit : Domain.public_unit) ->
                   Option.iter
                     (fun actor ->
                       self#draw_indicators actor;
                       let before =
                         if index > 0 then combat.strikes.(index - 1).health.(i) else unit.hp
                       in
                       if
                         (not self#frame.reduced_motion)
                         && strike.health.(i) < before
                         && Map.has actors unit.id
                       then self#transition actor Hit 260. at)
                     (self#find_actor unit.id))
                 combat.units;
               self#invalidate)
             ~invalidate:(fun () -> self#invalidate)
             ~fail:(fun error -> self#fail error));
      let _ = native##game##events##on "blur" blur in
      let _ = native##game##events##on "poststep" animate in
      let render_start : (int * float) option ref = ref None in
      let _ =
        native##events##on "prerender" (fun[@u] () ->
            render_start :=
              Option.map
                (fun (config : Performance_trace.config) -> (config.id, now ()))
                (Performance_trace.configuration ()))
      in
      let _ =
        native##events##on "render" (fun[@u] () ->
            (match (!render_start, Performance_trace.configuration ()) with
            | Some (id, start), Some config when config.id = id ->
                Performance_trace.instant "Battlefield CPU render submission"
                  ~args:(Performance_trace.obj [ ("durationMs", Js.Json.number (now () -. start)) ])
            | _ -> ());
            if has_frame && (not loading) && not failed then
              Option.iter
                (fun canvas ->
                  if style_visibility canvas##style = "hidden" then (
                    set_visibility canvas##style "visible";
                    presented <- true;
                    self#report_loading;
                    Performance_trace.instant "Battlefield first complete frame"
                      ~args:(Performance_trace.obj [ ("scene", Js.Json.string identity) ])))
                (Js.Nullable.toOption native##game##canvas);
            if presented && (not loading) && not failed then (
              self#playback#ready;
              self#acknowledge_action))
      in
      self#refresh_safely

    method return_to_commander = Option.iter (fun controls -> controls#return_to_commander) controls

    method present (view : Frame.view) reduced_motion =
      if not (Option.fold ~none:false ~some:(fun old -> old == view.artwork) artwork_metadata) then (
        artwork_metadata <- Some view.artwork;
        image_metadata <- Map.create ();
        Array.iter
          (fun key ->
            Option.iter
              (fun (frame : Art.frame) ->
                let _ = Map.set image_metadata frame.url frame in
                ())
              (Art.find view.artwork key))
          (Art.keys view.artwork));
      let next = frame_cache#prepare view reduced_motion in
      if not (Option.fold ~none:false ~some:(fun (old : Frame.t) -> old.tiles == next.tiles) frame)
      then (
        tiles_by_id <- Map.create ();
        Array.iter
          (fun (tile : Frame.tile) ->
            let _ = Map.set tiles_by_id tile.terrain.id tile in
            ())
          next.tiles);
      if not (same_option action view.action) then (
        Option.iter (fun unwatch -> unwatch ()) unwatch_action;
        action <- view.action;
        action_finished <- false;
        action_lease <- Option.map Action_presentation.claim action;
        unwatch_action <-
          Option.map
            (fun action ->
              Phaser_sound.watch audio (Action_presentation.owner action) (fun () ->
                  self#invalidate))
            action);
      frame <- Some next;
      if ready then self#refresh_safely

    method private refresh_safely =
      try
        let changed =
          Performance_trace.measure "Phaser scene preparation" (fun () -> self#refresh)
        in
        if changed then self#invalidate;
        let visible = presented || (has_frame && not loading) in
        let _ = native##cameras##main##setVisible visible in
        Phaser.SceneMethods.input_enabled native##input visible;
        self#report_loading
      with error -> self#fail error

    method private queue_refresh =
      (* Coalesce work, not revisions: another completion may arrive while an
         earlier refresh is queued or being prepared. *)
      resource_epoch <- resource_epoch + 1;
      if not refresh_queued then (
        refresh_queued <- true;
        let _ =
          native##events##once "preupdate" (fun[@u] () ->
              refresh_queued <- false;
              if ready && not failed then self#refresh_safely)
        in
        ())

    method private report_loading =
      let audio_total =
        if Option.fold ~none:false ~some:(fun (frame : Frame.t) -> frame.view.sound) frame then
          Set.size required_audio
        else 0
      in
      let count keys ready =
        Array.fold_left
          (fun count key -> if ready key then count + 1 else count)
          0 (Set.values_array keys)
      in
      let audio_loaded =
        if audio_total > 0 then count required_audio (fun key -> native##cache##audio##exists key)
        else 0
      in
      let total = Set.size required_images + audio_total in
      let loaded = count required_images (fun key -> native##textures##exists key) + audio_loaded in
      let next =
        {
          Battlefield_view.identity;
          phase =
            (if presented then "ready"
             else if loaded < total || total = 0 then "loading"
             else "preparing");
          loaded;
          total;
          audio_loaded;
          audio_total;
          error = asset_error;
        }
      in
      if previous_progress <> Some next then (
        previous_progress <- Some next;
        progress next)

    method retry_resources kind =
      if Option.is_none kind then asset_error <- None;
      self#assets#retry kind;
      self#report_loading

    method private reset next_identity =
      identity <- next_identity;
      self#playback#stop;
      playback_frame <- None;
      plan_key <- None;
      Set.clear required_audio;
      presented <- false;
      asset_error <- None;
      let _ = native##cameras##main##setVisible false in
      native##tweens##killAll ();
      Map.clear pulses;
      has_frame <- false;
      Option.iter
        (fun canvas -> set_visibility canvas##style "hidden")
        (Js.Nullable.toOption native##game##canvas);
      let frame = self#frame in
      let units =
        Array_ext.filter
          (fun (unit : Domain.public_unit) -> Native.controllable frame.view.game unit.id)
          (Native.units frame.view.game)
      in
      let commander =
        match
          Array.find_opt
            (fun (unit : Domain.public_unit) -> unit.role = Some Domain.Role.Leader)
            units
        with
        | Some commander -> Some commander
        | None -> if Array.length units = 0 then None else Some units.(0)
      in
      let ids =
        Array_ext.filter_map
          (fun (unit : Domain.public_unit) ->
            if Option.fold ~none:false ~some:(fun commander -> commander == unit) commander then
              None
            else Some (Hex.key unit.tile))
          units
      in
      let ids =
        Array.append ids
          (Option.fold ~none:[||]
             ~some:(fun (unit : Domain.public_unit) -> [| Hex.key unit.tile |])
             commander)
      in
      Option.iter (fun controls -> controls#reset ids) controls;
      Map.each actors (fun[@u] actor _ ->
          actor.indicators##destroy ();
          actor.root##destroy ());
      Map.each dying (fun[@u] _ root -> root##destroy ());
      Map.clear actors;
      Set.clear active_actors;
      Map.clear dying;
      Option.iter (fun terrain -> terrain#destroy) terrain;
      let state = Native.state frame.view.game in
      let manifest = Phaser_resource_plan.texture state.map_id state.map_revision in
      terrain <-
        Some
          (new Phaser_terrain.terrain
             native manifest
             (Phaser_resource_plan.initial_terrain frame.view.catalog state.map_id
                state.map_revision frame.view.artwork)
             self#lighting self#resources);
      terrain_game <- None;
      Map.each ground_items (fun[@u] item _ -> item.root##destroy ());
      Map.each hexes (fun[@u] hex _ -> hex.root##destroy ());
      Map.each owner_markers (fun[@u] marker _ -> marker##destroy ());
      visibility#clear;
      let _ = self#items##removeAll true in
      let _ = self#outlines##removeAll true in
      Map.clear hexes;
      Map.clear ground_items;
      let _ = self#owners##removeAll true in
      Map.clear owner_markers;
      Option.iter (fun atmosphere -> atmosphere#destroy) atmosphere;
      atmosphere <- Some (new Phaser_fog.fog native manifest frame.terrain_tiles self#resources);
      Option.iter (fun controls -> controls#layout) controls

    method private refresh =
      match frame with
      | None -> false
      | Some _ when failed -> false
      | Some frame -> (
          if frame.reduced_motion then (
            Map.each pulses (fun[@u] pulse _ ->
                Option.iter (fun (hex : outline) -> hex.tile <- None) (Map.get hexes pulse.tile));
            Map.each pulses (fun[@u] pulse _ -> Pulse_tween.kill native##tweens pulse);
            Map.clear pulses;
            Map.each dying (fun[@u] death root ->
                native##tweens##killTweensOf (Phaser.Upcast.container root);
                visibility#delete death.key;
                let _ = Set.delete active_actors death.actor in
                root##destroy ());
            Map.clear dying);
          let state = Native.state frame.view.game in
          let next_identity = string_of_int frame.view.session_version ^ ":" ^ state.map_id in
          if next_identity <> identity then (
            self#reset next_identity;
            last_visual <- None;
            scene_game <- None);
          if not (same_option playback_frame frame.view.combat_playback) then (
            playback_frame <- frame.view.combat_playback;
            let visible = Set.create () in
            Map.each actors (fun[@u] _ id ->
                let _ = Set.add visible id in
                ());
            Array.iter
              (fun (unit : Domain.public_unit) ->
                let _ = Set.add visible unit.id in
                ())
              (Native.units frame.view.game);
            let owner =
              match action with
              | Some action -> Some (Action_presentation.owner action)
              | None ->
                  Option.map
                    (fun playback -> Phaser_sound.combat_owner playback.Combat_playback.combat)
                    frame.view.combat_playback
            in
            self#playback#present frame.view.combat_playback visible frame.reduced_motion owner);
          let previous = last_visual in
          let same_world previous =
            previous.game == frame.view.game
            && previous.motion = frame.reduced_motion
            && previous.sound = frame.view.sound && previous.epoch = resource_epoch
          in
          match previous with
          | Some previous
            when same_world previous
                 && Option.fold ~none:false
                      ~some:(fun artwork -> artwork == frame.view.artwork)
                      scene_artwork
                 && previous.tiles != frame.tiles ->
              Array.iteri
                (fun i tile ->
                  if i >= Array.length previous.tiles || previous.tiles.(i) != tile then
                    self#draw_outlines (Some tile.Frame.terrain.id))
                frame.tiles;
              self#describe_inspection;
              last_visual <- Some { previous with tiles = frame.tiles };
              true
          | Some previous when same_world previous && previous.tiles == frame.tiles ->
              self#describe_inspection;
              false
          | _ -> self#refresh_world frame)

    method private refresh_world (frame : Frame.t) =
      (* Asset admission may synchronously invalidate this pass. Remember the
         input epoch so the queued pass cannot mistake stale readiness for fresh. *)
      let preparation_epoch = resource_epoch in
      let fog = Set.create () in
      Array.iter
        (fun hex ->
          let _ = Set.add fog (Hex.key hex) in
          ())
        (Native.presentation frame.view.game).fog;
      Map.each actors (fun[@u] actor id ->
          if Option.is_none (Native.unit frame.view.game id) && Set.has fog actor.data.tile then (
            native##tweens##killTweensOf (Phaser.Upcast.container actor.root);
            Option.iter (Transition_tween.kill native##tweens) actor.transition;
            actor.indicators##destroy ();
            actor.root##destroy ();
            let _ = Map.delete actors id in
            let _ = Set.delete active_actors actor in
            visibility#delete ("actor:" ^ id);
            visibility#delete ("indicator:" ^ id)));
      Map.each dying (fun[@u] death root ->
          if Set.has fog death.tile then (
            visibility#delete death.key;
            native##tweens##killTweensOf (Phaser.Upcast.container root);
            root##destroy ();
            let _ = Set.delete active_actors death.actor in
            let _ = Map.delete dying root in
            ()));
      Option.iter
        (fun atmosphere ->
          atmosphere#update
            (Array.map Hex.key (Native.presentation frame.view.game).fog)
            frame.terrain_tiles)
        atmosphere;
      if
        (not (Option.fold ~none:false ~some:(fun old -> old == frame.view.game) scene_game))
        || (not
              (Option.fold ~none:false ~some:(fun old -> old == frame.view.artwork) scene_artwork))
        || Option.is_none scene_data
      then (
        scene_game <- Some frame.view.game;
        scene_artwork <- Some frame.view.artwork;
        scene_data <-
          Some (Performance_trace.measure "scene.project" (fun () -> Frame.scene_data frame)));
      let scene = Option.get scene_data and terrain = Option.get terrain in
      let urls = Set.create () in
      if not (Option.fold ~none:false ~some:(fun old -> old == frame.view.game) terrain_game) then (
        terrain_game <- Some frame.view.game;
        terrain_urls <-
          Performance_trace.measure "scene.terrain_plan" (fun () ->
              terrain#plan frame.terrain_tiles));
      Set.each terrain_urls (fun[@u] url ->
          let _ = Set.add urls url in
          ());
      Array.iter
        (fun unit ->
          let _ = Set.add urls (frame_url unit) in
          ())
        scene.units;
      Array.iter
        (fun (item : Frame.ground_item) ->
          let _ = Set.add urls item.url in
          ())
        scene.items;
      Array.iter
        (fun (unit : Domain.public_unit) ->
          Array.iter
            (fun (item : Domain.item) ->
              let _ =
                Set.add urls (Phaser_resource_plan.item_frame frame.view.artwork item.id).url
              in
              ())
            unit.items)
        (Native.units frame.view.game);
      required_images <- urls;
      if
        Option.is_none plan_key
        || not (Option.fold ~none:false ~some:(fun old -> old == frame.view.game) planned_game)
      then (
        planned_game <- Some frame.view.game;
        let next_key =
          Phaser_resource_plan.key ~session_version:frame.view.session_version ~game:frame.view.game
            ~catalog:frame.view.catalog
        in
        if not (Option.fold ~none:false ~some:(Phaser_resource_plan.same_key next_key) plan_key)
        then (
          plan_key <- Some next_key;
          required_audio <- Set.create ();
          Array.iter
            (fun (sample : Phaser_sound.sample) ->
              let _ = Set.add required_audio sample.key in
              ())
            (Phaser_resource_plan.current_map_audio frame.view.game frame.view.catalog);
          let plan =
            Phaser_resource_plan.prepare ~game:frame.view.game ~catalog:frame.view.catalog
              ~artwork:frame.view.artwork
          in
          Set.each terrain#retired_sources (fun[@u] key ->
              let _ = Map.delete plan key in
              ());
          let retain (image : Phaser.image) =
            let key = image##texture##key in
            if not (Map.has plan key) then
              let _ =
                Map.set plan key
                  {
                    Phaser_resource_plan.key;
                    url = key;
                    kind = Image;
                    priority = 5;
                    hud = false;
                    width = Some image##width;
                    height = Some image##height;
                  }
              in
              ()
          in
          Map.each actors (fun[@u] actor _ -> retain actor.image);
          (* Dying roots contain the retained actor image plus its base/indicators. *)
          Map.each dying (fun[@u] death _ -> retain death.actor.image);
          self#assets#set_plan plan));
      self#assets#require_images urls (Some image_metadata);
      Set.each urls (fun[@u] url ->
          Option.iter
            (fun failure -> failwith (Phaser_files.describe failure))
            (self#assets#error url));
      if (not presented) && frame.view.sound then
        Set.each required_audio (fun[@u] key ->
            Option.iter
              (fun failure -> failwith (Phaser_files.describe failure))
              (self#assets#error key));
      loading <-
        Array.exists (fun url -> not (native##textures##exists url)) (Set.values_array urls)
        || (not presented) && frame.view.sound
           && Array.exists
                (fun key -> not (native##cache##audio##exists key))
                (Set.values_array required_audio);
      let terrain_ready =
        Array.for_all (fun url -> native##textures##exists url) (Set.values_array terrain_urls)
      in
      self#actor_lighting#configure (Native.presentation frame.view.game).lighting;
      if terrain_ready then (
        terrain#update (Native.presentation frame.view.game).lighting;
        Set.each terrain#retired_sources (fun[@u] key ->
            if Set.delete terrain_urls key then
              let _ = Set.delete urls key in
              self#assets#retire_image key));
      self#draw_outlines None;
      let ready_items =
        Array_ext.filter
          (fun (item : Frame.ground_item) -> native##textures##exists item.url)
          scene.items
      in
      let live_items = Set.create () in
      Array.iter
        (fun (item : Frame.ground_item) ->
          let _ = Set.add live_items item.key in
          ())
        ready_items;
      Map.each ground_items (fun[@u] item key ->
          if not (Set.has live_items key) then (
            item.root##destroy ();
            visibility#delete ("item:" ^ key);
            let _ = Map.delete ground_items key in
            ()));
      Array.iteri
        (fun ordinal (item : Frame.ground_item) ->
          let record =
            match Map.get ground_items item.key with
            | Some record -> record
            | None ->
                let anchor = [%mel.obj { x = item.x; y = item.y }] in
                let root = native##add##container item.x item.y in
                let base = native##add##ellipse 0. 14. 24. 10. 0xd5bc68 in
                let _ = base##setFillStyle 0xd5bc68 0.4 in
                let _ = base##setSmoothness 20 in
                let data =
                  {
                    Phaser_actor_lighting.unlit = false;
                    light_anchor = None;
                    light_point = Some anchor;
                  }
                in
                let _ =
                  Phaser_actor_lighting.Pipeline_data.ellipse base "BattlefieldActorLighting" data
                    false
                in
                let image = native##add##image 0. 0. item.url in
                let _ = image##setDisplaySize 64. 64. in
                let _ =
                  Phaser_actor_lighting.Pipeline_data.image image "BattlefieldActorLighting" data
                    false
                in
                Phaser.SceneMethods.add_many root
                  [| Phaser.Upcast.ellipse base; Phaser.Upcast.image image |];
                let _ = self#items##add (Phaser.Upcast.container root) in
                let record = { root; image; anchor } in
                let _ = Map.set ground_items item.key record in
                record
          in
          Phaser.Geom.Vector2.set_x record.anchor item.x;
          Phaser.Geom.Vector2.set_y record.anchor item.y;
          let _ = record.root##setPosition item.x item.y in
          let _ = record.image##setTexture item.url in
          self#actor_lighting#prepare (Phaser.Upcast.image record.image);
          visibility#set ("item:" ^ item.key) self#items
            (Phaser.Upcast.container record.root)
            (bounds (item.x -. 32.) (item.y -. 32.) (item.x +. 32.) (item.y +. 32.))
            (float_of_int ordinal))
        ready_items;
      let ready_units =
        Array_ext.filter_map
          (fun (unit : Frame.unit_actor) ->
            if native##textures##exists (frame_url unit) then Some unit
            else
              Option.map (fun actor -> { unit with url = actor.data.url }) (Map.get actors unit.id))
          scene.units
      in
      Performance_trace.measure "scene.actors" (fun () -> self#sync_units ready_units fog);
      Map.each actors (fun[@u] actor _ ->
          let _ = Set.add urls actor.image##texture##key in
          ());
      Map.each dying (fun[@u] death _ ->
          let _ = Set.add urls death.actor.image##texture##key in
          ());
      self#draw_owners None;
      self#describe_inspection;
      Performance_trace.instant "Battlefield workload"
        ~args:
          (Performance_trace.obj
             [
               ("tiles", Performance_trace.int (Array.length frame.tiles));
               ("units", Performance_trace.int (Array.length scene.units));
               ("items", Performance_trace.int (Array.length scene.items));
               ( "lights",
                 Performance_trace.int
                   (Array.length (Native.presentation frame.view.game).lighting.lights) );
               ("fog", Performance_trace.int (Set.size fog));
             ]);
      has_frame <- terrain_ready;
      last_visual <-
        Some
          {
            game = frame.view.game;
            tiles = frame.tiles;
            motion = frame.reduced_motion;
            sound = frame.view.sound;
            epoch = preparation_epoch;
          };
      true

    method private sync_units (units : Frame.unit_actor array) fog =
      let live = Set.create () in
      Array.iter
        (fun (unit : Frame.unit_actor) ->
          let _ = Set.add live unit.id in
          ())
        units;
      let movement : (float * float) option ref = ref None in
      Map.each actors (fun[@u] actor id ->
          if not (Set.has live id) then (
            let _ = Map.delete actors id in
            let _ = Set.delete active_actors actor in
            visibility#delete ("actor:" ^ id);
            visibility#delete ("indicator:" ^ id);
            native##tweens##killTweensOf (Phaser.Upcast.container actor.root);
            Option.iter (Transition_tween.kill native##tweens) actor.transition;
            let _ = actor.image##clearTint () in
            let _ = actor.root##setPosition actor.data.x actor.data.y in
            let _ = actor.root##setAlpha 1. in
            if Set.has fog actor.data.tile then (
              actor.indicators##destroy ();
              actor.root##destroy ())
            else
              let _ = actor.indicators##setPosition 0. 0. in
              let _ = actor.root##add (Phaser.Upcast.graphics actor.indicators) in
              let start =
                match self#playback#death id with
                | Some start -> start
                | None -> if self#playback#holds id then infinity else now ()
              in
              let key = "dying:" ^ id in
              let _ =
                Map.set dying actor.root
                  {
                    tile = actor.data.tile;
                    key;
                    actor;
                    start;
                    owner = self#action_owner_option;
                    y = actor.data.y;
                  }
              in
              visibility#set key self#actor_roots
                (Phaser.Upcast.container actor.root)
                (bounds
                   (actor.root##x -. (actor.data.size /. 2.))
                   (actor.root##y +. 20. -. actor.data.size)
                   (actor.root##x +. (actor.data.size /. 2.))
                   (actor.root##y +. 30.))
                actor.root##depth));
      Array.iter
        (fun (unit : Frame.unit_actor) ->
          let previous = Option.map (fun actor -> actor.data) (Map.get actors unit.id) in
          let actor =
            match Map.get actors unit.id with
            | None ->
                let source = Phaser.Texture.source_image (native##textures##get (frame_url unit)) in
                if image_width source <> 64 || image_height source <> 64 then
                  failwith ("Units must use native 64×64 artwork: " ^ frame_url unit);
                let root = native##add##container unit.x unit.y in
                let _ = root##setDepth (20000. +. unit.y) in
                let base = native##add##ellipse 0. 19. 38. 14. (color unit.color) in
                let _ = base##setFillStyle (color unit.color) 0.5 in
                let _ = base##setSmoothness 20 in
                let _ =
                  Phaser_actor_lighting.Pipeline_data.ellipse base "BattlefieldActorLighting"
                    { unlit = true; light_anchor = None; light_point = None }
                    false
                in
                let image =
                  native##add##image (-.unit.size /. 2.) (20. -. unit.size) (frame_url unit)
                in
                let _ = image##setOrigin 0. 0. in
                let _ = image##setDisplaySize unit.size unit.size in
                let _ =
                  Phaser_actor_lighting.Pipeline_data.image image "BattlefieldActorLighting"
                    {
                      unlit = false;
                      light_anchor = Some (Phaser.Upcast.container_point root);
                      light_point = None;
                    }
                    false
                in
                let indicators = native##add##graphics () in
                let _ = indicators##setDepth 30000. in
                let _ = self#actor_roots##add (Phaser.Upcast.container root) in
                let _ = self#upper_indicators##add (Phaser.Upcast.graphics indicators) in
                Phaser.SceneMethods.add_many root
                  [| Phaser.Upcast.ellipse base; Phaser.Upcast.image image |];
                let actor =
                  {
                    data = unit;
                    root;
                    image;
                    concealment_outline = None;
                    base;
                    indicators;
                    transition = None;
                  }
                in
                let _ = Map.set actors unit.id actor in
                if has_frame && not self#frame.reduced_motion then
                  self#transition actor Spawn 220. (now ());
                actor
            | Some actor ->
                if actor.data.x <> unit.x || actor.data.y <> unit.y then (
                  let duration = if self#frame.reduced_motion then 0. else 240. in
                  let start, ending =
                    match !movement with
                    | Some movement -> movement
                    | None ->
                        let start = now () in
                        let ending =
                          (Phaser_sound.schedule_movement ~owner:self#action_owner audio
                             (start +. duration))
                            .impact_at
                        in
                        movement := Some (start, ending);
                        (start, ending)
                  in
                  if duration <> 0. then self#transition actor Move (ending -. start) start)
                else if
                  unit.hp < actor.data.hp && (not self#frame.reduced_motion)
                  && not (self#playback#holds unit.id)
                then self#transition actor Hit 260. (now ());
                (if actor.data.url <> unit.url || actor.data.size <> unit.size then
                   let _ = actor.image##setTexture (frame_url unit) in
                   let _ = actor.image##setDisplaySize unit.size unit.size in
                   ());
                actor.data <- unit;
                actor
          in
          let _ = Set.add active_actors actor in
          let _ = actor.base##setFillStyle (color unit.color) 0.5 in
          let _ = actor.root##setDepth (20000. +. unit.y +. (unit.x *. 0.00001)) in
          let concealed = unit.concealment = Domain.Unit_concealment_appearance.Concealed in
          let marked = unit.concealment <> Domain.Unit_concealment_appearance.Normal in
          let _ = actor.image##setAlpha (if concealed then 0.38 else 1.) in
          if marked then (
            let key = Phaser_unit_outline.texture native##textures (frame_url unit) in
            let outline =
              match actor.concealment_outline with
              | Some image ->
                  ignore (image##setTexture key);
                  image
              | None ->
                  let outline = native##add##image (-.unit.size /. 2.) (20. -. unit.size) key in
                  ignore (outline##setOrigin 0. 0.);
                  ignore (actor.root##add (Phaser.Upcast.image outline));
                  actor.concealment_outline <- Some outline;
                  outline
            in
            ignore (outline##setDisplaySize unit.size unit.size);
            ignore (outline##setVisible true);
            ignore (outline##setTint (if concealed then 0xbed1b5 else 0xe7b45d)))
          else Option.iter (fun image -> ignore (image##setVisible false)) actor.concealment_outline;
          if self#frame.reduced_motion then (
            Option.iter (Transition_tween.kill native##tweens) actor.transition;
            actor.transition <- None;
            let _ = actor.root##setPosition unit.x unit.y in
            let _ = actor.root##setAlpha 1. in
            let _ = actor.image##clearTint () in
            ());
          if
            not
              (Option.fold ~none:false
                 ~some:(fun (previous : Frame.unit_actor) ->
                   previous.health = unit.health && previous.spent = unit.spent
                   && previous.retreating = unit.retreating)
                 previous)
          then self#draw_indicators actor)
        units

    method private draw_indicators actor =
      let unit = actor.data and g = actor.indicators in
      let _ = g##clear () in
      let _ = g##fillStyle 0x18211b 1. in
      let _ = g##fillRect (-19.) 27. 38. 4. in
      let _ = g##fillStyle (if unit.spent then 0x60785e else 0xa9d0a5) 1. in
      let _ =
        g##fillRect (-19.) 27.
          (clamp (Option.value ~default:unit.health (self#playback#health_width unit.id)) 0. 38.)
          4.
      in
      let _ = g##lineStyle 2. 0xdbe8d7 1. in
      (if unit.spent then
         let _ = g##beginPath () in
         let _ = g##moveTo 22. 28. in
         let _ = g##lineTo 25. 31. in
         let _ = g##lineTo 30. 25. in
         let _ = g##strokePath () in
         ());
      if unit.retreating then
        let _ = g##lineStyle 2. 0xfff1c1 1. in
        let _ = g##beginPath () in
        let _ = g##moveTo (-22.) (-21.) in
        let _ = g##lineTo (-30.) (-21.) in
        let _ = g##lineTo (-26.) (-25.) in
        let _ = g##moveTo (-30.) (-21.) in
        let _ = g##lineTo (-26.) (-17.) in
        let _ = g##strokePath () in
        ()

    method private transition actor kind duration at =
      Option.iter (Transition_tween.kill native##tweens) actor.transition;
      actor.transition <-
        Some
          {
            kind;
            owner = self#action_owner_option;
            t = 0.;
            from_x = actor.root##x;
            from_y = actor.root##y;
            start = at;
            end_ = at +. duration;
          };
      let _ = Set.add active_actors actor in
      self#invalidate

    method private animate_actors =
      if Option.is_some frame && ready then (
        let at = now () in
        self#playback#update at;
        Map.each pulses (fun[@u] pulse key ->
            pulse.t <- clamp ((at -. pulse.start) /. pulse.duration) 0. 1.;
            (if pulse.t >= 1. then
               let _ = Map.delete pulses key in
               ());
            if pulse.kind = "select" then self#draw_outlines (Some pulse.tile);
            if pulse.kind = "capture" then self#draw_owners (Some pulse.tile);
            self#invalidate);
        Map.each dying (fun[@u] death root ->
            let start =
              Option.value ~default:death.start (self#playback#death death.actor.data.id)
            in
            if Float.is_finite start then death.start <- start;
            let duration = if self#frame.reduced_motion then 0. else 300. in
            let t =
              if at < start then 0.
              else if duration <> 0. then min 1. ((at -. start) /. duration)
              else 1.
            in
            (if
               t >= 1.
               || ((not (Float.is_finite start)) && not (self#playback#holds death.actor.data.id))
             then (
               let _ = Map.delete dying root in
               visibility#delete death.key;
               let _ = Set.delete active_actors death.actor in
               root##destroy ())
             else
               let _ = root##setAlpha (1. -. t) in
               let _ = root##setY (death.y +. (t *. 10.)) in
               visibility#set death.key self#actor_roots (Phaser.Upcast.container root)
                 (bounds
                    (root##x -. (death.actor.data.size /. 2.))
                    (root##y +. 20. -. death.actor.data.size)
                    (root##x +. (death.actor.data.size /. 2.))
                    (root##y +. 30.))
                 root##depth);
            self#invalidate);
        if Set.size active_actors > 0 then self#invalidate;
        Set.each active_actors (fun[@u] actor ->
            let dying_actor = Map.get dying actor.root in
            if
              Option.fold ~none:false
                ~some:(fun (death : death) ->
                  at >= Option.value ~default:death.start (self#playback#death actor.data.id))
                dying_actor
            then
              let _ = Set.delete active_actors actor in
              ()
            else
              let unit = actor.data and transition = actor.transition in
              let t =
                Option.fold ~none:1.
                  ~some:(fun (transition : transition) ->
                    clamp ((at -. transition.start) /. (transition.end_ -. transition.start)) 0. 1.)
                  transition
              in
              Option.iter
                (fun (transition : transition) ->
                  transition.t <- t;
                  if t >= 1. then actor.transition <- None)
                transition;
              let x = ref unit.x and y = ref unit.y and alpha = ref 1. in
              (match transition with
              | Some { kind = Move; from_x; from_y; _ } ->
                  let ease = t *. t *. (3. -. (2. *. t)) in
                  x := from_x +. ((unit.x -. from_x) *. ease);
                  y := from_y +. ((unit.y -. from_y) *. ease) -. (sin (t *. Float.pi) *. 5.)
              | Some { kind = Spawn; _ } ->
                  alpha := t;
                  y := !y -. ((1. -. t) *. 10.)
              | Some { kind = Hit; _ } -> x := !x +. (sin (t *. Float.pi *. 6.) *. (1. -. t) *. 4.)
              | _ -> ());
              let pulse = Map.get pulses ("attack:" ^ unit.tile) in
              Option.iter
                (fun pulse ->
                  if pulse.kind = "attack" then x := !x +. (sin (pulse.t *. Float.pi) *. 7.))
                pulse;
              let flash =
                match transition with
                | Some { kind = Hit; _ } -> sin (t *. Float.pi) *. 0.5
                | _ -> 0.
              in
              let shade = int_of_float (floor ((255. *. (1. -. flash)) +. 0.5)) in
              let _ = actor.image##setTint ((255 lsl 16) lor (shade lsl 8) lor shade) in
              let _ = actor.root##setPosition !x !y in
              let _ = actor.root##setAlpha !alpha in
              let _ =
                actor.indicators##setPosition
                  (if Option.is_some dying_actor then 0. else !x)
                  (if Option.is_some dying_actor then 0. else !y)
              in
              let _ = actor.indicators##setAlpha !alpha in
              self#actor_lighting#prepare (Phaser.Upcast.image actor.image);
              if Option.is_none dying_actor then (
                visibility#set ("actor:" ^ unit.id) self#actor_roots
                  (Phaser.Upcast.container actor.root)
                  (bounds
                     (!x -. (unit.size /. 2.))
                     (!y +. 20. -. unit.size)
                     (!x +. (unit.size /. 2.))
                     (!y +. 26.))
                  actor.root##depth;
                visibility#set ("indicator:" ^ unit.id) self#upper_indicators
                  (Phaser.Upcast.graphics actor.indicators)
                  (bounds (!x -. 31.) (!y -. 26.) (!x +. 31.) (!y +. 32.))
                  0.);
              if
                Option.is_none transition
                && not (Option.fold ~none:false ~some:(fun pulse -> pulse.kind = "attack") pulse)
              then
                let _ = Set.delete active_actors actor in
                ());
        self#actor_lighting#prepare_pending;
        let camera = native##cameras##main in
        let _ = camera##getWorldPoint 0. 0. world_origin in
        let _ = camera##getWorldPoint camera##width camera##height world_edge in
        let bounds = bounds world_origin##x world_origin##y world_edge##x world_edge##y in
        visibility#update bounds;
        Option.iter (fun terrain -> terrain#visible bounds) terrain;
        Option.iter (fun atmosphere -> atmosphere#visible bounds) atmosphere)

    method private tiles tile_id =
      match tile_id with
      | None -> self#frame.tiles
      | Some tile_id ->
          Option.fold ~none:[||] ~some:(fun tile -> [| tile |]) (Map.get tiles_by_id tile_id)

    method private draw_outlines tile_id =
      let tiles = self#tiles tile_id in
      if Option.is_none tile_id then (
        let live = Set.create () in
        Array.iter
          (fun (tile : Frame.tile) ->
            let _ = Set.add live tile.terrain.id in
            ())
          tiles;
        Map.each hexes (fun[@u] hex id ->
            if not (Set.has live id) then (
              hex.root##destroy ();
              visibility#delete ("hex:" ^ id);
              let _ = Map.delete hexes id in
              ())));
      Array.iter
        (fun (tile : Frame.tile) ->
          let existing = Map.get hexes tile.terrain.id in
          if
            Option.is_some tile_id
            || not
                 (Option.fold ~none:false
                    ~some:(fun (hex : outline) ->
                      Option.fold ~none:false ~some:(fun old -> old == tile) hex.tile)
                    existing)
          then (
            let classes = String.split_on_char ' ' tile.display.classes in
            let has value = List.mem value classes in
            let ink = ref 0x39472b
            and width = ref Phaser_hex_outline.idle_line_width
            and alpha =
              ref
                (if
                   Lighting.ambient tile.terrain.x tile.terrain.y
                     (Native.presentation self#frame.view.game).lighting
                   < 0.5
                 then 0.14
                 else 0.3)
            in
            if has "selected" then (
              ink := 0xffe5a1;
              width := 3.);
            if has "reachable" then (
              ink := 0x95bfd2;
              width := 2.4;
              alpha := 1.);
            if has "target" then (
              ink := 0xdb8980;
              width := 3.;
              alpha := 1.);
            if has "recruitable" then (
              ink := 0x8ec7a2;
              width := 3.;
              alpha := 1.);
            if has "objective" then (
              ink := if has "reachable" || has "selected" then 0x99c8ec else 0xffe5a1;
              if has "recruitable" then ink := 0x6fdec0;
              width := 3.5;
              alpha := 1.);
            let highlighted =
              Array.exists has [| "selected"; "reachable"; "target"; "recruitable"; "objective" |]
            in
            let pulse = Map.get pulses ("select:" ^ tile.terrain.id) in
            (match pulse with
            | Some { kind = "select"; t; _ } ->
                if not highlighted then (
                  ink := 0xffe5a1;
                  width := 3.);
                alpha :=
                  if t < 0.55 then 0.45 +. (0.55 *. t /. 0.55)
                  else 1. -. (0.2 *. (t -. 0.55) /. 0.45)
            | _ -> ());
            let dash =
              if has "objective" then Some (9., 5.)
              else if has "recruitable" then Some (4., 3.)
              else None
            in
            let hex =
              match existing with
              | Some hex -> hex
              | None ->
                  let outline = new Phaser_hex_outline.outline native in
                  let root = outline#root in
                  let _ = self#outlines##add (Phaser.Upcast.container root) in
                  let hex = { root; ordinal = Map.size hexes; outline; tile = None } in
                  let _ = Map.set hexes tile.terrain.id hex in
                  hex
            in
            hex.outline#update tile.terrain.x tile.terrain.y !ink !width !alpha
              (highlighted
              || Option.fold ~none:false ~some:(fun pulse -> pulse.kind = "select") pulse)
              dash;
            visibility#set ("hex:" ^ tile.terrain.id) self#outlines
              (Phaser.Upcast.container hex.root)
              (bounds (tile.terrain.x -. 42.) (tile.terrain.y -. 45.) (tile.terrain.x +. 42.)
                 (tile.terrain.y +. 45.))
              (float_of_int hex.ordinal);
            hex.tile <- Some tile))
        tiles

    method private draw_owners tile_id =
      let frame = self#frame in
      let tiles = self#tiles tile_id in
      if Option.is_none tile_id then (
        let live = Set.create () in
        Array.iter
          (fun (tile : Frame.tile) ->
            if
              Option.is_some tile.display.tile.owner
              && not (Native.fogged frame.view.game tile.terrain.id)
            then
              let _ = Set.add live tile.terrain.id in
              ())
          tiles;
        Map.each owner_markers (fun[@u] marker id ->
            if not (Set.has live id) then (
              marker##destroy ();
              visibility#delete ("owner:" ^ id);
              let _ = Map.delete owner_markers id in
              match Map.get pulses ("capture:" ^ id) with
              | Some ({ kind = "capture"; _ } as pulse) ->
                  Pulse_tween.kill native##tweens pulse;
                  let _ = Map.delete pulses ("capture:" ^ id) in
                  ()
              | _ -> ())));
      Array.iter
        (fun (tile : Frame.tile) ->
          if
            Option.is_some tile.display.tile.owner
            && not (Native.fogged frame.view.game tile.terrain.id)
          then
            let previous = Map.get owner_markers tile.terrain.id in
            let marker =
              match previous with
              | Some marker -> marker
              | None ->
                  let marker =
                    native##add##ellipse
                      (tile.terrain.x +. 23.) (tile.terrain.y +. 17.) 10. 10.
                      (color tile.display.owner_color)
                  in
                  let _ = marker##setSmoothness 20 in
                  let _ = Map.set owner_markers tile.terrain.id marker in
                  let _ = self#owners##add (Phaser.Upcast.ellipse marker) in
                  marker
            in
            let radius =
              match Map.get pulses ("capture:" ^ tile.terrain.id) with
              | Some { kind = "capture"; t; _ } -> 5. +. (sin (t *. Float.pi) *. 3.)
              | _ -> 5.
            in
            let fill = color tile.display.owner_color in
            if
              Option.is_none previous
              || marker##x <> tile.terrain.x +. 23.
              || marker##y <> tile.terrain.y +. 17.
              || marker##fillColor <> fill
              || marker##scaleX <> radius /. 5.
            then
              let _ = marker##setPosition (tile.terrain.x +. 23.) (tile.terrain.y +. 17.) in
              let _ = marker##setFillStyle fill 1. in
              let _ = marker##setScale (radius /. 5.) (radius /. 5.) in
              let ordinal =
                Option.fold ~none:0
                  ~some:(fun (hex : outline) -> hex.ordinal)
                  (Map.get hexes tile.terrain.id)
              in
              visibility#set ("owner:" ^ tile.terrain.id) self#owners (Phaser.Upcast.ellipse marker)
                (bounds (tile.terrain.x +. 15.) (tile.terrain.y +. 9.) (tile.terrain.x +. 31.)
                   (tile.terrain.y +. 25.))
                (float_of_int ordinal))
        tiles

    method private describe_inspection =
      let frame = self#frame in
      let tile = Option.bind frame.view.inspected_tile (Map.get tiles_by_id) in
      let label =
        match tile with
        | Some tile when Native.fogged frame.view.game tile.terrain.id ->
            "Fog-covered terrain, column "
            ^ string_of_int (Int32.to_int tile.display.tile.col + 1)
            ^ ", row "
            ^ string_of_int (Int32.to_int tile.display.tile.row + 1)
        | Some tile -> tile.display.label
        | None -> "Battlefield"
      in
      describe label;
      Option.iter
        (fun canvas -> canvas##setAttribute "aria-label" (Utf8.to_js ("Battlefield. " ^ label)))
        (Js.Nullable.toOption native##game##canvas)

    method pulse ?sound ?sound_owner ?(at = now ()) ?duration tile_id kind =
      if ready && Option.is_some frame && Map.has tiles_by_id tile_id then (
        Option.iter
          (fun sound ->
            let _ = Phaser_sound.play_interaction ?owner:sound_owner audio sound in
            ())
          sound;
        if not self#frame.reduced_motion then (
          let duration = Option.value ~default:(if kind = "select" then 180. else 300.) duration in
          let key = kind ^ ":" ^ tile_id in
          Option.iter
            (fun previous ->
              Pulse_tween.kill native##tweens previous;
              let _ = Map.delete pulses key in
              if previous.kind = "capture" then self#draw_owners (Some tile_id);
              if previous.kind = "select" then self#draw_outlines (Some tile_id))
            (Map.get pulses key);
          let owner = if kind = "select" then None else self#action_owner_option in
          let _ =
            Map.set pulses key { kind; tile = tile_id; t = 0.; start = at; duration; owner }
          in
          if kind = "attack" then
            Map.each actors (fun[@u] actor _ ->
                if actor.data.tile = tile_id then
                  let _ = Set.add active_actors actor in
                  ());
          self#invalidate;
          if kind = "select" then self#draw_outlines (Some tile_id);
          if kind = "capture" then self#draw_owners (Some tile_id)))
  end
