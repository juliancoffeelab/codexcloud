use hex_command_core::{data::{Catalogue, State}, logic::BattleSession, simulation::*, training::*};
use serde_json::{Value, json};
use std::{fs, io::{BufWriter, Write}};

fn presentation(rules: &Catalogue, state: &State) -> Value {
    let game = BattleSession::from_state(rules, state.clone());
    json!(["player", "enemy"].map(|id| {
        let view = game.presentation_for(id);
        json!({"faction": id, "fog": view.fog, "visibleUnits": view.visible_units})
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let rules = Catalogue::load_from(&args[1])?;
    let jobs: Vec<Value> = serde_json::from_str(&fs::read_to_string(&args[2])?)?;
    let mut output = BufWriter::new(fs::File::create(&args[3])?);
    let model = if let Some(path) = args.get(4) {
        let mut model: Model = serde_json::from_str(&fs::read_to_string(path)?)?;
        model.upgrade()?;
        Some(model)
    } else {
        None
    };
    let mut definitions = CampaignDefinitions::new(&rules);
    for job in jobs {
        let mut config: MatchConfig = serde_json::from_value(job["config"].clone())?;
        let MatchConfig::Campaign(ref mut campaign) = config else {
            return Err("expected a campaign configuration".into());
        };
        if model.is_some() {
            campaign.player_blocks = Some(learned_blocks(campaign.player_blocks.unwrap_or(campaign.player.blocks())));
        }
        definitions.prepare_match(&config)?;
        let (result, _) = scoped(None, model.clone(), None, || run(&rules, config, &definitions));
        let MatchResult::Campaign(result) = result? else { unreachable!() };
        let mission = &result.missions[0];
        let observations = json!({
            "initial": presentation(&rules, &mission.initial),
            "final": presentation(&rules, &mission.r#final)
        });
        serde_json::to_writer(&mut output, &json!({"job": job, "result": result, "observations": observations}))?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
