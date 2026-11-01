//! Data-driven periodic rituals. Charges and pending groups are durable game facts.
use super::battle::Game;
use crate::data::*;

/// Source coordinates are already part of each durable mark/death group identity.
/// The same identity keeps partially blocked groups anchored after save/restore.
pub(super) fn ritual_source(mark: &str) -> Option<&str> {
    let mut parts = mark.splitn(4, ':');
    let kind = parts.next()?;
    let _ritual = parts.next()?;
    let identity = parts.next()?;
    let source = parts.next()?;
    match kind {
        "charge" => {
            if identity.strip_prefix('u')?.parse::<u32>().ok()? == 0 {
                return None;
            }
            Some(source)
        }
        "ritual" => {
            if identity.parse::<u32>().ok()? == 0 {
                return None;
            }
            if let Some(tile) = source.strip_prefix("mark-") {
                Some(tile)
            } else {
                let (unit, tile) = source.strip_prefix("death-u")?.split_once('-')?;
                if unit.parse::<u32>().ok()? == 0 {
                    return None;
                }
                Some(tile)
            }
        }
        _ => None,
    }
}

impl Game<'_> {
    #[cfg(any(feature = "wasm", test))]
    pub(crate) fn ritual_notices(&self) -> Vec<String> {
        let Some(mission) = &self.mission else {
            return vec![];
        };
        mission
            .rituals
            .iter()
            .filter(|ritual| {
                self.event_condition_met(&ritual.when)
                    && ritual
                        .notice_when
                        .as_ref()
                        .is_none_or(|c| self.event_condition_met(c))
            })
            .map(|ritual| {
                let next = if self.round < ritual.first_round {
                    ritual.first_round
                } else {
                    self.round + ritual.interval
                        - (self.round - ritual.first_round) % ritual.interval
                };
                format!("Next toll: turn {next}. Move captives off active marks.")
            })
            .collect()
    }

    pub(crate) fn charge_deaths(&mut self, defeated: &[Unit]) {
        let Some(mission) = self.mission.clone() else {
            return;
        };
        for ritual in &mission.rituals {
            if !self.event_condition_met(&ritual.when) {
                continue;
            }
            for unit in defeated
                .iter()
                .filter(|u| ritual.unit_types.contains(&self.rules.units[u.kind].id))
            {
                let mark = format!(
                    "charge:{}:u{}:{}",
                    ritual.id, unit.id, self.board.ids[unit.tile]
                );
                let state = self.campaign.as_mut().unwrap();
                if !state.marked(&mark) {
                    state.marks.push(mark);
                    self.record("A red light gathers where the captives fell.".into());
                }
            }
        }
    }

    pub(crate) fn ritual_lights(&self) -> Vec<BattlefieldLight> {
        let Some(mission) = &self.mission else {
            return vec![];
        };
        let Some(state) = &self.campaign else {
            return vec![];
        };
        let mut lights = vec![];
        for ritual in &mission.rituals {
            if !self.event_condition_met(&ritual.when) {
                continue;
            }
            let prefix = format!("charge:{}:", ritual.id);
            for mark in state.marks.iter().filter(|m| m.starts_with(&prefix)) {
                let Some((_, tile)) = mark.rsplit_once(':') else {
                    continue;
                };
                let Some(&site) = self.board.indices.get(tile) else {
                    continue;
                };
                let mut light = ritual.death_light.clone();
                (light.col, light.row) = self.board.positions[site];
                lights.push(light);
            }
        }
        lights
    }

    pub(crate) fn evaluate_rituals(&mut self, at_turn_start: bool) {
        if self.turn != Side::Player || self.winner.is_some() {
            return;
        }
        let Some(mission) = self.mission.clone() else {
            return;
        };
        for ritual in &mission.rituals {
            if !self.event_condition_met(&ritual.when) {
                continue;
            }
            let tick = format!("toll:{}:{}", ritual.id, self.round);
            let due = at_turn_start
                && self.round >= ritual.first_round
                && (self.round - ritual.first_round) % ritual.interval == 0
                && !self.campaign.as_ref().unwrap().marked(&tick);
            if due {
                self.campaign.as_mut().unwrap().marks.push(tick);
                self.record(ritual.label.clone());
                let source = self.rules.terrains[&ritual.source_terrain];
                let spent = self.rules.terrains[&ritual.spent_terrain];
                let sites: Vec<_> = self
                    .terrain
                    .iter()
                    .enumerate()
                    .filter(|(_, terrain)| **terrain == source)
                    .map(|(tile, _)| tile)
                    .collect();
                for tile in sites {
                    let victims: Vec<Unit> = self
                        .units
                        .iter()
                        .filter(|u| {
                            u.tile == tile
                                && ritual.unit_types.contains(&self.rules.units[u.kind].id)
                        })
                        .cloned()
                        .collect();
                    self.set_terrain(tile, spent);
                    if victims.is_empty() {
                        continue;
                    }
                    // A sacrifice spends this source once; it must not create a second death charge.
                    for unit in &victims {
                        self.ground_items
                            .extend(unit.items.iter().map(|item| GroundItem {
                                tile: self.board.ids[tile].clone(),
                                item: item.clone(),
                            }));
                    }
                    self.units.retain(|u| !victims.iter().any(|v| v.id == u.id));
                    self.roster_changed();
                    self.campaign.as_mut().unwrap().marks.push(format!(
                        "ritual:{}:{}:mark-{}",
                        ritual.id, self.round, self.board.ids[tile]
                    ));
                    self.record("The bell takes the captives still standing on the mark.".into());
                }
                let prefix = format!("charge:{}:", ritual.id);
                let charges: Vec<_> = self
                    .campaign
                    .as_ref()
                    .unwrap()
                    .marks
                    .iter()
                    .filter(|m| m.starts_with(&prefix))
                    .cloned()
                    .collect();
                for charge in &charges {
                    let source = charge.strip_prefix(&prefix).unwrap().replace(':', "-");
                    self.campaign.as_mut().unwrap().marks.push(format!(
                        "ritual:{}:{}:death-{}",
                        ritual.id, self.round, source
                    ));
                }
                self.campaign
                    .as_mut()
                    .unwrap()
                    .marks
                    .retain(|m| !charges.contains(m));
            }
            // Retry only the unplaced members of activated finite groups.
            let prefix = format!("ritual:{}:", ritual.id);
            let groups: Vec<_> = self
                .campaign
                .as_ref()
                .unwrap()
                .marks
                .iter()
                .filter(|m| m.starts_with(&prefix))
                .cloned()
                .collect();
            for group in groups {
                for incoming in &ritual.spawn.units {
                    let mark = format!("spawn:{group}:{}", incoming.id);
                    if self.campaign.as_ref().unwrap().marked(&mark) {
                        continue;
                    }
                    let kind = self.kind(&incoming.r#type);
                    let origin = self.site(ritual_source(&group).expect("validated ritual source"));
                    let site = (0..self.terrain.len())
                        .filter(|&tile| {
                            self.board.distance(tile, origin) <= ritual.spawn.radius
                                && self.at(tile).is_none()
                                && self.cost(kind, tile) != u16::MAX
                        })
                        .min_by_key(|&tile| (self.board.distance(tile, origin), tile));
                    if let Some(site) = site {
                        let id = self.add(kind, incoming.side, site);
                        let unit = self.units.iter_mut().find(|u| u.id == id).unwrap();
                        unit.faction = incoming
                            .faction
                            .clone()
                            .or_else(|| ritual.spawn.faction.clone());
                        unit.ai_group = ritual.spawn.ai_group.clone();
                        unit.name = incoming.name.clone();
                        unit.role = incoming.role;
                        self.campaign.as_mut().unwrap().marks.push(mark);
                    }
                }
            }
        }
    }
}
