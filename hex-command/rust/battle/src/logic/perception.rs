//! Shared perception for units with limited visibility.
//!
//! The intent is to put players, rule-based AI and neural-network AI on roughly
//! equal footing through the same visibility rules and permitted knowledge.
//!
//! AI nudges may prevent stalls or trivial play under obstructed visibility, but
//! they are explicit exceptions. A human must authorize each exception and the
//! way it obtains and uses information beyond ordinary perception.
//!
//! Visibility rules and authorized exceptions are authored in JSON. Rust
//! interprets those definitions and executes the shared mechanics, rather than
//! embedding unit-, map- or controller-specific visibility policy.
//!
//! Detection operates on physical facts without evaluating campaign flags or
//! callbacks.
use super::BattlePosition;
use crate::data::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedFixedLight {
    pub tile: usize,
    pub emission: Option<LightEmission>,
    pub appearance: BattlefieldLight,
}

pub(crate) struct Perception<'a, 'catalog> {
    game: &'a BattlePosition<'catalog>,
    positions: [Option<(u32, usize)>; 3],
}
impl BattlePosition<'_> {
    #[inline]
    pub(crate) fn vision_group(&self, faction: &str) -> VisionGroupKey {
        let id = &self
            .faction(faction)
            .expect("validated faction")
            .vision_group;
        VisionGroupKey(
            self.vision_groups
                .iter()
                .position(|g| &g.id == id)
                .expect("validated vision group"),
        )
    }
    #[inline]
    pub(crate) fn unit_vision_group(&self, unit: &Unit) -> VisionGroupKey {
        unit.vision_group
    }
    #[inline]
    pub(crate) fn perception(&self, position: Option<(u32, usize)>) -> Perception<'_, '_> {
        Perception {
            game: self,
            positions: [position, None, None],
        }
    }
    pub(crate) fn reveal_weapon_user(&mut self, id: u32, recipient: VisionGroupKey) {
        self.reveal_contact(id, recipient);
    }
    pub(crate) fn reveal_contact(&mut self, id: u32, recipient: VisionGroupKey) {
        let slot = self.unit(id).expect("weapon user").perception.0;
        if !self.unit_perception[slot].revealed_to.contains(&recipient) {
            self.runtime.unit_perception[slot]
                .revealed_to
                .push(recipient);
        }
    }
    pub(crate) fn fresh_perception(&mut self) -> PerceptionId {
        let id = PerceptionId(self.unit_perception.len());
        self.runtime
            .unit_perception
            .push(UnitPerceptionFacts::default());
        id
    }
}
impl<'a, 'catalog> Perception<'a, 'catalog> {
    #[inline]
    pub(crate) fn with_position(mut self, id: u32, tile: usize) -> Self {
        let slot = self
            .positions
            .iter()
            .position(|p| p.is_some_and(|(unit, _)| unit == id))
            .or_else(|| self.positions.iter().position(Option::is_none))
            .expect("three projected positions");
        self.positions[slot] = Some((id, tile));
        self
    }
    #[inline]
    fn tile(&self, unit: &Unit) -> usize {
        self.positions
            .iter()
            .flatten()
            .find(|(id, _)| *id == unit.id)
            .map_or(unit.tile, |(_, tile)| *tile)
    }
    fn profile(&self, unit: &Unit) -> ResolvedSightProfile {
        let group = &self.game.vision_groups[self.game.unit_vision_group(unit).0];
        let ability = &self.game.rules.units[unit.kind].perception;
        let mut penetrates = ConcealmentMask::of(&group.sight.penetrates)
            .union(ConcealmentMask::of(&ability.penetrates));
        for effect in &self.game.unit_perception[unit.perception.0].effects {
            penetrates = penetrates.union(ConcealmentMask::of(&effect.penetrates));
        }
        let mut range = ability.range.resolve(group.sight.range.resolve(None));
        for item in self.game.inventory(unit) {
            if let Some(perception) = &item.perception {
                penetrates = penetrates.union(ConcealmentMask::of(&perception.penetrates));
                match perception.range {
                    SightRange::Extent(SightExtent::Inherited) => {}
                    SightRange::Extent(SightExtent::Unlimited) => range = None,
                    SightRange::Range(item) => {
                        if let Some(base) = &mut range {
                            *base = (*base).max(item);
                        }
                    }
                }
            }
        }
        ResolvedSightProfile { range, penetrates }
    }
    fn applies(&self, concealment: &UnitConcealment, tile: usize) -> bool {
        concealment.applies(
            self.game.rules.terrain_roles[self.game.terrain[tile]],
            || self.illuminated(tile),
        )
    }

