//! AI integration against validated mechanical snapshots; engine-only tests live in battle.
use crate::{data::*, logic::BattleSession};
fn ability_rules() -> JsonBundle {
    crate::catalogue_fixture::neutral_combat()
}

#[test]
fn uncertain_kills_allow_actual_replies_and_ai_does_not_peek_at_combat_seed() {
    let mut rules = ability_rules();
    crate::catalogue_fixture::unit(&mut rules, "archer", |unit| {
        unit["attacks"][0]["damage"] = serde_json::json!(4);
        unit["attacks"][0]["spread"] = serde_json::json!(4);
        unit["attacks"][0]["count"] = serde_json::json!(1);
    });
    let rules = Catalogue::from_json_bundle(&rules).unwrap();
    let mut outcomes = [false; 2];
    let mut decision = None;
    for seed in 0..32 {
        let mut state = BattleSession::new(&rules, "mirror").state();
        state.units.clear();
        state.combat_rng = seed;
        state.gold = Gold::two(0, 0);
        let mut add = |faction: &str, tile: Hex, hp| {
            let id = state.next_id;
            state.next_id += 1;
            state.units.push(PublicUnit {
                faction: faction.into(),
                revealed_to: vec![],
                perception_effects: vec![],
                items: vec![],
                ai_group: None,
                id: format!("u{id}"),
                r#type: "archer".into(),
                tile: tile.into(),
                hp,
                moved: false,
                attack_spent: false,
                experience: Some(0),
                character: None,
                role: None,
                retreating: None,
                rebuilding: None,
            });
            id
        };
        let a = add("player", Hex::new(2, 3), rules.unit("archer").max_hp);
        let b = add("enemy", Hex::new(3, 3), 4);
        let mut game = BattleSession::try_from_state(&rules, state).unwrap();
        let actor = game.unit(a).unwrap();
        let target = game.unit(b).unwrap();
        assert!((game.exchange(actor, target, 0, actor.tile).death[1] - 5. / 9.).abs() < 1e-12);
        assert!(game.preview(actor, target, 0, actor.tile).2.is_some());
        let mut workspace =
            crate::ai::PlanningWorkspace::new(7, 0, None, crate::ai::Diplomacy::Unrestricted);
        let command =
            crate::ai::choose_game_command(&game, crate::ai::Policy::Balanced, &mut workspace)
                .unwrap();
        assert_eq!(decision.get_or_insert(command.clone()), &command);
        #[cfg(feature = "simulation")]
        game.start_recording();
        game.execute(Command::Attack {
            unit: UnitId(a),
            target: UnitId(b),
            weapon: "bow".into(),
        })
        .unwrap();
        #[cfg(feature = "simulation")]
        let records = game.take_recordings().unwrap();
        #[cfg(feature = "simulation")]
        let record = &records.combats[0];
        if game.unit(b).is_none() {
            outcomes[0] = true;
            #[cfg(feature = "simulation")]
            assert!(record.response_rolls.is_empty());
        } else {
            outcomes[1] = true;
            #[cfg(feature = "simulation")]
            assert_eq!(record.response_rolls.len(), 1);
        }
    }
    assert_eq!(outcomes, [true, true]);
}

#[test]
fn best_attack_avoids_lethal_reply_and_melee_cannot_repeat() {
    let rules = Catalogue::from_json_bundle(&crate::catalogue_fixture::deterministic()).unwrap();
    let mut state = BattleSession::new(&rules, "mirror").state();
    state.units.clear();
    for tile in &mut state.tiles {
        if tile.terrain != "plains" {
            tile.connections = None;
            tile.walls = None;
            tile.dead_ends = None;
        }
        tile.terrain = "plains".into();
        tile.owner = None;
    }
    let mut add = |kind: &str, faction: &str, tile: Hex, hp| {
        let id = state.next_id;
        state.next_id += 1;
        state.units.push(PublicUnit {
            faction: faction.into(),
            revealed_to: vec![],
            perception_effects: vec![],
            items: vec![],
            ai_group: None,
            id: format!("u{id}"),
            r#type: kind.into(),
            tile: tile.into(),
            hp,
            moved: false,
            attack_spent: false,
            experience: Some(0),
            character: None,
            role: None,
            retreating: None,
            rebuilding: None,
        });
        id
    };
    let scout = add("scout", "player", Hex::new(2, 3), 5);
    let bow = add(
        "archer",
        "enemy",
        Hex::new(3, 3),
        rules.unit("archer").max_hp,
    );
    let mut game = BattleSession::try_from_state(&rules, state).unwrap();
    let actor = game.unit(scout).unwrap();
    let target = game.unit(bow).unwrap();
    let knife = game.with_capabilities(actor, actor.tile, |caps| {
        caps.attacks()
            .position(|attack| attack.id() == "knife")
            .unwrap()
    });
    assert_eq!(
        crate::ai::policies::combat::best_attack(game.view(), actor, target, actor.tile),
        Some(knife)
    );
    game.execute(Command::Attack {
        unit: UnitId(scout),
        target: UnitId(bow),
        weapon: "knife".into(),
    })
    .unwrap();
    assert_eq!(game.last_combat().unwrap().sound, "attack-melee");
    assert!(
        game.execute(Command::Attack {
            unit: UnitId(scout),
            target: UnitId(bow),
            weapon: "sling".into()
        })
        .is_err()
    );
}

/// Restore an edited checkpoint for integration fixtures without exposing engine storage.
pub(crate) fn edit(game: &mut BattleSession<'_>, change: impl FnOnce(&mut State)) {
    let pools = game.pools().cloned();
    let first = game.arena_first();
    let mut state = game.state();
    change(&mut state);
    *game = game.restored(state).unwrap();
    game.configure_arena(pools, first);
}
