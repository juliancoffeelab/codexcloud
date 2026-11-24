"""Render retained per-map, per-chapter and actual training-allocation diagnostics."""
from pathlib import Path
from collections import defaultdict,deque
import json,math
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
ROOT=Path(__file__).resolve().parent

def read(path):
 if not path.exists():return []
 rows=[]
 for line in path.read_text().splitlines():
  try:rows.append(json.loads(line))
  except json.JSONDecodeError:break
 return rows

def title(cid):
 folder=ROOT/'catalog/game/campaigns'/cid
 for p in folder.glob('*.json'):
  j=json.loads(p.read_text())
  if isinstance(j,dict) and j.get('id')==cid and (j.get('name') or j.get('title')):return j.get('name') or j.get('title')
 return cid.removeprefix('rl-').replace('-',' ').title()

def main():
 path=ROOT/'evaluation/hourly-summary.json'
 if not path.exists():return
 records=json.loads(path.read_text());out=ROOT/'plots';out.mkdir(exist_ok=True);spec=json.loads((ROOT/'catalog/simulation/rl.json').read_text());maps=defaultdict(dict);chapters=defaultdict(dict)
 lookup={a['id']:(a['samplingGroup'],a['samplingMap']) for a in spec['arenas']}
 lookup.update({cid:(vs[0]['samplingGroup'],vs[0]['samplingMap']) for cid,vs in spec['campaigns'].items()})
 for record in records:
  hour=record['hour'];by=defaultdict(lambda:[0,0])
  for panel in ['training_diagnostic','lantern_showcase']:
   rows=read(ROOT/f"evaluation/{record['label']}/monitor/{panel}.jsonl")
   for row in rows:
    c=row['config'];cid=c.get('campaign',c.get('landscape'));family,logical=lookup.get(cid,('showcase',cid));result=row['result'];v=by[(family,logical)];v[0]+=result['success'];v[1]+=1
    if family in ['campaign','showcase']:
     for mission in result.get('missions',[]):
      key=(cid,c['objective'],mission['id']);d=chapters[key].setdefault(hour,[0,0]);d[0]+=mission['winner'] in ['player','playerVictory'];d[1]+=1
  for key,value in by.items():maps[key][hour]=value
 for family in ['arena','targeted','campaign']:
  selected=sorted((k,v) for k,v in maps.items() if k[0]==family)
  if not selected:continue
  cols=4 if family!='campaign' else 3;rows=math.ceil(len(selected)/cols);fig,axes=plt.subplots(rows,cols,figsize=(cols*4,rows*3),squeeze=False)
  for ax,(key,points) in zip(axes.flat,selected):
   xs=sorted(points);ax.plot(xs,[100*points[h][0]/points[h][1] for h in xs],'o-');ax.set(title=title(key[1]),xlabel='Trainer hours',ylabel='Wins / attempts (%)',ylim=(0,100));ax.grid(alpha=.25)
  for ax in list(axes.flat)[len(selected):]:ax.set_visible(False)
  fig.suptitle(f'Fixed gameplay diagnostics · {family} · baseline included');fig.tight_layout();fig.savefig(out/f'{family}-per-map.png',dpi=140);plt.close(fig)
 campaigns=sorted({k[:2] for k in chapters})
 if campaigns:
  fig,axes=plt.subplots(len(campaigns),1,figsize=(11,max(4,3*len(campaigns))),squeeze=False)
  for ax,key in zip(axes.flat,campaigns):
   for k,points in sorted(chapters.items()):
    if k[:2]!=key:continue
    xs=sorted(points);ax.plot(xs,[100*points[h][0]/points[h][1] for h in xs],'o-',label=k[2])
   ax.set(title=title(key[0])+' · '+key[1],xlabel='Trainer hours',ylabel='Clears / chapter entrants (%)',ylim=(0,100));ax.grid(alpha=.25);ax.legend(fontsize=7)
  fig.tight_layout();fig.savefig(out/'broad-campaign-chapters.png',dpi=140);plt.close(fig)
 metrics=read(ROOT/'rl/metrics.jsonl');fig,axes=plt.subplots(1,3,figsize=(14,4))
 for ax,family in zip(axes,['arena','targeted','campaign']):
  window=deque(maxlen=50);xs=[];ys=[]
  for row in metrics:
   if row.get('event')!='episode' or row.get('sampling_group')!=family:continue
   window.append(row['episode_return']);xs.append(row['seconds']/3600);ys.append(sum(window)/len(window))
  if xs:ax.plot(xs,ys,linewidth=1)
  ax.set(title=f'{family}: mean of last 50 completed episodes',xlabel='Trainer hours',ylabel='Training return');ax.grid(alpha=.25)
 fig.tight_layout();fig.savefig(out/'training-return-by-pool.png',dpi=150);plt.close(fig)
 exposures=read(ROOT/'allocation-history.jsonl')
 if exposures:
  first=exposures[0]['time'];fig,ax=plt.subplots(figsize=(10,4))
  for pool in ['arena','targeted','campaign']:
   valid=[r for r in exposures if sum(r['pools'].values())]
   ax.plot([(r['time']-first)/3600 for r in valid],[100*r['pools'].get(pool,0)/sum(r['pools'].values()) for r in valid],label=pool)
  ax.set(xlabel='Wall hours since first exposure snapshot',ylabel='Actual cumulative decision share (%)',ylim=(0,100));ax.grid(alpha=.25);ax.legend();fig.tight_layout();fig.savefig(out/'actual-allocation.png',dpi=150);plt.close(fig)
 detail=dict(maps=[dict(group=k[0],map=k[1],hours={str(h):dict(wins=v[0],attempts=v[1]) for h,v in values.items()}) for k,values in maps.items()],chapters=[dict(campaign=k[0],visibility=k[1],chapter=k[2],hours={str(h):dict(clears=v[0],entrants=v[1]) for h,v in values.items()}) for k,values in chapters.items()])
 (ROOT/'evaluation/per-map-and-chapter-curves.json').write_text(json.dumps(detail,indent=2)+'\n')
if __name__=='__main__':main()
