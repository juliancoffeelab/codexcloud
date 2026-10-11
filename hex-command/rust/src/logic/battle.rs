//! Compact battle sessions borrowing catalog rules and indexing the live roster.

use crate::data::*;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct Board {
    pub(crate) ids: Vec<String>,
    pub(crate) positions: Vec<(i32, i32)>,
    pub(crate) indices: HashMap<String, usize>,
    pub(crate) adjacent: Vec<Vec<usize>>,
    pub(crate) distances: Vec<u16>,
}
impl Board {
    pub(crate) fn new(tiles: &[Tile]) -> Rc<Self> {
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
        Rc::new(Self {
            ids,
            positions,
            indices,
            adjacent,
            distances,
        })
    }
    pub(crate) fn distance(&self, first: usize, second: usize) -> u16 {
        self.distances[first * self.ids.len() + second]
    }
}
fn hex_distance(first: (i32, i32), second: (i32, i32)) -> u16 {
    let q = first.0 - first.1.div_euclid(2) - second.0 + second.1.div_euclid(2);
    let r = first.1 - second.1;
    ((q.abs() + r.abs() + (q + r).abs()) / 2) as u16
}
#[derive(Debug)]
pub(crate) struct Game<'catalog> {
    pub(crate) rules: &'catalog Rules,
    pub(crate) board: Rc<Board>,
    pub(crate) terrain: Vec<usize>,
    pub(crate) owners: Vec<Option<Side>>,
    pub(crate) units: Vec<Unit>,
    pub(crate) map_id: String,
    pub(crate) map_revision: u32,
    pub(crate) cols: i32,
    pub(crate) rows: i32,
    pub(crate) round: u32,
    pub(crate) turn: Side,
    pub(crate) gold: [i32; 2],
    pub(crate) winner: Option<Side>,
    pub(crate) next_id: u32,
    pub(crate) log: Vec<String>,
    pub(crate) campaign: Option<CampaignState>,
    pub(crate) last_combat: Option<LastCombat>,
    pub(crate) combat_present: bool,
    pub(crate) pools: Option<[Vec<usize>; 2]>,
    pub(crate) arena_first: Option<Side>,
    pub(crate) mission: Option<Rc<Mission>>,
    pub(crate) recordings: Option<Records>,
    pub(super) route_cache: RefCell<HashMap<(usize, Vec<usize>, u64), Rc<Vec<u16>>>>,
    pub(super) terrain_revision: u64,
    pub(super) attack_geometry: super::geometry::AttackGeometry,
    pub(super) route_order: RefCell<VecDeque<(usize, Vec<usize>, u64)>>,
    pub(super) occupancy: Vec<Option<usize>>,
    pub(super) unit_indices: HashMap<u32, usize>,
}
impl<'catalog> Game<'catalog> {
    pub(crate) fn from_state(rules: &'catalog Rules, state: State) -> Self {
        let board = Board::new(&state.tiles);
        Self::from_state_with_board(rules, state, board)
    }
    pub(crate) fn from_state_with_board(
        rules: &'catalog Rules,
        state: State,
        board: Rc<Board>,
    ) -> Self {
        let units: Vec<Unit> = state
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
        let last_combat = state.last_combat.flatten();
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
            route_cache: RefCell::new(HashMap::new()),
            terrain_revision: 0,
            attack_geometry: Rc::new(RefCell::new(HashMap::new())),
            route_order: RefCell::new(VecDeque::new()),
            occupancy: vec![],
            unit_indices: HashMap::new(),
        };
        game.roster_changed();
        game.refresh_mission();
        game
    }
    pub(crate) fn new(rules: &'catalog Rules, map: &str) -> Self {
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
    /// Materialize the protocol snapshot without changing battle state.
    pub(crate) fn state(&self) -> State {
        State {
            map_id: self.map_id.clone(),
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: Gold {
                player: self.gold[0],
                enemy: self.gold[1],
            },
            winner: self.winner,
            next_id: self.next_id,
            log: self.log.clone(),
            tiles: self.tiles(),
            units: self.public_units(),
            campaign: self.campaign.clone(),
            last_combat: self.combat_present.then(|| self.last_combat.clone()),
        }
    }
    pub(crate) fn tiles(&self) -> Vec<Tile> {
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
    pub(crate) fn public_unit(&self, unit: &Unit) -> PublicUnit {
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
    pub(crate) fn public_units(&self) -> Vec<PublicUnit> {
        self.units
            .iter()
            .map(|unit| self.public_unit(unit))
            .collect()
    }
    pub(crate) fn unit(&self, id: u32) -> Option<&Unit> {
        self.index(id).map(|index| &self.units[index])
    }
    pub(crate) fn at(&self, tile: usize) -> Option<&Unit> {
        self.occupancy
            .get(tile)
            .copied()
            .flatten()
            .and_then(|index| self.units.get(index))
            .filter(|unit| unit.tile == tile)
    }
    pub(crate) fn index(&self, id: u32) -> Option<usize> {
        self.unit_indices
            .get(&id)
            .copied()
            .filter(|&index| self.units.get(index).is_some_and(|u| u.id == id))
    }
    pub(crate) fn site(&self, id: &str) -> usize {
        self.board.indices[id]
    }
    pub(crate) fn kind(&self, name: &str) -> usize {
        self.rules.types[name]
    }
    pub(crate) fn terrain_is(&self, tile: usize, terrain: Terrain) -> bool {
        self.rules.terrain_types[self.terrain[tile]] == terrain
    }
    pub(crate) fn record(&mut self, message: String) {
        self.log.insert(0, message);
        self.log.truncate(8);
    }
    pub(crate) fn add(&mut self, kind: usize, side: Side, tile: usize) -> u32 {
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
        self.unit_indices.insert(id, self.units.len() - 1);
        self.occupancy[tile] = Some(self.units.len() - 1);
        id
    }
    /// Refresh dense lookup tables after an internal roster transformation.
    pub(crate) fn roster_changed(&mut self) {
        self.occupancy = vec![None; self.terrain.len()];
        self.unit_indices.clear();
        for (index, unit) in self.units.iter().enumerate() {
            self.occupancy[unit.tile] = Some(index);
            self.unit_indices.insert(unit.id, index);
        }
    }
    /// Change terrain and invalidate only terrain-dependent route results.
    #[cfg(test)]
    pub(crate) fn set_terrain(&mut self, tile: usize, terrain: usize) {
        if self.terrain[tile] != terrain {
            self.terrain[tile] = terrain;
            self.attack_geometry = Rc::new(RefCell::new(HashMap::new()));
            self.terrain_revision = self.terrain_revision.wrapping_add(1);
            self.route_cache.get_mut().clear();
            self.route_order.get_mut().clear();
        }
    }

    pub(crate) fn refresh_mission(&mut self) {
        let Some(state) = &self.campaign else {
            self.mission = None;
            return;
        };
        let mut mission = Rc::new(self.rules.campaigns[&state.id].missions[state.mission].clone());
        if state.chosen("refuge-war")
            || state.chosen("green-dain-dead")
            || state.marked("dain-fell")
        {
            if let Some(patch) = mission.without_dain.clone() {
                Rc::make_mut(&mut mission).apply(&patch);
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
                Rc::make_mut(&mut mission).apply(&patch);
            }
        }
        if mission
            .intro
            .iter()
            .chain(&mission.victory)
            .any(|line| fallen.contains(&line.speaker))
        {
            let changed = Rc::make_mut(&mut mission);
            changed.intro.retain(|line| !fallen.contains(&line.speaker));
            changed
                .victory
                .retain(|line| !fallen.contains(&line.speaker));
        }
        if state.escalated() {
            if let Some(refuge) = mission.refuge.clone() {
                let changed = Rc::make_mut(&mut mission);
                changed.briefing = refuge.war_briefing;
                changed.hints = refuge.war_hints;
            }
        }
        self.mission = Some(mission);
    }
}

impl<'catalog> Game<'catalog> {
    pub(crate) fn fork(&self) -> Self {
        Self {
            rules: self.rules,
            board: self.board.clone(),
            terrain: self.terrain.clone(),
            owners: self.owners.clone(),
            units: self.units.clone(),
            map_id: self.map_id.clone(),
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: self.gold,
            winner: self.winner,
            next_id: self.next_id,
            log: self.log.clone(),
            campaign: self.campaign.clone(),
            last_combat: self.last_combat.clone(),
            combat_present: self.combat_present,
            pools: self.pools.clone(),
            arena_first: self.arena_first,
            mission: self.mission.clone(),
            recordings: None,
            route_cache: RefCell::new(HashMap::new()),
            terrain_revision: self.terrain_revision,
            attack_geometry: self.attack_geometry.clone(),
            route_order: RefCell::new(VecDeque::new()),
            occupancy: self.occupancy.clone(),
            unit_indices: self.unit_indices.clone(),
        }
    }
    pub(crate) fn action(&mut self, value: ActionRecord) {
        if let Some(records) = &mut self.recordings {
            records.actions.push(value);
        }
    }
    pub(crate) fn promotion_options(&self, unit: &Unit) -> Vec<usize> {
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
    pub(crate) fn advance(&mut self, id: u32, kind: usize) {
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
        self.action(ActionRecord::Advance {
            unit: format!("u{id}"),
            unit_type: self.rules.units[kind].id.clone(),
        });
    }

    pub(crate) fn recruitment_options(&self, side: Side) -> Vec<usize> {
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
        ["swordsman", "spearman", "archer", "scout", "rider"]
            .iter()
            .map(|name| self.kind(name))
            .collect()
    }
    pub(crate) fn placement_tiles(&self, leader: &Unit, kind: usize, price: bool) -> Vec<usize> {
        if !self.rules.units[leader.kind].leader
            || self.rules.units[kind].leader
            || !self.recruitment_options(leader.side).contains(&kind)
            || self.winner.is_some()
            || leader.side != self.turn
            || !self.terrain_is(leader.tile, Terrain::Keep)
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
    pub(crate) fn recruit(&mut self, id: u32, kind: usize, tile: usize) -> u32 {
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
        self.action(ActionRecord::Recruit {
            unit: format!("u{id}"),
            unit_type: self.rules.units[kind].id.clone(),
            tile: self.board.ids[tile].clone(),
        });
        added
    }
}
