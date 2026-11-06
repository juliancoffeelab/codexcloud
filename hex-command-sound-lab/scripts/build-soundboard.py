#!/usr/bin/env python3
"""Build browser-friendly listening copies and catalog from the locked originals.

Requires Python 3.11+, FFmpeg and FFprobe. Originals are never changed.
"""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import hashlib,json,re,subprocess,zipfile,shutil,sys
ROOT=Path(__file__).resolve().parents[1]
VENDOR=ROOT/'vendor/sfx'; OUT=ROOT/'dist'
SCHEME='mp3-mono-44100-96k-v1'
def run(args):
 p=subprocess.run(args,capture_output=True,text=True,check=True);return p.stdout+p.stderr
sources=json.loads((VENDOR/'sources.json').read_text())
packs={p['id']:p for p in sources['packs']}
validation={p['path']:p for p in json.loads((VENDOR/'validation.json').read_text())['files'] if 'duration_seconds' in p}
inventory=[p for p in json.loads((VENDOR/'inventory.json').read_text()) if p['path'] in validation]
old={p['path']:p for p in json.loads((OUT/'catalog.json').read_text()) if p['pack']=='current'}
(OUT/'audio/listening').mkdir(parents=True,exist_ok=True)
previous={p['id']:p for p in json.loads((OUT/'catalog.json').read_text())}
def build(entry):
 path=entry['path'];src=VENDOR/path;pack=packs[entry['pack']];info=validation[path]
 if hashlib.sha256(src.read_bytes()).hexdigest()!=entry['sha256']:raise ValueError(f'Original hash mismatch: {path}')
 digest=hashlib.sha256((entry['sha256']+SCHEME).encode()).hexdigest()[:20]
 dest=OUT/'audio/listening'/f'{digest}.mp3';name=src.stem
 # Retain existing Kenney favorite identifiers.
 family={'kenney/impact-sounds':'impact-sounds','kenney/rpg-audio':'rpg-audio'}.get(entry['pack'],entry['pack'])
 id=f'{family}/{name}' if family in ('impact-sounds','rpg-audio') else f'{entry["pack"]}/{path.split(entry["pack"]+"/",1)[1]}'
 prev=previous.get(id)
 if prev and prev.get('sourceHash')==entry['sha256'] and dest.exists() and prev.get('takeMethod') in ('silence42-fallback30-v2','bow-review-v1'):return prev
 duration=info['duration_seconds']
 analysis=run(['ffmpeg','-hide_banner','-nostdin','-i',str(src),'-af','volumedetect,silencedetect=noise=-42dB:d=0.25','-f','null','-'])
 peak=float(re.search(r'max_volume: ([-\d.]+) dB',analysis)[1])
 gain=min(18,max(-12,-3-peak))
 run(['ffmpeg','-hide_banner','-loglevel','error','-nostdin','-y','-i',str(src),'-map_metadata','-1','-vn','-ac','1','-ar','44100','-codec:a','libmp3lame','-b:a','96k',str(dest)])
 if entry['pack'].startswith('still-north-media/') and 'silence_start' not in analysis:
  analysis=run(['ffmpeg','-hide_banner','-nostdin','-i',str(src),'-af','silencedetect=noise=-30dB:d=0.25','-f','null','-'])
 # Auto-suggest takes only for the multi-take weapon sessions. Full recordings
 # remain playable, including all quiet tails and pauses. Boundaries are reviewable.
 intervals=[]
 if entry['pack'].startswith('still-north-media/'):
  starts=[float(v) for v in re.findall(r'silence_start: ([\d.]+)',analysis)]
  ends=[float(v) for v in re.findall(r'silence_end: ([\d.]+)',analysis)]
  cursor=0
  for a,b in zip(starts,ends):
   if a-cursor>=.025:intervals.append((max(0,cursor-.018),min(duration,a+.10)))
   cursor=b
  if duration-cursor>=.025:intervals.append((max(0,cursor-.018),duration))
  if not intervals:intervals=[(0,duration)]
 else:intervals=[(0,duration)]
 # Nearby contacts belong to a single take; preserve room for decay.
 merged=[]
 for start,end in intervals:
  if merged and start-merged[-1][1]<.20:merged[-1]=(merged[-1][0],end)
  else:merged.append((start,end))
 text=(name+' '+entry['pack']).lower()
 categories=set()
 if any(w in text for w in ['footstep','trot','cloth','leather']):categories.add('movement')
 if any(w in text for w in ['swing','shoot','spell','magic','blade','chop','knife']):categories.add('attack')
 if any(w in text for w in ['impact','blade on','haft on','mace','hurt','hit','stone','wood']):categories.add('impact')
 if any(w in text for w in ['creature','ghost','dog-','horse-vocals']):categories.add('voice')
 if any(w in text for w in ['spell','magic','fire']):categories.add('magic')
 if any(w in text for w in ['draw','sheath','nock','fletch','book','coin','lock','belt','latch','place','trigger']):categories.add('equipment')
 return dict(takeMethod='silence42-fallback30-v2',id=id,name=name,path=dest.relative_to(OUT).as_posix(),pack=family,packLabel=entry['pack'].replace('/',' · '),license=pack['license'],source=pack['source_url'],creator=pack['creator'],kind=pack['kind'],notes=pack.get('notes',''),sourcePath=path,sourceHash=entry['sha256'],duration=round(duration,4),gainDb=round(gain,1),categories=sorted(categories or {'equipment'}),takes=[dict(start=round(a,4),duration=round(b-a,4)) for a,b in merged])
