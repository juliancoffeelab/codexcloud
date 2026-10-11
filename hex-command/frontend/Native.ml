type bridge
type 'a response
external initialize : unit -> unit Js.Promise.t = "default" [@@mel.module "@wasm/hex_command"]
external construct : unit -> bridge = "NativeEngineFactory" [@@mel.new] [@@mel.module "@wasm/hex_command"]
external raw_catalog : bridge -> Domain.catalog_view response = "catalog" [@@mel.send]
external raw_create : bridge -> Domain.start -> float -> Domain.battle_handle response = "create" [@@mel.send]
external raw_restore : bridge -> Domain.saved_game -> Domain.battle_handle response = "restore" [@@mel.send]
external raw_inspect : bridge -> float -> float -> Domain.battle_inspection response = "inspect" [@@mel.send]
external raw_dispatch : bridge -> float -> float -> Domain.command -> Domain.dispatch_result response = "dispatch" [@@mel.send]
external raw_ai : bridge -> float -> float -> Domain.ai_options -> Domain.ai_decision response = "chooseAiCommand" [@@mel.send]
external raw_save : bridge -> float -> float -> Domain.saved_game response = "save" [@@mel.send]
external raw_dispose : bridge -> float -> unit response = "disposeBattle" [@@mel.send]
external raw_free : bridge -> unit = "free" [@@mel.send]
external succeeded : 'a response -> bool = "ok" [@@mel.get]
external response_value : 'a response -> 'a = "value" [@@mel.get]
external response_error : 'a response -> Js.Json.t = "error" [@@mel.get]
external error_message : Js.Json.t -> string Js.undefined = "message" [@@mel.get]
external rule_error : Js.Json.t -> Js.Json.t Js.undefined = "error" [@@mel.get]
external error_code : Js.Json.t -> string = "code" [@@mel.get]
external freeze : 'a -> 'a = "freeze" [@@mel.scope "Object"]
let own = Frozen.own
external console_error : exn -> unit = "error" [@@mel.scope "console"]
let failure_reason error = match Js.Exn.asJsExn error with
  | Some error -> Option.value ~default:"Native publication failed." (Js.Exn.message error)
  | None -> match error with Failure reason | Invalid_argument reason -> reason | _ -> "Native publication failed."

let value response =
  if succeeded response then response_value response else
    let error = response_error response in
    Js.Exn.raiseError (match Js.Undefined.toOption (error_message error) with
        | Some message -> message
        | None -> (match Js.Undefined.toOption (rule_error error) with
            | Some rule -> (match Js.Json.decodeString rule with Some message -> message | None -> Js.Json.stringify rule)
            | None -> error_code error))

let index key values =
  let result = Js.Dict.empty () in
  Array.iter (fun entry -> Js.Dict.set result (key entry) entry) values;
  result

type view = {
  revision : float;
  state : Domain.state;
  inspection : Domain.battle_inspection;
  tiles : Domain.tile Js.Dict.t;
  units : Domain.public_unit Js.Dict.t;
  occupancy : Domain.public_unit Js.Dict.t;
  choices : Domain.unit_actions Js.Dict.t;
}
and factory = {
  bridge : bridge;
  catalog : Catalog.t;
  mutable disposed : bool;
  mutable battles : battle list;
}
and battle = {
  factory : factory;
  mutable id : float;
  mutable current : view;
  mutable released : bool;
  mutable failed : (string -> unit) option;
}

let publication bridge id (source : Domain.engine_view) =
  let state = own ((source.Domain.state)) in
  let inspection = own (value (raw_inspect bridge id ((source.Domain.revision)))) in
  if (source.Domain.revision) <> (inspection.Domain.revision) then
    Js.Exn.raiseError "Incoherent native publication";
  freeze { revision = source.Domain.revision; state; inspection;
           tiles = freeze (index (fun (tile : Domain.tile) -> tile.id) state.Domain.tiles);
           units = freeze (index (fun (unit : Domain.public_unit) -> unit.id) state.Domain.units);
           occupancy = freeze (index (fun (unit : Domain.public_unit) -> unit.tile) state.Domain.units);
           choices = freeze (index (fun (actions : Domain.unit_actions) -> actions.unit) inspection.Domain.units) }

