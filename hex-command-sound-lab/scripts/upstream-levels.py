#!/usr/bin/env python3
"""Summarize digital levels across the archived upstream game's complete cues."""
import json,statistics
from pathlib import Path
root=Path(__file__).resolve().parents[1];out=root/'dist'
report=json.loads((out/'loudness.json').read_text());reference=json.loads((root/'loudness-reference.json').read_text())
result={'units':'dBFS; not acoustic SPL','average':'Arithmetic mean of per-recording dB measurements, not an energy/time-weighted mix.','playback':'Includes original game-relative gain at the 55% board reference setting, reproducing game master 0.29.','groups':{}}
for name,entries in [('allGameCues',[x for x in report['entries'] if x['pack']=='current']),('attackReference',[x for x in report['entries'] if x['id'] in reference['referenceIds']])]:
 stats={'recordings':len(entries),'metrics':{}}
 for key in ['rmsDbFS','peakDbFS','strongest50msRmsDbFS','playbackRmsDbFS','playbackPeakDbFS','playbackStrongest50msRmsDbFS']:
  values=[x['full'][key] for x in entries];stats['metrics'][key]={k:round(v,2) for k,v in [('min',min(values)),('average',statistics.mean(values)),('median',statistics.median(values)),('max',max(values))]}
 result['groups'][name]=stats
(out/'upstream-levels.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result['groups']['attackReference']['metrics']['playbackStrongest50msRmsDbFS']))
