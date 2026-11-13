//! Live condition queries borrow battlefield facts; owned inputs are for devtools.
use super::Game;
pub use crate::data::presentation::{PresentationInput, matches, resolve};
use crate::data::{
    presentation::{self, ConditionContext, ConditionUnit, PresentationContext},
    *,
};

pub(crate) struct LiveContext<'a, 'catalog> {
    game: &'a Game<'catalog>,
    observer: &'a str,
    position: Option<(u32, usize)>,
}
impl<'a, 'catalog> LiveContext<'a, 'catalog> {
    fn new(game: &'a Game<'catalog>, position: Option<(u32, usize)>) -> Self {
        Self {
            game,
            observer: "player",
            position,
        }
    }
    pub(crate) fn for_faction(
        game: &'a Game<'catalog>,
        observer: &'a str,
        position: Option<(u32, usize)>,
    ) -> Self {
        Self {
            game,
            observer,
            position,
        }
    }
    pub(crate) fn current_sight(&self, tile: usize) -> bool {
        self.game
            .perception(self.position)
            .sees_terrain(self.game.vision_group(self.observer), tile)
    }
    pub(crate) fn unit_visible(&self, unit: &Unit) -> bool {
        self.game
            .perception(self.position)
            .sees_unit(self.game.vision_group(self.observer), unit)
    }
}
impl ConditionContext for LiveContext<'_, '_> {
    fn factions(&self) -> &[Faction] {
        &self.game.factions
    }
    fn observer_faction(&self) -> &str {
        self.observer
    }
    fn visible_unit(&self, unit: ConditionUnit<'_>) -> bool {
        self.game.units.iter().any(|u| {
            self.game.rules.units[u.kind].id == unit.kind
                && self.game.board.positions[self
                    .position
                    .filter(|(id, _)| *id == u.id)
                    .map_or(u.tile, |(_, tile)| tile)]
                    == unit.position.unwrap_or((-1, -1))
                && self.unit_visible(u)
        })
    }
    fn owns(&self, id: &TileRef, side: &str) -> bool {
        self.game
            .board
            .resolve(id)
            .is_some_and(|i| self.game.owners[i].as_deref() == Some(side))
    }
    fn position(&self, id: &TileRef) -> Option<(i32, i32)> {
        self.game
            .board
            .resolve(id)
            .map(|i| self.game.board.positions[i])
    }
    fn visible_at(&self, position: (i32, i32)) -> bool {
        self.game
            .board
            .indices
            .get(&Hex::new(position.0, position.1))
            .is_some_and(|&tile| {
                self.game
                    .perception(self.position)
                    .terrain_knowledge(self.game.vision_group(self.observer), tile)
                    != TerrainKnowledge::Unexplored
            })
    }
    #[inline(always)]
    fn any_unit(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        self.game.units.iter().any(|unit| {
            let tile = self
                .position
                .filter(|(actor, _)| *actor == unit.id)
                .map(|(_, tile)| tile)
                .unwrap_or(unit.tile);
            predicate(ConditionUnit {
                faction: Some(self.game.names.get(unit.faction)),
                side: unit.side,
                kind: &self.game.rules.units[unit.kind].id,
                character: self.game.names.resolve(unit.character),
                ai_group: self.game.names.resolve(unit.ai_group),
                position: Some(self.game.board.positions[tile]),
            })
        })
    }
    #[inline(always)]
    fn any_pending(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        let Some(mission) = self
            .game
            .story
            .mission
            .as_deref()
            .map(|m| &**m)
            .filter(|_| self.game.story.campaign.is_some())
        else {
            return false;
        };
        mission
            .events
            .iter()
            .filter(|event| {
                self.game
                    .story
                    .campaign
                    .as_deref()
                    .map(|s| s.marks.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .any(|m| m.strip_prefix("event:") == Some(event.id.as_str()))
            })
            .any(|event| {
                event.spawn.iter().any(|group| {
                    group.units.iter().any(|unit| {
                        let spawned = self
                            .game
                            .story
                            .campaign
                            .as_deref()
                            .map(|s| s.marks.as_slice())
                            .unwrap_or_default()
                            .iter()
                            .any(|m| {
                                m.strip_prefix("spawn:")
                                    .and_then(|s| s.strip_prefix(event.id.as_str()))
                                    .and_then(|s| s.strip_prefix(':'))
                                    == Some(unit.id.as_str())
                            });
                        !spawned
                            && predicate(ConditionUnit {
                                faction: unit.faction.as_deref().or(group.faction.as_deref()),
                                side: unit.side,
                                kind: &unit.r#type,
                                character: unit.character.as_deref(),
                                ai_group: group.ai_group.as_deref(),
                                position: None,
                            })
                    })
                })
            })
    }
    fn flag(&self, id: &str) -> bool {
        self.game
            .story
            .campaign
            .as_deref()
            .is_some_and(|s| flag_is_set(&s.flags, id))
    }
    fn marks(&self) -> &[String] {
        self.game
            .story
            .campaign
            .as_deref()
            .map(|s| s.marks.as_slice())
            .unwrap_or_default()
    }
    fn round(&self) -> u32 {
        self.game.round
    }
}
impl PresentationContext for LiveContext<'_, '_> {
    fn fog_hidden(&self, id: &Hex) -> bool {
        !self.visible_at((id.col, id.row))
    }
    fn lighting(&self) -> &MapLighting {
        static DEFAULT: MapLighting = MapLighting {
            time: TimeOfDay::Day,
            underground: Vec::new(),
        };
        self.game
            .rules
            .map_definitions
            .iter()
            .find(|map| map.id == self.game.map_id)
            .map(|m| &m.lighting)
            .unwrap_or(&DEFAULT)
    }
    fn visit_lights(&self, mut visitor: impl FnMut(&Hex, i32, i32, &TileLight)) {
        for (i, light) in self.game.light.iter().enumerate() {
            if let Some(light) = light {
                let (col, row) = self.game.board.positions[i];
                visitor(&self.game.board.ids[i], col, row, light);
            }
        }
    }
    fn visit_tiles(&self, mut visitor: impl FnMut(&Hex)) {
        for id in &self.game.board.ids {
            visitor(id);
        }
    }
}
impl<'catalog> Game<'catalog> {
    pub(crate) fn condition_context(&self) -> LiveContext<'_, 'catalog> {
        LiveContext::new(self, None)
    }
    pub(crate) fn event_condition_met(&self, condition: &Condition) -> bool {
        if matches(condition, &self.condition_context(), None) {
            return true;
        }
        if let Some((actor, path)) = &self.position.runtime.traversal {
            if self.position.runtime.units.iter().any(|u| u.id == *actor) {
                return path.iter().any(|&tile| {
                    matches(
                        condition,
                        &LiveContext::new(self, Some((*actor, tile))),
                        None,
                    )
                });
            }
        }
        false
    }
    pub(crate) fn group_active(&self, group: &EnemyGroup) -> bool {
        group.when.as_ref().is_none_or(|rule| {
            (rule.remember
                && self
                    .condition_memory
                    .iter()
                    .any(|m| m.strip_prefix("group:") == Some(group.id.as_str())))
                || matches(&rule.condition, &self.condition_context(), None)
        })
    }
    pub(crate) fn presentation_for(
        &self,
        observer: &str,
        knowledge: &crate::BattleView<'_, '_>,
    ) -> BattlefieldPresentation {
        let context = LiveContext::for_faction(self, observer, None);
        let lighting = context.lighting();
        let mut result = BattlefieldPresentation {
            lighting: BattlefieldLighting {
                time: lighting.time,
                underground: lighting.underground.clone(),
                lights: Vec::new(),
            },
            fog: self
                .board
                .ids
                .iter()
                .filter(|id| context.fog_hidden(id))
                .copied()
                .collect(),
            visible_units: Vec::new(),
            unit_appearances: Vec::new(),
        };
        result.visible_units = self
            .units
            .iter()
            .filter(|u| context.unit_visible(u))
            .map(|u| format!("u{}", u.id))
            .collect();
        // Only units are visibility-filtered. Illumination is a physical world fact.
        result.lighting.lights.clear();
        let perception = self.perception(None);
        perception.visit_light_sources(|_, tile, emission, appearance| {
            if let Some(appearance) = appearance {
                result.lighting.lights.push(appearance.clone());
            } else if let Some(emission) = emission {
                let (col, row) = self.board.positions[tile];
                result.lighting.lights.push(BattlefieldLight {
                    col,
                    row,
                    color: Some([255, 208, 128]),
                    fixture: Some(false),
                    dx: None,
                    dy: None,
                    radius: (f64::from(emission.radius) + 0.5) * 64.0,
                    power: 1.0,
                    when: None,
                });
            }
            false
        });
        let group = self.vision_group(observer);
        result.unit_appearances = knowledge
            .units()
            .iter()
            .filter(|u| u.vision_group == group)
            .map(|unit| knowledge.unit_appearance(unit))
            .collect();

        result
    }
    pub(crate) fn remember_conditions(&mut self) {
        let groups = self.story.mission.as_ref().is_some_and(|m| {
            m.enemy_groups
                .iter()
                .any(|g| g.when.as_ref().is_some_and(|r| r.remember))
        });
        let lights = self
            .light
            .iter()
            .flatten()
            .any(|l| l.when.as_ref().is_some_and(|r| r.remember));
        if !groups && !lights {
            self.remember_faction_sight();
            return;
        }
        let mut memory = std::mem::take(&mut self.position.runtime.condition_memory);

        let mut visit = |position| {
            let input = LiveContext::new(self, position);
            let added = presentation::discover(&input, &memory);
            if !added.is_empty() {
                std::rc::Rc::make_mut(&mut memory).extend(added);
            }
            if let Some(mission) = &self.story.mission {
                for group in &mission.enemy_groups {
                    if group
                        .when
                        .as_ref()
                        .is_some_and(|r| r.remember && matches(&r.condition, &input, None))
                        && !memory
                            .iter()
                            .any(|m| m.strip_prefix("group:") == Some(group.id.as_str()))
                    {
                        std::rc::Rc::make_mut(&mut memory).push(format!("group:{}", group.id));
                    }
                }
            }
        };
        visit(None);
        if let Some((actor, path)) = &self.position.runtime.traversal {
            if self.position.runtime.units.iter().any(|u| u.id == *actor) {
                for &tile in path {
                    visit(Some((*actor, tile)));
                }
            }
        }
        self.position.runtime.condition_memory = memory;
        self.remember_faction_sight();
    }
}

impl super::BattlePosition<'_> {
    pub(crate) fn observed_units(&self, observer: &str) -> Vec<Unit> {
        let group = self.vision_group(observer);
        self.units
            .iter()
            .filter(|u| self.perception(None).sees_unit(group, u))
            .copied()
            .collect()
    }
    pub(crate) fn unit_visible_to(
        &self,
        observer: &str,
        unit: &Unit,
        position: Option<(u32, usize)>,
    ) -> bool {
        self.perception(position)
            .sees_unit(self.vision_group(observer), unit)
    }
    pub(crate) fn tile_in_sight_to(&self, observer: &str, tile: usize) -> bool {
        self.perception(None)
            .sees_terrain(self.vision_group(observer), tile)
    }
    pub(crate) fn tile_visible_to(&self, observer: &str, tile: usize) -> bool {
        self.perception(None)
            .terrain_knowledge(self.vision_group(observer), tile)
            != TerrainKnowledge::Unexplored
    }
}
