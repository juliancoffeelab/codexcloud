open Browser_tools
module Weak_map = Host_collections.Weak_map

type owner = unit ref
type registration = { mutable pending : unit Js.Promise.t option; mutable owner : owner option }
type registry_state = { registrations : registration Js.Dict.t; mutable latest : owner option }

(* Private browser bookkeeping: one entry per tool name, with no inherited keys. *)
external create_registrations : unit Js.Nullable.t -> registration Js.Dict.t = "create"
[@@mel.scope "Object"]

external delete_registration : registration Js.Dict.t -> string -> unit = "deleteProperty"
[@@mel.scope "Reflect"]

external has_own : 'a -> string -> bool = "hasOwn" [@@mel.scope "Object"]

external from_entries : (string * Js.Json.t) array -> Js.Json.t = "fromEntries"
[@@mel.scope "Object"]

let fail message = Js.Exn.raiseError (Utf8.to_js message)
let obj fields = from_entries (Array.of_list fields)
let text value = Js.Json.string (Utf8.to_js value)
let strings values = Js.Json.array (Array.of_list (List.map text values))
let json serialized = Js.Json.parseExn (Utf8.to_js serialized)

let fields value =
  match Js.Json.decodeObject value with
  | Some fields -> fields
  | None -> fail "Expected tool inputs to be an object."

let get object_ key = if has_own object_ key then Js.Dict.get object_ key else None

let field object_ key =
  match get object_ key with
  | Some value -> value
  | None -> fail ("Missing tool result field: " ^ key)

let input value =
  match Js.Undefined.toOption value with
  | None -> Js.Dict.empty ()
  | Some value -> (
      match Js.Json.classify value with
      | Js.Json.JSONObject fields -> fields
      | _ -> fail "Expected tool inputs to be an object.")

let string input key =
  match Option.bind (get input key) Js.Json.decodeString with
  | Some value -> Utf8.of_js value
  | None -> fail ("Expected " ^ key ^ " to be a string.")

let array value =
  match Js.Json.decodeArray value with
  | Some values -> values
  | None -> fail "Malformed native tool result."

let string_property ?values ?description () =
  obj
    ([ ("type", text "string") ]
    @ (match values with None -> [] | Some values -> [ ("enum", strings values) ])
    @ match description with None -> [] | Some value -> [ ("description", text value) ])

let schema ?(required = []) properties =
  obj
    ([
       ("type", text "object");
       ("properties", obj properties);
       ("additionalProperties", Js.Json.boolean false);
     ]
    @ if required = [] then [] else [ ("required", strings required) ])

let snapshot session =
  let source = fields (json (Native.state_json (Session.view session).interaction_game)) in
  let sites terrains =
    Js.Json.array
      (array (field source "tiles")
      |> Array.to_list
      |> List.filter_map (fun tile ->
          let tile = fields tile in
          if List.exists (fun terrain -> field tile "terrain" = text terrain) terrains then
            Some (obj [ ("id", field tile "id"); ("owner", field tile "owner") ])
          else None)
      |> Array.of_list)
  in
  let campaign =
    Option.bind (get source "campaign") (fun value ->
        if Js.Json.classify value = Js.Json.JSONNull then None else Some value)
  in
  let site_values = sites [ "grove"; "fort"; "ruins" ] in
  obj
    (List.map
       (fun key -> (key, field source key))
       [ "mapId"; "mapRevision"; "round"; "turn"; "gold"; "winner"; "units" ]
    @ (match campaign with None -> [] | Some value -> [ ("campaign", value) ])
    @ [ ("villages", sites [ "village" ]) ]
    @
    if campaign <> None || Array.length (array site_values) > 0 then [ ("sites", site_values) ]
    else [])

