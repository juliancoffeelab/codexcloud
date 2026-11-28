from pathlib import Path
from collections import defaultdict
import json,math,gzip,hashlib,shutil
import torch
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
ROOT=Path(__file__).resolve().parent
NAMES={'rl-deadline-pines-relief8':'Splitpine Dispatch','rl-route-ambush':'Echo Gate','rl-route-pass-road':'North of Lantern Road','rl-defend-lower-gallery-0':'Lower Gallery Refuge','rl-interrupt-herring-tide-road-three':'Herring Tide Road Three','rl-lair-outworks':'Lair Outworks','rl-hollowbank-extraction-crossfire-late-guard':'Hollowbank east-road guard','rl-hollowbank-extraction-crossfire-nearside':'Hollowbank west-bank reserve','rl-ashbank-relay-fork-vigil':'Ashbank Relay','rl-ashbank-relay-fork-vigil-player-fog':'Ashbank Relay','rl-ashbank-relay-fork-vigil-both-fog':'Ashbank Relay'}
def main():
 records=json.loads((ROOT/'evaluation/summary.json').read_text());base=torch.load(ROOT/'hour10-policy.pt',map_location='cpu',weights_only=True);drift={}
 for record in records:
  if record['minute']==0:record['active_seconds']=0.;continue
  state=torch.load(Path(record['model']).parent/'resume.pt',map_location='cpu',weights_only=False);record['active_seconds']=state['elapsed'];current=state['policy'];parts={}
  for prefix in ['convs.','coarse.','gru.','head.']:
   keys=[k for k in base if k.startswith(prefix)];delta=sum(float(((current[k]-base[k]).float()**2).sum()) for k in keys);norm=sum(float((base[k].float()**2).sum()) for k in keys);parts[prefix]=dict(l2=math.sqrt(delta),relative_l2=math.sqrt(delta/max(norm,1e-30)))
  drift[str(record['minute'])]=parts
 minutes=[r['minute'] for r in records];by_map=defaultdict(dict);conditions=defaultdict(dict);chapters=defaultdict(dict);paired={}
 baseline={}
 for record in records:
  minute=record['minute'];maps=defaultdict(lambda:[0,0,0])
  for panel in ['targeted','ordinary_retention','lantern_showcase']:
   rows=[json.loads(l) for l in (ROOT/f'evaluation/minute-{minute:02d}/{panel}.jsonl').read_text().splitlines()]
   gained=lost=0
   for row in rows:
    c=row['config'];result=row['result'];key=json.dumps(c,sort_keys=True)
    if minute==0:baseline[key]=result['success']
    else:gained+=result['success'] and not baseline[key];lost+=baseline[key] and not result['success']
    if panel=='targeted':
     name=NAMES[c['campaign']];maps[name][0]+=result['success'];maps[name][1]+=1;maps[name][2]+=result['truncated']
     enemy='Full' if c.get('enemyBlocks',{}).get('planner')=='full' else c['enemy'];scope={'authored':'clear','player-fog':'player-only fog','both-fog':'both-sides fog'}[c['objective']];k=(name,scope,enemy);v=conditions[k].setdefault(minute,[0,0]);v[0]+=result['success'];v[1]+=1
    if panel=='lantern_showcase':
     for mission in result['missions']:
      v=chapters[mission['id']].setdefault(minute,[0,0]);v[0]+=mission['winner'] in ['player','playerVictory'];v[1]+=1
   if minute:paired.setdefault(str(minute),{})[panel]=dict(gained=gained,lost=lost)
  for name,values in maps.items():by_map[name][minute]=values
 plot=ROOT/'plots';plot.mkdir(exist_ok=True)
 fig,axes=plt.subplots(1,3,figsize=(13,3.8))
 for ax,panel,title in zip(axes,['targeted','ordinary_retention','lantern_showcase'],['Eligible targets · equal map weights','Ordinary retention · 28 cases','Lantern Road · 24 campaigns']):
  ax.plot(minutes,[r['panels'][panel]['score']*100 for r in records],'o-',linewidth=2);ax.set(title=title,xlabel='Active imitation minutes',ylabel='Completion / win (%)',ylim=(0,100),xticks=[0,1,3,5]);ax.grid(alpha=.25)
 fig.tight_layout();fig.savefig(plot/'gameplay.png',dpi=160);plt.close(fig)
 fig,axes=plt.subplots(3,3,figsize=(12,9))
 for ax,(name,values) in zip(axes.flat,sorted(by_map.items())):
  ax.plot(minutes,[100*values[m][0]/values[m][1] for m in minutes],'o-');ax.set(title=name,xlabel='Active minutes',ylabel='Wins (%)',ylim=(0,100),xticks=[0,1,3,5]);ax.grid(alpha=.25)
 fig.suptitle('Eligible targeted maps · fixed matched seeds · pilot, not confirmation');fig.tight_layout();fig.savefig(plot/'per-map.png',dpi=150);plt.close(fig)
 metrics=[json.loads(l) for l in (ROOT/'posttune/metrics.jsonl').read_text().splitlines()];v=[x for x in metrics if x['event']=='progress'];valid=[x for x in metrics if x['event']=='validation']
 fig,axes=plt.subplots(1,3,figsize=(12,3.5))
 axes[0].plot([x['seconds']/60 for x in v],[x['loss'] for x in v]);axes[0].set(title='Training window loss',ylabel='Weighted cross entropy')
 axes[1].plot([x['seconds']/60 for x in v],[x['gradient_norm'] for x in v]);axes[1].set(title='Gradient norm before clipping')
 if valid:axes[2].plot([x['seconds']/60 for x in valid],[100*x['accuracy'] for x in valid],'o-')
 axes[2].set(title='Imitation validation accuracy',ylabel='Teacher agreement (%)')
 for ax in axes:ax.set_xlabel('Active imitation minutes');ax.grid(alpha=.25)
 fig.tight_layout();fig.savefig(plot/'training.png',dpi=160);plt.close(fig)
 summary=dict(records=records,parameter_drift=drift,per_map={k:{str(m):dict(wins=v[0],games=v[1],caps=v[2]) for m,v in vals.items()} for k,vals in by_map.items()},conditions=[dict(map=k[0],visibility=k[1],enemy=k[2],minutes={str(m):dict(wins=v[0],games=v[1]) for m,v in vals.items()}) for k,vals in conditions.items()],lantern_chapters={k:{str(m):dict(clears=v[0],entrants=v[1]) for m,v in vals.items()} for k,vals in chapters.items()},paired_changes=paired)
 (ROOT/'analysis.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(dict(records=records,paired=paired),indent=2))
if __name__=='__main__':main()
