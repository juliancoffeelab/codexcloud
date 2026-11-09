//! Equipment moves with its owner, or remains on a tile when dropped.
use super::battle::Game;
use crate::data::*;
impl Game<'_> {
    #[inline]
    pub(crate) fn equipment_modifier(&self, unit: &Unit, damage: DamageType) -> i32 {
        if unit.inventory.0 == 0 {
            0
        } else {
            self.runtime.inventories.modifier(unit.inventory, damage)
        }
    }
    pub(crate) fn collect_items(&mut self, id: u32) {
        let index = self.index(id).unwrap();
        let tile = self.runtime.units[index].tile;
        let mut position = 0;
        while position < self.runtime.ground_items.len() {
            let ground = self.runtime.ground_items[position];
            let allowed = ground.tile == tile
                && validate_equipment(
                    self.inventory(&self.runtime.units[index])
                        .chain(std::iter::once(self.runtime.inventories.item(ground.item))),
                    &self.rules.units[self.runtime.units[index].kind].attacks,
                    "pickup",
                )
                .is_ok();
            if allowed {
                self.record(format!(
                    "{} picked up {}.",
                    self.unit_name(&self.runtime.units[index]),
                    self.runtime.inventories.item(ground.item).name
                ));
                self.give_existing_item(id, ground.item);
                self.runtime.ground_items.remove(position);
            } else {
                position += 1;
            }
        }
    }
    pub(crate) fn healing_from(&self, provider: &Unit, target: &Unit) -> i32 {
        if !self.allied(provider, target) {
            return 0;
        }
        let distance = self.runtime.board.distance(provider.tile, target.tile);
        let innate = if provider.id != target.id && distance == 1 {
            self.mechanics.units[provider.kind].healing
        } else {
            0
        };
        self.inventory(provider)
            .filter_map(|item| item.healing.as_ref())
            .filter(|aura| distance <= aura.radius)
            .map(|aura| aura.amount)
            .max()
            .unwrap_or(0)
            .max(innate)
    }
}
