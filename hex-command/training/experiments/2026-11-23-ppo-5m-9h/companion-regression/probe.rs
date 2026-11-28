use hex_command_core::{data::*, simulation::*, training::*};
use std::{fs,time::Instant};
#[global_allocator]
static ALLOCATOR:mimalloc::MiMalloc=mimalloc::MiMalloc;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();
 let mut model:Model=serde_json::from_str(&fs::read_to_string(&args[1])?)?;model.upgrade()?;
 let rules=Catalogue::load_from(std::path::PathBuf::from(&args[2]))?;
 let mut definitions=CampaignDefinitions::new(&rules);
 let cases:Vec<serde_json::Value>=serde_json::from_str(&fs::read_to_string(&args[3])?)?;
 for value in cases{
  let mut config:MatchConfig=serde_json::from_value(value.clone())?;
  match &mut config{
   MatchConfig::Arena(c)=>c.player_blocks=Some(learned_blocks(c.player_blocks.unwrap_or(c.player.blocks()))),
   MatchConfig::Campaign(c)=>c.player_blocks=Some(learned_blocks(c.player_blocks.unwrap_or(c.player.blocks()))),
  }
  definitions.prepare_match(&config)?;
  let start=Instant::now();
  let(result,_)=scoped(None,Some(model.clone()),None,||run(&rules,config,&definitions));
  let seconds=start.elapsed().as_secs_f64();
  let summary=match result?{
   MatchResult::Arena(r)=>serde_json::json!({"success":r.winner==Some(BattleResult::PlayerVictory),"truncated":r.winner.is_none(),"winner":r.winner,"rounds":r.rounds,"reason":r.reason}),
   MatchResult::Campaign(r)=>serde_json::json!({"success":r.completed,"truncated":r.missions.last().is_some_and(|m|m.winner.is_none()),"missions":r.missions})
  };
  println!("{}",serde_json::json!({"config":value,"result":summary,"seconds":seconds}));
 }
 Ok(())
}
