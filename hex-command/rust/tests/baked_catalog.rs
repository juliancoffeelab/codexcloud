#![cfg(feature = "wasm")]

use hex_command_core::data::{BAKED_RULES, Rules, Seed};

// Compare the complete resolved catalog, including every deferred map and its metadata.
// Destructuring deliberately requires new catalog fields to join the consistency check.
fn snapshot(rules: &Rules) -> serde_json::Value {
    let Rules {
        content_fingerprint,
        recruitment,
        rosters,
        arena,
        campaign_defaults,
        terrain_names,
        terrain_types,
        terrain_labels,
        terrain_definitions,
        race_definitions,
        map_definitions,
        units,
        movement_classes,
        route_classes,
        types,
        terrains,
        maps,
        arena_maps,
        campaigns,
    } = rules;
    serde_json::json!({
        "fingerprint": content_fingerprint,
        "recruitment": recruitment,
        "rosters": rosters,
        "arena": arena,
        "campaignDefaults": campaign_defaults,
        "terrainNames": terrain_names,
        "terrainTypes": terrain_types,
        "terrainLabels": terrain_labels,
        "terrain": terrain_definitions,
        "races": race_definitions,
        "mapDefinitions": map_definitions.iter().map(Seed::instantiate).collect::<Vec<_>>(),
        "units": units,
        "movementClasses": movement_classes,
        "routeClasses": route_classes,
        "types": types,
        "terrains": terrains,
        "maps": maps.iter().map(|(key, seed)| (key, seed.instantiate())).collect::<Vec<_>>(),
        "arenaMaps": arena_maps.iter().map(|(key, seed)| (key, seed.instantiate())).collect::<Vec<_>>(),
        "campaigns": campaigns,
    })
}

#[test]
fn baked_catalog_matches_native_resolution() {
    let native = Rules::load_from(concat!(env!("CARGO_MANIFEST_DIR"), "/data")).unwrap();
    assert_eq!(snapshot(&BAKED_RULES), snapshot(&native));
}