let ensure_factory factory = if factory.disposed then Js.Exn.raiseError "disposed"
let ensure battle =
  ensure_factory battle.factory;
  if battle.released then Js.Exn.raiseError "disposed"
let catalog factory = ensure_factory factory; factory.catalog
let create_factory () =
  Js.Promise.then_ (fun () ->
      let bridge = construct () in
      try
        let catalog = Catalog.of_view (value (raw_catalog bridge)) in
        Js.Promise.resolve {bridge; catalog; disposed = false; battles = []}
      with error ->
        (try raw_free bridge with _ -> ());
        Fault.raise_original error) (initialize ())

let accept factory (handle : Domain.battle_handle) =
  let id = (handle.Domain.id) in
  try
    let current = publication factory.bridge id ((handle.Domain.view)) in
    let battle = {factory; id; current; released = false; failed = None} in
    factory.battles <- battle :: factory.battles;
    battle
  with error ->
    (try ignore (raw_dispose factory.bridge id) with _ -> ());
    Fault.raise_original error

external is_integer : float -> bool = "isInteger" [@@mel.scope "Number"]
let valid_seed seed = is_integer seed && seed >= 0. && seed <= 4294967295.
let create factory start seed =
  ensure_factory factory;
  if not (valid_seed seed) then Js.Exn.raiseError "seed must be a uint32";
  accept factory (value (raw_create factory.bridge start seed))
let restore factory (saved : Domain.saved_game) =
  ensure_factory factory;
  if not (valid_seed ((saved.Domain.seed))) then Js.Exn.raiseError "saved seed must be a uint32";
  accept factory (value (raw_restore factory.bridge saved))

let dispose battle =
  if not battle.released then (
    battle.released <- true;
    battle.failed <- None;
    battle.factory.battles <- List.filter (fun current -> current != battle) battle.factory.battles;
    if not battle.factory.disposed then ignore (value (raw_dispose battle.factory.bridge battle.id)))

let dispose_factory factory =
  if not factory.disposed then (
    let error = ref None in
    List.iter (fun battle -> try dispose battle with failure ->
        if !error = None then error := Some failure) factory.battles;
    factory.disposed <- true;
    (try raw_free factory.bridge with failure -> if !error = None then error := Some failure);
    match !error with None -> () | Some failure -> Fault.raise_original failure)

let view battle = ensure battle; battle.current
let on_failure battle receive = ensure battle; battle.failed <- Some receive
let state view = view.state
let inspection view = view.inspection
let revision view = view.revision

let apply battle expected command =
  ensure battle;
  let result = value (raw_dispatch battle.factory.bridge battle.id expected command) in
  (* An accepted mutation must never leave a live handle with an old revision
     if its inspection cannot be published. The old detached view stays valid. *)
  (try battle.current <- publication battle.factory.bridge battle.id ((result.Domain.view))
   with error ->
     let receive = battle.failed in
     (try dispose battle with _ -> ());
     Option.iter (fun receive -> try receive (failure_reason error) with failure -> console_error failure) receive;
     Fault.raise_original error);
  own ((result.Domain.effects))
let dispatch battle command = apply battle battle.current.revision command
let enemy_step battle options =
  ensure battle;
  let decision = value (raw_ai battle.factory.bridge battle.id battle.current.revision options) in
  apply battle ((decision.Domain.revision)) ((decision.Domain.command))
let save battle =
  ensure battle;
  own (value (raw_save battle.factory.bridge battle.id battle.current.revision))
let replace battle saved =
  ensure battle;
  let replacement = restore battle.factory saved in
  let previous = battle.id in
  battle.id <- replacement.id;
  battle.current <- replacement.current;
  replacement.released <- true;
  battle.factory.battles <- List.filter (fun current -> current != replacement) battle.factory.battles;
  ignore (value (raw_dispose battle.factory.bridge previous))

