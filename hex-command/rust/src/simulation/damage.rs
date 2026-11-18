//! Exact damage tables use the same distributions as live forecasts.
use super::error::SimulationError;
use crate::{data::Catalogue, logic::damage::distribution};
use hex_command_catalog::diagnostics;
use std::{fmt::Write, path::Path};

pub(super) fn write(catalogue: &Catalogue, directory: &Path) -> Result<(), SimulationError> {
    let mut csv = String::from(
        "unit,level,weapon,category,damageType,count,rollMin,rollMax,incomingModifier,min,max,expected,kill10,kill20\n",
    );
    for unit in catalogue.units() {
        for attack in diagnostics::attacks(unit) {
            for modifier in [0, -2, -4, 1] {
                let outcomes = diagnostics::attack_outcomes(attack, modifier, 0);
                let (low, high) = outcomes.bounds();
                let d = distribution(outcomes);
                writeln!(
                    csv,
                    "{},{},{},{:?},{:?},{},{},{},{},{},{},{:.4},{:.6},{:.6}",
                    unit.id(),
                    unit.level(),
                    attack.id(),
                    attack.category(),
                    attack.damage_type(),
                    outcomes.strike_count(),
                    low.raw,
                    high.raw,
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
