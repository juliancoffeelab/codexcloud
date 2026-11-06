#!/usr/bin/env python3
"""Conservative edge-silence audit for every playable take; originals stay intact.

5ms RMS frames; gate 45dB below that take's strongest frame (floor -80dBFS).
Retain 5ms before activity and 12ms after. Only remove >=20ms of edge quiet.
Internal quiet is reported, never spliced out. Requires Python 3, Node and FFmpeg.
"""
import array,hashlib,json,math,subprocess,sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'dist';RATE=44100;HOP=220
apply='--apply' in sys.argv
catalog=json.loads((OUT/'catalog.json').read_text())
overrides={(e['id'],e['take']):e for e in json.loads((ROOT/'timing-overrides.json').read_text())['overrides']}
subprocess.run(['node',str(ROOT/'scripts/palette-refs.mjs')],check=True)
palette=json.loads((OUT/'palette-refs.json').read_text())['refs']
palette_by_id={}
for ref in palette:palette_by_id.setdefault(ref['id'],[]).append(ref)
previous_path=OUT/'timing-audit.json'
previous={x['id']:x for x in json.loads(previous_path.read_text()).get('entries',[])} if previous_path.exists() else {}
def analyze(item):
 blob=(OUT/item['path']).read_bytes();digest=hashlib.sha256(blob).hexdigest()
 a=array.array('f');a.frombytes(subprocess.check_output(['ffmpeg','-v','error','-nostdin','-i',str(OUT/item['path']),'-ac','1','-ar',str(RATE),'-f','f32le','-']))
 if sys.byteorder!='little':a.byteswap()
 ranges=item.get('takes') or [{'start':0,'duration':len(a)/RATE}]
 old=previous.get(item['id'])
 if old and old['listeningSha256']==digest and ranges==old['appliedRanges']:
  ranges=old['originalRanges']
 takes=[];applied=[]
 for index,take in enumerate(ranges):
  start=round(take['start']*RATE);end=min(len(a),round((take['start']+take['duration'])*RATE));chunk=a[start:end]
  energy=[sum(v*v for v in chunk[n:n+HOP])/len(chunk[n:n+HOP]) for n in range(0,len(chunk),HOP)]
  peak=max(energy,default=0);threshold=max(1e-8,peak*10**(-45/10));active=[i for i,e in enumerate(energy) if e>=threshold]
  if not active:
   takes.append({'take':index+1,'silent':True});applied.append(take);continue
  first=active[0]*HOP;last=min(len(chunk),(active[-1]+1)*HOP)
  lead=max(0,first/RATE-.005);tail=max(0,(len(chunk)-last)/RATE-.012)
  trim_start=lead if lead>=.020 else 0;trim_end=tail if tail>=.020 else 0
  # Keep valid source-time bounds, never lengthen or splice a take.
  new={'start':round(take['start']+trim_start,4),'duration':round((end-start)/RATE-trim_start-trim_end,4)}
  override=overrides.get((item['id'],index))
  if override:
   if item.get('sourceHash')!=override['sourceHash']:raise ValueError('Timing override source changed')
   new={k:override[k] for k in ['start','duration']}
   if new['start']<take['start'] or new['start']+new['duration']>take['start']+take['duration']+.0001:raise ValueError('Override exceeds source range')
   trim_start=new['start']-take['start'];trim_end=take['start']+take['duration']-new['start']-new['duration']
  if new['duration']<=0:raise ValueError('Empty timing cut: '+item['id'])
  quiet=[];cursor=None
  for n in range(active[0],active[-1]+1):
   if energy[n]<threshold and cursor is None:cursor=n
   if (energy[n]>=threshold or n==active[-1]) and cursor is not None:
    duration=(n-cursor)*HOP/RATE
    if duration>=.1:quiet.append({'start':round(take['start']+cursor*HOP/RATE,4),'duration':round(duration,4)})
    cursor=None
  applied.append(new)
  takes.append({'take':index+1,'leadingQuietMs':round(first/RATE*1000,1),'trailingQuietMs':round((len(chunk)-last)/RATE*1000,1),'trimmedLeadingMs':round(trim_start*1000,1),'trimmedTrailingMs':round(trim_end*1000,1),'gateDbFS':round(10*math.log10(threshold),1),'internalQuiet':quiet,'remainingInternalQuiet':[{'start':round(max(q['start'],new['start']),4),'duration':round(min(q['start']+q['duration'],new['start']+new['duration'])-max(q['start'],new['start']),4)} for q in quiet if min(q['start']+q['duration'],new['start']+new['duration'])-max(q['start'],new['start'])>=.1],'reviewedOverride':override.get('reason') if override else None})
 return {'id':item['id'],'name':item['name'],'listeningSha256':digest,'originalRanges':ranges,'appliedRanges':applied,'takes':takes,'paletteReferences':palette_by_id.get(item['id'],[])}
with ThreadPoolExecutor(max_workers=4) as pool:entries=list(pool.map(analyze,catalog))
changes=sum(any(t.get('trimmedLeadingMs') or t.get('trimmedTrailingMs') for t in e['takes']) for e in entries)
changed_takes=sum(bool(t.get('trimmedLeadingMs') or t.get('trimmedTrailingMs')) for e in entries for t in e['takes'])
if apply:
 for item,e in zip(catalog,entries):item['takes']=e['appliedRanges']
 (OUT/'catalog.json').write_text(json.dumps(catalog,indent=2)+'\n')
palette_issues=[{'id':e['id'],'take':ref['take']+1,'quiet':e['takes'][ref['take']].get('remainingInternalQuiet',[])} for e in entries for ref in e['paletteReferences'] if e['takes'][ref['take']].get('remainingInternalQuiet') or e['takes'][ref['take']].get('silent')]
report={'paletteReferences':len(palette),'paletteInternalQuietIssues':palette_issues,'method':__doc__.strip(),'applied':apply,'recordings':len(entries),'takes':sum(len(e['takes']) for e in entries),'changedRecordings':changes,'changedTakes':changed_takes,'catalogSha256':hashlib.sha256((OUT/'catalog.json').read_bytes()).hexdigest(),'entries':entries}
(OUT/('timing-audit.json' if apply else 'timing-audit-preview.json')).write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['entries','method']},indent=2))
