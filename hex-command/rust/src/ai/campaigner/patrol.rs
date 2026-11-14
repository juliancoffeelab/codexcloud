//! Idle field movement is a strategic order, shared by every planner.
use crate::{
    ai::{context::BattleView, orders::*, policies::tie_breaking::Random},
    data::*,
};
use rustc_hash::FxHashSet;

pub(super) fn assign(
    game: &BattleView<'_, '_>,
    seed: u32,
    orders: &mut TacticalOrders,
    guidance: impl Fn(&Unit) -> Option<hex_command_battle::SearchGuidance>,
) {
    // Ordinary combat is the common path. No routes, allocations or random draws there.
    if game
        .units()
        .iter()
        .any(|unit| game.hostile_to_faction(unit, game.observer_faction()))
    {
        return;
    }
    // Generic authored travel/capture work already supplies a destination.
    if orders.targets.iter().any(|target| {
        !target.tiles.is_empty()
            && target.actor.is_none()
            && target.kind != TargetKind::Defeat
            && !target.id.starts_with("village:")
    }) {
        return;
    }
    let mut destinations = FxHashSet::default();
    let mut busy: FxHashSet<u32> = orders.inactive.iter().map(|unit| unit.0).collect();
    busy.extend(orders.scoped.iter().map(|order| order.actor.0));
    for duty in &orders.objectives {
        match &duty.objective {
            TacticalObjective::GoTo { actor, destination }
            | TacticalObjective::ClearPassage { actor, destination } => {
                busy.insert(actor.0);
                destinations.insert(game.site(&destination.0));
            }
            TacticalObjective::Occupy { actor, location } => {
                busy.insert(actor.0);
                destinations.insert(game.site(&location.0));
            }
            TacticalObjective::Escort {
                actor, selector, ..
            } => {
                busy.insert(actor.0);
                busy.extend(
                    game.units()
                        .iter()
                        .filter(|unit| {
                            game.belongs_to_faction(unit, game.observer_faction())
                                && selector.accepts(game, unit.kind)
                        })
                        .map(|unit| unit.id),
                );
            }
        }
    }
    let villages: Vec<_> = (0..game.terrain().len())
        .filter(|&tile| {
            game.role_is(tile, TerrainRole::Village) && game.captures(tile, game.observer_faction())
        })
        .collect();
    for unit in game.units().iter().filter(|unit| {
        game.belongs_to_faction(unit, game.observer_faction())
            && game.can_act(unit.id)
            && !unit.moved
            && game.available_combatant(unit)
            && !busy.contains(&unit.id)
            && (!unit.is_leader()
                || (orders.leader_post.is_none()
                    && !orders
                        .leader_intents
                        .iter()
                        .any(|(actor, intent)| actor.0 == unit.id && *intent == Some(true))))
    }) {
        let reachable = game.reachable(unit);
        // Multi-source terrain routes are cached by movement kind and destination set.
        let village_step = if villages.is_empty() {
            None
        } else {
            let routes = game.routes(unit.kind, &villages);
            reachable
                .iter()
                .filter(|&&(tile, _)| {
                    routes[tile] < routes[unit.tile] && !destinations.contains(&tile)
                })
                .min_by_key(|&&(tile, _)| (routes[tile], tile))
                .map(|&(tile, _)| tile)
        };
        let destination = village_step.or_else(|| {
            let longest = reachable.iter().map(|&(_, cost)| cost).max()?;
            let shortest = reachable
                .iter()
                .filter_map(|&(tile, cost)| (tile != unit.tile && cost > 0).then_some(cost))
                .min()?;
            let budget = longest.div_ceil(2).max(shortest);
            let guidance = guidance(unit);
            let mut random = Random(
                seed ^ unit.id.wrapping_mul(0x9e3779b9)
                    ^ game.round().wrapping_mul(0x85ebca6b)
                    ^ game.turn().index() as u32,
            );
            let mut selected = None;
            let mut total_weight = 0.;
            // Short weighted random steps avoid a march to an exact hidden unit.
            // The coarse Campaigner hint biases shorter/equal/longer routes 3:2:1.
            // Weighted reservoir sampling allocates no destination list.
            for &(tile, cost) in reachable.iter() {
                if tile == unit.tile || cost == 0 || cost > budget || destinations.contains(&tile) {
                    continue;
                }
                let weight = guidance
                    .as_ref()
                    .map_or(1., |hint| match hint.progress(tile) {
                        std::cmp::Ordering::Less => 3.,
                        std::cmp::Ordering::Equal => 2.,
                        std::cmp::Ordering::Greater => 1.,
                    });
                total_weight += weight;
                if random.next() * total_weight < weight {
                    selected = Some(tile);
                }
            }
            selected
        });
        let Some(destination) = destination else {
            continue;
        };
        destinations.insert(destination);
        let id = format!("patrol:u{}", unit.id);
        let tile = game.board().ids[destination];
        orders.targets.push(Target {
            id: id.clone(),
            kind: TargetKind::Visit,
            tiles: vec![tile],
            weight: 4.,
            actor: Some(format!("u{}", unit.id)),
            family: None,
            hold: None,
        });
        orders.objectives.push(ObjectiveOrder {
            id,
            priority: ObjectivePriority::Normal,
            objective: TacticalObjective::GoTo {
                actor: UnitId(unit.id),
                destination: TileId(tile),
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Game;

    #[test]
    fn patrols_are_deterministic_read_only_and_preserve_duties() {
        let rules = Rules::load().unwrap();
        let visible = Game::new(&rules, "mirror");
        let mut untouched = TacticalOrders::default();
        assign(&visible, 17, &mut untouched, |_| None);
        assert!(untouched.objectives.is_empty());
        let mut state = visible.state();
        state.units.retain(|unit| unit.faction == "player");
        for tile in &mut state.tiles {
            if tile.terrain == "village" {
                tile.owner = Some("player".into());
            }
        }
        let mut unit = state.units[0].clone();
        unit.id = format!("u{}", state.next_id);
        let id = state.next_id;
        state.next_id += 1;
        unit.role = None;
        unit.character = None;
        unit.r#type = "swordsman".into();
        unit.hp = 10;
        unit.items.clear();
        unit.tile = state
            .tiles
            .iter()
            .find(|tile| {
                tile.terrain == "plains" && state.units.iter().all(|unit| unit.tile != tile.hex())
            })
            .unwrap()
            .hex();
        state.units.push(unit);
        let game = Game::from_state(&rules, state);
        let before = serde_json::to_value(game.state()).unwrap();
        let mut first = TacticalOrders::default();
        assign(&game, 17, &mut first, |_| None);
        let mut repeated = TacticalOrders::default();
        assign(&game, 17, &mut repeated, |_| None);
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&repeated).unwrap()
        );
        fn destination(order: &ObjectiveOrder) -> Option<(UnitId, TileId)> {
            match order.objective {
                TacticalObjective::GoTo { actor, destination } => Some((actor, destination)),
                TacticalObjective::Occupy { .. }
                | TacticalObjective::Escort { .. }
                | TacticalObjective::ClearPassage { .. } => None,
            }
        }
        assert!(
            first
                .objectives
                .iter()
                .filter_map(destination)
                .any(|(actor, destination)| {
                    actor.0 == id && destination.0 != game.board().ids[game.unit(id).unwrap().tile]
                })
        );
        let endpoints: Vec<_> = first
            .objectives
            .iter()
            .filter_map(destination)
            .map(|(_, tile)| tile.0)
            .collect();
        assert_eq!(
            endpoints.len(),
            endpoints.iter().collect::<FxHashSet<_>>().len()
        );
        assert_eq!(before, serde_json::to_value(game.state()).unwrap());
        let mut guarded = TacticalOrders::default();
        guarded.objectives.push(ObjectiveOrder {
            id: "guard".into(),
            priority: ObjectivePriority::Important,
            objective: TacticalObjective::Occupy {
                actor: UnitId(id),
                location: TileId(game.board().ids[game.unit(id).unwrap().tile]),
            },
        });
        assign(&game, 17, &mut guarded, |_| None);
        assert!(
            !guarded
                .objectives
                .iter()
                .any(|order| order.id == format!("patrol:u{id}"))
        );
    }
}