let unit_actions session id =
  let view = (Session.view session).interaction_game in
  let source =
    fields
      (json
         (match Native.actions_json view id with
         | Some actions -> actions
         | None -> fail "Unknown unit."))
  in
  let attacks = array (field source "attacks") in
  let target_attacks target =
    Array.to_list attacks |> List.filter (fun value -> field (fields value) "target" = text target)
  in
  let targets =
    Native.targets view id
    |> Array.map (fun (target : Domain.public_unit) ->
        let choices = target_attacks target.id in
        let best =
          match
            List.find_opt
              (fun value -> field (fields value) "recommended" = Js.Json.boolean true)
              choices
          with
          | Some value -> fields value
          | None -> fail "No recommended native attack."
        in
        let weapons =
          Array.of_list choices
          |> Array.map (fun value ->
              let details = fields value in
              from_entries
                (Array.append
                   (Js.Dict.entries (fields (field details "attack")))
                   (Js.Dict.entries details)))
        in
        obj
          [
            ("id", text target.id);
            ("damage", field best "damage");
            ("retaliation", field best "retaliation");
            ("recommendedAttackId", field (fields (field best "attack")) "id");
            ("weapons", Js.Json.array weapons);
          ])
  in
  let recruitment =
    Native.recruitment_options view "player"
    |> Array.to_list
    |> List.map (fun kind ->
        ( Utf8.to_js kind,
          strings
            (Array.to_list
               (Array.map (fun (tile : Domain.tile) -> tile.id) (Native.recruit_tiles view id kind)))
        ))
  in
  obj
    [
      ("promotions", field source "promotions");
      ("moves", field source "reachable");
      ("attacks", Js.Json.array targets);
      ("recruitment", obj recruitment);
    ]

let registries : (registry, registry_state) Weak_map.t = Weak_map.create ()

let registry_state registry =
  match Weak_map.get registries registry with
  | Some state -> state
  | None ->
      let state = { registrations = create_registrations Js.Nullable.null; latest = None } in
      let _ = Weak_map.set registries registry state in
      state

let optional action =
  try
    Js.Promise.catch
      (fun _ -> Js.Promise.resolve ())
      (Js.Promise.then_ (fun _ -> Js.Promise.resolve ()) (Js.Promise.resolve (action ())))
  with _ -> Js.Promise.resolve ()

