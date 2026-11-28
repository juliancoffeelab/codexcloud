use hex_command_core::{data::Catalogue, simulation::*, training::*};
use std::{fs, io::{self, Write}, path::PathBuf};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mut model: Option<Model> = if args[1] == "-" {
        None
    } else {
        Some(serde_json::from_str(&fs::read_to_string(&args[1])?)?)
    };
    if let Some(model) = &mut model {
        model.upgrade()?;
    }
    let rules = Catalogue::load_from(PathBuf::from(&args[2]))?;
    let mut definitions = CampaignDefinitions::new(&rules);
    let cases: Vec<MatchConfig> = serde_json::from_str(&fs::read_to_string(&args[3])?)?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    for mut config in cases {
        if let (Some(_), MatchConfig::Arena(arena)) = (&model, &mut config) {
            arena.player_blocks = Some(learned_blocks(arena.player_blocks.unwrap_or(arena.player.blocks())));
        }
        definitions.prepare_match(&config)?;
        let (result, _) = scoped(None, model.clone(), None, || run(&rules, config, &definitions));
        serde_json::to_writer(&mut output, &result?)?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
