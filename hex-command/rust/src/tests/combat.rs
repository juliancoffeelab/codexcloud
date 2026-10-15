//! Native weapon legality, damage equations and retaliation behavior.
use super::*;

// Isolate new mechanics from roster balance: one counted weapon, ample HP,
// neutral terrain/resistance and two independent aura sources.
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
            attack.damage = 4;
            attack.count = 2;
            attack.drain = 0;
        }
    }
    rules
}

#[test]
fn thorns_count_incoming_strikes_and_do_not_drain_or_require_a_surviving_defender() {
    for strikes in [1, 3] {
        let mut rules = ability_rules();
        let attacker = rules.types["scout"];
        let defender = rules.types["soldier"];
        rules.units[attacker].attacks[0].count = strikes;
        rules.units[defender].thorns = 3;
        rules.units[defender].attacks.truncate(1);
        rules.units[defender].attacks[0].count = 1;
        rules.units[defender].attacks[0].drain = 50;
        let mut game = Game::new(&rules, "mirror");
        game.units.clear();
        game.roster_changed();
        let a = add(&mut game, "scout", Side::Player, "2,3");
        let b = add(&mut game, "soldier", Side::Enemy, "3,3");
        let index = game.index(b).unwrap();
        game.units[index].hp = 90;
        let actor = game.unit(a).unwrap().clone();
        let target = game.unit(b).unwrap().clone();
        assert_eq!(
            game.preview(&actor, &target, 0, actor.tile).1,
            4 + 3 * i32::from(strikes)
        );
        let inspection = game.inspect(0);
        let details = inspection
            .units
            .iter()
            .find(|u| u.unit == format!("u{a}"))
            .unwrap()
            .attacks
            .iter()
            .find(|c| {
                c.target == format!("u{b}") && c.attack.id == rules.units[attacker].attacks[0].id
            })
            .unwrap();
        assert_eq!(details.thorn_damage, 3 * i32::from(strikes));
        assert_eq!(
            details.retaliation_healing, 2,
            "reflection never contributes drain healing"
        );
        let attack_id = rules.units[attacker].attacks[0].id.clone();
        hit(&mut game, a, b, &attack_id);
        assert_eq!(game.unit(a).unwrap().hp, 100 - 4 - 3 * i32::from(strikes));
        assert_eq!(game.unit(b).unwrap().hp, 92 - 4 * i32::from(strikes));
        game.begin_turn(Side::Player);
        let index = game.index(b).unwrap();
        game.units[index].hp = 1;
        let before = game.unit(a).unwrap().hp;
        hit(&mut game, a, b, &attack_id);
        assert!(game.unit(b).is_none());
        assert_eq!(game.unit(a).unwrap().hp, before - 3 * i32::from(strikes));
    }
}

#[test]
fn thorn_retaliation_stacks_own_shield_with_strongest_adjacent_friendly_aura() {
    let mut rules = ability_rules();
    rules.units[rules.types["soldier"]].thorns = 2;
    rules.units[rules.types["archer"]].thorn_aura = 1;
    rules.units[rules.types["scout"]].thorn_aura = 3;
    let mut game = Game::new(&rules, "mirror");
    game.units.clear();
    game.roster_changed();
    let a = add(&mut game, "soldier", Side::Player, "2,3");
    let b = add(&mut game, "soldier", Side::Enemy, "3,3");
    add(&mut game, "archer", Side::Enemy, "4,3");
    let aura = add(&mut game, "scout", Side::Enemy, "3,4");
    add(&mut game, "scout", Side::Player, "2,4");
    let preview = |game: &Game<'_>| {
        game.preview(
            game.unit(a).unwrap(),
            game.unit(b).unwrap(),
            0,
            game.unit(a).unwrap().tile,
        )
    };
    assert_eq!(
        preview(&game).0,
        8,
        "Thornshield never boosts initiating attacks"
    );
    assert_eq!(preview(&game).1, 18, "2 shield + strongest aura 3, per hit");
    let inspected = game.inspect(0);
    let choice = inspected
        .units
        .iter()
        .find(|u| u.unit == format!("u{a}"))
        .unwrap()
        .attacks
        .iter()
        .find(|c| c.target == format!("u{b}") && c.attack.id == "sword")
        .unwrap();
    assert_eq!(
        choice.retaliation_detail.as_ref().unwrap().total + choice.thorn_damage,
        choice.retaliation
    );
    hit(&mut game, a, b, "sword");
    assert_eq!(game.unit(a).unwrap().hp, 82);
    assert_eq!(game.unit(b).unwrap().hp, 92);
    game.units.retain(|u| u.id != aura);
    game.roster_changed();
    assert_eq!(
        preview(&game).1,
        14,
        "aura disappears when its source leaves"
    );
    let mut restored = Game::try_from_state(&rules, game.state()).unwrap();
    let bi = restored.index(b).unwrap();
    restored.units[bi].hp = 1;
    assert_eq!(
        preview(&restored),
        (8, 6, None),
        "a killing melee hit still triggers thorns"
    );
}

