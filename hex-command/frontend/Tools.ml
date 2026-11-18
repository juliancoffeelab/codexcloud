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

let fail message = Js.Exn.raiseError message
let obj fields = from_entries (Array.of_list fields)
let text value = Js.Json.string value
let strings values = Js.Json.array (Array.of_list (List.map text values))
let json serialized = Js.Json.parseExn serialized

let fields value =
  match Js.Json.decodeObject value with
  | Some fields -> fields
  | None -> fail {js|Expected tool inputs to be an object.|js}

let get object_ key = if has_own object_ key then Js.Dict.get object_ key else None

let field object_ key =
  match get object_ key with
  | Some value -> value
  | None -> fail {j|Missing tool result field: $(key)|j}

let input value =
  match Js.Undefined.toOption value with
  | None -> Js.Dict.empty ()
  | Some value -> (
      match Js.Json.classify value with
      | Js.Json.JSONObject fields -> fields
      | _ -> fail {js|Expected tool inputs to be an object.|js})

let string input key =
  match Option.bind (get input key) Js.Json.decodeString with
  | Some value -> value
  | None -> fail {j|Expected $(key) to be a string.|j}

let array value =
  match Js.Json.decodeArray value with
  | Some values -> values
  | None -> fail {js|Malformed native tool result.|js}

let string_property ?values ?description () =
  obj
    ([ ({js|type|js}, text {js|string|js}) ]
    @ (match values with None -> [] | Some values -> [ ({js|enum|js}, strings values) ])
    @ match description with None -> [] | Some value -> [ ({js|description|js}, text value) ])

let schema ?(required = []) properties =
  obj
    ([
       ({js|type|js}, text {js|object|js});
       ({js|properties|js}, obj properties);
       ({js|additionalProperties|js}, Js.Json.boolean false);
     ]
    @ if required = [] then [] else [ ({js|required|js}, strings required) ])

let snapshot session =
  let source = fields (json (Native.state_json (Session.view session).interaction_game)) in
  let sites terrains =
    Js.Json.array
      (array (field source {js|tiles|js})
      |> Array.to_list
      |> List.filter_map (fun tile ->
          let tile = fields tile in
          if List.exists (fun terrain -> field tile {js|terrain|js} = text terrain) terrains then
            Some
              (obj
                 [ ({js|id|js}, field tile {js|id|js}); ({js|owner|js}, field tile {js|owner|js}) ])
          else None)
      |> Array.of_list)
  in
  let campaign =
    Option.bind (get source {js|campaign|js}) (fun value ->
        if Js.Json.classify value = Js.Json.JSONNull then None else Some value)
  in
  let site_values = sites [ {js|grove|js}; {js|fort|js}; {js|ruins|js} ] in
  obj
    (List.map
       (fun key -> (key, field source key))
       [
         {js|mapId|js};
         {js|mapRevision|js};
         {js|round|js};
         {js|turn|js};
         {js|gold|js};
         {js|winner|js};
         {js|units|js};
       ]
    @ (match campaign with None -> [] | Some value -> [ ({js|campaign|js}, value) ])
    @ [ ({js|villages|js}, sites [ {js|village|js} ]) ]
    @
    if campaign <> None || Array.length (array site_values) > 0 then
      [ ({js|sites|js}, site_values) ]
    else [])

