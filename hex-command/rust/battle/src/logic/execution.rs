//! Live orchestration consumes physical results. Forecasts own BattlePosition
//! instead of Game and therefore cannot reach this campaign boundary.
use super::{Game, battle::BattlePosition};
use crate::{RuntimeCommand, data::*};
use std::rc::Rc;
impl Game<'_> {
    #[cfg(test)]
    pub(crate) fn move_unit(&mut self, id: u32, tile: usize) {
        self.execute_runtime(RuntimeCommand::Move {
            unit: UnitId(id),
            tile: TileIndex(tile),
        })
        .expect("legal move");
    }
    #[cfg(test)]
    pub(crate) fn attack(&mut self, id: u32, target: u32, weapon: usize) {
        let weapon = self.weapon(self.unit(id).unwrap(), weapon).key();
        self.execute_runtime(RuntimeCommand::Attack {
            unit: UnitId(id),
            target: UnitId(target),
            weapon,
        })
        .expect("legal attack");
    }
    pub(crate) fn begin_turn(&mut self, phase: TurnPhase) {
        self.position.begin_turn(phase);
        self.refresh_perception_sources();
        self.evaluate_campaign(true, None);
        self.refresh_planning_facts();
        self.remember_conditions();
    }
    pub(crate) fn end_turn(&mut self) {
        self.execute_runtime(RuntimeCommand::EndTurn)
            .expect("unfinished battle");
    }
    pub(crate) fn refresh_planning_facts(&mut self) {
        self.position.refresh_perception_flags();
        self.position.runtime.campaign_battle = self.story.campaign.is_some();
        self.position.runtime.tracks_path = self
            .story
            .mission
            .as_ref()
            .is_some_and(|m| m.scripted_events.iter().any(|event| event.when.uses_path()));
        for index in 0..self.position.runtime.units.len() {
            let unit = self.position.runtime.units[index];
            let available = self.available_combatant(&unit);
            self.position.runtime.units[index].field_available = available;
        }
        self.refresh_perception_sources();
    }
    pub(crate) fn sync_recruitment(&mut self) {
        let mut pools = self
            .story
            .mission
            .as_ref()
            .map(|m| m.recruitment.clone())
            .unwrap_or_default();
        if let Some(events) = &self.story.scripts {
            for (faction, pool) in &events.recruitment {
                pools.insert(faction.clone(), pool.clone());
            }
        }
        self.position.runtime.resolved_recruitment = Rc::new(pools);
    }
    pub(crate) fn refresh_perception_sources(&mut self) {
        // Clone only the shared progress reference. The query never mutates it.
        let progress = self.story.scripts.clone();
        self.refresh_perception_sources_with_marks(
            progress
                .as_deref()
                .map(|s| s.marks.as_slice())
                .unwrap_or_default(),
        );
    }
    pub(crate) fn refresh_perception_sources_with_marks(&mut self, marks: &[MapMark]) {
        use crate::data::presentation;
        if self.terrain_emitters.is_empty()
            && marks.iter().all(|m| m.sight.is_none() && m.light.is_none())
        {
            if !self.fixed_lights.is_empty() {
                self.position.runtime.fixed_lights = Rc::new(vec![]);
            }
            if !self.fixed_sights.is_empty() {
                self.position.runtime.fixed_sights = Rc::new(vec![]);
            }
            return;
        }
        let context = self.condition_context();
        let mut lights = Vec::new();
        let mut sights = Vec::new();
        let mut sight = |tile, emission: &SightEmission| {
            let group = VisionGroupKey(
                self.vision_groups
                    .iter()
                    .position(|g| g.id == emission.group)
                    .expect("validated source group"),
            );
            let base = &self.vision_groups[group.0].sight;
            sights.push((
                tile,
                group,
                ResolvedSightProfile {
                    range: emission.profile.range.resolve(base.range.resolve(None)),
                    penetrates: ConcealmentMask::of(&base.penetrates)
                        .union(ConcealmentMask::of(&emission.profile.penetrates)),
                },
            ));
        };
        for &tile in self.terrain_emitters.iter() {
            if let Some(source) = &self.tile_perception[tile].sight {
                sight(tile, source);
            }
            if let Some(light) = self.light[tile].as_ref() {
                if light.when.as_ref().is_none_or(|r| {
                    (r.remember
                        && self
                            .condition_memory
                            .iter()
                            .any(|m| m == &format!("light:{}", self.board.ids[tile])))
                        || presentation::matches(
                            &r.condition,
                            &context,
                            Some(&TileRef::Hex(self.board.ids[tile])),
                        )
                }) {
                    let (col, row) = self.board.positions[tile];
                    lights.push(super::perception::ResolvedFixedLight {
                        tile,
                        emission: light.emission.clone(),
                        appearance: light.at(col, row),
                    });
                }
            }
        }
        for mark in marks {
            let tile = self.board.indices[&mark.tile];
            if let Some(source) = &mark.sight {
                sight(tile, source);
            }
            if let Some(light) = mark.light.as_ref() {
                if mark.light_remembered
                    || light.when.as_ref().is_none_or(|r| {
                        presentation::matches(
                            &r.condition,
                            &context,
                            Some(&TileRef::Hex(mark.tile)),
                        )
                    })
                {
                    let (col, row) = self.board.positions[tile];
                    lights.push(super::perception::ResolvedFixedLight {
                        tile,
                        emission: light.emission.clone(),
                        appearance: light.at(col, row),
                    });
                }
            }
        }
        if *self.fixed_lights != lights {
            self.position.runtime.fixed_lights = Rc::new(lights);
        }
        if *self.fixed_sights != sights {
            self.position.runtime.fixed_sights = Rc::new(sights);
        }
    }
    pub(crate) fn accept_combat_deaths(&mut self, attacker: Unit, target: Unit, mediated: bool) {
        let defeated = std::mem::take(&mut self.position.runtime.mechanical_deaths);
        if self
            .story
            .mission
            .as_ref()
            .is_some_and(|m| !m.scripted_events.is_empty())
        {
            let initiated_by = self.unit_faction(&attacker).to_owned();
            for unit in &defeated {
                self.story
                    .event_deaths
                    .push(super::events::DeathOccurrence {
                        unit: *unit,
                        initiated_by: initiated_by.clone(),
                        killed_by: Some(
                            self.unit_faction(if unit.id == target.id {
                                &attacker
                            } else {
                                &target
                            })
                            .to_owned(),
                        ),
                        defending: unit.id == target.id,
                    });
            }
        }
        // Legacy companion bookkeeping remains only on the actual game. New
        // scripts record deaths as flags and Retry retains its incoming roster.
        for unit in &defeated {
            if self.story.campaign.is_none() && unit.is_leader() {
                let relation = self.faction_relation("player", self.unit_faction(unit));
                let remaining = self.units.iter().any(|u| {
                    u.is_leader()
                        && self.faction_relation("player", self.unit_faction(u)) == relation
                });
                if !remaining {
                    self.position.runtime.winner = match relation {
                        Relation::Hostile => Some(BattleResult::PlayerVictory),
                        Relation::Allied => Some(BattleResult::PlayerDefeat),
                        Relation::Neutral => None,
                    };
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(record) = self
            .position
            .runtime
            .recordings
            .as_mut()
            .and_then(|r| r.combats.last_mut())
        {
            record.mediation_at_risk = mediated;
        }
        #[cfg(target_arch = "wasm32")]
        let _ = mediated;
    }
}
impl<'catalog> BattlePosition<'catalog> {
    /// Build a physical position from permitted sightings once. Its occupancy,
    /// movement, threat and subsequent projections never consult the real roster.
    pub(crate) fn known_to(&self, faction: &str) -> Self {
        let group = self.vision_group(faction);
        let mut position = self.planning_fork();
        position
            .runtime
            .units
            .retain(|u| self.perception(None).sees_unit(group, u));
        position
            .runtime
            .ground_items
            .retain(|item| self.perception(None).sees_terrain(group, item.tile));
        // Unobserved eyes and torch carriers must not become enemy sensors in
        // a forecast. Their currently observable illumination is a world fact;
        // retain that footprint without retaining the hidden source/position.
        let known_ids: rustc_hash::FxHashSet<_> = position.units.iter().map(|u| u.id).collect();
        let visible = |tile| self.perception(None).sees_terrain(group, tile);
        let mut unmodeled_lights = Vec::new();
        self.perception(None)
            .visit_light_sources(|carrier, tile, emission, _| {
                if carrier.map_or(!visible(tile), |u| !known_ids.contains(&u.id)) {
                    if let Some(emission) = emission {
                        unmodeled_lights.push((tile, emission.clone()));
                    }
                }
                false
            });
        if !unmodeled_lights.is_empty() {
            let mut footprint = (*position.observed_light).clone();
            for tile in 0..self.terrain.len() {
                if unmodeled_lights
                    .iter()
                    .any(|(origin, emission)| self.board.distance(*origin, tile) <= emission.radius)
                    && visible(tile)
                {
                    footprint.insert(tile);
                }
            }
            position.runtime.observed_light = Rc::new(footprint);
        }
        if self
            .fixed_sights
            .iter()
            .any(|(origin, g, _)| *g != group && !visible(*origin))
        {
            position.runtime.fixed_sights = Rc::new(
                self.fixed_sights
                    .iter()
                    .filter(|(origin, g, _)| *g == group || visible(*origin))
                    .copied()
                    .collect(),
            );
        }
        if self.fixed_lights.iter().any(|light| !visible(light.tile)) {
            position.runtime.fixed_lights = Rc::new(
                self.fixed_lights
                    .iter()
                    .filter(|light| visible(light.tile))
                    .cloned()
                    .collect(),
            );
        }
        position.runtime.tracks_path = false;
        position.runtime.traversal = None;
        position.runtime.mechanical_deaths.clear();
        position.roster_changed();
        position.refresh_perception_flags();
        position
    }
}
// Explicit mutation facades preserve Rust's ordinary two-phase receiver borrows
// for live orchestration. The physical implementation remains single-owned.
impl Game<'_> {
    pub(crate) fn add(&mut self, kind: usize, faction: &str, tile: usize) -> u32 {
        let faction = self.position.intern_name(faction);
        self.position.add(kind, faction, tile)
    }
    pub(crate) fn set_terrain(&mut self, tile: usize, terrain: usize) {
        self.position.set_terrain(tile, terrain);
    }
    #[cfg(test)]
    pub(crate) fn recruit(&mut self, id: u32, kind: usize, tile: usize) {
        self.position.recruit(id, kind, tile);
    }
    #[cfg(test)]
    pub(crate) fn advance(&mut self, id: u32, kind: usize) {
        self.execute_runtime(RuntimeCommand::Advance {
            unit: UnitId(id),
            kind: UnitKind(kind),
        })
        .expect("legal advancement");
    }
}