#[test]
fn ranged_retaliation_never_gets_thorn_bonus() {
    let mut rules = ability_rules();
    let kind = rules.types["archer"];
    rules.units[kind].thorns = 20;
    rules.units[kind].thorn_aura = 20;
    let mut game = Game::new(&rules, "mirror");
    game.units.clear();
    game.roster_changed();
    let a = add(&mut game, "archer", Side::Player, "2,3");
    let b = add(&mut game, "archer", Side::Enemy, "3,3");
    add(&mut game, "archer", Side::Enemy, "4,3");
    assert_eq!(
        game.preview(
            game.unit(a).unwrap(),
            game.unit(b).unwrap(),
            0,
            game.unit(a).unwrap().tile
        ),
        (8, 8, Some(0))
    );
}

#[test]
fn drain_uses_actual_hp_loss_before_reply_and_cannot_overheal_or_revive() {
    let mut rules = ability_rules();
    let kind = rules.types["soldier"];
    for attack in &mut rules.units[kind].attacks {
        attack.drain = 50;
    }
    let mut game = Game::new(&rules, "mirror");
    game.units.clear();
    game.roster_changed();
    let a = add(&mut game, "soldier", Side::Player, "2,3");
    let b = add(&mut game, "soldier", Side::Enemy, "3,3");
    let ai = game.index(a).unwrap();
    game.units[ai].hp = 5;
    let inspected = game.inspect(0);
    let choice = inspected
        .units
        .iter()
        .find(|u| u.unit == format!("u{a}"))
        .unwrap()
        .attacks
        .iter()
        .find(|c| c.target == format!("u{b}") && c.attack.id == "sword")
        .unwrap();
    assert_eq!((choice.healing, choice.retaliation_healing), (4, 4));
    hit(&mut game, a, b, "sword");
    assert_eq!(game.unit(a).unwrap().hp, 1, "heal 4 before taking reply 8");
    assert_eq!(
        game.unit(b).unwrap().hp,
        96,
        "surviving defender drains on retaliation"
    );
    let mut restored = Game::try_from_state(&rules, game.state()).unwrap();
    restored.begin_turn(Side::Player);
    let bi = restored.index(b).unwrap();
    restored.units[bi].hp = 3;
    hit(&mut restored, a, b, "sword");
    assert_eq!(
        restored.unit(a).unwrap().hp,
        2,
        "overkill heals only half of 3 HP, rounded down"
    );
    assert!(restored.unit(b).is_none());
    restored.begin_turn(Side::Player);
    let c = add(&mut restored, "soldier", Side::Enemy, "3,3");
    let ai = restored.index(a).unwrap();
    restored.units[ai].hp = 99;
    hit(&mut restored, a, c, "sword");
    assert_eq!(
        restored.unit(a).unwrap().hp,
        92,
        "healing caps before retaliation"
    );
    restored.begin_turn(Side::Player);
    let ai = restored.index(a).unwrap();
    restored.units[ai].hp = 1;
    hit(&mut restored, a, c, "sword");
    assert!(
        restored.unit(a).is_none(),
        "insufficient drain does not resurrect a dead attacker"
    );
}

