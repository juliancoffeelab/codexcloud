//! Tests author ordinary JSON documents before constructing the opaque Catalogue.
use crate::data::*;
use serde_json::{Value, json};
use std::path::Path;

pub fn bundle() -> JsonBundle {
    fn read(root: &Path, directory: &Path, result: &mut JsonBundle) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                read(root, &path, result);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                result.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    std::fs::read_to_string(path).unwrap(),
                );
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|directory| directory.join("data"))
        .find(|directory| directory.join("manifest.json").exists())
        .expect("authored test data");
    let mut result = JsonBundle::new();
    read(&root, &root, &mut result);
    result
}
pub fn edit(bundle: &mut JsonBundle, path: &str, change: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_str(&bundle[path]).unwrap();
    change(&mut value);
    bundle.insert(path.into(), serde_json::to_string(&value).unwrap());
}
pub fn unit_path(bundle: &JsonBundle, id: &str) -> String {
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    manifest["units"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .find(|path| serde_json::from_str::<Value>(&bundle[*path]).unwrap()["id"] == id)
        .expect("fixture unit")
        .to_owned()
}
pub fn unit(bundle: &mut JsonBundle, id: &str, change: impl FnOnce(&mut Value)) {
    let path = unit_path(bundle, id);
    edit(bundle, &path, change);
}
pub fn deterministic() -> JsonBundle {
    let mut bundle = bundle();
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    for path in manifest["units"].as_array().unwrap() {
        edit(&mut bundle, path.as_str().unwrap(), |unit| {
            for attack in unit["attacks"].as_array_mut().unwrap() {
                attack["spread"] = json!(0);
            }
        });
    }
    bundle
}
pub fn effects(bundle: &mut JsonBundle, id: &str, change: impl FnOnce(&mut Vec<ScopedEffect>)) {
    let path = unit_path(bundle, id);
    let unit: Value = serde_json::from_str(&bundle[&path]).unwrap();
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    let mut effects = Vec::new();
    for ability in unit["abilities"].as_array().unwrap() {
        let source = manifest["abilities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|path| path.as_str().unwrap())
            .find_map(|path| {
                let value: Value = serde_json::from_str(&bundle[path]).unwrap();
                (value["id"] == *ability).then_some(value)
            })
            .expect("fixture ability");
        effects.extend(
            serde_json::from_value::<Vec<ScopedEffect>>(source["effects"].clone()).unwrap(),
        );
    }
    change(&mut effects);
    let ability_id = format!("fixture-{id}");
    let ability_path = format!("abilities/{ability_id}.json");
    bundle.insert(ability_path.clone(), json!({"id":ability_id,"name":"Fixture effects","description":"Authored mechanical fixture","effects":effects}).to_string());
    edit(bundle, "manifest.json", |manifest| {
        let abilities = manifest["abilities"].as_array_mut().unwrap();
        if !abilities.iter().any(|path| path == &ability_path) {
            abilities.push(json!(ability_path));
        }
    });
    edit(bundle, &path, |unit| {
        unit["abilities"] = json!([ability_id])
    });
}
pub fn environment(bundle: &mut JsonBundle, id: &str, damage: i32) {
    let terrain: Value = serde_json::from_str(&bundle["game/terrain.json"]).unwrap();
    let roles: std::collections::BTreeSet<_> = terrain
        .as_array()
        .unwrap()
        .iter()
        .map(|terrain| terrain["role"].as_str().unwrap().to_owned())
        .collect();
    unit(bundle, id, |unit| {
        for role in roles {
            let modifiers = unit["terrainModifiers"].as_object_mut().unwrap();
            let value = modifiers.entry(role).or_insert_with(|| json!({"cost":0,"passable":null,"damageModifiers":{"melee":0,"ranged":0,"blade":0,"pierce":0,"impact":0}}));
            value["damageModifiers"] =
                json!({"melee":damage,"ranged":damage,"blade":0,"pierce":0,"impact":0});
        }
    });
}
pub fn neutral_combat() -> JsonBundle {
    let mut bundle = deterministic();
    edit(&mut bundle, "game/terrain.json", |terrains| {
        for terrain in terrains.as_array_mut().unwrap() {
            terrain["damageModifiers"] =
                json!({"melee":0,"ranged":0,"blade":0,"pierce":0,"impact":0});
        }
    });
    for id in ["soldier", "archer", "scout"] {
        unit(&mut bundle, id, |unit| {
            unit["hp"] = json!(100);
            unit["damageModifiers"] = json!({"blade":0,"pierce":0,"impact":0});
            for attack in unit["attacks"].as_array_mut().unwrap() {
                attack["spread"] = json!(0);
                attack["damage"] = json!(4);
                attack["count"] = json!(2);
                attack["drain"] = json!(0);
            }
        });
        environment(&mut bundle, id, 0);
        crate::effect_fixture::thorns(&mut bundle, id, 0);
        crate::effect_fixture::thorn_aura(&mut bundle, id, 0);
    }
    bundle
}
pub fn item(bundle: &mut JsonBundle, id: &str, value: Value) {
    let path = format!("game/items/{id}.json");
    bundle.insert(path.clone(), value.to_string());
    edit(bundle, "manifest.json", |manifest| {
        let items = manifest["items"].as_array_mut().unwrap();
        if !items.iter().any(|entry| entry == &path) {
            items.push(json!(path));
        }
    });
}
pub fn campaign_events(bundle: &mut JsonBundle, id: &str, chapter: usize, events: Value) {
    let campaign_path = format!("game/campaigns/{id}/campaign.json");
    let campaign: Value = serde_json::from_str(&bundle[&campaign_path]).unwrap();
    let stage_path = Path::new(&campaign_path)
        .parent()
        .unwrap()
        .join(campaign["stages"][chapter].as_str().unwrap());
    let stage: Value = serde_json::from_str(&bundle[stage_path.to_str().unwrap()]).unwrap();
    let scripts_path = stage_path
        .parent()
        .unwrap()
        .join(stage["scripts"].as_str().unwrap());
    edit(bundle, scripts_path.to_str().unwrap(), |scripts| {
        scripts["events"] = events
    });
}
pub fn ability(bundle: &mut JsonBundle, id: &str, effects: Vec<ScopedEffect>) {
    let path = format!("abilities/{id}.json");
    bundle.insert(path.clone(), json!({"id":id,"name":"Fixture ability","description":"Authored fixture ability","effects":effects}).to_string());
    edit(bundle, "manifest.json", |manifest| {
        let abilities = manifest["abilities"].as_array_mut().unwrap();
        if !abilities.iter().any(|entry| entry == &path) {
            abilities.push(json!(path));
        }
    });
}
pub fn kind(bundle: &JsonBundle, id: &str) -> usize {
    let path = unit_path(bundle, id);
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    manifest["units"]
        .as_array()
        .unwrap()
        .iter()
        .position(|entry| entry == &path)
        .unwrap()
}
pub fn unit_kind(bundle: &mut JsonBundle, kind: usize, change: impl FnOnce(&mut Value)) {
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    edit(bundle, manifest["units"][kind].as_str().unwrap(), change);
}
pub fn id(bundle: &JsonBundle, kind: usize) -> String {
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    let value: Value =
        serde_json::from_str(&bundle[manifest["units"][kind].as_str().unwrap()]).unwrap();
    value["id"].as_str().unwrap().to_owned()
}
pub fn copy_unit(bundle: &mut JsonBundle, source: usize, target: usize) {
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    let source_path = manifest["units"][source].as_str().unwrap();
    let target_path = manifest["units"][target].as_str().unwrap();
    let mut source: Value = serde_json::from_str(&bundle[source_path]).unwrap();
    let target: Value = serde_json::from_str(&bundle[target_path]).unwrap();
    source["id"] = target["id"].clone();
    bundle.insert(target_path.to_owned(), source.to_string());
}
/// Test battles borrow immutable parsed catalogues. Cache distinct authored fixtures
/// for the test process, rather than retaining mutable source-stat tables.
pub fn frozen(bundle: &JsonBundle) -> &'static Catalogue {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static FIXTURES: OnceLock<Mutex<HashMap<[u8; 32], &'static Catalogue>>> = OnceLock::new();
    let catalogue = Catalogue::from_json_bundle(bundle).unwrap();
    let mut cache = FIXTURES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    *cache
        .entry(catalogue.content_fingerprint)
        .or_insert_with(|| Box::leak(Box::new(catalogue)))
}
pub fn value(bundle: &JsonBundle, kind: usize) -> Value {
    let manifest: Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    serde_json::from_str(&bundle[manifest["units"][kind].as_str().unwrap()]).unwrap()
}
