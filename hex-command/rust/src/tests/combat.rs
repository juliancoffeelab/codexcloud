//! Native weapon legality, damage equations and retaliation behavior.
use super::*;

#[test]
fn zero_damage_attacks_consume_actions_without_xp_and_keep_retaliation() {
    let mut game = flat();
    let attacker = add(&mut game, "bonearcher", Side::Player, "2,3");
    let target = add(&mut game, "soldier", Side::Enemy, "3,3");
    game.set_terrain(game.site("3,3"), game.rules.terrains["hill"]);
    let hp = game.unit(target).unwrap().hp;
    let result = hit(&mut game, attacker, target, "shard");
    assert_eq!(result.damage, Some(0));
    assert!(result.retaliation.unwrap() > 0);
    assert_eq!(game.unit(target).unwrap().hp, hp);
    let unit = game.unit(attacker).unwrap();
    assert!(unit.moved);
    assert_eq!(unit.remaining_attacks[weapon(&game, attacker, "shard")], 0);
    assert_eq!(unit.experience, Some(0));
    assert_eq!(
        game.execute(Command::Attack {
            unit: UnitId(attacker),
            target: UnitId(target),
            weapon: "shard".into()
        })
        .unwrap_err(),
        CommandError::IllegalAttack
    );
}

#[test]
fn kills_suppress_retaliation_and_leader_death_finishes_battle() {
    for leader in [false, true] {
        let mut game = flat();
        let attacker = add(&mut game, "soldier", Side::Player, "2,3");
        let target = add(
            &mut game,
            if leader { "commander" } else { "scout" },
            Side::Enemy,
            "3,3",
        );
        let index = game.index(target).unwrap();
        game.units[index].hp = 1;
        let hp = game.unit(attacker).unwrap().hp;
        assert_eq!(
            hit(&mut game, attacker, target, "sword").retaliation,
            Some(0)
        );
        assert!(game.unit(target).is_none());
        assert_eq!(game.unit(attacker).unwrap().hp, hp);
        assert_eq!(game.winner, leader.then_some(Side::Player));
        if leader {
            assert_eq!(
                game.execute(Command::EndTurn).unwrap_err(),
                CommandError::BattleFinished
            );
        }
    }
}

#[test]
fn ranged_categories_and_intervening_cover_determine_legal_replies() {
    let mut game = flat();
    let bow = add(&mut game, "archer", Side::Player, "2,3");
    let target = add(&mut game, "archer", Side::Enemy, "4,3");
    let archer = game.unit(bow).unwrap().clone();
    let defender = game.unit(target).unwrap().clone();
    let attack = weapon(&game, bow, "bow");
    let (damage, reply, response) = game.preview(&archer, &defender, attack, archer.tile);
    assert_eq!(damage, reply);
    assert_eq!(response, Some(attack));
    assert!(!game.can_attack(&archer, archer.tile, &defender, weapon(&game, bow, "knife")));
    for terrain in ["forest", "hill"] {
        let blockers: Vec<_> = game.board.adjacent[archer.tile]
            .iter()
            .copied()
            .filter(|&t| game.board.distance(t, defender.tile) == 1)
            .collect();
        for &tile in &blockers {
            game.set_terrain(tile, game.rules.terrains[terrain]);
        }
        assert!(!game.can_attack(&archer, archer.tile, &defender, attack));
        game.set_terrain(blockers[0], game.rules.terrains["plains"]);
        assert!(game.can_attack(&archer, archer.tile, &defender, attack));
    }
}

#[test]
fn best_attack_avoids_lethal_reply_and_exhausted_types_cannot_repeat() {
    let mut game = flat();
    let scout = add(&mut game, "scout", Side::Player, "2,3");
    let bow = add(&mut game, "archer", Side::Enemy, "3,3");
    let index = game.index(scout).unwrap();
    game.units[index].hp = 5;
    let a = game.unit(scout).unwrap();
    let b = game.unit(bow).unwrap();
    assert_eq!(
        game.best_attack(a, b, a.tile),
        Some(weapon(&game, scout, "knife"))
    );
    let uses = game.unit(scout).unwrap().remaining_attacks[weapon(&game, scout, "knife")];
    game.units[index].hp = 100;
    let b = game.index(bow).unwrap();
    game.units[b].hp = 100;
    for _ in 0..uses {
        hit(&mut game, scout, bow, "knife");
    }
    assert_eq!(game.last_combat.as_ref().unwrap().sound, "attack-melee");
    assert!(
        game.execute(Command::Attack {
            unit: UnitId(scout),
            target: UnitId(bow),
            weapon: "knife".into()
        })
        .is_err()
    );
}

