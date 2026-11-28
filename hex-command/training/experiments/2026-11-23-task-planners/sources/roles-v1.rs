use hex_command_core::training::{Choices,DecisionController,spatial::{ACTOR_CHANNELS,KERNEL}};
use hex_command_battle::RuntimeCommand;
use serde_json::{Value,json};
use std::{collections::{HashMap,BinaryHeap},cmp::Reverse,rc::Rc,cell::RefCell};
fn distance(a:usize,b:usize,w:usize)->f32 {let ar=(a/w) as i32;let aq=(a%w) as i32;let br=(b/w) as i32;let bq=(b%w) as i32;((ar-br).abs()+(aq-bq).abs()+(ar+aq-br-bq).abs()) as f32/2.}
fn uid(c:&RuntimeCommand)->Option<u32>{match c{RuntimeCommand::Move{unit,..}|RuntimeCommand::Attack{unit,..}|RuntimeCommand::Advance{unit,..}=>Some(unit.0),_=>None}}
fn val(v:&Value,k:&str)->usize{v[k].as_u64().unwrap_or(0) as usize}
fn distances(c:&Choices,actor:usize,goal:usize,risk:&[f32],alpha:f32)->Vec<f32>{
 let b=&c.spatial.as_ref().unwrap().board;let n=b.width*b.height;let Some(ctx)=b.contexts.iter().find(|x|x.actor_site==actor)else{return vec![f32::INFINITY;n]};
 let cost=ACTOR_CHANNELS.iter().position(|&x|x==38).unwrap();let pass=ACTOR_CHANNELS.iter().position(|&x|x==39).unwrap();
 let mut d=vec![f32::INFINITY;n];let mut q=BinaryHeap::new();d[goal]=0.;q.push(Reverse((0u32,goal)));
 while let Some(Reverse((price,p)))=q.pop(){if price as f32>d[p]*100.+1.{continue;}let row=(p/b.width) as i32;let col=(p%b.width) as i32;let edge=ctx.values[cost*n+p].max(0.1)+alpha*if p==goal{0.}else{risk[p]};
  for &(dr,dq) in KERNEL.iter().skip(1){let r=row+dr;let k=col+dq;if r<0||k<0||r>=b.height as i32||k>=b.width as i32{continue;}let next=r as usize*b.width+k as usize;if b.values[next]<0.5||ctx.values[pass*n+next]<0.5{continue;}let candidate=d[p]+edge;if candidate+0.0001<d[next]{d[next]=candidate;q.push(Reverse(((candidate*100.).round() as u32,next)));}}
 }d
}
pub fn controller(plan:Value,log:Rc<RefCell<Vec<Value>>>)->DecisionController{
 let task=plan["task"].as_str().unwrap().to_owned();let variant=val(&plan,"variant");let anchor=val(&plan,"anchor");let home=val(&plan,"home");let staging=val(&plan,"staging");let supply=val(&plan,"supply");let healer_home=val(&plan,"healer_home");let mut sites:HashMap<u32,usize>=HashMap::new();let mut previous:HashMap<u32,usize>=HashMap::new();let mut supplied=false;
 Box::new(move|c:&Choices|{
 let b=&c.spatial.as_ref().unwrap().board;let n=b.width*b.height;let round=c.context.round;
 let mut actors=HashMap::new();for (i,cmd) in c.commands.iter().enumerate(){if let Some(id)=uid(cmd){let ix=c.spatial.as_ref().unwrap().indices[i];if ix[0]>=0{sites.insert(id,ix[0] as usize);actors.entry(id).or_insert(i);}}}
 sites.retain(|_,p|*p<n&&b.values[n+*p]>0.5);
 let foes:Vec<_>=(0..n).filter(|&p|b.values[2*n+p]>0.5).collect();let friendly:Vec<_>=(0..n).filter(|&p|b.values[n+p]>0.5).collect();let healers:Vec<_>=(0..n).filter(|&p|b.values[n+p]>0.5&&(b.values[10*n+p]>0.||b.values[11*n+p]>0.)).collect();
 let marked:Vec<_>=foes.iter().copied().filter(|&p|b.values[51*n+p]>0.5).collect();let mut risk=vec![0.;n];
 for &e in &foes{let melee=b.values[18*n+e]*10.*b.values[19*n+e]*5.;let ranged=b.values[26*n+e]*10.*b.values[27*n+e]*5.;let reach=(b.values[20*n+e]*5.).max(b.values[28*n+e]*5.).max(1.);let movement=b.values[6*n+e]*10.;for p in 0..n{let d=distance(p,e,b.width);if d<=reach+movement {risk[p]+=melee.max(ranged)*if d<=reach{1.}else{0.5};}}}
 let holder=if task=="tide"{2}else if task=="gallery"{5}else{0};let holder_site=sites.get(&holder).copied().unwrap_or(anchor);
 if supply<n&&b.values[42*n+supply]>0.5{supplied=true;}
 let nearby=sites.iter().filter(|(id,p)|**id!=holder&&**id!=1&&distance(**p,anchor,b.width)<=1.).count();
 let strong_near=sites.iter().filter(|(id,p)|**id!=holder&&**id!=1&&distance(**p,anchor,b.width)<=3.&&b.values[35*n+**p]>0.6).count();
 let ready=if task=="tide"{nearby>=if variant==0{2}else{1}}else{strong_near>=if variant==0{3}else{2}&&foes.iter().filter(|&&e|distance(e,anchor,b.width)<=4.).count()<=if variant==0{2}else{4}};
 let mut routes:HashMap<u32,Vec<f32>>=HashMap::new();let mut goals:HashMap<u32,usize>=HashMap::new();let mut stage=String::new();
 for (&id,&i) in &actors{
  let f=&c.features[i];let at=sites[&id];let mut g=anchor;
  if task=="tide"{
   if id==1{g=home;}else if id==2{g=if ready||at==anchor{anchor}else{staging};stage=if ready{"capture/hold"}else{"form screen"}.into();}
   else if (id==5||id==6)&&!supplied{g=supply;}
   else if id==3{g=val(&plan,"guard1");}else if id==4{g=val(&plan,"guard2");}else if id==11{g=val(&plan,"guard3");}else{g=anchor;}
  }else if task=="briar"{
   let wounded=sites.get(&2).is_some_and(|&p|b.values[35*n+p]<if variant==0{0.85}else{0.7});
   if id==1{g=home;}else if id==2&&wounded{g=staging;}else if id==4{g=if wounded{healer_home}else{sites.get(&2).copied().unwrap_or(staging)};}
   else {g=marked.iter().chain(foes.iter()).min_by_key(|&&p|distance(at,p,b.width) as u32).copied().unwrap_or(anchor);}
  }else if task=="rally"{
   if id==1{g=home;}else if (id==2||id==18)&&variant==0{g=staging;}
   else if f[8]<0.45{g=(0..n).filter(|&p|b.values[41*n+p]>0.5&&b.values[42*n+p]>0.5).min_by_key(|&p|distance(at,p,b.width) as u32).unwrap_or(home);}
  }else if task=="crown"{
   if id==1{g=home;}
   else if round<if variant==0{4}else{3}{g=staging;}
  }else if task=="gallery"{
   if id==holder{g=if ready||at==anchor{anchor}else{staging};stage=if ready{"escort/hold"}else{"wait for guard"}.into();}
   else if id==1{g=home;}else if id==10||id==12||id==4{g=if ready{val(&plan,if id==10{"guard1"}else if id==12{"guard2"}else{"guard3"})}else{holder_site};}
  }
  if g>=n||b.values[g]<0.5{g=anchor;}goals.insert(id,g);
  let alpha=if task=="crown"{if variant==0{0.15}else{0.3}}else if task=="rally"{0.05}else{0.02};routes.insert(id,distances(c,at,g,&risk,alpha));
 }
 let mut best=(0,0f32);let mut chosen=Value::Null;
 for (i,(cmd,f)) in c.commands.iter().zip(&c.features).enumerate().skip(1){let Some(id)=uid(cmd)else{continue};let ix=c.spatial.as_ref().unwrap().indices[i];if ix[0]<0{continue;}let at=ix[0] as usize;let critical=id==1||id==holder||(task=="briar"&&id==2);let mut score;
 if f[3]>0.5{score=300.;}
 else if f[2]>0.5{
  if (critical&&f[27]>f[9]*0.45)||(!critical&&f[27]>=f[9]&&f[28]<0.8){continue;}
  if task=="briar"&&id==2&&f[8]<if variant==0{0.85}else{0.7}{continue;}
  score=100.+120.*f[28]+35.*(f[26]-if critical{f[27]}else{0.35*f[27]});
  if ix[2]>=0{let t=ix[2] as usize;if b.values[51*n+t]>0.5{score+=60.;}if (task=="tide"||task=="gallery")&&distance(t,holder_site,b.width)<=4.{score+=30.;}}
 }else if f[1]>0.5&&ix[1]>=0{
  let to=ix[1] as usize;let goal=goals[&id];let route=&routes[&id];if !route[at].is_finite()||!route[to].is_finite(){continue;}
  if (task=="tide"||task=="gallery")&&id==holder&&at==anchor{continue;}
  if (task=="crown"||task=="rally")&&to==anchor{score=10000.;}
  else{
   let gain=route[at]-route[to];let safer=risk[at]-risk[to];if gain<=0.01&&!(critical&&safer>4.){continue;}
   score=8.*gain-0.3*route[to]+f[21]*8.;if critical{score+=safer*0.8;}
   if (task=="tide"||task=="gallery")&&id==holder{score+=60.;}
   if task=="briar"&&id==2&&f[8]<0.85{score+=60.;}
   if task=="tide"&&(id==3||id==4||id==11)&&!ready{score+=20.;}
   if task=="tide"&&(id==5||id==6)&&!supplied{score+=20.;}
   if previous.get(&id)==Some(&to)&&!critical{score-=20.;}
   if id==1&&risk[to]>f[9]*30.*0.7{score-=100.;}
   if to==goal{score+=4.;}
  }
 }else{continue;}
 if score>best.1{best=(i,score);chosen=json!({"unit":id,"origin":at,"goal":goals.get(&id),"score":score,"hp":f[9]*30.,"protectedFeature":f[15],"occupyAssigned":f[179],"command":format!("{:?}",cmd)});}
 }
 if let Some(id)=uid(&c.commands[best.0]){let ix=c.spatial.as_ref().unwrap().indices[best.0];if c.features[best.0][1]>0.5&&ix[1]>=0{previous.insert(id,ix[0] as usize);sites.insert(id,ix[1] as usize);}}
 if log.borrow().len()<600{log.borrow_mut().push(json!({"context":c.context,"boardSize":[b.height,b.width],"supplyOwned":b.values[42*n+supply],"round":round,"stage":stage,"supplied":supplied,"ready":ready,"nearby":nearby,"strongNear":strong_near,"choice":chosen}));}
 Some(best.0)
 })
}
