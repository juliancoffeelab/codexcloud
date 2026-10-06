use crate::model::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Board {
    pub ids: Vec<String>,
    pub positions: Vec<(i32, i32)>,
    pub indices: HashMap<String, usize>,
    pub adjacent: Vec<Vec<usize>>,
    pub distances: Vec<u16>,
}
impl Board {
    pub fn new(tiles: &[Tile]) -> Arc<Self> {
        let ids: Vec<String> = tiles.iter().map(|tile| tile.id.clone()).collect();
        let positions: Vec<_> = tiles.iter().map(|tile| (tile.col, tile.row)).collect();
        let indices = ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.clone(), index))
            .collect();
        let adjacent = (0..tiles.len())
            .map(|first| {
                (0..tiles.len())
                    .filter(|&second| hex_distance(positions[first], positions[second]) == 1)
                    .collect()
            })
            .collect();
        let coordinates = &positions;
        let distances = (0..tiles.len())
            .flat_map(|first| {
                (0..tiles.len())
                    .map(move |second| hex_distance(coordinates[first], coordinates[second]))
            })
            .collect();
        Arc::new(Self {
            ids,
            positions,
            indices,
            adjacent,
            distances,
        })
    }
    pub fn distance(&self, first: usize, second: usize) -> u16 {
        self.distances[first * self.ids.len() + second]
    }
}
fn hex_distance(first: (i32, i32), second: (i32, i32)) -> u16 {
    let q = first.0 - first.1.div_euclid(2) - second.0 + second.1.div_euclid(2);
    let r = first.1 - second.1;
    ((q.abs() + r.abs() + (q + r).abs()) / 2) as u16
}
#[derive(Clone, Debug)]
pub struct Game {
    pub rules: Arc<Rules>,
    pub board: Arc<Board>,
    pub terrain: Vec<usize>,
    pub owners: Vec<Option<Side>>,
    pub units: Vec<Unit>,
    pub map_id: String,
    pub map_revision: u32,
    pub cols: i32,
    pub rows: i32,
    pub round: u32,
    pub turn: Side,
    pub gold: [i32; 2],
    pub winner: Option<Side>,
    pub next_id: u32,
    pub log: Vec<String>,
    pub campaign: Option<CampaignState>,
    pub last_combat: Option<LastCombat>,
    pub combat_present: bool,
    pub pools: Option<[Vec<usize>; 2]>,
    pub arena_first: Option<Side>,
    pub mission: Option<Arc<Mission>>,
    pub recordings: Option<Arc<Mutex<Records>>>,
    route_cache: Arc<Mutex<HashMap<(usize, Vec<usize>, Vec<usize>), Vec<u16>>>>,
}
impl Game {
    pub fn from_state(rules: Arc<Rules>, state: State) -> Self {
        let board = Board::new(&state.tiles);
        let units = state
            .units
            .iter()
            .map(|unit| Unit {
                id: unit.id.strip_prefix('u').unwrap().parse().unwrap(),
                kind: rules.types[&unit.r#type],
                side: unit.side,
                tile: board.indices[&unit.tile],
                hp: unit.hp,
                moved: unit.moved,
                attacked: unit.attacked,
                experience: unit.experience,
                name: unit.name.clone(),
                role: unit.role.clone(),
                retreating: unit.retreating,
                rebuilding: unit.rebuilding,
            })
            .collect();
        let terrain = state
            .tiles
            .iter()
            .map(|tile| rules.terrains[&tile.terrain])
            .collect();
        let owners = state.tiles.iter().map(|tile| tile.owner).collect();
        let combat_present = state.last_combat.is_some();
        let last_combat = state
            .last_combat
            .filter(|v| !v.is_null())
            .map(|v| serde_json::from_value(v).unwrap());
        let mut game = Self {
            rules,
            board,
            terrain,
            owners,
            units,
            map_id: state.map_id,
            map_revision: state.map_revision,
            cols: state.cols,
            rows: state.rows,
            round: state.round,
            turn: state.turn,
            gold: [state.gold.player, state.gold.enemy],
            winner: state.winner,
            next_id: state.next_id,
            log: state.log,
            campaign: state.campaign,
            last_combat,
            combat_present,
            pools: None,
            arena_first: None,
            mission: None,
            recordings: None,
            route_cache: Arc::new(Mutex::new(HashMap::new())),
        };
        game.refresh_mission();
        game
    }
    pub fn new(rules: Arc<Rules>, map: &str) -> Self {
        let state = rules
            .maps
            .iter()
            .filter(|((id, _), _)| id == map)
            .max_by_key(|((_, revision), _)| revision)
            .unwrap()
            .1
            .clone();
        Self::from_state(rules, state)
    }
    pub fn state(&self) -> Value {
        let mut value = json!({"mapRevision":self.map_revision,"mapId":self.map_id,"cols":self.cols,"rows":self.rows,"round":self.round,"turn":self.turn,"gold":{"player":self.gold[0],"enemy":self.gold[1]},"winner":self.winner,"nextId":self.next_id,"log":self.log,"tiles":self.tiles(),"units":self.public_units()});
        if self.combat_present {
            value["lastCombat"] = serde_json::to_value(&self.last_combat).unwrap();
        }
        if let Some(campaign) = &self.campaign {
            value["campaign"] = serde_json::to_value(campaign).unwrap();
        }
        value
    }
    pub fn tiles(&self) -> Vec<Tile> {
        self.board
            .ids
            .iter()
            .enumerate()
            .map(|(index, id)| Tile {
                id: id.clone(),
                col: self.board.positions[index].0,
                row: self.board.positions[index].1,
                terrain: self.rules.terrain_names[self.terrain[index]].clone(),
                owner: self.owners[index],
            })
            .collect()
    }
    pub fn public_unit(&self, unit: &Unit) -> PublicUnit {
        PublicUnit {
            id: format!("u{}", unit.id),
            r#type: self.rules.units[unit.kind].id.clone(),
            side: unit.side,
            tile: self.board.ids[unit.tile].clone(),
            hp: unit.hp,
            moved: unit.moved,
            attacked: unit.attacked,
            experience: unit.experience,
            name: unit.name.clone(),
            role: unit.role.clone(),
            retreating: unit.retreating,
            rebuilding: unit.rebuilding,
        }
    }
    pub fn public_units(&self) -> Vec<PublicUnit> {
        self.units
            .iter()
            .map(|unit| self.public_unit(unit))
            .collect()
    }
    pub fn unit(&self, id: u32) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.id == id)
    }
    pub fn at(&self, tile: usize) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.tile == tile)
    }
    pub fn index(&self, id: u32) -> Option<usize> {
        self.units.iter().position(|unit| unit.id == id)
    }
    pub fn site(&self, id: &str) -> usize {
        self.board.indices[id]
    }
    pub fn kind(&self, name: &str) -> usize {
        self.rules.types[name]
    }
    pub fn terrain_is(&self, tile: usize, name: &str) -> bool {
        self.rules.terrain_names[self.terrain[tile]] == name
    }
    pub fn cost(&self, kind: usize, tile: usize) -> u16 {
        self.rules.units[kind].costs[self.terrain[tile]].unwrap_or(u16::MAX)
    }
    pub fn record(&mut self, message: String) {
        self.log.insert(0, message);
        self.log.truncate(8);
    }
    pub fn add(&mut self, kind: usize, side: Side, tile: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
            id,
            kind,
            side,
            tile,
            hp: self.rules.units[kind].hp,
            moved: false,
            attacked: false,
            experience: Some(0),
            name: None,
            role: None,
            retreating: None,
            rebuilding: None,
        });
        id
    }
    pub fn occupied(&self) -> Vec<bool> {
        let mut occupied = vec![false; self.terrain.len()];
        for unit in &self.units {
            occupied[unit.tile] = true;
        }
        occupied
    }
    pub fn reachable(&self, unit: &Unit) -> Vec<(usize, u16)> {
        if unit.moved || unit.attacked || self.winner.is_some() {
            return vec![];
        }
        self.reach_positions(unit.kind, unit.tile, &self.occupied())
    }
    pub fn reach_positions(
        &self,
        kind: usize,
        start: usize,
        occupied: &[bool],
    ) -> Vec<(usize, u16)> {
        let limit = self.rules.units[kind].r#move;
        let mut costs = vec![u16::MAX; self.terrain.len()];
        let mut order = Vec::new();
        let mut buckets = vec![Vec::new(); limit + 1];
        costs[start] = 0;
        buckets[0].push(start);
        for spent in 0..=limit {
            for cursor in 0..buckets[spent].len() {
                let current = buckets[spent][cursor];
                if costs[current] as usize != spent {
                    continue;
                }
                for &next in &self.board.adjacent[current] {
                    if occupied[next] {
                        continue;
                    }
                    let entry = self.cost(kind, next);
                    if entry == u16::MAX {
                        continue;
                    }
                    let cost = spent as u16 + entry;
                    if cost as usize > limit || cost >= costs[next] {
                        continue;
                    }
                    if costs[next] == u16::MAX {
                        order.push(next);
                    }
                    costs[next] = cost;
                    buckets[cost as usize].push(next);
                }
            }
        }
        order.into_iter().map(|tile| (tile, costs[tile])).collect()
    }
    pub fn routes(&self, kind: usize, goals: &[usize]) -> Vec<u16> {
        let key = (kind, goals.to_vec(), self.terrain.clone());
        if let Some(costs) = self.route_cache.lock().unwrap().get(&key) {
            return costs.clone();
        }
        let mut costs = vec![u16::MAX; self.terrain.len()];
        let mut pending = Vec::new();
        for &goal in goals {
            if costs[goal] == 0 {
                continue;
            }
            costs[goal] = 0;
            pending.push(goal);
        }
        while !pending.is_empty() {
            let index = (0..pending.len())
                .min_by_key(|&i| costs[pending[i]])
                .unwrap();
            let current = pending.remove(index);
            let entry = self.cost(kind, current);
            if entry == u16::MAX {
                continue;
            }
            for &next in &self.board.adjacent[current] {
                if self.cost(kind, next) == u16::MAX {
                    continue;
                }
                let cost = costs[current] + entry;
                if cost >= costs[next] {
                    continue;
                }
                costs[next] = cost;
                if !pending.contains(&next) {
                    pending.push(next);
                }
            }
        }
        let mut cache = self.route_cache.lock().unwrap();
        if cache.len() >= 128 {
            cache.clear();
        }
        cache.insert(key, costs.clone());
        costs
    }
    pub fn can_attack(&self, unit: &Unit, from: usize, target: &Unit, weapon: usize) -> bool {
        if unit.side == target.side {
            return false;
        }
        let attack = &self.rules.units[unit.kind].attacks[weapon];
        let gap = self.board.distance(from, target.tile);
        if gap < 1 || gap > attack.range as u16 {
            return false;
        }
        attack.category != Category::Ranged
            || gap != 2
            || self.board.adjacent[from].iter().any(|&between| {
                self.board.distance(between, target.tile) == 1
                    && !self.terrain_is(between, "forest")
                    && !self.terrain_is(between, "hill")
                    && !self.terrain_is(between, "grove")
            })
    }
    pub fn attack_options(&self, unit: &Unit, target: &Unit, from: usize) -> Vec<usize> {
        (0..self.rules.units[unit.kind].attacks.len())
            .filter(|&weapon| self.can_attack(unit, from, target, weapon))
            .collect()
    }
    pub fn damage(&self, attacker: &Unit, target: &Unit, weapon: usize) -> i32 {
        let attack = &self.rules.units[attacker.kind].attacks[weapon];
        let rule = &self.rules.units[target.kind];
        (attack.damage
            + rule.damage_modifiers.get(attack.damage_type)
            + rule.environment[self.terrain[target.tile]][attack.category.index()]
                [attack.damage_type.index()])
        .max(0)
    }
    pub fn preview(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> (i32, i32, Option<usize>) {
        let damage = self.damage(unit, target, weapon);
        if target.hp <= damage {
            return (damage, 0, None);
        }
        let mut actor = unit.clone();
        actor.tile = from;
        let category = self.rules.units[unit.kind].attacks[weapon].category;
        let mut best = None;
        let mut response = 0;
        for attack in self.attack_options(target, &actor, target.tile) {
            if self.rules.units[target.kind].attacks[attack].category != category {
                continue;
            }
            let dealt = self.damage(target, &actor, attack);
            if best.is_none() || dealt > response {
                best = Some(attack);
                response = dealt;
            }
        }
        (damage, response, best)
    }
    pub fn best_attack(&self, unit: &Unit, target: &Unit, from: usize) -> Option<usize> {
        let mut best = None;
        let mut score = f64::NEG_INFINITY;
        for attack in self.attack_options(unit, target, from) {
            let (damage, retaliation, _) = self.preview(unit, target, attack, from);
            let value = damage as f64 + if damage >= target.hp { 18.0 } else { 0.0 }
                - retaliation as f64 * 0.7
                - if retaliation >= unit.hp { 40.0 } else { 0.0 };
            if value > score {
                score = value;
                best = Some(attack);
            }
        }
        best
    }
    pub fn income(&self, side: Side) -> i32 {
        self.owners
            .iter()
            .enumerate()
            .filter(|(tile, owner)| **owner == Some(side) && self.terrain_is(*tile, "village"))
            .count() as i32
            * 2
    }
    pub fn refresh_mission(&mut self) {
        let Some(state) = &self.campaign else {
            self.mission = None;
            return;
        };
        let mut mission = self.rules.campaigns[&state.id].missions[state.mission].clone();
        if state.chosen("refuge-war")
            || state.chosen("green-dain-dead")
            || state.marked("dain-fell")
        {
            if let Some(patch) = mission.without_dain.clone() {
                Arc::make_mut(&mut mission).apply(&patch);
            }
        }
        let mut fallen = state.fallen_companions.clone().unwrap_or_default();
        for mark in &state.marks {
            if let Some(name) = mark.strip_prefix("fallen:") {
                if !fallen.iter().any(|n| n == name) {
                    fallen.push(name.to_owned());
                }
            }
        }
        for name in &fallen {
            if let Some(patch) = mission
                .companion_losses
                .as_ref()
                .and_then(|patches| patches.get(name))
                .cloned()
            {
                Arc::make_mut(&mut mission).apply(&patch);
            }
        }
        if mission
            .intro
            .iter()
            .chain(&mission.victory)
            .any(|line| fallen.contains(&line.speaker))
        {
            let changed = Arc::make_mut(&mut mission);
            changed.intro.retain(|line| !fallen.contains(&line.speaker));
            changed
                .victory
                .retain(|line| !fallen.contains(&line.speaker));
        }
        if state.escalated() {
            if let Some(refuge) = mission.refuge.clone() {
                let changed = Arc::make_mut(&mut mission);
                changed.briefing = refuge.war_briefing;
                changed.hints = refuge.war_hints;
            }
        }
        self.mission = Some(mission);
    }
}

