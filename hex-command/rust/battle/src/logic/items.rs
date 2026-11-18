//! Equipment moves with its owner, or remains on a tile when dropped.
use super::battle::BattlePosition;
use crate::data::*;
impl BattlePosition<'_> {
    pub(crate) fn can_equip(&self, target: &Unit, equipment: &[&ItemDefinition]) -> bool {
        let mut ids = std::collections::HashSet::new();
        if !equipment.iter().all(|item| ids.insert(item.id())) {
            return false;
        }
        let capabilities = self.definitions.unit(target.kind).compose(UnitInputs {
            vision_group_sight: &self.vision_groups[target.vision_group.0].sight,
            equipment,
            gained_abilities: &[],
            selected_effects: &[],
            map_modifiers: &self.map_modifiers,
            perception_contributions: &[],
        });
        let mut ids = std::collections::HashSet::new();
        capabilities.attacks().all(|attack| ids.insert(attack.id()))
    }
    pub(crate) fn arrival_tiles(
        &self,
        kind: usize,
        faction: NameKey,
        items: &[String],
        tiles: &[usize],
    ) -> Vec<usize> {
        let mut projection = self.fork();
        let inventory =
            std::sync::Arc::make_mut(&mut projection.runtime.inventories).load(items.to_vec());
        tiles
            .iter()
            .copied()
            .filter(|&tile| {
                let mut unit = projection.prospective(kind, faction, tile);
                unit.inventory = inventory;
                projection.with_capabilities_at(&unit, tile, &projection.units, &[], |caps| {
                    caps.step_cost(projection.terrain_definition(tile))
                        .is_some()
                })
            })
            .collect()
    }
    pub(crate) fn collect_items(&mut self, id: u32) {
        let index = self.index(id).unwrap();
        let tile = self.runtime.units[index].tile;
        let mut position = 0;
        while position < self.runtime.ground_items.len() {
            let ground = self.runtime.ground_items[position];
            let target = &self.runtime.units[index];
            let incoming = self
                .catalogue
                .item(self.runtime.inventories.item(ground.item));
            let mut equipment: Vec<_> = self.inventory(target).collect();
            equipment.push(incoming);
            let allowed = ground.tile == tile && self.can_equip(target, &equipment);
            if allowed {
                self.record(format!(
                    "{} picked up {}.",
                    self.unit_name(&self.runtime.units[index]),
                    incoming.name()
                ));
                self.give_existing_item(id, ground.item);
                self.runtime.ground_items.remove(position);
            } else {
                position += 1;
            }
        }
    }
}
