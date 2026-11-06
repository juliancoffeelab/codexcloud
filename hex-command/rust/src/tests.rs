//! AI integration against validated mechanical snapshots; engine-only tests live in battle.
use crate::{data::*, logic::Game};
fn ability_rules() -> Rules {
    let mut rules = Rules::load().unwrap();
    for id in ["soldier", "archer", "scout"] {
        let kind = rules.types[id];
        let rule = &mut rules.units[kind];
        rule.hp = 100;
        rule.environment.fill([[0; 3]; 2]);
        rule.damage_modifiers = DamageModifiers {
            blade: 0,
            pierce: 0,
            impact: 0,
        };
        rule.thorns = 0;
        rule.thorn_aura = 0;
        for attack in &mut rule.attacks {
            attack.spread = 0;
            attack.damage = 4;
            attack.count = 2;
            attack.drain = 0;
        }
    }
    rules
}

#[test]
fn uncertain_kills_allow_actual_replies_and_ai_does_not_peek_at_combat_seed() {
    let mut rules = ability_rules();
    let kind = rules.types["archer"];
    rules.units[kind].attacks[0].damage = 4;
    rules.units[kind].attacks[0].spread = 4;
    rules.units[kind].attacks[0].count = 1;
    let mut outcomes = [false; 2];
    let mut decision = None;
    for seed in 0..32 {
        let mut state = Game::new(&rules, "mirror").state();
        state.units.clear();
        state.combat_rng = seed;
        state.gold = Gold::two(0, 0);
        let mut add = |side, tile: &str, hp| {
            let id = state.next_id;
            state.next_id += 1;
            state.units.push(PublicUnit {
                faction: None,
                items: vec![],
                ai_group: None,
                id: format!("u{id}"),
                r#type: "archer".into(),
                side,
                tile: tile.into(),
                hp,
                moved: false,
                attacked: false,
                experience: Some(0),
                character: None,
                role: None,
                retreating: None,
                rebuilding: None,
            });
            id
        };
        let a = add(Side::Player, "hex-0408-edbae05560", rules.units[kind].hp);
        let b = add(Side::Enemy, "hex-0760-150b2b6a75", 4);
        let mut game = Game::try_from_state(&rules, state).unwrap();
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
        game.start_recording();
        game.execute(Command::Attack {
            unit: UnitId(a),
            target: UnitId(b),
            weapon: "bow".into(),
        })
        .unwrap();
        let records = game.take_recordings().unwrap();
        let record = &records.combats[0];
        if game.unit(b).is_none() {
            outcomes[0] = true;
            assert!(record.response_rolls.is_empty());
        } else {
            outcomes[1] = true;
            assert_eq!(record.response_rolls.len(), 1);
        }
    }
    assert_eq!(outcomes, [true, true]);
}

#[test]
fn best_attack_avoids_lethal_reply_and_melee_cannot_repeat() {
    let mut rules = Rules::load().unwrap();
    for unit in &mut rules.units {
        for attack in &mut unit.attacks {
            attack.spread = 0;
        }
    }
    let mut state = Game::new(&rules, "mirror").state();
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
    let mut add = |kind: &str, side, tile: &str, hp| {
        let id = state.next_id;
        state.next_id += 1;
        state.units.push(PublicUnit {
            faction: None,
            items: vec![],
            ai_group: None,
            id: format!("u{id}"),
            r#type: kind.into(),
            side,
            tile: tile.into(),
            hp,
            moved: false,
            attacked: false,
            experience: Some(0),
            character: None,
            role: None,
            retreating: None,
            rebuilding: None,
        });
        id
    };
    let scout = add("scout", Side::Player, "hex-0408-edbae05560", 5);
    let bow = add(
        "archer",
        Side::Enemy,
        "hex-0760-150b2b6a75",
        rules.units[rules.types["archer"]].hp,
    );
    let mut game = Game::try_from_state(&rules, state).unwrap();
    let actor = game.unit(scout).unwrap();
    let target = game.unit(bow).unwrap();
    let knife = rules.units[actor.kind]
        .attacks
        .iter()
        .position(|weapon| weapon.id == "knife")
        .unwrap();
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
pub(crate) fn edit(game: &mut Game<'_>, change: impl FnOnce(&mut State)) {
    let pools = game.pools().cloned();
    let first = game.arena_first();
    let mut state = game.state();
    change(&mut state);
    *game = Game::from_state(game.catalogue(), state);
    game.configure_arena(pools, first);
}
