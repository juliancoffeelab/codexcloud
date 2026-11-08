(* Localized historical-map repairs. Upstream's v5/v6 steps leave these real
   published saves at revisions rejected by current native restoration. *)
open Historical_types
module C = Historical_codec
module J = Migration_json

let invalid () = invalid_arg "Invalid historical Lantern geometry"
let require condition = if not condition then invalid ()

let append_unique values value =
  if Array.mem value values then values else Array.append values [| value |]

let coordinate (tile : V5.tile) =
  let col = int_of_float tile.col and row = int_of_float tile.row in
  (col - ((row - (row land 1)) / 2), row)

let distance a b =
  let q, r = coordinate a and oq, or_ = coordinate b in
  max (abs (q - oq)) (max (abs (r - or_)) (abs (q + r - oq - or_)))

type placed_unit = { mutable value : V5.unit_ }

let expand data (state : V5.state) : V5.state option =
  if state.map_id <> J.text data "mapId" || state.map_revision <> J.num data "fromRevision" then
    None
  else (
    require (state.cols = J.num data "fromCols" && state.rows = J.num data "fromRows");
    let costs = Lazy.force Frozen_data.history_catalog_v6 in
    let mapping = J.field data "mapping" in
    let destination id = match J.get mapping id with None -> invalid () | Some x -> J.string x in
    let old_ids = Hashtbl.create (Array.length state.tiles) in
    require (Array.length state.tiles = List.length (J.keys mapping));
    let terrain_order = Array.map J.string (J.array (J.field costs "terrainOrder")) in
    Array.iter
      (fun (tile : V5.tile) ->
        require
          (tile.id = Printf.sprintf "%.0f,%.0f" tile.col tile.row
          && not (Hashtbl.mem old_ids tile.id));
        let _ = destination tile.id in
        require
          (Array.mem tile.terrain terrain_order
          && List.mem tile.owner [ `Null; `String "player"; `String "enemy" ]);
        List.iter
          (Option.iter (fun edges ->
               require
                 (List.length (List.sort_uniq compare (Array.to_list edges)) = Array.length edges)))
          [ tile.connections; tile.dead_ends ];
        Hashtbl.add old_ids tile.id tile)
      state.tiles;
    let fields = Array.map J.string (J.array (J.field data "tileFields")) in
    let fog =
      match J.field data "fog" with
      | value when value = Js.Json.null -> None
      | value -> Some (C.Fragment.rule11 value)
    in
    let tiles =
      Array.map
        (fun source ->
          let values = J.array source in
          require (Array.length fields = Array.length values);
          let entries =
            Array.to_list (Array.mapi (fun i key -> (key, values.(i))) fields)
            |> List.filter (fun (key, value) -> value <> Js.Json.null || key = "owner")
          in
          let tile = (C.Fragment.tiles11 (J.a [ J.obj entries ])).(0) in
          { tile with owner = `Null; fog })
        (J.array (J.field data "tiles"))
    in
    let by_id = Hashtbl.create (Array.length tiles) in
    Array.iteri (fun i (tile : V5.tile) -> Hashtbl.replace by_id tile.id i) tiles;
    let tile id = match Hashtbl.find_opt by_id id with Some i -> tiles.(i) | None -> invalid () in
    Array.iter
      (fun (old : V5.tile) ->
        let id = destination old.id in
        let i = Hashtbl.find by_id id in
        tiles.(i) <- { (tiles.(i)) with owner = old.owner })
      state.tiles;
    let legal (unit : V5.unit_) (tile : V5.tile) =
      match J.get (J.field data "newUnitCosts") unit.type_ with
      | Some costs -> J.field costs tile.terrain <> Js.Json.null
      | None ->
          let terrain =
            Option.fold ~none:tile.terrain ~some:J.string
              (J.get (J.field data "costTerrain") tile.terrain)
          in
          let i =
            match Array.find_index (( = ) terrain) terrain_order with
            | Some i -> i
            | None -> invalid ()
          in
          (J.array (J.field (J.field (J.field costs "units") unit.type_) "costs")).(i)
          <> Js.Json.null
    in
    let occupied = Hashtbl.create 32 and origins = Hashtbl.create 32 in
    let roster = Array.map (fun value -> { value }) state.units in
    Array.iter
      (fun unit ->
        require (Hashtbl.mem old_ids unit.value.tile && not (Hashtbl.mem origins unit.value.tile));
        Hashtbl.add origins unit.value.tile ())
      roster;
    let place entries =
      let pending = ref [] in
      Array.iter
        (fun (unit, preferred) ->
          if legal unit.value preferred && not (Hashtbl.mem occupied preferred.id) then (
            unit.value <- { unit.value with tile = preferred.id };
            Hashtbl.add occupied preferred.id ())
          else pending := (unit, preferred) :: !pending)
        entries;
      List.iter
        (fun (unit, preferred) ->
          let best =
            Array.fold_left
              (fun best candidate ->
                if (not (legal unit.value candidate)) || Hashtbl.mem occupied candidate.id then best
                else
                  match best with
                  | None -> Some candidate
                  | Some old ->
                      if distance preferred candidate < distance preferred old then Some candidate
                      else best)
              None tiles
          in
          match best with
          | None -> invalid ()
          | Some fresh ->
              unit.value <- { unit.value with tile = fresh.id };
              Hashtbl.add occupied fresh.id ())
        (List.rev !pending)
    in
    let next_id = ref state.next_id in
    let fresh =
      if state.winner <> `Null then [||]
      else (
        require (!next_id >= 1. && floor !next_id = !next_id);
        let fresh =
          Array.map
            (fun raw ->
              let id = Printf.sprintf "u%.0f" !next_id in
              require (not (Array.exists (fun (u : V5.unit_) -> u.id = id) state.units));
              let input = J.copy raw in
              J.set input "id" (J.s id);
              let unit = C.read_save2_checkpoint_state_units_item input in
              next_id := !next_id +. 1.;
              { value = unit })
            (J.array (J.field data "newUnits"))
        in
        place (Array.map (fun unit -> (unit, tile unit.value.tile)) fresh);
        fresh)
    in
    place (Array.map (fun unit -> (unit, tile (destination unit.value.tile))) roster);
    let remap_marker marker =
      let coordinate = function
        | [ prefix; location ] when List.mem prefix [ "held"; "sealed"; "fog" ] ->
            [ prefix; destination location ]
        | [ "charge"; ritual; unit; location ] -> [ "charge"; ritual; unit; destination location ]
        | [ "ritual"; ritual; round; source ] ->
            let rewrite prefix location = prefix ^ destination location in
            let source =
              if String.starts_with ~prefix:"mark-" source then
                rewrite "mark-" (String.sub source 5 (String.length source - 5))
              else if String.starts_with ~prefix:"death-u" source then
                match String.index_from_opt source 7 '-' with
                | None -> invalid ()
                | Some split ->
                    rewrite
                      (String.sub source 0 (split + 1))
                      (String.sub source (split + 1) (String.length source - split - 1))
              else invalid ()
            in
            [ "ritual"; ritual; round; source ]
        | parts -> parts
      in
      String.concat ":" (coordinate (String.split_on_char ':' marker))
    in
    let memory =
      ref (Option.fold ~none:[||] ~some:(Array.map remap_marker) state.condition_memory)
    in
    Array.iter
      (fun (old : V5.tile) ->
        let fresh = tile (destination old.id) in
        if Option.fold ~none:false ~some:(fun (fog : V5.fog) -> fog.remember = Some `True) fresh.fog
        then memory := append_unique !memory ("fog:" ^ fresh.id))
      state.tiles;
    let campaign = match state.campaign with Some c -> c | None -> invalid () in
    let projection = J.field data "leaderProjection" in
    let campaign =
      {
        campaign with
        story_veterans =
          Option.map
            (Array.map (fun (u : V5.veteran) ->
                 if u.role = Some `Leader || u.name = Some (J.text data "leaderName") then
                   {
                     u with
                     type_ = Option.fold ~none:u.type_ ~some:J.string (J.get projection u.type_);
                   }
                 else u))
            campaign.story_veterans;
        marks = Array.map remap_marker campaign.marks;
        holds =
          Option.map
            (fun holds ->
              String_map.fold
                (fun key value out -> String_map.add (destination key) value out)
                holds String_map.empty)
            campaign.holds;
        evacuated =
          Option.map
            (Array.map (fun (e : V5.evacuated) ->
                 { e with unit_ = { e.unit_ with tile = destination e.unit_.tile } }))
            campaign.evacuated;
      }
    in
    let factions = ref (Option.value state.factions ~default:[||]) in
    Array.iter
      (fun (faction : V5.faction) ->
        if not (Array.exists (fun (old : V5.faction) -> old.id = faction.id) !factions) then
          factions := Array.append !factions [| faction |])
      (C.Fragment.factions7 (J.field data "factions"));
    Some
      {
        state with
        units = Array.map (fun u -> u.value) (Array.append roster fresh);
        next_id = !next_id;
        ground_items =
          Option.map
            (Array.map (fun (g : V5.ground_items) -> { g with tile = destination g.tile }))
            state.ground_items;
        condition_memory =
          (if Array.length !memory > 0 || state.condition_memory <> None then Some !memory else None);
        campaign = Some campaign;
        factions =
          (if Array.length !factions > 0 || state.factions <> None then Some !factions else None);
        tiles;
        cols = J.num data "cols";
        rows = J.num data "rows";
        map_revision = J.num data "toRevision";
      })

let mark (campaign : V5.campaign) value =
  { campaign with marks = append_unique campaign.marks value }

let marked (campaign : V5.campaign) value = Array.mem value campaign.marks

let ferry (state : V5.state) : V5.state =
  match expand (Lazy.force Frozen_data.history_lantern_ferry_v5) state with
  | None -> state
  | Some state ->
      let campaign = ref (Option.get state.campaign) in
      let add value = campaign := mark !campaign value in
      if marked !campaign "event:ferryman" then add "event:house-visit";
      if state.winner = `Player then
        List.iter add
          [ "event:house-visit"; "event:village-1"; "event:village-2"; "event:villages-secured" ];
      Array.iter
        (fun (id, event) ->
          if
            Array.exists
              (fun (tile : V5.tile) -> tile.id = id && tile.owner = `String "player")
              state.tiles
          then (
            add ("event:" ^ event);
            List.iter
              (fun member -> add ("spawn:" ^ event ^ ":" ^ member))
              [ "member-1"; "member-2" ]))
        [| ("21,5", "village-1"); ("21,12", "village-2") |];
      { state with campaign = Some !campaign }

let villages ~preplaced_sella (state : V6.state) : V6.state =
  match expand (Lazy.force Frozen_data.history_lantern_villages_v6) state with
  | None -> state
  | Some state ->
      let campaign = ref (Option.get state.campaign) in
      let add value = campaign := mark !campaign value in
      let named (u : V6.unit_) = u.name = Some "Warden Sella" in
      let known =
        preplaced_sella || Array.exists named state.units
        || marked !campaign "fallen:Warden Sella"
        || marked !campaign "spawn:village-contact:sella"
        || Option.fold ~none:false ~some:(Array.mem "Warden Sella") !campaign.fallen_companions
        || Option.fold ~none:false
             ~some:(Array.exists (fun (e : V6.evacuated) -> named e.unit_))
             !campaign.evacuated
        || state.winner = `Player
      in
      if known then (
        List.iter add [ "event:village-contact"; "spawn:village-contact:sella" ];
        campaign :=
          {
            !campaign with
            choices =
              Some
                (append_unique
                   (Option.value !campaign.choices ~default:[||])
                   "hidden-village-found");
          });
      if state.winner = `Player then
        List.iter add
          [
            "event:foothold-established";
            "event:prisoners-found";
            "event:prisoners-freed";
            "event:village-safe";
            "event:eastern-departure";
          ];
      if
        not
          (Array.exists
             (fun (u : V6.unit_) -> u.name = Some "The Tollkeeper" && u.side = `Enemy)
             state.units)
      then (
        add "event:tollkeeper-fallen";
        campaign :=
          {
            !campaign with
            choices =
              Some
                (append_unique (Option.value !campaign.choices ~default:[||]) "tollkeeper-defeated");
          });
      { state with campaign = Some !campaign }