let unit_actions session id =
  let view = (Session.view session).interaction_game in
  let source =
    fields
      (json
         (match Native.actions_json view id with
         | Some actions -> actions
         | None -> fail {js|Unknown unit.|js}))
  in
  let attacks = array (field source {js|attacks|js}) in
  let target_attacks target =
    Array.to_list attacks
    |> List.filter (fun value -> field (fields value) {js|target|js} = text target)
  in
  let targets =
    Native.targets view id
    |> Array.map (fun (target : Domain.public_unit) ->
        let choices = target_attacks target.id in
        let best =
          match
            List.find_opt
              (fun value -> field (fields value) {js|recommended|js} = Js.Json.boolean true)
              choices
          with
          | Some value -> fields value
          | None -> fail {js|No recommended native attack.|js}
        in
        let weapons =
          Array.of_list choices
          |> Array.map (fun value ->
              let details = fields value in
              from_entries
                (Array.append
                   (Js.Dict.entries (fields (field details {js|attack|js})))
                   (Js.Dict.entries details)))
        in
        obj
          [
            ({js|id|js}, text target.id);
            ({js|damage|js}, field best {js|damage|js});
            ({js|retaliation|js}, field best {js|retaliation|js});
            ({js|recommendedAttackId|js}, field (fields (field best {js|attack|js})) {js|id|js});
            ({js|weapons|js}, Js.Json.array weapons);
          ])
  in
  let recruitment =
    Native.recruitment_options view {js|player|js}
    |> Array.to_list
    |> List.map (fun kind ->
        ( kind,
          strings
            (Array.to_list
               (Array.map (fun (tile : Domain.tile) -> tile.id) (Native.recruit_tiles view id kind)))
        ))
  in
  obj
    [
      ({js|promotions|js}, field source {js|promotions|js});
      ({js|moves|js}, field source {js|reachable|js});
      ({js|attacks|js}, Js.Json.array targets);
      ({js|recruitment|js}, obj recruitment);
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
    | Some registry when Js.typeof (registration registry) <> {js|function|js} -> absent
    | Some registry ->
        let active = ref true in
        let owner = ref () in
        let registry_state = registry_state registry in
        registry_state.latest <- Some owner;
        let ensure () = if not !active then fail {js|This battle session has closed.|js} in
        let commands = Session.commands session in
        let catalog = (Session.view session).catalog in
        let unit_ids =
          List.map (fun (unit : Domain.wiki_unit) -> unit.id) (Catalog.units catalog)
        in
        let recruitable = unit_ids in
        let campaign_ids =
          List.map
            (fun (campaign : Domain.browser_campaign) -> campaign.id)
            (Catalog.campaigns catalog)
        in
        let maps =
          List.map (fun (map : Domain.browser_map) -> map.id) (Catalog.scenario_maps catalog)
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
              description;
              inputSchema = input_schema;
              annotations =
                (if readonly then
                   Js.Undefined.return (obj [ ({js|readOnlyHint|js}, Js.Json.boolean true) ])
                 else Js.Undefined.empty);
              execute;
            }]
        in
        let tools =
          [|
            descriptor {js|advance_unit|js}
              {js|Advance a ready unit after it earns enough experience. Preserves wounds and consumes this turn.|js}
              (schema ~required:[ {js|unitId|js}; {js|type|js} ]
                 [
                   ({js|unitId|js}, string_property ());
                   ({js|type|js}, string_property ~values:unit_ids ());
                 ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      let kind = string input {js|type|js} in
                      member unit_ids kind {js|Unknown advancement.|js};
                      Session.advance_unit session (string input {js|unitId|js}) kind)));
            descriptor {js|start_campaign|js}
              {js|Replace the current battle with a new story campaign.|js}
              (schema ~required:[ {js|id|js} ]
                 [ ({js|id|js}, string_property ~values:campaign_ids ()) ])
              (immediate
                 (result (fun value ->
                      let id = string (input value) {js|id|js} in
                      member campaign_ids id {js|Unknown campaign.|js};
                      commands.start_campaign id)));
            descriptor {js|advance_dialogue|js}
              {js|Read the next line in an active campaign conversation.|js} (schema [])
              (immediate (fun _ ->
                   commands.dialogue false;
                   Js.Json.null));
            descriptor ~readonly:true {js|get_battle_state|js}
              {js|Read units, gold, village ownership, the current turn, and victory state.|js}
              (schema [])
              (immediate (fun _ -> snapshot session));
            descriptor ~readonly:true {js|get_unit_actions|js}
              {js|Read legal moves, attacks, and recruiting locations for a unit.|js}
              (schema ~required:[ {js|unitId|js} ] [ ({js|unitId|js}, string_property ()) ])
              (immediate (fun value -> unit_actions session (string (input value) {js|unitId|js})));
            descriptor {js|move_unit|js}
              {js|Move one of your units to a legal hex. Captures a village when entered.|js}
              (schema
                 ~required:[ {js|unitId|js}; {js|tileId|js} ]
                 [ ({js|unitId|js}, string_property ()); ({js|tileId|js}, string_property ()) ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      Session.move_unit session (string input {js|unitId|js})
                        (string input {js|tileId|js}))));
            descriptor {js|attack_unit|js}
              {js|Attack an enemy in range and resolve damage and retaliation.|js}
              (schema
                 ~required:[ {js|unitId|js}; {js|targetId|js} ]
                 [
                   ({js|unitId|js}, string_property ());
                   ({js|targetId|js}, string_property ());
                   ( {js|attackId|js},
                     string_property
                       ~description:
                         {js|Weapon id from get_unit_actions; omit to choose the safest effective attack.|js}
                       () );
                 ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      let weapon =
                        match get input {js|attackId|js} with
                        | None -> None
                        | Some _ -> Some (string input {js|attackId|js})
                      in
                      Session.attack_unit session (string input {js|unitId|js})
                        (string input {js|targetId|js}) weapon)));
            descriptor {js|recruit_unit|js}
              {js|Spend gold to recruit a troop from this mission’s pool adjacent to your leader on the keep.|js}
              (schema
                 ~required:[ {js|commanderId|js}; {js|type|js}; {js|tileId|js} ]
                 [
                   ({js|commanderId|js}, string_property ());
                   ({js|type|js}, string_property ~values:recruitable ());
                   ({js|tileId|js}, string_property ());
                 ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      let kind = string input {js|type|js} in
                      member recruitable kind {js|Unknown recruit type.|js};
                      Session.recruit_unit session
                        (string input {js|commanderId|js})
                        kind (string input {js|tileId|js}))));
            descriptor {js|undo_move|js}
              {js|Undo the latest move in the current player turn, restoring its village ownership changes. Attacking, recruiting, or ending the turn clears earlier moves.|js}
              (schema [])
              (immediate (result (fun _ -> commands.undo ())));
            descriptor {js|end_turn|js}
              {js|End your turn, let the enemy army act, and return the resulting battle state.|js}
              (schema []) (fun _ ->
                Js.Promise.then_
                  (function
                    | Error error ->
                        Js.Promise.resolve
                          (obj [ ({js|error|js}, text (Session.error_message error)) ])
                    | Ok () ->
                        ensure ();
                        Js.Promise.resolve (snapshot session))
                  (commands.end_turn ()));
            descriptor {js|start_new_battle|js}
              {js|Replace the current battle with a fresh battle.|js}
              (schema [ ({js|mapId|js}, string_property ~values:maps ()) ])
              (immediate
                 (result (fun value ->
                      let input = input value in
                      commands.reset
                        (match get input {js|mapId|js} with
                        | None -> {js|mirror|js}
                        | Some _ -> string input {js|mapId|js}))));
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
                        if Js.typeof (unregistration registry) = {js|function|js} then
                          unregister_tool registry (tool_name tool)
                        else Js.Json.null
                    | _ -> Js.Json.null))
              tools)
  with _ -> absent
