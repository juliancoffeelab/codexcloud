from pathlib import Path
import json
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.ticker import PercentFormatter
ROOT=Path(__file__).resolve().parent
records=json.loads((ROOT/'evaluation/hourly-summary.json').read_text())
folder=ROOT/'catalog/game/campaigns/lantern-road'
campaign=json.loads(next(folder.glob('*.json')).read_text())
stages=[json.loads((folder/p).read_text()) for p in campaign['stages']]
ids=[s['id'] for s in stages]
names=[json.loads((folder/path).with_name('map.json').read_text())['name'] for path in campaign['stages']]
series=[]
for record in records:
 p=ROOT/f"evaluation/{record['label']}/monitor/lantern_showcase.json"
 if not p.exists():continue
 panel=json.loads(p.read_text());rows=panel['rows'];counts={sid:[0,0] for sid in ids}
 for row in rows:
  for m in row['result'].get('missions',[]):
   if m['id'] in counts:
    counts[m['id']][1]+=1;counts[m['id']][0]+=m['winner'] in ['player','playerVictory']
 series.append(dict(hour=record['hour'],hours=record['actual_seconds']/3600,wins=panel['wins'],runs=panel['runs'],caps=panel['caps'],chapters=counts))
plt.rcParams.update({'font.size':11,'axes.spines.top':False,'axes.spines.right':False})
fig,axes=plt.subplots(2,2,figsize=(13,8.8))
ax=axes.flat[0];xs=[x['hours'] for x in series]
ax.plot(xs,[100*x['wins']/x['runs'] for x in series],'-o',color='#285fa5',lw=2)
for x in series:ax.annotate(f"{x['wins']}/{x['runs']}",(x['hours'],100*x['wins']/x['runs']),xytext=(0,9),textcoords='offset points',ha='center',fontsize=9)
ax.set(title='Full campaign completion',ylim=(0,28),ylabel='Cleared / 24 campaign starts')
for ax,sid,name,color in zip(list(axes.flat)[1:],ids,names,['#218c74','#b87617','#8753a4']):
 ys=[100*x['chapters'][sid][0]/x['chapters'][sid][1] if x['chapters'][sid][1] else float('nan') for x in series]
 ax.plot(xs,ys,'-o',color=color,lw=2)
 for x,y in zip(series,ys):
  w,n=x['chapters'][sid]
  if n:ax.annotate(f'{w}/{n}',(x['hours'],y),xytext=(0,8),textcoords='offset points',ha='center',fontsize=9)
 ax.set(title=name,ylim=(0,112),ylabel='Cleared / starts of this chapter')
for ax in axes.flat:
 ax.set(xlabel='Additional PPO training hours (0 = original hour-10 model)',xticks=list(range(12)))
 ax.yaxis.set_major_formatter(PercentFormatter());ax.grid(alpha=.2)
fig.suptitle('Lantern Road · fixed 24-case showcase · checkpoints 0–11',fontsize=17,y=.99)
fig.text(.5,.012,'Chapter rates are conditional on entering that chapter. Labels show clears / entrants; later chapters have fewer attempts.\nSame seeds and opponent panel each hour; natural campaign carryover. No separate fresh-start chapter tests.',ha='center',fontsize=10)
fig.tight_layout(rect=(0,.065,1,.96));fig.savefig(ROOT/'plots/lantern-focused-curves.png',dpi=160)
(ROOT/'evaluation/lantern-focused-curves.json').write_text(json.dumps(dict(stages=dict(zip(ids,names)),series=series),indent=2)+'\n')
print(json.dumps(dict(names=names,last=series[-1]),indent=2))
