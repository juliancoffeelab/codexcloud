//! Mechanical battle API is owned by the internal engine crate.
pub use hex_command_battle::{
    BattleSession, StateError, conditions, damage, exchange, geometry, validate_state,
};
pub mod battle {
    pub use hex_command_battle::{BattleSession, Board, hex_distance};
}
#[cfg(not(target_arch = "wasm32"))]
pub use hex_command_battle::{validate_state_geometry, validate_state_geometry_with_campaign};

pub use hex_command_battle::data::{Command, CommandError};

#[cfg(any(
    feature = "wasm",
    all(feature = "simulation", not(target_arch = "wasm32"))
))]
pub(crate) fn place_selected_leader(
    rules: &crate::data::Catalogue,
    state: &mut crate::data::State,
    faction: &str,
    unit_type: &str,
) -> Result<(), &'static str> {
    use crate::data::{PublicUnit, Role};

    let tile = state
        .tiles
        .iter()
        .find(|tile| {
            tile.owner.as_deref() == Some(faction)
                && rules
                    .terrains()
                    .find(|terrain| terrain.id().as_str() == tile.terrain)
                    .expect("authored terrain")
                    .allows_recruitment()
        })
        .ok_or("Selected armies require an owned keep for each side")?
        .hex();
    state.units.push(PublicUnit {
        id: format!("u{}", state.next_id),
        r#type: unit_type.into(),
        faction: faction.into(),
        tile,
        hp: rules.unit(unit_type).max_hp,
        revealed_to: vec![],
        perception_effects: vec![],
        items: vec![],
        ai_group: None,
        moved: false,
        attack_spent: false,
        experience: Some(0),
        character: None,
        role: Some(Role::Leader),
        retreating: None,
        rebuilding: None,
    });
    state.next_id += 1;
    Ok(())
}