with ThreadPoolExecutor(max_workers=4) as pool:
 result=[]
 for n,item in enumerate(pool.map(build,inventory),1):
  result.append(item)
  if n%50==0:print(f'Prepared {n}/{len(inventory)}',flush=True)
for item in old.values():
 item.setdefault('packLabel','Current Hexcommand');item['categories']=['baseline'];item['kind']='current-game-synthesis';item['gainDb']=0
 item['duration']=float(run(['ffprobe','-v','error','-show_entries','format=duration','-of','csv=p=0',str(OUT/item['path'])]).strip())
 result.append(item)
# Reviewed boundaries override automatic proposals, including cached catalog rows.
review=json.loads((ROOT/'take-boundaries.json').read_text())
by_id={item['id']:item for item in result}
for entry in review['recordings']:
 item=by_id[entry['id']]
 if item['sourceHash']!=entry['sourceHash']:raise ValueError('Reviewed source changed: '+entry['id'])
 item['takes']=entry['takes'];item['takeMethod']=review['version']
 item['previousTakes']=entry['previousTakes']
shutil.copy(ROOT/'take-boundaries.json',OUT/'take-boundaries.json')
(OUT/'catalog.json').write_text(json.dumps(result,indent=2)+'\n')
subprocess.run([sys.executable,str(ROOT/'scripts/build-edits.py')],check=True)
result=json.loads((OUT/'catalog.json').read_text())
(OUT/'sources.json').write_text(json.dumps(sources,indent=2)+'\n')
(OUT/'coverage.json').write_text((VENDOR/'coverage.json').read_text())
(OUT/'roster.json').write_text((VENDOR/'roster.json').read_text())
shutil.copy(VENDOR/'CC0-1.0.txt',OUT/'licenses/CC0-1.0.txt')
# Source references remain offline-readable without publishing the huge originals.
(OUT/'licenses/packs').mkdir(exist_ok=True)
for key,p in packs.items():
 d=VENDOR/key
 for name in ['SOURCE.txt','CREDITS.txt','License.txt']:
  if (d/name).exists():shutil.copy(d/name,OUT/'licenses/packs'/f'{key.replace("/","--")}--{name}')
(OUT/'provenance.json').write_text(json.dumps({'originals':'vendor/sfx/ in the repository; all originals unchanged','processing':SCHEME,'reviewedTakeBoundaries':'take-boundaries.json; bow/crossbow/arrow sessions use authored action windows', 'takeDetection':'silencedetect -42 dB, 0.25 s; fallback -30 dB if no silence; approximate boundaries only','playbackGain':'Per-take strongest-50ms RMS matched toward the median live melee/bow output; loudness-reference.json and playback-levels.json. Sample-peak ceiling -1.5 dBFS, gains bounded -18/+24 dB; authored quiet edits retain zero gain.','sourceManifest':'sources.json','sourceHashes':'catalog.json sourceHash','auditionEdits':'audio-edits.json; derived edits keep their intended gain when level matching is enabled'},indent=2)+'\n')
if (ROOT/'curation-notes.json').exists():shutil.copy(ROOT/'curation-notes.json',OUT/'curation-notes.json')
# Measure exact listening takes, generate bounded playback gains, then report effective levels.
for args in [["check-loudness.py", "--legacy-gains"], ["build-playback-levels.py"], ["check-loudness.py"]]:
 subprocess.run([sys.executable, str(ROOT / "scripts" / args[0]), *args[1:]], check=True)
subprocess.run(['node',str(ROOT/'scripts/check-cue-durations.mjs')],check=True)
with zipfile.ZipFile(OUT/'sound-library.zip','w',zipfile.ZIP_DEFLATED) as z:
 for path in sorted({item['path'] for item in result}):z.write(OUT/path,path)
 for f in (OUT/'licenses').rglob('*'):
  if f.is_file():z.write(f,f.relative_to(OUT))
 for name in ['cue-duration-review.json','take-boundaries.json','catalog.json','sources.json','coverage.json','roster.json','provenance.json','workbench.json','audio-edits.json','curation-notes.json','playback-levels.json','loudness-reference.json','loudness.json','loudness.csv','workbench/terrain-portraits/provenance.json']:
  if (OUT/name).exists():z.write(OUT/name,name)
print(json.dumps(dict(recordings=len(result),cc0=len(inventory),takes=sum(len(x.get('takes',[])) or 1 for x in result),listeningMiB=round(sum((OUT/x['path']).stat().st_size for x in result)/1048576,2)),indent=2))
