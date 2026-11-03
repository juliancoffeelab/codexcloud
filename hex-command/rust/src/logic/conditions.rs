//! Live condition queries borrow battlefield facts; owned inputs are for devtools.
use super::Game;
pub use crate::data::presentation::{PresentationInput, matches, resolve};
use crate::data::{
    presentation::{self, ConditionContext, ConditionUnit, PresentationContext},
    *,
};

pub(crate) struct LiveContext<'a, 'catalog> {
    game: &'a Game<'catalog>,
    position: Option<(u32, usize)>,
}
impl ConditionContext for LiveContext<'_, '_> {
    fn factions(&self) -> &[Faction] {
        &self.game.factions
    }
    fn owns(&self, id: &str, side: &str) -> bool {
        self.game
            .board
            .indices
            .get(id)
            .is_some_and(|&i| self.game.owners[i].as_deref() == Some(side))
    }
    fn position(&self, id: &str) -> Option<(i32, i32)> {
        self.game
            .board
            .indices
            .get(id)
            .map(|&i| self.game.board.positions[i])
    }
    fn visible_at(&self, position: (i32, i32)) -> bool {
        self.game
            .board
            .positions
            .iter()
            .position(|&p| p == position)
            .is_some_and(|tile| {
                self.game.fog[tile].as_ref().is_none_or(|rule| {
                    (rule.remember && self.fog_remembered(&self.game.board.ids[tile]))
                        || crate::data::presentation::matches_fog(
                            &rule.condition,
                            self,
                            Some(&self.game.board.ids[tile]),
                        )
                })
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
                faction: unit.faction.as_deref(),
                side: unit.side,
                kind: &self.game.rules.units[unit.kind].id,
                character: unit.character.as_deref(),
                ai_group: unit.ai_group.as_deref(),
                position: Some(self.game.board.positions[tile]),
            })
        })
    }
    #[inline(always)]
    fn any_pending(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        let (Some(mission), Some(state)) = (&self.game.mission, &self.game.campaign) else {
            return false;
        };
        mission
            .events
            .iter()
            .filter(|event| state.marked_prefixed("event:", &event.id))
            .any(|event| {
                event.spawn.iter().any(|group| {
                    group.units.iter().any(|unit| {
                        let spawned = state.marks.iter().any(|m| {
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
    fn choices(&self) -> &[String] {
        self.game
            .campaign
            .as_ref()
            .and_then(|s| s.choices.as_deref())
            .unwrap_or_default()
    }
    fn marks(&self) -> &[String] {
        self.game
            .campaign
            .as_ref()
            .map(|s| s.marks.as_slice())
            .unwrap_or_default()
    }
    fn round(&self) -> u32 {
        self.game.round
    }
}
impl PresentationContext for LiveContext<'_, '_> {
    fn fog_index(&self, id: &str) -> usize {
        self.game.board.indices[id]
    }
    fn fog_remembered(&self, id: &str) -> bool {
        let i = self.fog_index(id);
        self.game
            .fog_memory
            .get(i / 64)
            .is_some_and(|word| word & (1u64 << (i % 64)) != 0)
    }
    fn lighting(&self) -> &MapLighting {
        static DEFAULT: MapLighting = MapLighting {
            time: TimeOfDay::Day,
        };
        self.game
            .rules
            .map_definitions
            .iter()
            .find(|map| map.id == self.game.map_id)
            .map(|m| &m.lighting)
            .unwrap_or(&DEFAULT)
    }
    fn visit_lights(&self, mut visitor: impl FnMut(&str, i32, i32, &TileLight)) {
        for (i, light) in self.game.light.iter().enumerate() {
            if let Some(light) = light {
                let (col, row) = self.game.board.positions[i];
                visitor(&self.game.board.ids[i], col, row, light);
            }
        }
    }
    fn visit_fog(&self, mut visitor: impl FnMut(&str, &ConditionRule)) {
        for (i, rule) in self.game.fog.iter().enumerate() {
            if let Some(rule) = rule {
                visitor(&self.game.board.ids[i], rule);
            }
        }
    }
}
impl Game<'_> {
    pub(crate) fn condition_context(&self) -> LiveContext<'_, '_> {
        LiveContext {
            game: self,
            position: None,
        }
    }
    pub(crate) fn event_condition_met(&self, condition: &Condition) -> bool {
        if matches(condition, &self.condition_context(), None) {
            return true;
        }
        if let Some((actor, path)) = &self.traversal {
            if self.units.iter().any(|u| u.id == *actor) {
                return path.iter().any(|&tile| {
                    matches(
                        condition,
                        &LiveContext {
                            game: self,
                            position: Some((*actor, tile)),
                        },
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
    pub(crate) fn presentation(&self) -> BattlefieldPresentation {
        let mut result = presentation::render(&self.condition_context(), &self.condition_memory);
        result.lighting.lights.extend(self.ritual_lights());
        result
    }
    pub(crate) fn remember_conditions(&mut self) {
        let groups = self.mission.as_ref().is_some_and(|m| {
            m.enemy_groups
                .iter()
                .any(|g| g.when.as_ref().is_some_and(|r| r.remember))
        });
        let lights = self
            .light
            .iter()
            .flatten()
            .any(|l| l.when.as_ref().is_some_and(|r| r.remember));
        if !groups
            && !lights
            && !self
                .fog
                .iter()
                .any(|f| f.as_ref().is_some_and(|r| r.remember))
        {
            return;
        }
        let mut memory = std::mem::take(&mut self.condition_memory);
        let mut known = std::mem::take(&mut self.fog_memory);
        let mut visit = |position| {
            let input = LiveContext {
                game: self,
                position,
            };
            let added = presentation::discover(&input, &memory, &mut known);
            if !added.is_empty() {
                std::rc::Rc::make_mut(&mut memory).extend(added);
            }
            if let Some(mission) = &self.mission {
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
        if let Some((actor, path)) = &self.traversal {
            if self.units.iter().any(|u| u.id == *actor) {
                for &tile in path {
                    visit(Some((*actor, tile)));
                }
            }
        }
        self.condition_memory = memory;
        self.fog_memory = known;
    }
}
