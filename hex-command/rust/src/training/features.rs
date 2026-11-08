//! Versioned, stat-based observations. No unit or terrain identity embeddings.
use crate::{
    ai::{context::BattleView, orders::*},
    data::*,
};
use std::{collections::HashMap, rc::Rc};
pub const FEATURES: usize = 172;
pub const FEATURE_NAMES: [&str; FEATURES] = [
    "end_turn",
    "move",
    "attack",
    "advance",
    "round",
    "allies",
    "enemies",
    "gold",
    "hp_fraction",
    "hp",
    "leader",
    "moved",
    "attacked",
    "cost",
    "movement",
    "protected",
    "nearest_enemy",
    "enemy_distance_gain",
    "nearest_ally",
    "allies_adjacent",
    "enemies_adjacent",
    "captures",
    "goal_distance",
    "goal_distance_gain",
    "goal_weight",
    "hold_goal",
    "damage",
    "retaliation",
    "kill_probability",
    "target_hp",
    "target_leader",
    "objective_count",
    "actor_max_hp",
    "actor_cost",
    "actor_movement",
    "actor_level",
    "actor_experience_fraction",
    "actor_promotion_ready",
    "actor_healing",
    "actor_item_healing",
    "actor_item_healing_radius",
    "actor_thorns",
    "actor_thorn_aura",
    "actor_blade_modifier",
    "actor_pierce_modifier",
    "actor_impact_modifier",
    "actor_melee_damage",
    "actor_melee_strikes",
    "actor_melee_range",
    "actor_melee_spread",
    "actor_melee_drain",
    "actor_melee_blade",
    "actor_melee_pierce",
    "actor_melee_impact",
    "actor_ranged_damage",
    "actor_ranged_strikes",
    "actor_ranged_range",
    "actor_ranged_spread",
    "actor_ranged_drain",
    "actor_ranged_blade",
    "actor_ranged_pierce",
    "actor_ranged_impact",
    "actor_hp_fraction",
    "actor_moved",
    "actor_attacked",
    "actor_leader",
    "target_max_hp",
    "target_cost",
    "target_movement",
    "target_level",
    "target_experience_fraction",
    "target_promotion_ready",
    "target_healing",
    "target_item_healing",
    "target_item_healing_radius",
    "target_thorns",
    "target_thorn_aura",
    "target_blade_modifier",
    "target_pierce_modifier",
    "target_impact_modifier",
    "target_melee_damage",
    "target_melee_strikes",
    "target_melee_range",
    "target_melee_spread",
    "target_melee_drain",
    "target_melee_blade",
    "target_melee_pierce",
    "target_melee_impact",
    "target_ranged_damage",
    "target_ranged_strikes",
    "target_ranged_range",
    "target_ranged_spread",
    "target_ranged_drain",
    "target_ranged_blade",
    "target_ranged_pierce",
    "target_ranged_impact",
    "target_hp_fraction",
    "target_moved",
    "target_attacked",
    "target_leader",
    "nearest_enemy_max_hp",
    "nearest_enemy_cost",
    "nearest_enemy_movement",
    "nearest_enemy_level",
    "nearest_enemy_experience_fraction",
    "nearest_enemy_promotion_ready",
    "nearest_enemy_healing",
    "nearest_enemy_item_healing",
    "nearest_enemy_item_healing_radius",
    "nearest_enemy_thorns",
    "nearest_enemy_thorn_aura",
    "nearest_enemy_blade_modifier",
    "nearest_enemy_pierce_modifier",
    "nearest_enemy_impact_modifier",
    "nearest_enemy_melee_damage",
    "nearest_enemy_melee_strikes",
    "nearest_enemy_melee_range",
    "nearest_enemy_melee_spread",
    "nearest_enemy_melee_drain",
    "nearest_enemy_melee_blade",
    "nearest_enemy_melee_pierce",
    "nearest_enemy_melee_impact",
    "nearest_enemy_ranged_damage",
    "nearest_enemy_ranged_strikes",
    "nearest_enemy_ranged_range",
    "nearest_enemy_ranged_spread",
    "nearest_enemy_ranged_drain",
    "nearest_enemy_ranged_blade",
    "nearest_enemy_ranged_pierce",
    "nearest_enemy_ranged_impact",
    "nearest_enemy_hp_fraction",
    "nearest_enemy_moved",
    "nearest_enemy_attacked",
    "nearest_enemy_leader",
    "destination_melee_blade",
    "destination_melee_pierce",
    "destination_melee_impact",
    "destination_ranged_blade",
    "destination_ranged_pierce",
    "destination_ranged_impact",
    "destination_entry_cost",
    "destination_healing",
    "destination_keep",
    "destination_owned",
    "target_terrain_melee_blade",
    "target_terrain_melee_pierce",
    "target_terrain_melee_impact",
    "target_terrain_ranged_blade",
    "target_terrain_ranged_pierce",
    "target_terrain_ranged_impact",
    "target_terrain_entry_cost",
    "target_terrain_healing",
    "target_terrain_keep",
    "target_terrain_owned",
    "goal_path_cost",
    "goal_travel_turns",
    "goal_path_gain",
    "goal_path_reachable",
    "enemy_path_cost",
    "enemy_travel_turns",
    "enemy_path_gain",
    "enemy_path_reachable",
    "move_cost",
    "weapon_damage",
    "weapon_strikes",
    "weapon_range",
    "weapon_spread",
    "weapon_drain",
    "weapon_blade",
    "weapon_pierce",
    "weapon_impact",
    "weapon_melee",
];
#[derive(Clone)]
pub struct Choices {
    pub spatial: Option<super::spatial::Spatial>,
    pub commands: Vec<Command>,
    pub features: Vec<[f32; FEATURES]>,
}
fn actor_matches(actor: &Option<String>, unit: &Unit) -> bool {
    actor
        .as_ref()
        .is_none_or(|a| a == &format!("u{}", unit.id) || unit.character.as_ref() == Some(a))
}
pub fn candidates(game: &BattleView<'_, '_>, orders: &TacticalOrders) -> Choices {
    let mut commands = vec![Command::EndTurn];
    for unit in game
        .units()
        .iter()
        .filter(|u| game.belongs_to_phase(u, game.turn()) && game.can_act(u.id))
    {
        if !unit.moved {
            for &(tile, _) in game.reachable(unit).iter() {
                if tile != unit.tile {
                    let c = Command::Move {
                        unit: UnitId(unit.id),
                        tile: TileId(game.board().ids[tile].clone()),
                    };
                    if game.permits(&c) {
                        commands.push(c);
                    }
                }
            }
        }
        if !unit.attacked {
            for target in game.units().iter().filter(|t| game.hostile(unit, t)) {
                for weapon in game.attack_options(unit, target, unit.tile) {
                    let c = Command::Attack {
                        unit: UnitId(unit.id),
                        target: UnitId(target.id),
                        weapon: WeaponId(game.weapon(unit, weapon).id.clone()),
                    };
                    if game.permits(&c) {
                        commands.push(c);
                    }
                }
            }
        }
        if let Some(kind) = orders.promotion(game, unit) {
            commands.push(Command::Advance {
                unit: UnitId(unit.id),
                kind: UnitTypeId(game.rules().units[kind].id.clone()),
            });
        }
    }
    let extra = Extra::new(game, orders);
    let features = commands
        .iter()
        .map(|c| encode(game, orders, c, &extra))
        .collect();
    let spatial = super::needs_spatial().then(|| super::spatial::observe(game, orders, &commands));
    Choices {
        commands,
        features,
        spatial,
    }
}
fn encode(
    g: &BattleView<'_, '_>,
    orders: &TacticalOrders,
    c: &Command,
    extra: &Extra,
) -> [f32; FEATURES] {
    let mut x = [0.; FEATURES];
    let allies: Vec<_> = g
        .units()
        .iter()
        .filter(|u| g.belongs_to_phase(u, g.turn()))
        .collect();
    let enemies: Vec<_> = g
        .units()
        .iter()
        .filter(|u| g.hostile_to_phase(u, g.turn()))
        .collect();
    x[4] = g.round() as f32 / 30.;
    x[5] = allies.len() as f32 / 10.;
    x[6] = enemies.len() as f32 / 10.;
    x[7] = g.gold_for_phase(g.turn()) as f32 / 100.;
    x[31] = orders.targets.len() as f32 / 10.;
    let (id, tile, target, weapon) = match c {
        Command::EndTurn => {
            x[0] = 1.;
            return x;
        }
        Command::Move { unit, tile } => {
            x[1] = 1.;
            (unit.0, Some(g.site(&tile.0)), None, None)
        }
        Command::Attack {
            unit,
            target,
            weapon,
        } => {
            x[2] = 1.;
            (unit.0, None, Some(target.0), Some(&weapon.0))
        }
        Command::Advance { unit, .. } => {
            x[3] = 1.;
            (unit.0, None, None, None)
        }
        _ => unreachable!(),
    };
    let u = g.unit(id).unwrap();
    let r = &g.rules().units[u.kind];
    let tile = tile.unwrap_or(u.tile);
    x[8] = u.hp as f32 / r.hp.max(1) as f32;
    x[9] = u.hp as f32 / 30.;
    x[10] = u.is_leader() as u8 as f32;
    x[11] = u.moved as u8 as f32;
    x[12] = u.attacked as u8 as f32;
    x[13] = r.cost as f32 / 30.;
    x[14] = r.r#move as f32 / 10.;
    x[15] = orders.protects(id) as u8 as f32;
    let dist = |a, b| g.board().distance(a, b) as f32;
    let before = enemies
        .iter()
        .map(|e| dist(u.tile, e.tile))
        .fold(30., f32::min);
    let after = enemies
        .iter()
        .map(|e| dist(tile, e.tile))
        .fold(30., f32::min);
    x[16] = after / 20.;
    x[17] = (before - after) / 10.;
    x[18] = allies
        .iter()
        .filter(|a| a.id != id)
        .map(|a| dist(tile, a.tile))
        .fold(30., f32::min)
        / 20.;
    x[19] = allies
        .iter()
        .filter(|a| a.id != id && dist(tile, a.tile) == 1.)
        .count() as f32
        / 6.;
    x[20] = enemies.iter().filter(|a| dist(tile, a.tile) == 1.).count() as f32 / 6.;
    x[21] = g.captures(tile, g.turn()) as u8 as f32;
    if let Some((goal, d)) = orders
        .targets
        .iter()
        .filter(|o| actor_matches(&o.actor, u) && o.family.as_ref().is_none_or(|f| *f == r.family))
        .filter_map(|o| {
            o.tiles
                .iter()
                .map(|t| dist(tile, g.site(t)))
                .reduce(f32::min)
                .map(|d| (o, d))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
    {
        let old = goal
            .tiles
            .iter()
            .map(|t| dist(u.tile, g.site(t)))
            .fold(30., f32::min);
        x[22] = d / 20.;
        x[23] = (old - d) / 10.;
        x[24] = goal.weight as f32 / 10.;
        x[25] = goal.hold.unwrap_or(false) as u8 as f32;
    }
    x[32..66].copy_from_slice(&extra.profiles[&id]);
    terrain_features(g, u, tile, &mut x[134..144]);
    if let Some(enemy) = enemies
        .iter()
        .min_by_key(|e| (g.board().distance(tile, e.tile), e.id))
    {
        x[100..134].copy_from_slice(&extra.profiles[&enemy.id]);
    }
    let paths = &extra.paths[&id];
    path_features(&paths.0, u.tile, tile, r.r#move, &mut x[154..158]);
    path_features(&paths.1, u.tile, tile, r.r#move, &mut x[158..162]);
    if matches!(c, Command::Move { .. }) {
        x[162] = g
            .reachable(u)
            .iter()
            .find(|&&(t, _)| t == tile)
            .map_or(0., |&(_, cost)| cost as f32 / 10.);
    }
    if let Some(t) = target.and_then(|id| g.unit(id)) {
        let w = g.weapons(u).position(|w| Some(&w.id) == weapon).unwrap();
        x[66..100].copy_from_slice(&extra.profiles[&t.id]);
        terrain_features(g, t, t.tile, &mut x[144..154]);
        let attack = g.weapon(u, w);
        weapon_features(attack, &mut x[163..171]);
        x[171] = (attack.category == Category::Melee) as u8 as f32;
        let forecast = g.exchange(u, t, w, u.tile);
        x[26] = forecast.effective[0] as f32 / 30.;
        x[27] = forecast.damage[1] as f32 / 30.;
        x[28] = forecast.death[1] as f32;
        x[29] = t.hp as f32 / 30.;
        x[30] = t.is_leader() as u8 as f32;
    }
    x
}

// Built once per decision; the engine caches route fields by movement profile,
// goal set and terrain revision, so candidates only index shared distances.
struct Extra {
    profiles: HashMap<u32, [f32; 34]>,
    paths: HashMap<u32, (Rc<Vec<u16>>, Rc<Vec<u16>>)>,
}
impl Extra {
    fn new(g: &BattleView<'_, '_>, orders: &TacticalOrders) -> Self {
        let profiles = g
            .units()
            .iter()
            .map(|u| (u.id, unit_features(g, u)))
            .collect();
        let mut paths = HashMap::new();
        let mut enemies: Vec<_> = g
            .units()
            .iter()
            .filter(|u| g.hostile_to_phase(u, g.turn()))
            .map(|u| u.tile)
            .collect();
        enemies.sort_unstable();
        enemies.dedup();
        for u in g
            .units()
            .iter()
            .filter(|u| g.belongs_to_phase(u, g.turn()) && g.can_act(u.id))
        {
            let mut goals: Vec<_> = orders
                .targets
                .iter()
                .filter(|o| {
                    actor_matches(&o.actor, u)
                        && o.family
                            .as_ref()
                            .is_none_or(|f| *f == g.rules().units[u.kind].family)
                })
                .flat_map(|o| o.tiles.iter().map(move |t| (o, g.site(t))))
                .filter(|(o, t)| match o.kind {
                    TargetKind::Capture => g.captures(*t, g.turn()),
                    TargetKind::Defeat => g.at(*t).is_some_and(|e| g.hostile(u, e)),
                    TargetKind::Visit => true,
                })
                .map(|(_, t)| t)
                .collect();
            goals.sort_unstable();
            goals.dedup();
            paths.insert(u.id, (g.routes(u.kind, &goals), g.routes(u.kind, &enemies)));
        }
        Self { profiles, paths }
    }
}
fn weapon_features(w: &Weapon, x: &mut [f32]) {
    x[0] = w.damage as f32 / 10.;
    x[1] = w.count as f32 / 5.;
    x[2] = w.range as f32 / 5.;
    x[3] = w.spread as f32 / 10.;
    x[4] = w.drain as f32 / 100.;
    x[5 + w.damage_type.index()] = 1.;
}
pub(super) fn unit_features(g: &BattleView<'_, '_>, u: &Unit) -> [f32; 34] {
    let mut x = [0.; 34];
    let r = &g.rules().units[u.kind];
    x[0] = r.hp as f32 / 30.;
    x[1] = r.cost as f32 / 30.;
    x[2] = r.r#move as f32 / 10.;
    x[3] = r.level as f32 / 5.;
    x[4] = r
        .experience_needed
        .map_or(0., |n| u.experience.unwrap_or(0) as f32 / n.max(1) as f32);
    x[5] = (r
        .experience_needed
        .is_some_and(|n| u.experience.unwrap_or(0) >= n)
        && !r.advances_to.is_empty()) as u8 as f32;
    x[6] = r.healing as f32 / 10.;
    if let Some(aura) = u
        .items
        .iter()
        .filter_map(|i| i.healing.as_ref())
        .max_by_key(|a| (a.amount, a.radius))
    {
        x[7] = aura.amount as f32 / 10.;
        x[8] = aura.radius as f32 / 10.;
    }
    x[9] = r.thorns as f32 / 10.;
    x[10] = r.thorn_aura as f32 / 10.;
    for (i, damage) in [DamageType::Blade, DamageType::Pierce, DamageType::Impact]
        .into_iter()
        .enumerate()
    {
        x[11 + i] = (r.damage_modifiers.get(damage)
            + u.items
                .iter()
                .filter_map(|i| i.armor.as_ref())
                .map(|a| a.get(damage))
                .sum::<i32>()) as f32
            / 10.;
    }
    for (i, category) in [Category::Melee, Category::Ranged].into_iter().enumerate() {
        if let Some(w) = g
            .weapons(u)
            .filter(|w| w.category == category)
            .max_by_key(|w| w.damage * i32::from(w.count))
        {
            weapon_features(w, &mut x[14 + i * 8..22 + i * 8]);
        }
    }
    x[30] = u.hp as f32 / r.hp.max(1) as f32;
    x[31] = u.moved as u8 as f32;
    x[32] = u.attacked as u8 as f32;
    x[33] = u.is_leader() as u8 as f32;
    x
}
fn terrain_features(g: &BattleView<'_, '_>, u: &Unit, tile: usize, x: &mut [f32]) {
    let r = &g.rules().units[u.kind];
    for (i, &v) in r.environment[g.terrain()[tile]]
        .iter()
        .flatten()
        .enumerate()
    {
        x[i] = v as f32 / 10.;
    }
    let cost = g.cost(u.kind, tile);
    x[6] = if cost == u16::MAX {
        -1.
    } else {
        cost as f32 / 10.
    };
    x[7] = g.terrain_healing(tile) as f32 / 10.;
    x[8] = g.is_keep(tile) as u8 as f32;
    x[9] = g.owns(tile, u.side) as u8 as f32;
}
fn path_features(route: &[u16], from: usize, to: usize, movement: usize, x: &mut [f32]) {
    let cost = route[to];
    if cost == u16::MAX || (movement == 0 && cost > 0) {
        return;
    }
    x[0] = cost as f32 / 30.;
    x[1] = cost as f32 / movement.max(1) as f32 / 10.;
    if route[from] != u16::MAX {
        x[2] = (route[from] as f32 - cost as f32) / 10.;
    }
    x[3] = 1.;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Game;
    #[test]
    fn observations_follow_current_stats_and_terrain_without_type_ids() {
        let rules = Rules::load().unwrap();
        let game = Game::new(&rules, "mirror");
        let actor = game.units().iter().find(|u| u.side == game.turn()).unwrap();
        let baseline = unit_features(&game.view(), actor);
        let mut changed = rules.clone();
        let kind = actor.kind;
        changed.units[kind].hp += 7;
        changed.units[kind].healing += 2;
        changed.units[kind].damage_modifiers.blade -= 1;
        let tile = actor.tile;
        let terrain = game.terrain()[tile];
        changed.units[kind].environment[terrain][0][0] -= 2;
        let altered = Game::from_state(&changed, game.state());
        let altered_actor = altered.units().iter().find(|u| u.id == actor.id).unwrap();
        let after = unit_features(&altered.view(), altered_actor);
        assert_ne!(baseline[0], after[0]);
        assert_ne!(baseline[6], after[6]);
        assert_ne!(baseline[11], after[11]);
        let mut old_terrain = [0.; 10];
        let mut new_terrain = [0.; 10];
        terrain_features(&game.view(), actor, tile, &mut old_terrain);
        terrain_features(&altered.view(), altered_actor, tile, &mut new_terrain);
        assert_ne!(old_terrain[0], new_terrain[0]);
        // Cosmetic identity does not enter the mechanical profile.
        changed.units[kind].id = "renamed-for-observation-probe".into();
        let renamed = Game::from_state(&changed, game.state());
        assert_eq!(
            after,
            unit_features(
                &renamed.view(),
                renamed.units().iter().find(|u| u.id == actor.id).unwrap()
            )
        );
    }
    #[test]
    fn travel_estimates_distinguish_cost_and_unreachable() {
        let mut x = [0.; 4];
        path_features(&[9, 3], 0, 1, 3, &mut x);
        assert_eq!(x, [0.1, 0.1, 0.6, 1.]);
        path_features(&[3, 3], 0, 1, 3, &mut x);
        assert_eq!(x, [0.1, 0.1, 0., 1.]);
        let mut unreachable = [0.; 4];
        path_features(&[0, u16::MAX], 0, 1, 3, &mut unreachable);
        assert_eq!(unreachable, [0.; 4]);
    }
}
