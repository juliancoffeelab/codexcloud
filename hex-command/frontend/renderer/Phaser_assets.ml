open Phaser_files
open Browser_time
module Map = Host_collections.Map
module Set = Host_collections.Set
open Phaser_resource_plan

type phase = Queued | Active | Ready | Failed
type entry = { resource : resource; mutable state : phase }
type waiter = { resolve : Terrain_preview.image -> unit; reject : exn -> unit }

external splice : 'a array -> int -> int -> 'a array = "splice" [@@mel.send]

class assets (scene : Phaser.scene) (resources : Phaser_resources.resources)
  (hud : Terrain_preview.t) ~(changed : resource option -> unit)
  ~(report : resource -> Phaser_files.failure -> unit)
  ~(audio_readiness : bool -> Phaser_files.failure option -> unit) =
  object (self)
    val files : (string, Phaser.file) Map.t = Map.create ()
    val mutable plan : (string, resource) Map.t = Map.create ()
    val failed : (string, Phaser_files.failure) Map.t = Map.create ()
    val demand : (string, resource) Map.t = Map.create ()
    val images : string Set.t = Set.create ()
    val states : (string, entry) Map.t = Map.create ()
    val mutable queues : entry array array = Array.init 5 (fun _ -> [||])
    val mutable cursors = Array.make 5 0
    val mutable pending = Array.make 5 0
    val mutable decoded : Phaser.file array = [||]
    val mutable required : string Set.t = Set.create ()
    val adoptions : (string, Terrain_preview.image) Map.t = Map.create ()
    val mutable adoption_queues : entry array array = Array.init 5 (fun _ -> [||])
    val mutable adoption_cursors = Array.make 5 0
    val mutable audio_total = 0
    val mutable audio_loaded = 0
    val mutable audio_error : Phaser_files.failure option = None
    val waiters : (string, waiter array) Map.t = Map.create ()
    val mutable detach = fun () -> ()
    val mutable disposed = false
    val mutable suspended = false

    initializer
      let admit = fun[@u] () -> self#admit in
      let completed = fun[@u] key -> self#completed key in
      let errored = fun[@u] file -> self#errored file in
      let schedule = fun[@u] () -> self#schedule in
      let _ = scene##events##on {js|preupdate|js} admit in
      let unbind = Terrain_preview.bind hud self#request in
      on_key scene##load {js|filecomplete|js} completed;
      on_error scene##load {js|loaderror|js} errored;
      let _ = scene##load##on {js|complete|js} schedule in
      detach <-
        (fun () ->
          unbind ();
          let _ = scene##events##off {js|preupdate|js} admit in
          off_key scene##load {js|filecomplete|js} completed;
          off_error scene##load {js|loaderror|js} errored;
          let _ = scene##load##off {js|complete|js} schedule in
          ())

    method files = files

    (* Background tabs cannot be expected to finish downloads or decode before
       their deadlines. Requeue unfinished work without reporting a failure. *)
    method suspend =
      if (not disposed) && not suspended then (
        suspended <- true;
        Map.each files (fun[@u] file key -> self#cancel key file))

    method resume = if not disposed then suspended <- false

    method private cached resource =
      match resource.kind with
      | Image -> scene##textures##exists resource.key
      | Audio -> scene##cache##audio##exists resource.key

    method set_plan next =
      plan <- next;
      Map.each demand (fun[@u] resource key ->
          let _ = Map.set plan key resource in
          ());
      Map.each files (fun[@u] file key -> if not (Map.has plan key) then self#cancel key file);
      Set.each images (fun[@u] key ->
          if not (Map.has plan key) then (
            let _ = scene##textures##remove key in
            resources#remove key;
            let _ = Set.delete images key in
            ()));
      Map.each plan (fun[@u] resource _ ->
          (match (resource.kind, resource.width, resource.height) with
          | Image, Some width, Some height when width <> 0. && height <> 0. ->
              let _ = resources#check resource.key width height in
              ()
          | _ -> ());
          if resource.kind = Image && resource.hud && not (scene##textures##exists resource.key)
          then
            let frame : Art.frame =
              {
                url = resource.url;
                width = 64.;
                height = 64.;
                anchor_x = 0.;
                anchor_y = 0.;
                z = 0.;
              }
            in
            Option.iter
              (fun image ->
                Option.iter (self#cancel resource.key) (Map.get files resource.key);
                let _ = Map.set adoptions resource.key image in
                ())
              (Terrain_preview.ready hud frame));
      self#rebuild;
      self#pump;
      self#publish_audio

    method private phase resource =
      if self#cached resource then Ready
      else if Map.has failed resource.key then Failed
      else if Map.has files resource.key then Active
      else Queued

    method private rebuild =
      queues <- Array.init 5 (fun _ -> [||]);
      cursors <- Array.make 5 0;
      pending <- Array.make 5 0;
      Map.clear states;
      adoption_queues <- Array.init 5 (fun _ -> [||]);
      adoption_cursors <- Array.make 5 0;
      Map.each adoptions (fun[@u] _ key ->
          if not (Map.has plan key) then
            let _ = Map.delete adoptions key in
            ());
      audio_total <- 0;
      audio_loaded <- 0;
      audio_error <- None;
      Map.each plan (fun[@u] resource _ ->
          let entry = { resource; state = self#phase resource } in
          let _ = Map.set states resource.key entry in
          (if Map.has adoptions resource.key then
             let _ = Array_ext.append_value adoption_queues.(resource.priority - 1) entry in
             ());
          if entry.state = Queued then self#enqueue entry;
          if resource.kind = Audio then (
            if resource.priority = 1 then (
              audio_total <- audio_total + 1;
              if entry.state = Ready then audio_loaded <- audio_loaded + 1);
            if entry.state = Failed && Option.is_none audio_error then
              audio_error <- Map.get failed resource.key))

    method private enqueue entry =
      entry.state <- Queued;
      let priority = entry.resource.priority - 1 in
      let _ = Array_ext.append_value queues.(priority) entry in
      pending.(priority) <- pending.(priority) + 1

    method private promote resource =
      (match Map.get states resource.key with
      | Some old when old.state = Queued ->
          pending.(old.resource.priority - 1) <- pending.(old.resource.priority - 1) - 1
      | _ -> ());
      let entry = { resource; state = self#phase resource } in
      let _ = Map.set states resource.key entry in
      (if Map.has adoptions resource.key then
         let _ = Array_ext.append_value adoption_queues.(resource.priority - 1) entry in
         ());
      if entry.state = Queued then self#enqueue entry

    method private owns (file : Phaser.file) =
      Option.fold ~none:false ~some:(fun current -> current == file) (Map.get files file##key)

    method private admit_decoded index =
      let removed = splice decoded index 1 in
      let file = removed.(0) in
      if self#owns file then (
        resources#remove
          (let js_text_part_1 = file##key in
           {j|pending:$(js_text_part_1)|j});
        self#check_image file##key (Phaser_files.image_data file);
        Performance_trace.measure {js|Phaser image admission|js} (fun () -> Phaser_files.admit file))

    method private admit_adoption key image =
      let _ = Map.delete adoptions key in
      if Map.has plan key && not (scene##textures##exists key) then (
        self#check_image key image;
        let _ =
          Performance_trace.measure {js|Phaser image admission|js} (fun () ->
              scene##textures##addImage key image)
        in
        let _ = Set.add images key in
        (match Map.get states key with
        | None -> ()
        | Some entry ->
            if entry.state = Queued then
              pending.(entry.resource.priority - 1) <- pending.(entry.resource.priority - 1) - 1;
            entry.state <- Ready);
        changed (Map.get plan key);
        self#schedule)

    method private admit =
      if (not disposed) && (Array.length decoded > 0 || Map.size adoptions > 0) then (
        let rec foreground () =
          let index = ref (-1) in
          Array.iteri
            (fun i (file : Phaser.file) ->
              if !index < 0 && Set.has required file##key then index := i)
            decoded;
          if !index >= 0 then (
            self#admit_decoded !index;
            foreground ())
        in
        foreground ();
        Set.each required (fun[@u] key ->
            Option.iter (self#admit_adoption key) (Map.get adoptions key));
        let best = ref (-1) and priority = ref 6 in
        Array.iteri
          (fun i (file : Phaser.file) ->
            let p =
              Option.fold ~none:5 ~some:(fun resource -> resource.priority) (Map.get plan file##key)
            in
            if p < !priority then (
              priority := p;
              best := i))
          decoded;
        let adoption = ref None and p = ref 0 in
        while !p < min !priority 5 && Option.is_none !adoption do
          let queue = adoption_queues.(!p) in
          while adoption_cursors.(!p) < Array.length queue && Option.is_none !adoption do
            let entry = queue.(adoption_cursors.(!p)) in
            if
              Option.fold ~none:false
                ~some:(fun current -> current == entry)
                (Map.get states entry.resource.key)
              && Map.has adoptions entry.resource.key
            then adoption := Some entry
            else adoption_cursors.(!p) <- adoption_cursors.(!p) + 1
          done;
          incr p
        done;
        match !adoption with
        | Some entry ->
            adoption_cursors.(entry.resource.priority - 1) <-
              adoption_cursors.(entry.resource.priority - 1) + 1;
            self#admit_adoption entry.resource.key
              (Option.get (Map.get adoptions entry.resource.key))
        | None -> if !best >= 0 then self#admit_decoded !best)

    method private check_image key image =
      let expected = Map.get plan key in
      let width = Phaser_dom.natural_width image and height = Phaser_dom.natural_height image in
      if
        Option.fold ~none:false
          ~some:(fun resource ->
            Option.fold ~none:false
              ~some:(fun expected -> expected <> float_of_int width)
              resource.width
            || Option.fold ~none:false
                 ~some:(fun expected -> expected <> float_of_int height)
                 resource.height)
          expected
      then
        failwith
          (let js_text_part_3 = string_of_int width in
           let js_text_part_5 = string_of_int height in
           {j|Invalid natural artwork dimensions: $(key); received $(js_text_part_3)×$(js_text_part_5)|j});
      resources#image key image

    method retire_image key =
      let _ = Map.delete plan key in
      (match Map.get states key with
      | Some entry when entry.state = Queued ->
          pending.(entry.resource.priority - 1) <- pending.(entry.resource.priority - 1) - 1
      | _ -> ());
      let _ = Map.delete states key in
      let _ = Map.delete adoptions key in
      Option.iter (self#cancel key) (Map.get files key);
      let _ = scene##textures##remove key in
      let _ = Set.delete images key in
      resources#remove key

    method require_images keys (metadata : (string, Art.frame) Map.t option) =
      required <- Set.create ();
      Set.each keys (fun[@u] key ->
          let _ = Set.add required key in
          ());
      let promoted = ref false in
      Set.each keys (fun[@u] key ->
          let old = Map.get plan key in
          if not (Option.fold ~none:false ~some:(fun resource -> resource.priority = 1) old) then (
            let frame = Option.bind metadata (fun metadata -> Map.get metadata key) in
            let dimension get_old get_frame =
              match Option.bind old get_old with
              | Some value -> Some value
              | None -> Option.map get_frame frame
            in
            let resource =
              {
                key;
                url = key;
                kind = Image;
                priority = 1;
                hud = Option.fold ~none:false ~some:(fun old -> old.hud) old;
                width =
                  dimension
                    (fun resource -> resource.width)
                    (fun (frame : Art.frame) -> frame.width);
                height =
                  dimension
                    (fun resource -> resource.height)
                    (fun (frame : Art.frame) -> frame.height);
              }
            in
            let _ = Map.set plan key resource in
            self#promote resource;
            promoted := true));
      if !promoted then self#pump

    method error key = Map.get failed key
    method private schedule = microtask (fun () -> self#pump)

    method private priority =
      let result = ref (-1) in
      Array.iteri (fun i count -> if !result < 0 && count > 0 then result := i) pending;
      !result

    method private pump =
      if (not disposed) && not suspended then (
        let priority = self#priority + 1 in
        (if priority > 0 then
           let needed =
             ref
               (min pending.(priority - 1) scene##load##maxParallelDownloads
               - (scene##load##maxParallelDownloads - Map.size files))
           in
           Map.each files (fun[@u] file key ->
               if
                 !needed > 0
                 && Option.fold ~none:5 ~some:(fun resource -> resource.priority) (Map.get plan key)
                    > priority
               then (
                 self#cancel key file;
                 decr needed)));
        let rec fill () =
          let priority = self#priority in
          if Map.size files < scene##load##maxParallelDownloads && priority >= 0 then (
            let queue = queues.(priority) in
            let entry = ref None in
            while cursors.(priority) < Array.length queue && Option.is_none !entry do
              let candidate = queue.(cursors.(priority)) in
              cursors.(priority) <- cursors.(priority) + 1;
              if
                candidate.state = Queued
                && Option.fold ~none:false
                     ~some:(fun current -> current == candidate)
                     (Map.get states candidate.resource.key)
              then entry := Some candidate
            done;
            let entry =
              match !entry with
              | Some entry -> entry
              | None -> failwith {js|Asset FIFO count mismatch|js}
            in
            pending.(priority) <- pending.(priority) - 1;
            let resource = entry.resource in
            entry.state <- Active;
            (if not (Map.has adoptions resource.key) then
               let file =
                 match resource.kind with
                 | Image ->
                     Some
                       (Phaser_files.image scene##load resource.url
                          (Some
                             (fun file ->
                               if (not disposed) && self#owns file then (
                                 let image = Phaser_files.image_data file in
                                 resources#set
                                   (let js_text_part_1 = file##key in
                                    {j|pending:$(js_text_part_1)|j})
                                   {js|decoded pending images|js}
                                   (float_of_int (Phaser_dom.natural_width image)
                                   *. float_of_int (Phaser_dom.natural_height image)
                                   *. 4.)
                                   0.;
                                 let _ = Array_ext.append_value decoded file in
                                 ()))))
                 | Audio -> (
                     match Js.Nullable.toOption scene##game##sound##context with
                     | None ->
                         self#failure resource
                           {
                             Phaser_files.message =
                               (let js_text_part_1 = resource.key in
                                {j|Web Audio unavailable for $(js_text_part_1)|j});
                             cause = None;
                           };
                         None
                     | Some context ->
                         Some
                           (Phaser_files.audio scene##load resource.key resource.url context
                              (fun cause ->
                                self#failure resource
                                  {
                                    message =
                                      (let js_text_part_1 = resource.key in
                                       let js_text_part_3 = Phaser_files.describe cause in
                                       {j|Audio preparation failed: $(js_text_part_1)\n$(js_text_part_3)|j});
                                    cause = cause.cause;
                                  })))
               in
               Option.iter
                 (fun file ->
                   let _ = Map.set files resource.key file in
                   let _ = scene##load##addFile file in
                   ())
                 file);
            fill ())
        in
        fill ();
        if Map.size files > 0 && not (scene##load##isLoading ()) then
          let _ = scene##load##start () in
          ())

    method private completed key =
      if Map.has files key && not disposed then (
        let completed_file = Option.get (Map.get files key) in
        Boot_process.resource ~key ~url:completed_file##src ~phase:{js|resource-ready|js};
        let _ = Map.delete files key in
        microtask (fun () -> Phaser_files.settle scene##load completed_file);
        let resource = Map.get plan key in
        Option.iter (fun entry -> entry.state <- Ready) (Map.get states key);
        (match resource with
        | Some { kind = Audio; priority; _ } ->
            if priority = 1 then audio_loaded <- audio_loaded + 1;
            let buffer = scene##cache##audio##get key in
            resources#set key {js|decoded audio|js}
              (float_of_int (Phaser_files.audio_length buffer)
              *. float_of_int (Phaser_files.audio_channels buffer)
              *. 4.)
              0.
        | Some { kind = Image; hud = is_hud; _ } ->
            let _ = Set.add images key in
            let image = Phaser.Texture.source_image (scene##textures##get key) in
            if is_hud then Terrain_preview.adopt hud key image;
            Option.iter (Array.iter (fun waiter -> waiter.resolve image)) (Map.get waiters key);
            let _ = Map.delete waiters key in
            let _ = Map.delete demand key in
            ()
        | _ -> ());
        self#publish_audio;
        changed resource;
        self#schedule)

    method private errored (file : Phaser.file) =
      if self#owns file then (
        let _ = Map.delete files file##key in
        Option.iter
          (fun resource ->
            if not (Map.has failed file##key) then
              let failure =
                match Phaser_files.image_failure file with
                | Some failure -> failure
                | None ->
                    {
                      Phaser_files.message =
                        (let js_text_part_1 = file##key in
                         let js_text_part_3 = file##src in
                         {j|Resource failed: $(js_text_part_1) ($(js_text_part_3)); download or decode error|j});
                      cause = None;
                    }
              in
              self#failure resource failure)
          (Map.get plan file##key);
        self#publish_audio;
        self#schedule)

    method private failure (resource : resource) (error : Phaser_files.failure) =
      if (not disposed) && not (Map.has failed resource.key) then (
        let _ = Map.delete files resource.key in
        let _ = Map.set failed resource.key error in
        Option.iter (fun entry -> entry.state <- Failed) (Map.get states resource.key);
        if resource.kind = Audio && Option.is_none audio_error then audio_error <- Some error;
        Option.iter
          (Array.iter (fun waiter -> waiter.reject (Failure (Phaser_files.describe error))))
          (Map.get waiters resource.key);
        let _ = Map.delete waiters resource.key in
        report resource error;
        self#publish_audio;
        self#schedule)

    method private publish_audio =
      if not disposed then audio_readiness (audio_loaded = audio_total) audio_error

    method request (frame : Art.frame) =
      match Terrain_preview.ready hud frame with
      | Some image -> Js.Promise.resolve image
      | None ->
          let resource =
            {
              key = frame.url;
              url = frame.url;
              kind = Image;
              priority = 1;
              hud = true;
              width = Some frame.width;
              height = Some frame.height;
            }
          in
          let _ = Map.set demand frame.url resource in
          let _ = Map.set plan frame.url resource in
          let _ = Map.delete failed frame.url in
          if scene##textures##exists frame.url then (
            let image = Phaser.Texture.source_image (scene##textures##get frame.url) in
            Terrain_preview.adopt hud frame.url image;
            let _ = Map.delete demand frame.url in
            Js.Promise.resolve image)
          else
            let promise =
              Js.Promise.make (fun ~resolve ~reject ->
                  let waiting =
                    match Map.get waiters frame.url with Some waiting -> waiting | None -> [||]
                  in
                  let _ =
                    Array_ext.append_value waiting
                      {
                        resolve = (fun image -> (resolve image [@u]));
                        reject = (fun error -> (reject error [@u]));
                      }
                  in
                  let _ = Map.set waiters frame.url waiting in
                  ())
            in
            self#promote resource;
            self#pump;
            promise

    method retry (kind : kind option) =
      Map.each failed (fun[@u] _ key ->
          if
            Option.is_none kind
            || Option.bind (Map.get plan key) (fun resource -> Some resource.kind) = kind
          then
            let _ = Map.delete failed key in
            ());
      self#rebuild;
      self#publish_audio;
      self#pump;
      changed None

    method private cancel key file =
      let _ = Map.delete files key in
      decoded <- Array_ext.filter (fun (file : Phaser.file) -> file##key <> key) decoded;
      resources#remove {j|pending:$(key)|j};
      (match Map.get states key with
      | Some entry when (not disposed) && entry.state = Active && Map.has plan key ->
          self#enqueue entry
      | _ -> ());
      Phaser_files.cancel scene##load file

    method destroy =
      disposed <- true;
      detach ();
      decoded <- [||];
      Map.clear adoptions;
      Map.each files (fun[@u] file key -> self#cancel key file);
      Map.each waiters (fun[@u] waiting _ ->
          Array.iter
            (fun waiter -> waiter.reject (Failure {js|Artwork preparation canceled|js}))
            waiting);
      Map.clear waiters
  end
