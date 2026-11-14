(* Frozen v44 -> v45: campaign opponent IDs become explicit authored identities.
   Vision groups, palettes, phase values and diplomatic selectors are unchanged. *)
open Historical_types
module V = V44
module Map = String_map

let opponent = function
  | "ashen-dawn" -> Some "masters-host"
  | "ashen-quarry" -> Some "quarry-guards"
  | "ashen-refuge" -> Some "charter-patrol"
  | "bell-night" -> Some "bell-host"
  | "tide-nets" -> Some "factors-guards"
  | "tide-sluice" -> Some "sluice-guards"
  | "tide-wreck" -> Some "wreck-guards"
  | "darkness-hunters-arena" -> Some "crypt-hunters"
  | "darkness-hidden-arena" -> Some "crypt-defenders"
  | "green-border" | "green-vigil" -> Some "charter-guards"
  | "green-pact" -> Some "rootbinders-host"
  | "lantern-bell" -> Some "bell-host"
  | "lantern-ferry" -> Some "road-skeletons"
  | "lantern-villages" -> Some "toll-skeletons"
  | "mirror-night" -> Some "night-host"
  | "forest-hunters-arena" -> Some "forest-hunters"
  | "forest-hidden-arena" -> Some "forest-defenders"
  | _ -> None

let rename_keys rename map =
  Map.fold
    (fun key value result ->
      let key = rename key in
      Migration_json.require (not (Map.mem key result)) "Conflicting migrated faction IDs";
      Map.add key value result)
    map Map.empty

let state (value : V.state) : V.state =
  match opponent value.map_id with
  | None -> value
  | Some name ->
      let rename id = if id = "enemy" then name else id in
      let owner = function `Null -> `Null | `String id -> `String (rename id) in
      let rec condition :
          save40_member0_checkpoint_state_tiles_item_fog_member1_condition ->
          save40_member0_checkpoint_state_tiles_item_fog_member1_condition = function
        | `Owner v -> `Owner { v with side = rename v.side }
        | `All v -> `All { v with conditions = Array.map condition v.conditions }
        | `Any v -> `Any { v with conditions = Array.map condition v.conditions }
        | v -> v
      in
      let rule (v : save41_member0_checkpoint_state_factions_item_transfer_member1_when_) =
        { v with condition = condition v.condition }
      in
      let light (v : V.light) = { v with when_ = Option.map rule v.when_ } in
      let unit_ (v : V.unit_) = { v with faction = rename v.faction } in
      let faction (v : V.faction) =
        {
          v with
          id = rename v.id;
          relations = Option.map (rename_keys rename) v.relations;
          transfer =
            Option.map
              (fun (t : V.transfer) -> { t with to_ = rename t.to_; when_ = rule t.when_ })
              v.transfer;
        }
      in
      let scripts (v : save41_member0_checkpoint_state_campaign_member1_scripts_member1) =
        {
          v with
          recruitment = rename_keys rename v.recruitment;
          pending =
            Array.map
              (fun (p :
                     save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item)
                 -> { p with faction = rename p.faction })
              v.pending;
          marks =
            Array.map
              (fun (m : save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item)
                 -> { m with light = Option.map light m.light })
              v.marks;
        }
      in
      let campaign (v : V.campaign) =
        {
          v with
          scripts = Option.map scripts v.scripts;
          evacuated =
            Option.map
              (Array.map
                 (fun
                   (e : save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item) ->
                   { e with unit_ = unit_ e.unit_ }))
              v.evacuated;
        }
      in
      {
        value with
        factions = Array.map faction value.factions;
        gold = rename_keys rename value.gold;
        units = Array.map unit_ value.units;
        tiles =
          Array.map
            (fun (v : V.tile) -> { v with owner = owner v.owner; light = Option.map light v.light })
            value.tiles;
        campaign = Option.map campaign value.campaign;
      }

let v44 (value : V44.t) : V45.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo
      ~encode_old:Historical_codec.encode_save44_member0_checkpoint_state
      ~decode_old:Historical_codec.read_save44_member0_checkpoint_state
      ~encode_new:Historical_codec.encode_save44_member0_checkpoint_state state checkpoint undo
  in
  match value with
  | `Current v ->
      let state, undo = convert v.checkpoint.state (Some v.undo) in
      `Current
        {
          wire = v.wire;
          pending_enemy = v.pending_enemy;
          undo = Option.get undo;
          progress = v.progress;
          version = `V45;
          checkpoint = { v.checkpoint with state };
          setup = v.setup;
        }
  | `Turn v ->
      let state, _ = convert v.checkpoint.state None in
      `Turn
        {
          wire = v.wire;
          pending_enemy = v.pending_enemy;
          undo = v.undo;
          progress = v.progress;
          version = `V45;
          checkpoint = { v.checkpoint with state };
          setup = v.setup;
        }