    pub(crate) fn unit_lights(&self, unit: &Unit, mut visit: impl FnMut(&LightEmission)) {
        if let Some(light) = &self.game.rules.units[unit.kind].perception.light {
            visit(light);
        }
        for effect in &self.game.unit_perception[unit.perception.0].effects {
            if let Some(light) = &effect.light {
                visit(light);
            }
        }
        for item in self.game.inventory(unit) {
            if let Some(light) = item.perception.as_ref().and_then(|p| p.light.as_ref()) {
                visit(light);
            }
        }
    }
    /// Shared active sources. Carrier visibility never controls emitted light.
    pub(crate) fn visit_light_sources(
        &self,
        mut visit: impl FnMut(
            Option<&Unit>,
            usize,
            Option<&LightEmission>,
            Option<&BattlefieldLight>,
        ) -> bool,
    ) {
        for source in self.game.fixed_lights.iter() {
            if visit(
                None,
                source.tile,
                source.emission.as_ref(),
                Some(&source.appearance),
            ) {
                return;
            }
        }
        for unit in self.game.units.iter() {
            let mut stopped = false;
            self.unit_lights(unit, |emission| {
                if !stopped {
                    stopped = visit(Some(unit), self.tile(unit), Some(emission), None);
                }
            });
            if stopped {
                return;
            }
        }
    }
    pub(crate) fn illuminated(&self, tile: usize) -> bool {
        if self.game.observed_light.contains(tile) {
            return true;
        }
        let mut lit = false;
        self.visit_light_sources(|_, origin, emission, _| {
            if !lit {
                lit = emission.is_some_and(|e| self.game.board.distance(origin, tile) <= e.radius);
            }
            lit
        });
        lit
    }
    /// Indicator eligibility and physically active cover are distinct. An authored
    /// visual rule may be defeated by light or by leaving its eligible terrain.
    pub(crate) fn visual_cover(&self, unit: &Unit) -> (bool, bool) {
        let profile = &self.game.rules.units[unit.kind].perception;
        let facts = &self.game.unit_perception[unit.perception.0];
        let rules = self.game.tile_perception[self.tile(unit)]
            .concealment
            .iter()
            .chain(&profile.concealment)
            .chain(facts.effects.iter().flat_map(|e| &e.concealment))
            .chain(
                self.game
                    .inventory(unit)
                    .filter_map(|i| i.perception.as_ref())
                    .flat_map(|p| &p.concealment),
            );
        let mut eligible = false;
        let mut active = false;
        for rule in rules.filter(|r| self.game.rules.concealment_visuals.contains(r.kind)) {
            eligible = true;
            active |= self.applies(rule, self.tile(unit));
        }
        (eligible, active)
    }
    pub(crate) fn fixed_detection_facts(
        &self,
        group: VisionGroupKey,
        origin: usize,
        profile: ResolvedSightProfile,
        target: &Unit,
    ) -> DetectionFacts {
        self.detect_from(group, origin, profile, Some(target), self.tile(target))
    }
    /// Spatial lighting changes with the projected carrier's position as well as
    /// the target. This same function is used by live attacks and threat probes.
    pub(crate) fn detection_facts(
        &self,
        source: &Unit,
        target: Option<&Unit>,
        tile: usize,
    ) -> DetectionFacts {
        self.detect_from(
            self.game.unit_vision_group(source),
            self.tile(source),
            self.profile(source),
            target,
            tile,
        )
    }
    #[inline]
    fn remembered(&self, group: VisionGroupKey, tile: usize) -> bool {
        self.game.tile_perception[tile].remember
            && self.game.fog_memory[&self.game.vision_groups[group.0].id]
                .get(tile / 64)
                .is_some_and(|bits| bits & (1 << (tile % 64)) != 0)
    }
    fn detect_from(
        &self,
        group: VisionGroupKey,
        origin: usize,
        profile: ResolvedSightProfile,
        target: Option<&Unit>,
        tile: usize,
    ) -> DetectionFacts {
        let distance = self.game.board.distance(origin, tile);
        let revealed = target.is_some_and(|unit| {
            self.game.unit_perception[unit.perception.0]
                .revealed_to
                .contains(&group)
        });
        if profile.range.is_some_and(|range| distance > range) {
            return DetectionFacts {
                in_range: false,
                revealed,
                ..Default::default()
            };
        }
        let mut facts = detects(
            profile,
            distance,
            self.game.tile_perception[tile]
                .concealment
                .iter()
                .filter(|rule| !self.remembered(group, tile) && self.applies(rule, tile)),
        );
        facts.revealed = revealed;
        if let Some(unit) = target {
            let ability = &self.game.rules.units[unit.kind].perception;
            let apply = |c: &&UnitConcealment| self.applies(c, tile);
            let abilities = ability.concealment.iter().filter(apply);
            let effects = self.game.unit_perception[unit.perception.0]
                .effects
                .iter()
                .flat_map(|e| &e.concealment)
                .filter(apply);
            let equipment = self
                .game
                .inventory(unit)
                .filter_map(|i| i.perception.as_ref())
                .flat_map(|p| &p.concealment)
                .filter(apply);
            let personal = detects(profile, distance, abilities.chain(effects).chain(equipment));
            facts.active = facts.active.union(personal.active);
            facts.blocking = facts.blocking.union(personal.blocking);
        }
        facts
    }
    pub(crate) fn sees_terrain(&self, group: VisionGroupKey, tile: usize) -> bool {
        self.game.fixed_sights.iter().any(|(origin, g, profile)| {
            *g == group
                && self
                    .detect_from(group, *origin, *profile, None, tile)
                    .detected()
        }) || self.game.units.iter().any(|source| {
            self.game.unit_vision_group(source) == group
                && self.detection_facts(source, None, tile).detected()
        })
    }
    pub(crate) fn sees_unit(&self, group: VisionGroupKey, unit: &Unit) -> bool {
        self.game.unit_vision_group(unit) == group
            || self.game.unit_perception[unit.perception.0]
                .revealed_to
                .contains(&group)
            || self.game.fixed_sights.iter().any(|(origin, g, profile)| {
                *g == group
                    && self
                        .detect_from(group, *origin, *profile, Some(unit), self.tile(unit))
                        .detected()
            })
            || self.game.units.iter().any(|source| {
                self.game.unit_vision_group(source) == group
                    && self
                        .detection_facts(source, Some(unit), self.tile(unit))
                        .detected()
            })
    }
    pub(crate) fn terrain_knowledge(&self, group: VisionGroupKey, tile: usize) -> TerrainKnowledge {
        if self.sees_terrain(group, tile) {
            TerrainKnowledge::Visible
        } else if self.remembered(group, tile) {
            TerrainKnowledge::Remembered
        } else {
            TerrainKnowledge::Unexplored
        }
    }
}
impl BattlePosition<'_> {
    fn perception_area(&self, area: &PerceptionArea) -> Vec<usize> {
        match area {
            PerceptionArea::Map => (0..self.terrain.len()).collect(),
            PerceptionArea::Tiles(tiles) => tiles
                .iter()
                .map(|t| self.board.resolve(t).expect("validated perception area"))
                .collect(),
        }
    }
    pub(crate) fn add_concealment(&mut self, area: &PerceptionArea, rule: &ConcealmentRule) {
        self.runtime.perception_sensitive = true;
        let tiles = self.perception_area(area);
        let perception = std::rc::Rc::make_mut(&mut self.runtime.tile_perception);
        for tile in tiles {
            if !perception[tile].concealment.contains(rule) {
                perception[tile].concealment.push(rule.clone());
            }
        }
    }
    pub(crate) fn remove_concealment(&mut self, area: &PerceptionArea, kind: ConcealmentKind) {
        let tiles = self.perception_area(area);
        if tiles.iter().any(|&t| {
            self.tile_perception[t]
                .concealment
                .iter()
                .any(|r| r.kind == kind)
        }) {
            let perception = std::rc::Rc::make_mut(&mut self.runtime.tile_perception);
            for tile in tiles {
                perception[tile].concealment.retain(|r| r.kind != kind);
            }
        }
    }
    pub(crate) fn grant_perception(&mut self, id: u32, effect: PerceptionEffect) {
        self.runtime.perception_sensitive = true;
        let unit = self.index(id).expect("effect recipient");
        if self.units[unit].perception.0 == 0 {
            let fresh = self.fresh_perception();
            self.runtime.units[unit].perception = fresh;
        }
        let slot = self.units[unit].perception.0;
        let effects = &mut self.runtime.unit_perception[slot].effects;
        if let Some(previous) = effects.iter_mut().find(|e| e.id == effect.id) {
            *previous = effect;
        } else {
            effects.push(effect);
        }
    }
    pub(crate) fn reveal_terrain(&mut self, id: &str, area: &PerceptionArea) {
        let group = self
            .vision_groups
            .iter()
            .position(|g| g.id == id)
            .expect("validated reveal group");
        let tiles = self.perception_area(area);
        let memory = std::rc::Rc::make_mut(&mut self.runtime.fog_memory)
            .get_mut(id)
            .unwrap();
        let groups = std::rc::Rc::make_mut(&mut self.runtime.vision_groups);
        for tile in tiles {
            if memory[tile / 64] & (1 << (tile % 64)) == 0 {
                memory[tile / 64] |= 1 << (tile % 64);
                groups[group].explored.push(self.runtime.board.ids[tile]);
            }
        }
    }
    pub(crate) fn assign_vision_group(&mut self, faction: &str, destination: &str) {
        self.runtime.perception_sensitive = true;
        let source = self.vision_group(faction);
        let target = VisionGroupKey(
            self.vision_groups
                .iter()
                .position(|g| g.id == destination)
                .expect("validated destination group"),
        );
        if source == target {
            return;
        }
        let history = self.vision_groups[source.0].explored.clone();
        self.reveal_terrain(
            destination,
            &PerceptionArea::Tiles(history.into_iter().map(TileRef::Hex).collect()),
        );
        for facts in &mut *self.runtime.unit_perception {
            if facts.revealed_to.contains(&source) && !facts.revealed_to.contains(&target) {
                facts.revealed_to.push(target);
            }
        }
        std::rc::Rc::make_mut(&mut self.runtime.factions)
            .iter_mut()
            .find(|f| f.id == faction)
            .unwrap()
            .vision_group = destination.to_owned();
        let key = self.runtime.names.find(faction).unwrap();
        for unit in &mut self.runtime.units {
            if unit.faction == key {
                unit.vision_group = target;
            }
        }
    }
}
impl BattlePosition<'_> {
    /// Fast rejection for ordinary arenas. No visibility work or extra threat
    /// caches are needed when there is no spatial perception restriction.
    #[inline]
    pub(crate) fn perception_sensitive(&self) -> bool {
        self.runtime.perception_sensitive
    }
    pub(crate) fn refresh_perception_flags(&mut self) {
        self.runtime.perception_sensitive = self.calculate_perception_sensitive();
    }
    fn calculate_perception_sensitive(&self) -> bool {
        self.tile_perception
            .iter()
            .any(|p| !p.concealment.is_empty())
            || self
                .vision_groups
                .iter()
                .any(|g| matches!(g.sight.range, SightRange::Range(_)))
            || self.units.iter().any(|u| {
                let p = &self.rules.units[u.kind].perception;
                !p.concealment.is_empty()
                    || matches!(p.range, SightRange::Range(_))
                    || !self.unit_perception[u.perception.0].effects.is_empty()
                    || self.inventory(u).any(|item| item.perception.is_some())
            })
    }
    /// The caller already checked geometric coverage. Preserve the particular
    /// firing origin while checking group spotting; one attacker cannot stand
    /// beside its target and fire from a different hex in the same projection.
    pub(crate) fn can_threaten_from(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        origins: &super::geometry::TileSet,
    ) -> bool {
        if !self.perception_sensitive() {
            return true;
        }
        let attack = self.weapon(attacker, weapon);
        let profile = self.perception(None).profile(attacker);
        let facts = &self.unit_perception[target.perception.0];
        if facts.revealed_to.contains(&attacker.vision_group) {
            return true;
        }
        let unrestricted = |rule: &ConcealmentRule| {
            profile.penetrates.contains(rule.kind) || rule.reveal_within >= u16::from(attack.range)
        };
        let eligible = |c: &&UnitConcealment| self.perception(None).applies(c, target.tile);
        if profile
            .range
            .is_none_or(|range| range >= u16::from(attack.range))
            && self.tile_perception[target.tile]
                .concealment
                .iter()
                .all(unrestricted)
            && self.rules.units[target.kind]
                .perception
                .concealment
                .iter()
                .filter(eligible)
                .all(|c| unrestricted(c))
            && facts
                .effects
                .iter()
                .flat_map(|e| &e.concealment)
                .filter(eligible)
                .all(|c| unrestricted(c))
            && self
                .inventory(target)
                .filter_map(|item| item.perception.as_ref())
                .flat_map(|p| &p.concealment)
                .filter(eligible)
                .all(|c| unrestricted(c))
        {
            return true;
        }
        let firing = self.attack_origins(attack.category, attack.range);
        let group = attacker.vision_group;
        let mut detected = false;
        origins.for_each(|origin| {
            if detected || !firing[target.tile].contains(origin) {
                return;
            }
            let projected = || {
                self.perception(Some((attacker.id, origin)))
                    .with_position(target.id, target.tile)
            };
            if projected().sees_unit(group, target) {
                detected = true;
                return;
            }
            // Other already-known group members may move to provide a sighting.
            // Ignore competing friendly destinations for a conservative threat.
            for spotter in self
                .units
                .iter()
                .filter(|u| u.id != attacker.id && u.vision_group == group)
            {
                let mut blocked = self.occupied_tiles.clone();
                blocked.remove(target.tile);
                blocked.remove(attacker.tile);
                let reach = self.threat_reach_blocked(spotter, blocked);
                reach.for_each(|tile| {
                    if !detected
                        && tile != origin
                        && projected()
                            .with_position(spotter.id, tile)
                            .detection_facts(spotter, Some(target), target.tile)
                            .detected()
                    {
                        detected = true;
                    }
                });
                if detected {
                    break;
                }
            }
        });
        detected
    }
}