#[test]
fn cover_and_exposure_apply_to_attacks_and_retaliation_and_clamp_at_zero() {
    for terrain in ["plains", "forest", "hill", "ford", "marsh"] {
        let mut game = flat();
        let a = add(&mut game, "soldier", Side::Player, "2,3");
        let b = add(&mut game, "soldier", Side::Enemy, "3,3");
        game.set_terrain(game.site("2,3"), game.rules.terrains[terrain]);
        let actor = game.unit(a).unwrap();
        let defender = game.unit(b).unwrap();
        let attack = weapon(&game, a, "sword");
        let (_, reply, _) = game.preview(actor, defender, attack, actor.tile);
        let rule = &game.rules.units[actor.kind];
        let base = rule
            .attacks
            .iter()
            .map(|a| a.damage + rule.damage_modifiers.get(a.damage_type))
            .max()
            .unwrap();
        let expected = base
            + match terrain {
                "hill" => -1,
                "ford" | "marsh" => 1,
                _ => 0,
            };
        assert_eq!(reply, expected);
    }
    let mut game = flat();
    let a = add(&mut game, "bonearcher", Side::Player, "2,3");
    let b = add(&mut game, "soldier", Side::Enemy, "3,3");
    game.set_terrain(game.site("3,3"), game.rules.terrains["hill"]);
    assert_eq!(
        game.damage(
            game.unit(a).unwrap(),
            game.unit(b).unwrap(),
            weapon(&game, a, "shard")
        ),
        0
    );
}

// A synthetic pair tests mechanics without freezing authored unit statistics.
#[test]
fn attack_type_budgets_apply_flat_armor_per_hit_and_survive_restore() {
    let mut rules = Rules::load().unwrap();
    let kind = rules.types["soldier"];
    let rule = &mut rules.units[kind];
    rule.hp = 100;
    rule.level = 1;
    rule.experience_needed = Some(100);
    rule.damage_modifiers = DamageModifiers {
        blade: -1,
        pierce: -1,
        impact: -1,
    };
    rule.environment.fill([[0; 3]; 2]);
    rule.attacks[0].count = 3;
    rule.attacks[0].damage = 2;
    rule.attacks[1].count = 1;
    rule.attacks[1].damage = 6;
    let mut game = Game::new(&rules, "mirror");
    game.units.clear();
    game.roster_changed();
    let a = add(&mut game, "soldier", Side::Player, "2,3");
    let b = add(&mut game, "soldier", Side::Enemy, "3,3");
    game.recordings = Some(Records::default());
    // One heavy reply per strike, free even after the defender's own budget is spent.
    let bi = game.index(b).unwrap();
    game.units[bi].attacked = true;
    game.units[bi].remaining_attacks.fill(0);
    for _ in 0..3 {
        let actor = game.unit(a).unwrap();
        let defender = game.unit(b).unwrap();
        assert_eq!(
            game.preview(actor, defender, 0, actor.tile),
            (1, 5, Some(1))
        );
        assert_eq!(hit(&mut game, a, b, "sword").damage, Some(1));
    }
    assert_eq!(game.unit(b).unwrap().hp, 97);
    assert_eq!(game.unit(a).unwrap().hp, 85);
    assert_eq!(game.unit(a).unwrap().remaining_attacks, [0, 1]);
    assert_eq!(game.unit(a).unwrap().experience, Some(3));
    assert!(game.reachable(game.unit(a).unwrap()).is_empty());
    assert!(hit_result(&mut game, a, b, "sword").is_err());
    let saved = game.state();
    let mut restored = Game::try_from_state(&rules, saved.clone()).unwrap();
    assert_eq!(hit(&mut restored, a, b, "mace").damage, Some(5));
    assert_eq!(restored.unit(b).unwrap().hp, 92);
    assert!(restored.unit(a).unwrap().attacked);
    assert_eq!(
        game.recordings.as_ref().unwrap().combats[2].remaining_attacks["sword"],
        0
    );
    let mut invalid = saved.clone();
    invalid.units[0]
        .remaining_attacks
        .as_mut()
        .unwrap()
        .insert("sword".into(), 4);
    assert!(Game::try_from_state(&rules, invalid).is_err());
    let mut legacy = saved;
    for unit in &mut legacy.units {
        unit.remaining_attacks = None;
    }
    let legacy = Game::try_from_state(&rules, legacy).unwrap();
    assert_eq!(legacy.unit(a).unwrap().remaining_attacks, [3, 1]);
    assert_eq!(legacy.unit(b).unwrap().remaining_attacks, [0, 0]);
    restored.begin_turn(Side::Player);
    assert_eq!(restored.unit(a).unwrap().remaining_attacks, [3, 1]);
    let bi = restored.index(b).unwrap();
    restored.units[bi].hp = 1;
    assert_eq!(hit(&mut restored, a, b, "sword").retaliation, Some(0));
    assert!(restored.unit(b).is_none());
    assert_eq!(restored.unit(a).unwrap().experience, Some(10));
    let c = add(&mut restored, "soldier", Side::Enemy, "3,3");
    let ai = restored.index(a).unwrap();
    restored.units[ai].hp = 1;
    assert_eq!(hit(&mut restored, a, c, "sword").retaliation, Some(5));
    assert!(restored.unit(a).is_none());
    assert_eq!(restored.unit(c).unwrap().experience, Some(6));
}
fn hit_result(game: &mut Game<'_>, a: u32, b: u32, attack: &str) -> Result<Effects, CommandError> {
    game.execute(Command::Attack {
        unit: UnitId(a),
        target: UnitId(b),
        weapon: attack.into(),
    })
}
