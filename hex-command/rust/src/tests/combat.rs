//! Native weapon legality, damage equations, retaliation and balance regressions.
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
    assert!(unit.moved && unit.attacked);
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
fn best_attack_avoids_lethal_reply_and_melee_cannot_repeat() {
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
    hit(&mut game, scout, bow, "knife");
    assert_eq!(game.last_combat.as_ref().unwrap().sound, "attack-melee");
    assert!(
        game.execute(Command::Attack {
            unit: UnitId(scout),
            target: UnitId(bow),
            weapon: "sling".into()
        })
        .is_err()
    );
}

#[test]
fn preview_and_resolution_agree_across_weapons_and_terrain() {
    for (a, b, name, terrain, tile) in [
        ("soldier", "bonearcher", "mace", "plains", "3,3"),
        ("archer", "soldier", "bow", "forest", "4,3"),
        ("scout", "archer", "sling", "plains", "4,3"),
        ("thorncaller", "spearman", "thorns", "grove", "3,3"),
        ("boar", "skeleton", "trample", "plains", "3,3"),
        ("harpooner", "swordsman", "harpoon", "ford", "3,3"),
        (
            "skeleton-impact-caster",
            "soldier",
            "boneburst",
            "plains",
            "4,3",
        ),
        (
            "skeleton-impact-caster-l2",
            "soldier",
            "staff",
            "hill",
            "3,3",
        ),
    ] {
        let mut game = flat();
        let attacker = add(&mut game, a, Side::Player, "2,3");
        let target = add(&mut game, b, Side::Enemy, tile);
        game.set_terrain(game.site(tile), game.rules.terrains[terrain]);
        let actor = game.unit(attacker).unwrap();
        let defender = game.unit(target).unwrap();
        let (damage, retaliation, _) =
            game.preview(actor, defender, weapon(&game, attacker, name), actor.tile);
        let ahp = actor.hp;
        let bhp = defender.hp;
        let result = hit(&mut game, attacker, target, name);
        assert_eq!(result.damage, Some(damage));
        assert_eq!(result.retaliation, Some(retaliation));
        assert_eq!(game.unit(attacker).unwrap().hp, ahp - retaliation);
        assert_eq!(game.unit(target).unwrap().hp, bhp - damage);
    }
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
        let base = game.rules.units[actor.kind].attacks[attack].damage - 1;
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
