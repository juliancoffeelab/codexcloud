//! Group exploration records terrain facts, never unseen enemy positions.
use super::BattlePosition;
use crate::data::*;
use std::rc::Rc;
impl BattlePosition<'_> {
    pub(crate) fn remember_faction_sight(&mut self) {
        let mut additions = Vec::new();
        for (index, group) in self.vision_groups.iter().enumerate() {
            let key = VisionGroupKey(index);
            let known = &self.fog_memory[&group.id];
            let mut discovered = Vec::new();
            for (tile, rule) in self.tile_perception.iter().enumerate() {
                if !rule.remember || known[tile / 64] & (1 << (tile % 64)) != 0 {
                    continue;
                }
                let met = |position| self.perception(position).sees_terrain(key, tile);
                if met(None)
                    || self.traversal.as_ref().is_some_and(|(actor, path)| {
                        self.units.iter().any(|u| u.id == *actor)
                            && path.iter().any(|&t| met(Some((*actor, t))))
                    })
                {
                    discovered.push(tile);
                }
            }
            if !discovered.is_empty() {
                additions.push((index, discovered));
            }
        }
        if additions.is_empty() {
            return;
        }
        let memory = Rc::make_mut(&mut self.runtime.fog_memory);
        let groups = Rc::make_mut(&mut self.runtime.vision_groups);
        for (index, tiles) in additions {
            let bits = memory
                .get_mut(&groups[index].id)
                .expect("group exploration cache");
            for tile in tiles {
                bits[tile / 64] |= 1 << (tile % 64);
                groups[index].explored.push(self.runtime.board.ids[tile]);
            }
        }
    }
}