(* Browser registry operations may finish asynchronously. Serialize each name
   across mounts, including cleanup, so a stale completion cannot replace or
   unregister a newer owner's tool. Other names proceed independently. *)
let enqueue state name action =
  let registrations = state.registrations in
  let registration =
    match Js.Dict.get registrations name with
    | Some registration -> registration
    | None ->
        let registration = { pending = None; owner = None } in
        Js.Dict.set registrations name registration;
        registration
  in
  let previous = registration.pending in
  let finish = ref (fun () -> ()) in
  let pending =
    Js.Promise.make (fun ~resolve ~reject:_ ->
        finish :=
          fun () ->
            let completed = () in
            (resolve completed [@u]))
  in
  registration.pending <- Some pending;
  let _ =
    Js.Promise.then_
      (fun () ->
        (match registration.pending with
        | Some latest when latest == pending ->
            registration.pending <- None;
            if registration.owner = None then delete_registration registrations name
        | _ -> ());
        Js.Promise.resolve ())
      pending
  in
  let run () =
    Js.Promise.then_
      (fun () ->
        !finish ();
        Js.Promise.resolve ())
      (optional (fun () -> action registration))
  in
  let _ = match previous with None -> run () | Some previous -> Js.Promise.then_ run previous in
  ()

let register session =
  let absent () = () in
  try
    match Js.Nullable.toOption (model_context document) with
    | None -> absent
    | Some registry when Js.typeof (registration registry) <> "function" -> absent
    | Some registry ->
        let active = ref true in
        let owner = ref () in
        let registry_state = registry_state registry in
        registry_state.latest <- Some owner;
        let ensure () = if not !active then fail "This battle session has closed." in
        let commands = Session.commands session in
        let catalog = (Session.view session).catalog in
        let unit_ids =
          List.map (fun (unit : Domain.unit_rule) -> unit.id) (Catalog.units catalog)
        in
        let recruitable = unit_ids in
        let campaign_ids =
          List.map (fun (campaign : Domain.campaign) -> campaign.id) (Catalog.campaigns catalog)
        in
        let maps =
          List.map (fun (map : Domain.map_definition) -> map.id) (Catalog.scenario_maps catalog)
        in
        let member values value message = if not (List.mem value values) then fail message in
        let result action value =
          action value;
          snapshot session
        in
        let immediate action value = Js.Promise.resolve (action value) in
        let descriptor ?(readonly = false) name description input_schema action : tool =
          let execute value =
            Js.Promise.then_
              (fun () ->
                ensure ();
                action value)
              (Js.Promise.resolve ())
          in
          [%mel.obj
            {
              name;
              description = Utf8.to_js description;
              inputSchema = input_schema;
              annotations =
                (if readonly then
                   Js.Undefined.return (obj [ ("readOnlyHint", Js.Json.boolean true) ])
                 else Js.Undefined.empty);
              execute;
            }]
        in
        let tools =
          [|
            descriptor "advance_unit"
              "Advance a ready unit after it earns enough experience. Preserves wounds and \
               consumes this turn."
              (schema ~required:[ "unitId"; "type" ]
                 [ ("unitId", string_property ()); ("type", string_property ~values:unit_ids ()) ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      let kind = string input "type" in
                      member unit_ids kind "Unknown advancement.";
                      Session.advance_unit session (string input "unitId") kind)));
            descriptor "start_campaign" "Replace the current battle with a new story campaign."
              (schema ~required:[ "id" ] [ ("id", string_property ~values:campaign_ids ()) ])
              (immediate
                 (result (fun value ->
                      let id = string (input value) "id" in
                      member campaign_ids id "Unknown campaign.";
                      commands.start_campaign id)));
            descriptor "advance_dialogue" "Read the next line in an active campaign conversation."
              (schema [])
              (immediate (fun _ ->
                   commands.dialogue false;
                   Js.Json.null));
            descriptor ~readonly:true "get_battle_state"
              "Read units, gold, village ownership, the current turn, and victory state."
              (schema [])
              (immediate (fun _ -> snapshot session));
            descriptor ~readonly:true "get_unit_actions"
              "Read legal moves, attacks, and recruiting locations for a unit."
              (schema ~required:[ "unitId" ] [ ("unitId", string_property ()) ])
              (immediate (fun value -> unit_actions session (string (input value) "unitId")));
            descriptor "move_unit"
              "Move one of your units to a legal hex. Captures a village when entered."
              (schema ~required:[ "unitId"; "tileId" ]
                 [ ("unitId", string_property ()); ("tileId", string_property ()) ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      Session.move_unit session (string input "unitId") (string input "tileId"))));
            descriptor "attack_unit" "Attack an enemy in range and resolve damage and retaliation."
              (schema ~required:[ "unitId"; "targetId" ]
                 [
                   ("unitId", string_property ());
                   ("targetId", string_property ());
                   ( "attackId",
                     string_property
                       ~description:
                         "Weapon id from get_unit_actions; omit to choose the safest effective \
                          attack."
                       () );
                 ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      let weapon =
                        match get input "attackId" with
                        | None -> None
                        | Some _ -> Some (string input "attackId")
                      in
                      Session.attack_unit session (string input "unitId") (string input "targetId")
                        weapon)));
            descriptor "recruit_unit"
              "Spend gold to recruit a troop from this mission’s pool adjacent to your leader on \
               the keep."
              (schema
                 ~required:[ "commanderId"; "type"; "tileId" ]
                 [
                   ("commanderId", string_property ());
                   ("type", string_property ~values:recruitable ());
                   ("tileId", string_property ());
                 ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      let kind = string input "type" in
                      member recruitable kind "Unknown recruit type.";
                      Session.recruit_unit session (string input "commanderId") kind
                        (string input "tileId"))));
            descriptor "undo_move"
              "Undo the latest move in the current player turn, restoring its village ownership \
               changes. Attacking, recruiting, or ending the turn clears earlier moves."
              (schema [])
              (immediate (result (fun _ -> commands.undo ())));
            descriptor "end_turn"
              "End your turn, let the enemy army act, and return the resulting battle state."
              (schema []) (fun _ ->
                Js.Promise.then_
                  (function
                    | Error error ->
                        Js.Promise.resolve (obj [ ("error", text (Session.error_message error)) ])
                    | Ok () ->
                        ensure ();
                        Js.Promise.resolve (snapshot session))
                  (commands.end_turn ()));
            descriptor "start_new_battle" "Replace the current battle with a fresh battle."
              (schema [ ("mapId", string_property ~values:maps ()) ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      commands.reset
                        (match get input "mapId" with
                        | None -> "mirror"
                        | Some _ -> string input "mapId"))));
          |]
        in
        Array.iter
          (fun tool ->
            enqueue registry_state (tool_name tool) (fun registration ->
                if !active && Option.exists (fun current -> current == owner) registry_state.latest
                then (
                  registration.owner <- Some owner;
                  register_tool registry tool)
                else Js.Json.null))
          tools;
        fun () ->
          if !active then (
            active := false;
            if Option.exists (fun current -> current == owner) registry_state.latest then
              registry_state.latest <- None;
            Array.iter
              (fun tool ->
                enqueue registry_state (tool_name tool) (fun registration ->
                    match registration.owner with
                    | Some current when current == owner ->
                        registration.owner <- None;
                        if Js.typeof (unregistration registry) = "function" then
                          unregister_tool registry (tool_name tool)
                        else Js.Json.null
                    | _ -> Js.Json.null))
              tools)
  with _ -> absent
