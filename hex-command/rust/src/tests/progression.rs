//! Native advancement, XP and campaign carryover regressions.
use super::*;
use std::collections::HashSet;

#[test]
fn skeletal_mages_recruit_cast_advance_and_restore() {
    let mut game = flat();
    let mage = game.kind("skeleton-impact-caster");
    let roster = game
        .rules
        .rosters
        .iter()
        .find(|r| r.id == "undead")
        .unwrap();
    let pool: Vec<_> = roster.pool.iter().map(|id| game.kind(id)).collect();
    game.pools = Some([pool.clone(), pool]);
    let leader = add(&mut game, "necromancer", Side::Player, "2,3");
    let target = add(&mut game, "soldier", Side::Enemy, "4,3");
    let keep = game.site("2,3");
    game.set_terrain(keep, game.rules.terrains["keep"]);
    game.owners[keep] = Some(Side::Player);
    game.gold[Side::Player.index()] = game.rules.units[mage].cost;
    let id = game
        .execute(Command::Recruit {
            unit: UnitId(leader),
            kind: "skeleton-impact-caster".into(),
            tile: "3,3".into(),
        })
        .unwrap()
        .added_unit
        .unwrap()
        .0;
    assert_eq!(game.gold[Side::Player.index()], 0);
    assert!(game.unit(id).unwrap().attacked);
    game.end_turn();
    game.end_turn();
    let before_hp = game.unit(target).unwrap().hp;
    let effects = hit(&mut game, id, target, "boneburst");
    assert_eq!(effects.damage, Some(11));
    assert_eq!(effects.retaliation, Some(0));
    assert_eq!(game.unit(target).unwrap().hp, before_hp - 11);
    let index = game.index(id).unwrap();
    game.units[index].experience = game.rules.units[mage].experience_needed;
    game.units[index].hp -= 4;
    game.units[index].moved = false;
    game.units[index].attacked = false;
    game.execute(Command::Advance {
        unit: UnitId(id),
        kind: "skeleton-impact-caster-l2".into(),
    })
    .unwrap();
    let unit = game.unit(id).unwrap();
    assert_eq!(unit.hp, game.rules.units[unit.kind].hp - 4);
    let state = game.state();
    let restored = Game::try_from_state(rules(), state.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(restored.state()).unwrap(),
        serde_json::to_value(state).unwrap()
    );
    for mission in 0..rules().campaigns["ashen-oath"].missions.len() {
        let campaign = chapter("ashen-oath", mission);
        assert!(campaign.recruitment_options(Side::Player).contains(&mage));
    }
    let patch = rules().campaigns["ashen-oath"]
        .missions
        .last()
        .unwrap()
        .without_dain
        .as_ref()
        .unwrap();
    assert!(
        patch
            .recruits
            .as_ref()
            .unwrap()
            .iter()
            .any(|id| id == "skeleton-impact-caster")
    );
}

#[test]
fn macers_recruit_from_coastal_pools_and_advance_with_wounds() {
    let mut game = flat();
    let kind = game.kind("merfolk-maceguard");
    let roster = game
        .rules
        .rosters
        .iter()
        .find(|r| r.id == "coastal")
        .unwrap();
    let pool: Vec<_> = roster.pool.iter().map(|id| game.kind(id)).collect();
    game.pools = Some([pool.clone(), pool]);
    let leader = add(&mut game, "tidekeeper", Side::Player, "2,3");
    let keep = game.site("2,3");
    game.set_terrain(keep, game.rules.terrains["keep"]);
    game.owners[keep] = Some(Side::Player);
    game.gold[Side::Player.index()] = game.rules.units[kind].cost;
    let id = game
        .execute(Command::Recruit {
            unit: UnitId(leader),
            kind: "merfolk-maceguard".into(),
            tile: "3,3".into(),
        })
        .unwrap()
        .added_unit
        .unwrap()
        .0;
    assert_eq!(game.gold[Side::Player.index()], 0);
    let index = game.index(id).unwrap();
    game.units[index].experience = game.rules.units[kind].experience_needed;
    game.units[index].hp -= 4;
    game.units[index].moved = false;
    game.units[index].attacked = false;
    game.execute(Command::Advance {
        unit: UnitId(id),
        kind: "merfolk-maceguard-l2".into(),
    })
    .unwrap();
    let unit = game.unit(id).unwrap();
    assert_eq!(unit.hp, game.rules.units[unit.kind].hp - 4);
    let state = game.state();
    let restored = Game::try_from_state(rules(), state.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(restored.state()).unwrap(),
        serde_json::to_value(state).unwrap()
    );
    for mission in 0..rules().campaigns["borrowed-tide"].missions.len() {
        let campaign = chapter("borrowed-tide", mission);
        assert!(campaign.recruitment_options(Side::Player).contains(&kind));
    }
}

