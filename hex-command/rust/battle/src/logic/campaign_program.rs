//! Event-engine-owned chapter programs compiled from immutable Catalogue definitions.
use super::{event_conditions::ScriptTree, setup};
use crate::data::*;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct CampaignProgram {
    data: Arc<ProgramData>,
}
#[derive(Clone, Debug)]
struct ProgramData {
    id: Text,
    name: Text,
    characters: Vec<Character>,
    carryover: CampaignCarryover,
    ending: Vec<Speech>,
    chapters: Vec<ChapterProgram>,
}
#[derive(Clone, Debug)]
pub(crate) struct ChapterLayout {
    pub id: Text,
    pub revision: u32,
    pub units: Vec<PublicUnit>,
}
#[derive(Clone, Debug)]
pub(crate) struct ChapterProgram {
    pub id: Text,
    pub map: ChapterLayout,
    pub subtitle: Text,
    pub briefing: ConditionalText,
    pub debrief: Option<ConditionalText>,
    pub hints: Vec<ConditionalHint>,
    pub intro: Vec<Speech>,
    pub victory: Vec<Speech>,
    pub scripted_events: Vec<ScriptEvent>,
    pub required_companions: Vec<Text>,
    pub enemy_groups: Vec<EnemyGroup>,
    pub defend_objectives: bool,
    pub enemy_leader_post: Option<TileRef>,
    pub gold: Gold,
    pub arrivals: Vec<CompanionArrival>,
    pub recruitment: BTreeMap<Text, Vec<FactionRecruit>>,
    pub item_ids: Vec<Text>,
    initial: State,
}
impl CampaignProgram {
    pub fn compile(
        catalogue: &Catalogue,
        definition: &CampaignDefinition,
        route: Option<&SimulationObjective>,
    ) -> Self {
        let chapters = definition
            .chapters()
            .iter()
            .map(|chapter| {
                let map = catalogue.map(chapter.map_id(), chapter.map_revision());
                let initial = setup::initial_state(catalogue, map);
                let story = chapter.story();
                let scripts = chapter.scripts();
                let mut events = scripts.events.clone();
                if let Some(changes) = route.and_then(|route| route.stages.get(chapter.map_id())) {
                    for (id, condition) in &changes.script_conditions {
                        events
                            .iter_mut()
                            .find(|event| &event.id == id)
                            .expect("route condition references an authored event")
                            .when = condition.clone();
                    }
                    for event in &changes.script_events {
                        assert!(
                            !events.iter().any(|existing| existing.id == event.id),
                            "route event identity is unique"
                        );
                        events.push(event.clone());
                    }
                }
                // Equal-priority events retain their authored order.
                events.sort_by_key(|event| std::cmp::Reverse(event.priority));
                let mut item_ids: Vec<_> = initial
                    .units
                    .iter()
                    .flat_map(|unit| unit.items.iter().cloned())
                    .collect();
                for event in &events {
                    for action in event.actions_recursive() {
                        match action {
                            EventAction::Give(value) => item_ids.push(value.item.clone()),
                            EventAction::PlaceItem(value) => item_ids.push(value.item.clone()),
                            EventAction::Spawn(value) => {
                                for entry in &value.units {
                                    if let SpawnEntry::Member(member) = entry {
                                        item_ids.extend(member.items.iter().cloned());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                item_ids.sort();
                item_ids.dedup();
                for id in &item_ids {
                    catalogue.item(id);
                }
                let recruitment = initial
                    .factions
                    .iter()
                    .filter_map(|faction| {
                        faction
                            .recruits
                            .as_ref()
                            .map(|pool| (faction.id.clone(), pool.clone()))
                    })
                    .collect();
                ChapterProgram {
                    id: chapter.id().to_owned(),
                    map: ChapterLayout {
                        id: map.id().to_owned(),
                        revision: map.revision(),
                        units: initial.units.clone(),
                    },
                    subtitle: story.subtitle.clone(),
                    briefing: story.briefing.clone(),
                    debrief: story.debrief.clone(),
                    hints: story.hints.clone(),
                    intro: story.intro.clone(),
                    victory: story.victory.clone(),
                    scripted_events: events,
                    required_companions: scripts.required_companions.clone(),
                    enemy_groups: scripts.enemy_groups.clone(),
                    defend_objectives: scripts.defend_objectives,
                    enemy_leader_post: scripts.enemy_leader_post.clone(),
                    gold: initial.gold.clone(),
                    arrivals: map.roster().arrivals.clone(),
                    recruitment,
                    item_ids,
                    initial,
                }
            })
            .collect();
        Self {
            data: Arc::new(ProgramData {
                id: definition.id().to_owned(),
                name: definition.name().to_owned(),
                characters: definition.characters().to_vec(),
                carryover: definition.carryover().clone(),
                ending: definition.ending().to_vec(),
                chapters,
            }),
        }
    }
    pub fn id(&self) -> &str {
        &self.data.id
    }
    pub fn name(&self) -> &str {
        &self.data.name
    }
    pub fn characters(&self) -> &[Character] {
        &self.data.characters
    }
    pub fn carryover(&self) -> &CampaignCarryover {
        &self.data.carryover
    }
    pub fn ending(&self) -> &[Speech] {
        &self.data.ending
    }
    pub fn chapter_count(&self) -> usize {
        self.data.chapters.len()
    }
    pub(crate) fn chapters(&self) -> &[ChapterProgram] {
        &self.data.chapters
    }
    pub fn same_definition(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
    pub fn identity(&self) -> usize {
        Arc::as_ptr(&self.data) as usize
    }
}
impl ChapterProgram {
    pub(crate) fn initial_state(&self) -> State {
        self.initial.clone()
    }
    pub(crate) fn companion_ids(&self) -> impl Iterator<Item = &str> {
        self.map
            .units
            .iter()
            .filter_map(|unit| unit.character.as_deref())
            .chain(
                self.arrivals
                    .iter()
                    .map(|arrival| arrival.character.as_str()),
            )
            .chain(self.required_companions.iter().map(String::as_str))
    }
}