let tile view id = Js.Dict.get view.tiles id
let unit view id = Js.Dict.get view.units id
let at view id = Js.Dict.get view.occupancy id
let actions view id = Js.Dict.get view.choices id
let lookup dict id = match Js.Dict.get dict id with Some values -> values | None -> [||]
let tiles_for view ids = freeze (Array.of_list (Array.to_list ids |> List.filter_map (tile view)))
let neighbors view id = tiles_for view (lookup ((view.inspection.Domain.neighbors)) id)
let reachable view id =
  let result = Js.Dict.empty () in
  (match actions view id with None -> () | Some choices ->
      Array.iter (fun (entry : Domain.unit_actions_reachable_item) -> Js.Dict.set result entry.tile entry.cost)
        ((choices.Domain.reachable)));
  freeze result
let attack_options view id target = match actions view id with
  | None -> [||]
  | Some choices -> freeze (Array.of_list (Array.to_list choices.Domain.attacks
                                           |> List.filter (fun (attack : Domain.attack_details) -> attack.target = target)))
let targets view id = match unit view id with
  | None -> [||]
  | Some actor when (actor.Domain.attacked) || Js.Nullable.toOption ((view.state.Domain.winner)) <> None -> [||]
  | Some _ ->
    let ids = Js.Dict.empty () in
    (match actions view id with None -> () | Some choices ->
        Array.iter (fun (attack : Domain.attack_details) -> Js.Dict.set ids ((attack.Domain.target)) true) ((choices.Domain.attacks)));
    freeze (Array.of_list (Array.to_list view.state.Domain.units
                           |> List.filter (fun (candidate : Domain.public_unit) -> Js.Dict.get ids candidate.id <> None)))
let best_attack view id target = match unit view id, unit view target with
  | Some actor, Some defender ->
    let score (attack : Domain.attack_details) = (attack.Domain.damage)
                                                 +. (if (attack.Domain.damage) >= (defender.Domain.hp) then 18. else 0.)
                                                 -. (attack.Domain.retaliation) *. 0.7
                                                 -. (if (attack.Domain.retaliation) >= (actor.Domain.hp) then 40. else 0.) in
    Array.fold_left (fun best attack -> match best with
        | None -> Some attack
        | Some previous -> if score attack > score previous then Some attack else best)
      None (attack_options view id target)
  | _ -> None
let attack_details view id target weapon = match actions view id with
  | None -> None
  | Some choices -> Array.find_opt (fun (attack : Domain.attack_details) ->
      attack.target = target && attack.attack.id = weapon) choices.Domain.attacks
let promotion_options view id = match actions view id with
  | None -> [||] | Some choices -> (choices.Domain.promotions)
let recruitment_options view (side : Domain.side) = lookup view.inspection.recruitment (side :> string)
let recruit_tiles view id kind = match actions view id with
  | None -> [||]
  | Some choices -> tiles_for view (lookup ((choices.Domain.recruitment)) kind)
let movement_cost view kind terrain =
  match Js.Dict.get ((view.inspection.Domain.costs)) kind with
  | None -> infinity
  | Some costs -> (match Js.Dict.get costs terrain with
      | None -> infinity
      | Some cost -> (match Js.Nullable.toOption cost with None -> infinity | Some value -> value))
let income view side = if side = Domain.side_player then view.inspection.income.player
  else view.inspection.income.enemy
let unit_id id =
  let length = String.length id in
  let unknown () = Js.Exn.raiseError "Unknown unit." in
  if length < 2 || id.[0] <> 'u' || id.[1] < '1' || id.[1] > '9' then unknown ();
  for index = 2 to length - 1 do
    if id.[index] < '0' || id.[index] > '9' then unknown ()
  done;
  let number = float_of_string (String.sub id 1 (length - 1)) in
  if number > 4294967295. then unknown ();
  number
