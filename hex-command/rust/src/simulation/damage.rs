//! Exact damage tables use the same distributions as live forecasts.
use super::error::SimulationError;
use crate::{data::Rules, logic::damage::distribution};
use std::{fmt::Write, path::Path};

pub(super) fn write(rules: &Rules, directory: &Path) -> Result<(), SimulationError> {
    let mut csv = String::from(
        "unit,level,weapon,category,damageType,count,rollMin,rollMax,incomingModifier,min,max,expected,kill10,kill20\n",
    );
    for unit in rules.units.iter() {
        for attack in unit.attacks.iter() {
            for modifier in [0, -2, -4, 1] {
                let d = distribution(attack, modifier);
                writeln!(
                    csv,
                    "{},{},{},{:?},{:?},{},{},{},{},{},{},{:.4},{:.6},{:.6}",
                    unit.id,
                    unit.level,
                    attack.id,
                    attack.category,
                    attack.damage_type,
                    attack.count,
                    attack.damage - attack.spread,
                    attack.damage + attack.spread,
                    modifier,
                    d.min,
                    d.max,
                    d.mean,
                    d.chance(10),
                    d.chance(20)
                )
                .unwrap();
            }
        }
    }
    std::fs::write(directory.join("damage-tables.csv"), csv)?;
    Ok(())
}
