use hex_command_core::{ai::Blocks, data::*, simulation::*, training::*};
use std::{fs, io::Write};
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let rules = Catalogue::load_from(std::path::PathBuf::from(&args[1]))?;
    let mut definitions = CampaignDefinitions::new(&rules);
    let jobs: Vec<serde_json::Value> = serde_json::from_str(&fs::read_to_string(&args[2])?)?;
    for job in jobs {
        let mut config: MatchConfig = serde_json::from_value(job["config"].clone())?;
        let teacher: PretrainImitator = serde_json::from_value(job["teacher"].clone())?;
        let controller = imitator::controller(&teacher, &rules, &config)?;
        let mut blocks = match &config {
            MatchConfig::Arena(c) => c.player_blocks.unwrap_or(c.player.blocks()),
            MatchConfig::Campaign(c) => c.player_blocks.unwrap_or(c.player.blocks()),
        };
        if controller.is_some() {
            blocks = learned_blocks(blocks);
        } else {
            match &teacher {
                PretrainImitator::Named(name) => blocks.planner = serde_json::from_value(serde_json::json!(name))?,
                PretrainImitator::Blocks { .. } => blocks = serde_json::from_value::<Blocks>(serde_json::to_value(&teacher)?)?,
                _ => unreachable!(),
            }
        }
        match &mut config {
            MatchConfig::Arena(c) => c.player_blocks = Some(blocks),
            MatchConfig::Campaign(c) => c.player_blocks = Some(blocks),
        }
        definitions.prepare_match(&config)?;
        let data = if controller.is_some() { Some(Dataset { spatial: true, ..Default::default() }) } else { None };
        let (result, _) = scoped(data, None, controller, || run(&rules, config, &definitions));
        let summary = match result? {
            MatchResult::Arena(r) => serde_json::json!({"success":r.winner==Some(BattleResult::PlayerVictory),"truncated":r.winner.is_none(),"rounds":r.rounds,"reason":r.reason}),
            MatchResult::Campaign(r) => serde_json::json!({"success":r.completed,"truncated":r.missions.last().is_some_and(|m|m.winner.is_none()),"missions":r.missions.iter().map(|m|serde_json::json!({"id":m.id,"round":m.round,"winner":m.winner})).collect::<Vec<_>>()}),
        };
        println!("{}", serde_json::json!({"config":job["config"],"teacher":job["teacher"],"result":summary}));
        std::io::stdout().flush()?;
    }
    Ok(())
}