#[test]
fn authored_progression_ladders_are_acyclic_and_preserve_race_and_increase_level() {
    fn visit(rules: &Rules, kind: usize, path: &mut HashSet<usize>) {
        assert!(path.insert(kind), "cyclic advancement");
        let unit = &rules.units[kind];
        for next in &unit.advances_to {
            let advanced = rules.types[next];
            let rule = &rules.units[advanced];
            assert_eq!(rule.race, unit.race);
            assert!(rule.level > unit.level);
            visit(rules, advanced, path);
        }
        path.remove(&kind);
    }
    for kind in 0..rules().units.len() {
        let family = rules().types[rules().units[kind].family.as_str()];
        assert!(!rules().units[family].advances_to.is_empty());
        visit(rules(), kind, &mut HashSet::new());
    }
}

#[test]
fn xp_requires_real_damage_and_promotion_preserves_identity_and_wounds() {
    let mut game = flat();
    let id = add(&mut game, "swordsman", Side::Player, "3,3");
    let target = add(&mut game, "archer", Side::Enemy, "4,3");
    let index = game.index(id).unwrap();
    game.units[index].hp -= 4;
    let target_index = game.index(target).unwrap();
    game.units[target_index].hp = 3;
    hit(&mut game, id, target, "sword");
    assert_eq!(game.unit(id).unwrap().experience, Some(6));
    assert!(game.promotion_options(game.unit(id).unwrap()).is_empty());
    let needed = game.rules.units[game.kind("swordsman")].experience_needed;
    let index = game.index(id).unwrap();
    game.units[index].experience = needed;
    game.units[index].moved = false;
    game.units[index].attacked = false;
    let before = game.state();
    game.execute(Command::Advance {
        unit: UnitId(id),
        kind: "soldier".into(),
    })
    .unwrap();
    let promoted = game.unit(id).unwrap();
    assert_eq!(promoted.id, id);
    assert_eq!(promoted.hp, game.rules.units[promoted.kind].hp - 4);
    assert_eq!(promoted.experience, Some(0));
    assert!(promoted.moved && promoted.attacked);
    assert_eq!(before.units[0].r#type, "swordsman");
    assert_eq!(
        game.execute(Command::Advance {
            unit: UnitId(id),
            kind: "soldier".into()
        })
        .unwrap_err(),
        CommandError::IllegalAdvance
    );
}

#[test]
fn promoted_leader_veterans_and_named_companions_carry_forward_and_retry_rewinds_xp() {
    let mut game = chapter("borrowed-tide", 0);
    let leader = game
        .units
        .iter()
        .find(|u| u.side == Side::Player && u.role == Some(Role::Leader))
        .unwrap()
        .id;
    let troop = game
        .units
        .iter()
        .find(|u| u.side == Side::Player && game.rules.units[u.kind].id == "reefguard")
        .unwrap()
        .id;
    for (id, kind) in [(leader, "tideseer"), (troop, "tridentguard")] {
        let index = game.index(id).unwrap();
        game.units[index].experience = game.rules.units[game.units[index].kind].experience_needed;
        game.execute(Command::Advance {
            unit: UnitId(id),
            kind: kind.into(),
        })
        .unwrap();
    }
    let pilot = game
        .units
        .iter()
        .position(|u| u.name.as_deref() == Some("Edda"))
        .unwrap();
    game.units[pilot].experience = Some(7);
    game.winner = Some(Side::Player);
    game.execute(Command::RecallMission {
        units: vec![UnitId(troop)],
    })
    .unwrap();
    assert!(
        game.units
            .iter()
            .any(|u| u.role == Some(Role::Leader) && game.rules.units[u.kind].id == "tideseer")
    );
    assert!(
        game.units
            .iter()
            .any(|u| game.rules.units[u.kind].id == "tridentguard" && u.experience == Some(0))
    );
    assert_eq!(
        game.units
            .iter()
            .filter(|u| u.name.as_deref() == Some("Edda"))
            .count(),
        1
    );
    let pilot = game
        .units
        .iter()
        .position(|u| u.name.as_deref() == Some("Edda"))
        .unwrap();
    assert_eq!(game.units[pilot].experience, Some(7));
    game.units[pilot].experience = Some(10);
    game.execute(Command::RetryMission).unwrap();
    assert_eq!(
        game.units
            .iter()
            .find(|u| u.name.as_deref() == Some("Edda"))
            .unwrap()
            .experience,
        Some(7)
    );
    let restored = Game::try_from_state(rules(), game.state()).unwrap();
    assert_eq!(
        serde_json::to_value(restored.state()).unwrap(),
        serde_json::to_value(game.state()).unwrap()
    );
}
