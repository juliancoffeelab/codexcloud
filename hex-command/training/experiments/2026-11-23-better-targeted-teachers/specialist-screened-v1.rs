#[path="/workspace/codexcloud/hex-command/tmp/rl/targeted-map-1-draft/probe/src/objective_bot.rs"]
mod path_bot;
fn distance(a:usize,b:usize,w:usize)->i32 {
 let ar=(a/w) as i32; let aq=(a%w) as i32; let br=(b/w) as i32; let bq=(b%w) as i32;
 ((ar-br).abs()+(aq-bq).abs()+((ar+aq)-(br+bq)).abs())/2
}
fn specialist(kind:String,anchor:usize)->DecisionController {
 Box::new(move |c:&Choices| {
  let s=c.spatial.as_ref().unwrap();let b=&s.board;let n=b.width*b.height;
  let foes:Vec<_>=(0..n).filter(|&p|b.values[2*n+p]>0.5).collect();
  let friends:Vec<_>=(0..n).filter(|&p|b.values[n+p]>0.5).collect();
  if kind=="pursue_visible" || kind=="gank_visible" {
   let marked:Vec<_>=foes.iter().copied().filter(|&p|b.values[51*n+p]>0.5).collect();
   let targets=if marked.is_empty(){&foes}else{&marked};
   let goal=targets.iter().min_by_key(|&&p|friends.iter().map(|&a|distance(a,p,b.width)).min().unwrap_or(999));
   if let Some(&g)=goal {
    let fallback=path_bot::controller(g)(c);
    if kind=="gank_visible" {
     let mut best:Option<(usize,f32)>=None;
     for (i,(f,ix)) in c.features.iter().zip(&s.indices).enumerate().skip(1) {
      if f[10]>0.5 || f[15]>0.5 || f[2]<0.5 || ix[2]!=g as i64 || f[26]<=0. {continue;}
      let score=120.*f[28]+40.*f[26]-10.*f[27];
      if best.is_none_or(|(_,v)|score>v){best=Some((i,score));}
     }
     if let Some((i,_))=best {return Some(i);}
    }
    return fallback;
   }else{return Some(0);}

  }
  let protected:Vec<_>=c.features.iter().zip(&s.indices).filter(|(f,ix)|f[15]>0.5&&ix[0]>=0).map(|(_,ix)|ix[0] as usize).collect();
  let threat=foes.iter().min_by_key(|&&p|distance(p,anchor,b.width)).copied();
  let goal=threat.filter(|&p|distance(p,anchor,b.width)<=7).unwrap_or(anchor);
  let mut best=(0,0f32);
  for (i,(f,ix)) in c.features.iter().zip(&s.indices).enumerate().skip(1) {
   if ix[0]<0 {continue;}let actor=ix[0] as usize;let holding=f[15]>0.5&&actor==anchor;
   let score=if f[3]>0.5 {120.}
    else if f[2]>0.5 {
     if f[27]>=f[9] && f[28]<0.9 {continue;}
     80.+80.*f[28]+30.*(f[26]-0.25*f[27]) + if ix[2]>=0&&distance(ix[2] as usize,anchor,b.width)<=3 {25.}else{0.}
    }else if f[1]>0.5&&ix[1]>=0 {
     if holding {continue;}let to=ix[1] as usize;
     if f[15]>0.5 {100.+15.*(distance(actor,anchor,b.width)-distance(to,anchor,b.width)) as f32-distance(to,anchor,b.width) as f32}
     else if actor==anchor&&!protected.contains(&anchor) {90.-distance(to,anchor,b.width) as f32}
     else {
      let gain=distance(actor,goal,b.width)-distance(to,goal,b.width);
      let adj=foes.iter().filter(|&&p|distance(p,to,b.width)<=1).count();
      10.*gain as f32 - 0.3*distance(to,goal,b.width) as f32 - 3.*adj.saturating_sub(1) as f32
       + if f[38]>0.&&distance(to,anchor,b.width)==1 {8.}else{0.}
     }
    }else{continue;};
   if score>best.1 {best=(i,score);}
  }
  Some(best.0)
 })
}
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
        let custom=job.get("specialist");
        let teacher: PretrainImitator = if custom.is_some(){PretrainImitator::Named("queued".into())}else{serde_json::from_value(job["teacher"].clone())?};
        let controller = if let Some(kind)=custom {Some(specialist(kind.as_str().unwrap().to_owned(),job["anchor"].as_u64().unwrap_or(0) as usize))}else{imitator::controller(&teacher, &rules, &config)?};
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
        println!("{}", serde_json::json!({"role":job["role"],"config":job["config"],"teacher":job.get("specialist").unwrap_or(&job["teacher"]),"result":summary}));
        std::io::stdout().flush()?;
    }
    Ok(())
}
