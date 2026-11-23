use hex_command_core::{data::*, logic::BattleSession, simulation::CampaignDefinitions};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args_os().nth(1).ok_or("catalog is required")?;
    let rules = Catalogue::load_from(std::path::PathBuf::from(root))?;
    let mut definitions = CampaignDefinitions::new(&rules);
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let fixture: Value = serde_json::from_str(&line?)?;
        let campaign = fixture["config"]["campaign"].as_str().ok_or("campaign is required")?;
        let objective = fixture["config"]["objective"].as_str().ok_or("objective is required")?;
        definitions.prepare(campaign, objective)?;
        let definition = definitions.get(campaign, objective)?;
        for position in fixture["positions"].as_array().ok_or("positions are required")? {
            let raw = &position["request"]["state"];
            let mut game = BattleSession::from_state_with_campaign(&rules, serde_json::from_value(raw.clone())?, Some(definition));
            let acting = if fixture["side"] == "player" { "player" } else { "enemy" };
            game.set_active_faction(Some(acting.into()));
            let view = game.view();
            for faction in ["player", "enemy"] {
                let presentation = game.presentation_for(faction);
                let living_observers = game.authoritative_units().iter()
                    .filter(|unit| game.unit_faction(unit) == faction).count();
                let hostile: Vec<_> = game.authoritative_units().iter()
                    .filter(|unit| game.faction_relation(faction, game.unit_faction(unit)) == Relation::Hostile)
                    .map(|unit| json!({"id":format!("u{}", unit.id), "tile":view.board().ids[unit.tile]}))
                    .collect();
                if faction == acting {
                    let planner: Vec<_> = view.units().iter().map(|unit| format!("u{}", unit.id)).collect();
                    for unit in &hostile {
                        let id = unit["id"].as_str().unwrap();
                        assert_eq!(planner.iter().any(|v| v == id), presentation.visible_units.iter().any(|v| v == id));
                    }
                }
                serde_json::to_writer(&mut output, &json!({
                    "faction":faction, "config":fixture["config"], "label":position["label"],
                    "side":fixture["side"], "round":view.round(), "fog":presentation.fog,
                    "visible":presentation.visible_units, "hostile":hostile, "living_observers":living_observers,
                }))?;
                writeln!(output)?;
            }
        }
    }
    Ok(())
}
