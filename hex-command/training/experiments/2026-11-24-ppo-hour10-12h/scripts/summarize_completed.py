from pathlib import Path
from collections import defaultdict
import json
ROOT=Path(__file__).resolve().parent
records=json.loads((ROOT/'evaluation/hourly-summary.json').read_text());selection=json.loads((ROOT/'evaluation/selection.json').read_text())
spec=json.loads((ROOT/'catalog/simulation/rl.json').read_text())
lookup={a['id']:(a['samplingGroup'],a['samplingMap']) for a in spec['arenas']}
lookup.update({cid:(vs[0]['samplingGroup'],vs[0]['samplingMap']) for cid,vs in spec['campaigns'].items()})
out=[]
for record in records:
 panels={}
 for panel in ['heldout','training_diagnostic','lantern_showcase']:
  rows=json.loads((ROOT/f"evaluation/{record['label']}/monitor/{panel}.json").read_text())['rows'];groups=defaultdict(lambda:dict(wins=0,runs=0,caps=0));chapters=defaultdict(lambda:dict(clears=0,entrants=0,caps=0,defeats=0))
  for row in rows:
   c=row['config'];result=row['result'];cid=c.get('campaign',c.get('landscape'));family,logical=lookup.get(cid,('campaign' if 'campaign' in c else 'arena',cid))
   key='|'.join([family,logical or 'unknown',c.get('objective','unknown')]);g=groups[key];g['wins']+=result['success'];g['runs']+=1;g['caps']+=result['truncated']
   for m in result.get('missions',[]):
    k='|'.join([cid,c.get('objective','unknown'),m['id']]);v=chapters[k];v['entrants']+=1;v['clears']+=m['winner'] in ['player','playerVictory'];v['caps']+=m['winner'] in ['cap','truncated'];v['defeats']+=m['winner'] in ['enemy','playerDefeat']
  panels[panel]=dict(groups=dict(groups),chapters=dict(chapters),summary=record['panels'][panel])
 out.append(dict(hour=record['hour'],actual_seconds=record['actual_seconds'],strata=record['strata'],panels=panels))
result=dict(selected_hour=selection['selected']['hour'],criterion=selection['criterion'],hours=out)
(ROOT/'evaluation/comparison-and-chapters.json').write_text(json.dumps(result,indent=2)+'\n')
print('selected',result['selected_hour'])
for h in [0,result['selected_hour'],12]:
 x=out[h];print('hour',h,x['actual_seconds']);print('panels',{p:v['summary'] for p,v in x['panels'].items()});print('strata',x['strata']);print('diagnostics',x['panels']['training_diagnostic']['groups'])
