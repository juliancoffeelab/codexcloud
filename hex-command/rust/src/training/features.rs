//! Versioned, stat-based observations. No unit or terrain identity embeddings.
use crate::{
    ai::{context::BattleView, orders::*},
    data::*,
};
use hex_command_battle::RuntimeCommand;
use std::{collections::HashMap, rc::Rc};
pub const LEGACY_FEATURES: usize = 172;
pub const DUTY_FEATURES: usize = 219;
pub const DIPLOMACY_FEATURES: usize = 227;
pub const CAMPAIGN_FEATURES: usize = LEGACY_FEATURES + super::duties::NAMES.len();
pub const VISIBILITY_FEATURES: usize = CAMPAIGN_FEATURES + super::visibility::NAMES.len();
pub const IDLE_FEATURES: usize = VISIBILITY_FEATURES + 2;
pub const UNIT_HISTORY_NAMES: [&str; 9] = [
    "unit_history_known",
    "unit_previous_action_move",
    "unit_previous_action_attack",
    "unit_previous_action_advance",
    "unit_move_history_known",
    "unit_turns_since_move",
    "unit_attack_history_known",
    "unit_turns_since_attack",
    "move_returns_to_previous_position",
];
pub const FEATURES: usize = IDLE_FEATURES + UNIT_HISTORY_NAMES.len();
pub const LEGACY_FEATURE_NAMES: [&str; LEGACY_FEATURES] = [
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
    // Stable trained feature label; its value comes from Unit::attack_spent.
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
pub const FEATURE_NAMES: [&str; FEATURES] = {
    let old = LEGACY_FEATURE_NAMES;
    let mut names = [""; FEATURES];
    let mut i = 0;
    while i < LEGACY_FEATURES {
        names[i] = old[i];
        i += 1;
    }
    let mut j = 0;
    while j < super::duties::NAMES.len() {
        names[i + j] = super::duties::NAMES[j];
        j += 1;
    }
    let mut k = 0;
    while k < super::visibility::NAMES.len() {
        names[CAMPAIGN_FEATURES + k] = super::visibility::NAMES[k];
        k += 1;
    }
    names[VISIBILITY_FEATURES] = "consecutive_idle_turns";
    names[VISIBILITY_FEATURES + 1] = "acted_this_turn";
    let mut h = 0;
    while h < UNIT_HISTORY_NAMES.len() {
        names[IDLE_FEATURES + h] = UNIT_HISTORY_NAMES[h];
        h += 1;
    }
    names
};
#[derive(Clone)]
pub struct Choices {
    pub context: super::DecisionContext,
    pub spatial: Option<super::spatial::Spatial>,
    pub commands: Vec<RuntimeCommand>,
    pub features: Vec<[f32; FEATURES]>,
}
fn actor_matches(g: &BattleView<'_, '_>, actor: &Option<String>, unit: &Unit) -> bool {
    actor
        .as_ref()
        .is_none_or(|a| a == &format!("u{}", unit.id) || g.name(unit.character) == Some(a.as_str()))
}
pub fn candidates(game: &BattleView<'_, '_>, orders: &TacticalOrders) -> Choices {
    let mut commands = vec![RuntimeCommand::EndTurn];
    let mut move_costs = vec![0.];
    for unit in game
        .units()
        .iter()
        .filter(|u| game.belongs_to_faction(u, game.observer_faction()) && game.can_act(u.id))
    {
        if !unit.moved {
            for &(tile, cost) in game.reachable(unit).iter() {
                if tile != unit.tile {
                    let c = RuntimeCommand::Move {
                        unit: UnitId(unit.id),
                        tile: TileIndex(tile),
                    };
                    if game.permits_runtime(&c) {
                        commands.push(c);
                        move_costs.push(cost as f32 / 10.);
                    }
                }
            }
        }
        if !unit.attack_spent {
            for target in game.units().iter().filter(|t| game.hostile(unit, t)) {
                for weapon in game.attack_options(unit, target, unit.tile) {
                    if !orders.attack_allowed(game, unit, target, weapon, unit.tile) {
                        continue;
                    }
                    let c = RuntimeCommand::Attack {
                        unit: UnitId(unit.id),
                        target: UnitId(target.id),
                        weapon: game.weapon_key(unit, weapon),
                    };
                    if game.permits_runtime(&c) {
                        commands.push(c);
                        move_costs.push(0.);
                    }
                }
            }
        }
        if let Some(kind) = orders.promotion(game, unit) {
            commands.push(RuntimeCommand::Advance {
                unit: UnitId(unit.id),
                kind: UnitKind(kind),
            });
            move_costs.push(0.);
        }
    }
    let extra = Extra::new(game, orders);
    let history = super::history_features(game);
    let features = commands
        .iter()
        .zip(move_costs)
        .map(|(c, cost)| {
            let mut row = encode(game, orders, c, cost, &extra);
            row[VISIBILITY_FEATURES..IDLE_FEATURES].copy_from_slice(&history);
            row[IDLE_FEATURES..].copy_from_slice(&super::unit_history_features(game, c));
            row
        })
        .collect();
    let spatial = super::needs_spatial()
        .then(|| super::spatial::observe_with_duties(game, orders, &commands, &extra.duties));
    Choices {
        context: super::decision_context(game),
        commands,
        features,
        spatial,
    }
}
fn encode(
    g: &BattleView<'_, '_>,
    orders: &TacticalOrders,
    c: &RuntimeCommand,
    move_cost: f32,
    extra: &Extra,
) -> [f32; FEATURES] {
    let mut x = [0.; FEATURES];
    let allies = &extra.allies;
    let enemies = &extra.enemies;
    x[4] = g.round() as f32 / 30.;
    x[5] = allies.len() as f32 / 10.;
    x[6] = enemies.len() as f32 / 10.;
    x[7] = g.gold_for_faction(g.observer_faction()) as f32 / 100.;
    x[31] = orders.targets.len() as f32 / 10.;
    extra
        .duties
        .encode(g, orders, c, &mut x[LEGACY_FEATURES..CAMPAIGN_FEATURES]);
    x[CAMPAIGN_FEATURES..VISIBILITY_FEATURES].copy_from_slice(&g.visibility_action_features(c));
    let (id, tile, target, weapon) = match c {
        RuntimeCommand::EndTurn => {
            x[0] = 1.;
            return x;
        }
        RuntimeCommand::Move { unit, tile } => {
            x[1] = 1.;
            (unit.0, Some(tile.0), None, None)
        }
        RuntimeCommand::Attack {
            unit,
            target,
            weapon,
        } => {
            x[2] = 1.;
            (unit.0, None, Some(target.0), Some(*weapon))
        }
        RuntimeCommand::Advance { unit, .. } => {
            x[3] = 1.;
            (unit.0, None, None, None)
        }
        _ => unreachable!(),
    };
    let u = g.unit(id).unwrap();
    let r = g.unit_definition(u.kind);
    let tile = tile.unwrap_or(u.tile);
    x[8] = u.hp as f32 / r.max_hp.max(1) as f32;
    x[9] = u.hp as f32 / 30.;
    x[10] = u.is_leader() as u8 as f32;
    x[11] = u.moved as u8 as f32;
    x[12] = u.attack_spent as u8 as f32;
    x[13] = g.recruit_cost(u, u.kind) as f32 / 30.;
    x[14] = g.movement_limit(u) as f32 / 10.;
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
    x[21] = g.captures(tile, g.observer_faction()) as u8 as f32;
    if let Some((goal, d)) = orders
        .targets
        .iter()
        .filter(|o| {
            actor_matches(g, &o.actor, u) && o.family.as_ref().is_none_or(|f| *f == r.family())
        })
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
    path_features(
        &paths.0,
        u.tile,
        tile,
        g.movement_limit(u),
        &mut x[154..158],
    );
    path_features(
        &paths.1,
        u.tile,
        tile,
        g.movement_limit(u),
        &mut x[158..162],
    );
    x[162] = move_cost;
    if let Some(t) = target.and_then(|id| g.unit(id)) {
        let w = g
            .weapon_index(u, weapon.expect("attack candidate has a weapon"))
            .expect("candidate weapon belongs to actor");
        x[66..100].copy_from_slice(&extra.profiles[&t.id]);
        terrain_features(g, t, t.tile, &mut x[144..154]);
        g.with_capabilities(u, u.tile, |caps| {
            let attack = caps.attacks().nth(w).expect("attack index is valid");
            weapon_features(caps, attack, &mut x[163..171]);
            x[171] = f32::from(attack.category() == Category::Melee);
        });
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
struct Extra<'a> {
    allies: Vec<&'a Unit>,
    enemies: Vec<&'a Unit>,
    duties: super::duties::Duties,
    profiles: HashMap<u32, [f32; 34]>,
    paths: HashMap<u32, (Rc<Vec<u16>>, Rc<Vec<u16>>)>,
}
impl<'a> Extra<'a> {
    fn new(g: &'a BattleView<'_, '_>, orders: &TacticalOrders) -> Self {
        let allies = g
            .units()
            .iter()
            .filter(|u| g.belongs_to_faction(u, g.observer_faction()))
            .collect();
        let enemy_units = g
            .units()
            .iter()
            .filter(|u| g.hostile_to_faction(u, g.observer_faction()))
            .collect();
        let profiles = g
            .units()
            .iter()
            .map(|u| (u.id, unit_features(g, u)))
            .collect();
        let mut paths = HashMap::new();
        let mut enemies: Vec<_> = g
            .units()
            .iter()
            .filter(|u| g.hostile_to_faction(u, g.observer_faction()))
            .map(|u| u.tile)
            .collect();
        enemies.sort_unstable();
        enemies.dedup();
        for u in g
            .units()
            .iter()
            .filter(|u| g.belongs_to_faction(u, g.observer_faction()) && g.can_act(u.id))
        {
            let mut goals: Vec<_> = orders
                .targets
                .iter()
                .filter(|o| {
                    actor_matches(g, &o.actor, u)
                        && o.family
                            .as_ref()
                            .is_none_or(|f| *f == g.unit_definition(u.kind).family())
                })
                .flat_map(|o| o.tiles.iter().map(move |t| (o, g.site(t))))
                .filter(|(o, t)| match o.kind {
                    TargetKind::Capture => g.captures(*t, g.observer_faction()),
                    TargetKind::Defeat => g.at(*t).is_some_and(|e| g.hostile(u, e)),
                    TargetKind::Visit => true,
                })
                .map(|(_, t)| t)
                .collect();
            goals.sort_unstable();
            goals.dedup();
            paths.insert(u.id, (g.routes(u, &goals), g.routes(u, &enemies)));
        }
        Self {
            allies,
            enemies: enemy_units,
            profiles,
            paths,
            duties: super::duties::Duties::new(g, orders),
        }
    }
}
fn weapon_features(caps: &UnitCapabilities<'_>, attack: AttackHandle<'_>, x: &mut [f32]) {
    let stats = caps.neural_pure_attack_features(attack);
    x[0] = stats.mean_damage as f32 / 10.;
    x[1] = f32::from(stats.strike_count) / 5.;
    x[2] = f32::from(attack.range()) / 5.;
    x[3] = stats.spread as f32 / 10.;
    x[4] = f32::from(stats.drain_percent) / 100.;
    x[5 + attack.damage_type().index()] = 1.;
}
pub(super) fn unit_features(g: &BattleView<'_, '_>, u: &Unit) -> [f32; 34] {
    let mut x = [0.; 34];
    let definition = g.unit_definition(u.kind);
    x[0] = definition.max_hp as f32 / 30.;
    x[1] = g.recruit_cost(u, u.kind) as f32 / 30.;
    x[2] = g.movement_limit(u) as f32 / 10.;
    x[3] = definition.level() as f32 / 5.;
    let experience = u.experience.expect("resolved unit experience");
    x[4] = if definition.max_xp == 0 {
        0.
    } else {
        experience as f32 / definition.max_xp as f32
    };
    x[5] = f32::from(
        definition.max_xp > 0
            && experience >= definition.max_xp
            && !definition.promotion_ids().is_empty(),
    );
    g.with_capabilities(u, u.tile, |caps| {
        let profile = caps.neural_unit_features(
            g.terrain_definition(u.tile),
            Category::Melee,
            DamageType::Blade,
        );
        x[6] = profile.owned_healing_bound as f32 / 10.;
        x[7] = x[6];
        let radius = caps
            .effects()
            .filter(|effect| matches!(effect.value(), EffectValue::Healing { power } if *power > 0))
            .map(|effect| effect.scope().radius)
            .max();
        x[8] = match radius {
            Some(radius) => f32::from(radius) / 10.,
            None => 0.,
        };
        x[9] = profile.reflection_per_strike as f32 / 10.;
        x[10] = profile.owned_thorn_aura_bound as f32 / 10.;
        for (i, kind) in [DamageType::Blade, DamageType::Pierce, DamageType::Impact]
            .into_iter()
            .enumerate()
        {
            x[11 + i] = caps
                .neural_unit_features(g.terrain_definition(u.tile), Category::Melee, kind)
                .armour as f32
                / 10.;
        }
        for (i, category) in [Category::Melee, Category::Ranged].into_iter().enumerate() {
            let attacks = caps.attacks();
            if let Some(attack) = attacks
                .into_iter()
                .filter(|attack| attack.category() == category)
                .max_by(|a, b| {
                    let a = caps.neural_pure_attack_features(*a);
                    let b = caps.neural_pure_attack_features(*b);
                    (a.mean_damage * f64::from(a.strike_count))
                        .total_cmp(&(b.mean_damage * f64::from(b.strike_count)))
                })
            {
                weapon_features(caps, attack, &mut x[14 + i * 8..22 + i * 8]);
            }
        }
    });
    x[30] = u.hp as f32 / definition.max_hp.max(1) as f32;
    x[31] = f32::from(u.moved);
    x[32] = f32::from(u.attack_spent);
    x[33] = f32::from(u.is_leader());
    x
}
fn terrain_features(g: &BattleView<'_, '_>, u: &Unit, tile: usize, x: &mut [f32]) {
    for (i, (category, damage)) in [Category::Melee, Category::Ranged]
        .into_iter()
        .flat_map(|c| {
            [DamageType::Blade, DamageType::Pierce, DamageType::Impact].map(move |d| (c, d))
        })
        .enumerate()
    {
        x[i] = g.with_capabilities(u, tile, |caps| {
            caps.neural_unit_features(g.terrain_definition(tile), category, damage)
                .terrain
        }) as f32
            / 10.;
    }
    let cost = g.step_cost(u, tile);
    x[6] = if cost == u16::MAX {
        -1.
    } else {
        cost as f32 / 10.
    };
    x[7] = g.terrain_healing_at(u, tile) as f32 / 10.;
    x[8] = g.is_keep(tile) as u8 as f32;
    x[9] = g.owns(tile, g.unit_faction(&u)) as u8 as f32;
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
    use crate::logic::BattleSession;
    #[test]
    fn observations_follow_current_stats_and_terrain_without_type_ids() {
        let rules = Catalogue::load().unwrap();
        let game = BattleSession::new(&rules, "mirror");
        let actor = game
            .units()
            .iter()
            .find(|u| u.phase == game.turn())
            .unwrap();
        let baseline = unit_features(&game.view(), actor);
        let mut changed = crate::catalogue_fixture::bundle();
        let kind = actor.kind;
        let increased_healing = (baseline[6] * 10.).round() as i32 + 2;
        crate::effect_fixture::healing_kind(
            &mut changed,
            kind,
            Some((increased_healing, 1, false, HealingTargets::Allies)),
        );
        crate::catalogue_fixture::unit_kind(&mut changed, kind, |unit| {
            let hp = unit["hp"].as_i64().unwrap();
            unit["hp"] = serde_json::json!(hp + 7);
            let blade = unit["damageModifiers"]["blade"].as_i64().unwrap();
            unit["damageModifiers"]["blade"] = serde_json::json!(blade - 1);
        });
        let tile = actor.tile;
        let terrain: serde_json::Value =
            serde_json::from_str(&changed["game/terrain.json"]).unwrap();
        let terrain_role = terrain
            .as_array()
            .unwrap()
            .iter()
            .find(|terrain| terrain["id"] == game.terrain_definition(tile).id().as_str())
            .unwrap()["role"]
            .as_str()
            .unwrap()
            .to_owned();
        crate::catalogue_fixture::unit_kind(&mut changed, kind, |unit| {
            let modifiers = unit["terrainModifiers"].as_object_mut().unwrap().entry(terrain_role).or_insert_with(|| serde_json::json!({"cost":0,"passable":null,"damageModifiers":{"melee":0,"ranged":0,"blade":0,"pierce":0,"impact":0}}));
            let value = modifiers["damageModifiers"]["melee"].as_i64().unwrap();
            modifiers["damageModifiers"]["melee"] = serde_json::json!(value - 2);
        });
        let altered_rules = crate::catalogue_fixture::frozen(&changed);
        let altered = BattleSession::from_state(altered_rules, game.state());
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
        // Cosmetic display metadata does not enter the mechanical profile.
        crate::catalogue_fixture::unit_kind(&mut changed, kind, |unit| {
            unit["name"] = serde_json::json!("Renamed observation probe")
        });
        let renamed_rules = crate::catalogue_fixture::frozen(&changed);
        let renamed = BattleSession::from_state(renamed_rules, game.state());
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
