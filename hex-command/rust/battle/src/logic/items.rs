//! Equipment moves with its owner, or remains on a tile when dropped.
use super::battle::Game;
use crate::data::*;
impl Game<'_> {
    pub(crate) fn equipment_modifier(&self, unit: &Unit, damage: DamageType) -> i32 {
        unit.items
            .iter()
            .filter_map(|item| item.armor.as_ref())
            .map(|armor| armor.get(damage))
            .sum()
    }
    pub(crate) fn collect_items(&mut self, id: u32) {
        let index = self.index(id).unwrap();
        let tile = self.board.ids[self.units[index].tile].clone();
        let mut remaining = vec![];
        for ground in std::mem::take(&mut self.ground_items) {
            let mut inventory = self.units[index].items.clone();
            inventory.push(ground.item.clone());
            if ground.tile == tile
                && validate_equipment(
                    &inventory,
                    &self.rules.units[self.units[index].kind].attacks,
                    "pickup",
                )
                .is_ok()
            {
                self.record(format!(
                    "{} picked up {}.",
                    self.unit_name(&self.units[index]),
                    ground.item.name
                ));
                self.units[index].items = inventory;
            } else {
                remaining.push(ground);
            }
        }
        self.ground_items = remaining;
    }
    pub(crate) fn healing_from(&self, provider: &Unit, target: &Unit) -> i32 {
        if !self.allied(provider, target) {
            return 0;
        }
        let distance = self.board.distance(provider.tile, target.tile);
        let innate = if provider.id != target.id && distance == 1 {
            self.rules.units[provider.kind].healing
        } else {
            0
        };
        provider
            .items
            .iter()
            .filter_map(|item| item.healing.as_ref())
            .filter(|aura| distance <= aura.radius)
            .map(|aura| aura.amount)
            .max()
            .unwrap_or(0)
            .max(innate)
    }
}
