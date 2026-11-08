open Browser_time
module Map = Host_collections.Map
module Set = Host_collections.Set

type pending = {
  source : Combat_playback.frame;
  visible : string Set.t;
  reduced : bool;
  owner : Phaser_sound.owner;
}

type event = { at : float; run : unit -> unit }

type playing = {
  pending : pending;
  mutable health : int32 array;
  mutable events : event array;
  mutable next : int;
  mutable complete_at : float;
  death_at : (string, float) Map.t;
}

type state = Idle | Waiting of pending | Playing of playing | Complete of Domain.combat_animation

let same_combat a b = a == b

class playback (audio : Phaser_sound.t) (catalog : unit -> Catalog.t)
  ~(windup : string -> float -> unit) ~(impact : Domain.combat_animation -> int -> float -> unit)
  ~(invalidate : unit -> unit) ~(fail : exn -> unit) =
  object (self)
    val mutable state = Idle

    method present (source : Combat_playback.frame option) (visible : string Set.t) reduced
        (owner : Phaser_sound.owner option) =
      match source with
      | None -> self#stop
      | Some source ->
          let retained =
            match state with
            | Playing playing -> same_combat playing.pending.source.combat source.combat
            | Complete combat -> same_combat combat source.combat
            | _ -> false
          in
          if not retained then (
            (match state with
            | Waiting old when same_combat old.source.combat source.combat -> ()
            | _ -> self#stop);
            let owner =
              match owner with
              | Some owner -> owner
              | None -> Phaser_sound.combat_owner source.combat
            in
            state <- Waiting { source; visible; reduced; owner };
            invalidate ())

    method ready =
      match state with
      | Waiting pending -> (
          let source = pending.source and combat = pending.source.combat in
          try
            if
              not
                (Array.exists
                   (fun (unit : Domain.public_unit) -> Set.has pending.visible unit.id)
                   combat.units)
            then (
              state <- Complete combat;
              source.skip ())
            else if source.complete then state <- Complete combat
            else
              let health =
                if source.shown > 0 then combat.strikes.(source.shown - 1).health
                else Array.map (fun (unit : Domain.public_unit) -> unit.hp) combat.units
              in
              let playing =
                {
                  pending;
                  health;
                  events = [||];
                  next = 0;
                  complete_at = infinity;
                  death_at = Map.create ();
                }
              in
              Array.iteri
                (fun i (unit : Domain.public_unit) ->
                  if health.(i) <= 0l then
                    let _ = Map.set playing.death_at unit.id neg_infinity in
                    ())
                combat.units;
              state <- Playing playing;
              self#schedule_strike playing source.shown
                (now () +. if pending.reduced then 0. else 120.)
          with error ->
            self#stop;
            fail error)
      | _ -> ()

    method private schedule_strike playing index at =
      let combat = playing.pending.source.combat in
      if index >= Array.length combat.strikes then playing.complete_at <- now ()
      else
        let strike = combat.strikes.(index) in
        let actor =
          match
            Array.find_opt
              (fun (unit : Domain.public_unit) -> unit.id = strike.unit_id)
              combat.units
          with
          | Some actor -> actor
          | None -> failwith ("Combat strike has no actor: " ^ strike.unit_id)
        in
        let timing =
          Phaser_sound.schedule_cue ~use_impact:(not playing.pending.reduced) ~pitch_spread:0.8
            ~owner:playing.pending.owner
            ~key:("strike:" ^ string_of_int index)
            audio (self#family combat index) at Phaser_sound.Combat
        in
        let strike_at = timing.impact_at in
        playing.events <- [||];
        playing.next <- 0;
        (if not playing.pending.reduced then
           let _ =
             Array_ext.append_value playing.events
               { at = strike_at -. 120.; run = (fun () -> windup actor.id (strike_at -. 120.)) }
           in
           ());
        let _ =
          Array_ext.append_value playing.events
            {
              at = strike_at;
              run =
                (fun () ->
                  if not (playing.pending.source.advance index) then self#stop
                  else (
                    playing.health <- strike.health;
                    impact combat index strike_at;
                    if index + 1 < Array.length combat.strikes then
                      self#schedule_strike playing (index + 1)
                        (strike_at
                        +. if playing.pending.reduced && not timing.played then 0. else 550.)
                    else
                      playing.complete_at <-
                        (strike_at +. if playing.pending.reduced then 0. else 300.)));
            }
        in
        Array.iteri
          (fun i (unit : Domain.public_unit) ->
            if strike.health.(i) <= 0l && not (Map.has playing.death_at unit.id) then
              let _ = Map.set playing.death_at unit.id strike_at in
              if Set.has playing.pending.visible unit.id then
                let _ =
                  Phaser_sound.schedule_death ~owner:playing.pending.owner ~key:unit.id audio
                    unit.type_ strike_at
                in
                ())
          combat.units;
        invalidate ()

    method health_width id =
      match state with
      | Playing playing ->
          let found = ref None in
          Array.iteri
            (fun i (unit : Domain.public_unit) ->
              if unit.id = id then
                found :=
                  Some
                    (38.
                    *. Int32.to_float playing.health.(i)
                    /. Int32.to_float (Catalog.unit (catalog ()) unit.type_).hp))
            playing.pending.source.combat.units;
          !found
      | _ -> None

    method death id = match state with Playing playing -> Map.get playing.death_at id | _ -> None

    method holds id =
      match state with
      | Playing playing ->
          Array.exists
            (fun (unit : Domain.public_unit) -> unit.id = id)
            playing.pending.source.combat.units
      | _ -> false

    method update at =
      match state with
      | Playing playing ->
          let current () = match state with Playing current -> current == playing | _ -> false in
          while
            current ()
            && playing.next < Array.length playing.events
            && playing.events.(playing.next).at <= at
          do
            let event = playing.events.(playing.next) in
            playing.next <- playing.next + 1;
            event.run ()
          done;
          if current () then (
            if at >= playing.complete_at then (
              state <- Complete playing.pending.source.combat;
              playing.pending.source.finish ());
            invalidate ())
      | _ -> ()

    method private family (combat : Domain.combat_animation) index =
      let strike = combat.strikes.(index) in
      let actor =
        match
          Array.find_opt (fun (unit : Domain.public_unit) -> unit.id = strike.unit_id) combat.units
        with
        | Some actor -> actor
        | None -> failwith ("Combat strike has no actor: " ^ strike.unit_id)
      in
      let weapon =
        match
          List.find_opt
            (fun (weapon : Domain.weapon) -> weapon.id = strike.weapon_id)
            (Presentation.weapons (catalog ()) actor)
        with
        | Some weapon -> weapon
        | None -> failwith ("Combat strike has no weapon: " ^ strike.weapon_id)
      in
      Phaser_sound.sound_for_interaction
        {
          Phaser_sound.interaction with
          attack_type = Some actor.type_;
          attack_id = Some weapon.id;
          attack_sound = Some weapon.sound;
          attack_category = Some (Enum.string_of_category weapon.category);
          attack_damage_type = Some (Enum.string_of_damage_type weapon.damage_type);
        }

    method stop = state <- Idle

    method skip =
      let pending =
        match state with
        | Waiting pending -> Some pending
        | Playing playing -> Some playing.pending
        | _ -> None
      in
      self#stop;
      Option.iter
        (fun pending ->
          state <- Complete pending.source.combat;
          pending.source.skip ())
        pending
  end