#[test]
fn zero_damage_attacks_consume_actions_without_xp_and_keep_retaliation() {
    let mut game = flat();
    let attacker = add(&mut game, "bonearcher", Side::Player, "2,3");
    let target = add(&mut game, "soldier", Side::Enemy, "3,3");
    game.set_terrain(game.site("3,3"), game.rules.terrains["hill"]);
    let hp = game.unit(target).unwrap().hp;
    let attacker_hp = game.unit(attacker).unwrap().hp;
    hit(&mut game, attacker, target, "shard");
    assert!(game.unit(attacker).unwrap().hp < attacker_hp);
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
        hit(&mut game, attacker, target, "sword");
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
        let environment = match terrain {
            "hill" => -1,
            "ford" | "marsh" => 1,
            _ => 0,
        };
        let expected = rule
            .attacks
            .iter()
            .map(|a| {
                (a.damage + rule.damage_modifiers.get(a.damage_type) + environment).max(0)
                    * i32::from(a.count)
            })
            .max()
            .unwrap();
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

#[test]
fn counted_damage_keeps_the_existing_attack_and_restore_flow() {
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
    let bi = game.index(b).unwrap();
    game.units[bi].attacked = true;
    let actor = game.unit(a).unwrap();
    let target = game.unit(b).unwrap();
    assert_eq!(game.preview(actor, target, 0, actor.tile), (3, 5, Some(1)));
    assert_eq!(game.preview(actor, target, 1, actor.tile), (5, 5, Some(1)));
    hit(&mut game, a, b, "sword");
    assert_eq!(game.unit(a).unwrap().hp, 95);
    assert_eq!(game.unit(b).unwrap().hp, 97);
    assert!(game.unit(a).unwrap().attacked && game.unit(a).unwrap().moved);
    assert_eq!(game.unit(a).unwrap().experience, Some(1));
    let recorded = &game.recordings.as_ref().unwrap().combats[0];
    assert_eq!(
        (recorded.count, recorded.damage_per_hit, recorded.damage),
        (3, 1, 3)
    );
    let mut restored = Game::try_from_state(&rules, game.state()).unwrap();
    assert!(
        restored
            .execute(Command::Attack {
                unit: UnitId(a),
                target: UnitId(b),
                weapon: "mace".into()
            })
            .is_err()
    );
    restored.begin_turn(Side::Player);
    let bi = restored.index(b).unwrap();
    restored.units[bi].hp = 2;
    hit(&mut restored, a, b, "sword");
    assert_eq!(restored.unit(a).unwrap().hp, 95);
    assert!(restored.unit(b).is_none());
    assert_eq!(restored.unit(a).unwrap().experience, Some(7));
    restored.begin_turn(Side::Player);
    let c = add(&mut restored, "soldier", Side::Enemy, "3,3");
    let ai = restored.index(a).unwrap();
    restored.units[ai].hp = 1;
    hit(&mut restored, a, c, "sword");
    assert_eq!(restored.unit(c).unwrap().hp, 97);
    assert!(restored.unit(a).is_none());
    assert_eq!(restored.unit(c).unwrap().experience, Some(6));
}

#[test]
fn reflected_damage_records_separate_own_aura_and_effective_damage() {
    let mut rules = ability_rules();
    let soldier = rules.types["soldier"];
    rules.units[soldier].thorns = 2;
    rules.units[rules.types["scout"]].thorn_aura = 3;
    let mut game = Game::new(&rules, "mirror");
    game.units.clear();
    game.roster_changed();
    let a = add(&mut game, "soldier", Side::Player, "2,3");
    let b = add(&mut game, "soldier", Side::Enemy, "3,3");
    let aura = add(&mut game, "scout", Side::Enemy, "4,3");
    let index = game.index(a).unwrap();
    game.units[index].hp = 10;
    game.recordings = Some(Records::default());
    hit(&mut game, a, b, "sword");
    let thorns = game.recordings.as_ref().unwrap().combats[0]
        .thorns
        .as_ref()
        .unwrap();
    assert_eq!(thorns.damage, 10);
    assert_eq!(thorns.aura_damage, 6);
    assert_eq!(thorns.aura_sources, vec![format!("u{aura}")]);
    assert_eq!(thorns.effective_damage, 2); // ordinary retaliation removes 8 of 10 HP
}
