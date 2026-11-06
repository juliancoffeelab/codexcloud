#!/usr/bin/env python3
"""Report actual contextual gains for suggested palette choices and validate bands."""
import json,statistics
from pathlib import Path
root=Path(__file__).resolve().parents[1];out=root/'dist'
levels=json.loads((out/'playback-levels.json').read_text());loudness=json.loads((out/'loudness.json').read_text());refs=json.loads((out/'palette-refs.json').read_text())['refs']
measured={e['id']:e for e in loudness['entries']};rows=[]
for ref in refs:
 groups={}
 for use in ref['uses']:
  name=use.get('levelClass') or ('terrainExceptions' if use['entity'].startswith('terrain/') else 'movement' if use['action']=='move' else 'selection')
  groups.setdefault(name,[]).append(use)
 for name,uses in groups.items():
  item=measured[ref['id']];measurement=item.get('contexts',{}).get(name,item)['takes'][ref['take']]
  value=measurement['playbackStrongest50msRmsDbFS'];band=levels['eventRanges'].get(name)
  within=not band or band['minDbFS']-.03<=value<=band['maxDbFS']+.03
  row={'id':ref['id'],'name':item['name'],'take':ref['take']+1,'context':name,'playbackStrongest50msRmsDbFS':value,'playbackPeakDbFS':measurement['playbackPeakDbFS'],'inRange':within,'uses':uses}
  rows.append(row)
classes={}
for name in sorted({r['context'] for r in rows}):
 values=[r['playbackStrongest50msRmsDbFS'] for r in rows if r['context']==name]
 classes[name]={'choices':len(values),'min':min(values),'average':round(statistics.mean(values),2),'max':max(values),'referenceRange':levels['eventRanges'].get(name)}
report={'units':'dBFS, strongest half-overlapping 50ms RMS','volumePercent':loudness['volumePercent'],'matchLevels':loudness['matchLevels'],'method':'Whole-cue gain adjustment by event class; no waveform clipping, compression or new fades. Within-range choices retain their base gains. Selection, movement and quarry/bell exceptions keep base profiles.','classes':classes,'outOfRange':[r for r in rows if not r['inRange']],'rows':rows}
(out/'palette-levels.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(classes,indent=2))
if report['outOfRange']:raise ValueError('Suggested cues outside their event range: '+str(report['outOfRange']))