impl Game {
    pub fn fork(&self) -> Self {
        let mut game = self.clone();
        game.recordings = None;
        game
    }
    pub fn action(&self, value: Value) {
        if let Some(records) = &self.recordings {
            records.lock().unwrap().actions.push(value);
        }
    }
    pub fn promotion_options(&self, unit: &Unit) -> Vec<usize> {
        if self.winner.is_some() || unit.side != self.turn || unit.moved || unit.attacked {
            return vec![];
        }
        let rule = &self.rules.units[unit.kind];
        if rule
            .experience_needed
            .is_none_or(|needed| unit.experience.unwrap_or(0) < needed)
        {
            return vec![];
        }
        rule.advances_to
            .iter()
            .map(|name| self.kind(name))
            .filter(|&kind| self.cost(kind, unit.tile) != u16::MAX)
            .collect()
    }
    pub fn advance(&mut self, id: u32, kind: usize) {
        let index = self.index(id).unwrap();
        assert!(self.promotion_options(&self.units[index]).contains(&kind));
        let previous = &self.rules.units[self.units[index].kind];
        let name = self.units[index]
            .name
            .clone()
            .unwrap_or(previous.name.clone());
        let wounds = previous.hp - self.units[index].hp;
        let experience = self.units[index]
            .experience
            .unwrap_or(0)
            .saturating_sub(previous.experience_needed.unwrap_or(0));
        let unit = &mut self.units[index];
        unit.kind = kind;
        unit.experience = Some(experience);
        unit.hp = (self.rules.units[kind].hp - wounds).max(1);
        unit.moved = true;
        unit.attacked = true;
        self.record(format!(
            "{} advanced to {}.",
            name, self.rules.units[kind].name
        ));
        self.action(
            json!({"kind":"advance","unit":format!("u{id}"),"type":self.rules.units[kind].id}),
        );
    }
    pub fn move_unit(&mut self, id: u32, tile: usize) {
        let index = self.index(id).unwrap();
        let actor = self.units[index].clone();
        assert!(
            actor.side == self.turn
                && self.winner.is_none()
                && actor.tile != tile
                && self.reachable(&actor).iter().any(|&(site, _)| site == tile)
        );
        self.units[index].tile = tile;
        self.units[index].moved = true;
        if ["village", "grove", "fort", "ruins"]
            .iter()
            .any(|name| self.terrain_is(tile, name))
            && self.owners[tile] != Some(actor.side)
        {
            self.owners[tile] = Some(actor.side);
            let unit_name = self.rules.units[actor.kind].name.to_lowercase();
            let capture = if self.terrain_is(tile, "village") {
                "captured a village".to_owned()
            } else {
                format!(
                    "claimed {}",
                    self.rules.terrain_labels[self.terrain[tile]].to_lowercase()
                )
            };
            self.record(format!(
                "{} {unit_name} {capture}.",
                if actor.side == Side::Player {
                    "Your"
                } else {
                    "Enemy"
                }
            ));
        }
        self.evaluate_campaign(
            false,
            if actor.side == Side::Player {
                Some(tile)
            } else {
                None
            },
        );
        self.action(json!({"kind":"move","unit":format!("u{id}"),"tile":self.board.ids[tile]}));
    }
    pub fn mediation_at_risk(&self, target: &Unit) -> bool {
        self.campaign
            .as_ref()
            .is_some_and(|state| !state.escalated())
            && self
                .mission
                .as_ref()
                .is_some_and(|mission| mission.refuge.is_some())
            && target.side == Side::Enemy
            && (target.retreating == Some(true) || target.role.as_deref() == Some("leader"))
    }
    pub fn attack(&mut self, id: u32, target_id: u32, weapon: usize) -> (i32, i32) {
        let a = self.index(id).unwrap();
        let b = self.index(target_id).unwrap();
        let attacker = self.units[a].clone();
        let target = self.units[b].clone();
        assert!(
            attacker.side == self.turn
                && !attacker.attacked
                && self.winner.is_none()
                && self.can_attack(&attacker, attacker.tile, &target, weapon)
        );
        let (damage, retaliation, _) = self.preview(&attacker, &target, weapon, attacker.tile);
        let mediated = self.mediation_at_risk(&target);
        let attack = self.rules.units[attacker.kind].attacks[weapon].clone();
        let before = [self.public_unit(&attacker), self.public_unit(&target)];
        self.last_combat = Some(LastCombat {
            attacker_id: format!("u{id}"),
            attack_id: attack.id.clone(),
            sound: attack.sound.clone(),
        });
        self.combat_present = true;
        self.units[a].moved = true;
        self.units[a].attacked = true;
        self.units[b].hp -= damage;
        if self.units[b].hp > 0 {
            self.units[a].hp -= retaliation;
        }
        for (own, opponent, dealt) in [(a, b, damage), (b, a, retaliation)] {
            let limit = self.rules.units[self.units[own].kind]
                .experience_needed
                .unwrap_or(0);
            if self.units[own].hp > 0 && dealt > 0 && limit > 0 {
                let reward = if self.units[opponent].hp <= 0 {
                    6 * self.rules.units[self.units[opponent].kind].level
                } else {
                    1
                };
                self.units[own].experience =
                    Some(limit.min(self.units[own].experience.unwrap_or(0) + reward));
            }
        }
        let message = if retaliation > 0 {
            format!(" and took {retaliation} in retaliation")
        } else {
            String::new()
        };
        self.record(format!(
            "{} {} used {} for {damage} damage{message}.",
            if attacker.side == Side::Player {
                "Your"
            } else {
                "Enemy"
            },
            self.rules.units[attacker.kind].name.to_lowercase(),
            attack.name.to_lowercase()
        ));
        let defeated: Vec<Unit> = self
            .units
            .iter()
            .filter(|unit| unit.hp <= 0)
            .cloned()
            .collect();
        for unit in &defeated {
            if let Some(campaign) = &mut self.campaign {
                if unit.side == Side::Player {
                    if let Some(name) = &unit.name {
                        if self.rules.campaigns[&campaign.id]
                            .missions
                            .iter()
                            .any(|chapter| {
                                chapter.map.units.iter().any(|incoming| {
                                    incoming.side == Side::Player
                                        && incoming.name.as_ref() == Some(name)
                                })
                            })
                        {
                            let mark = format!("fallen:{name}");
                            if !campaign.marked(&mark) {
                                campaign.marks.push(mark);
                            }
                        }
                    }
                }
                if campaign.id == "green-accord"
                    && unit.name.as_deref() == Some("Captain Dain")
                    && !campaign.marked("dain-fell")
                {
                    campaign.marks.push("dain-fell".into());
                }
            }
            if self.mediation_at_risk(unit) {
                self.escalate_refuge();
            }
            if self.rules.units[unit.kind].leader && self.campaign.is_none() {
                self.winner = Some(unit.side.other());
            }
            self.record(format!(
                "{} {} fell.",
                if unit.side == Side::Player {
                    "Your"
                } else {
                    "Enemy"
                },
                self.rules.units[unit.kind].name.to_lowercase()
            ));
        }
        self.units.retain(|unit| unit.hp > 0);
        self.refresh_mission();
        self.evaluate_campaign(false, None);
        self.action(json!({"kind":"attack","unit":format!("u{id}"),"target":format!("u{target_id}"),"weapon":attack.id}));
        if let Some(records) = &self.recordings {
            let mut records = records.lock().unwrap();
            records.losses.extend(before.into_iter().filter(|unit| {
                !self
                    .units
                    .iter()
                    .any(|alive| format!("u{}", alive.id) == unit.id)
            }));
            records.combats.push(json!({"unit":format!("u{id}"),"target":format!("u{target_id}"),"weapon":attack.id,"from":self.board.ids[attacker.tile],"targetTile":self.board.ids[target.tile],"damage":damage,"retaliation":retaliation,"mediationAtRisk":mediated}));
        }
        (damage, retaliation)
    }
    pub fn recruitment_options(&self, side: Side) -> Vec<usize> {
        if let Some(pools) = &self.pools {
            return pools[side.index()].clone();
        }
        if let Some(mission) = &self.mission {
            return if side == Side::Player {
                &mission.recruits
            } else {
                &mission.enemy_recruits
            }
            .iter()
            .map(|name| self.kind(name))
            .collect();
        }
        ["swordsman", "spearman", "archer", "scout"]
            .iter()
            .map(|name| self.kind(name))
            .collect()
    }
    pub fn placement_tiles(&self, leader: &Unit, kind: usize, price: bool) -> Vec<usize> {
        if !self.rules.units[leader.kind].leader
            || self.rules.units[kind].leader
            || !self.recruitment_options(leader.side).contains(&kind)
            || self.winner.is_some()
            || leader.side != self.turn
            || !self.terrain_is(leader.tile, "keep")
            || self.owners[leader.tile] != Some(leader.side)
            || (price && self.gold[leader.side.index()] < self.rules.units[kind].cost)
        {
            return vec![];
        }
        self.board.adjacent[leader.tile]
            .iter()
            .copied()
            .filter(|&tile| self.cost(kind, tile) != u16::MAX && self.at(tile).is_none())
            .collect()
    }
    pub fn recruit(&mut self, id: u32, kind: usize, tile: usize) -> u32 {
        let leader = self.unit(id).unwrap().clone();
        assert!(self.placement_tiles(&leader, kind, true).contains(&tile));
        self.gold[leader.side.index()] -= self.rules.units[kind].cost;
        let added = self.add(kind, leader.side, tile);
        let unit = self.units.last_mut().unwrap();
        unit.moved = true;
        unit.attacked = true;
        self.record(format!(
            "{} recruited a {}. It acts next turn.",
            if leader.side == Side::Player {
                "You"
            } else {
                "Enemy"
            },
            self.rules.units[kind].name.to_lowercase()
        ));
        self.action(json!({"kind":"recruit","unit":format!("u{id}"),"type":self.rules.units[kind].id,"tile":self.board.ids[tile]}));
        added
    }
    pub fn begin_turn(&mut self, side: Side) {
        self.turn = side;
        self.last_combat = None;
        self.combat_present = true;
        self.gold[side.index()] += self.income(side);
        for unit in &mut self.units {
            if unit.side != side {
                continue;
            }
            unit.moved = false;
            unit.attacked = false;
            let terrain = &self.rules.terrain_names[self.terrain[unit.tile]];
            if (terrain == "village" || terrain == "grove") && self.owners[unit.tile] == Some(side)
            {
                unit.hp = (unit.hp + 2).min(self.rules.units[unit.kind].hp);
            }
        }
        for index in 0..self.units.len() {
            if self.units[index].side != side {
                continue;
            }
            let healing = self
                .units
                .iter()
                .filter(|ally| {
                    ally.side == side
                        && ally.id != self.units[index].id
                        && self.board.distance(ally.tile, self.units[index].tile) == 1
                })
                .map(|ally| self.rules.units[ally.kind].healing)
                .max()
                .unwrap_or(0);
            self.units[index].hp =
                (self.units[index].hp + healing).min(self.rules.units[self.units[index].kind].hp);
        }
        if side == Side::Enemy {
            self.spawn_waves();
        }
        self.evaluate_campaign(side == Side::Player, None);
    }
    pub fn end_turn(&mut self) {
        assert!(self.winner.is_none());
        if self.arena_first.is_some() {
            if Some(self.turn) != self.arena_first {
                self.round += 1;
            }
        } else if self.turn == Side::Enemy {
            self.round += 1;
        }
        self.begin_turn(self.turn.other());
    }
}
