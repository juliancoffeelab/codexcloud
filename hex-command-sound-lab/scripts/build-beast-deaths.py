#!/usr/bin/env python3
"""Bake selection-derived beast deaths: one shared fade/reflection treatment.

Uses the actual decoded listening take, including existing authored processing.
No voice pitch, saturation or modulation is added. Requires Python/Node/FFmpeg.
"""
import hashlib,json,re,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'dist'
catalog=json.loads((OUT/'catalog.json').read_text());by_id={x['id']:x for x in catalog}
active=json.loads(subprocess.check_output(['node',str(ROOT/'scripts/beast-death-selections.mjs')]))
prior_path=OUT/'beast-deaths.json'
prior=json.loads(prior_path.read_text()).get('edits',[]) if prior_path.exists() else []
plans={e['id']:{k:e[k] for k in ['id','sourceId','take','units']} for e in prior if e['sourceId'] in by_id}
plans.update({e['id']:e for e in active});active_ids={e['id'] for e in active};edits=[]
for plan in plans.values():
 source=by_id[plan['sourceId']];take=source['takes'][plan['take']];duration=take['duration']
 filters=[f"atrim=start={take['start']}:duration={duration}",'asetpts=PTS-STARTPTS','volume=-12dB',
          f'afade=t=out:st={duration*.25:.6f}:d={duration*.75:.6f}',
          'aecho=in_gain=0.95:out_gain=0.90:delays=19|37|61|89:decays=0.08|0.06|0.04|0.02']
 path=OUT/source['path']
 probe=subprocess.run(['ffmpeg','-hide_banner','-nostdin','-i',str(path),'-af',','.join(filters+['volumedetect']),'-f','null','-'],capture_output=True,text=True,check=True)
 peak=float(re.search(r'max_volume: ([-\d.]+) dB',probe.stderr)[1])
 if peak<=-100:raise ValueError('Silent beast death source: '+source['id'])
 filters.append(f'volume={-6-peak:.2f}dB')
 recipe={**plan,'sourceListeningSha256':hashlib.sha256(path.read_bytes()).hexdigest(),'sourceRange':take,
         'fadeReleaseFraction':.75,'roomReflectionsMs':[19,37,61,89],'filters':filters,'active':plan['id'] in active_ids}
 identity=json.dumps({'recipe':recipe,'format':'mp3-mono-44100-96k'},sort_keys=True)
 dest=OUT/'audio/listening'/(hashlib.sha256(identity.encode()).hexdigest()[:20]+'.mp3')
 subprocess.run(['ffmpeg','-v','error','-nostdin','-y','-i',str(path),'-map_metadata','-1','-af',','.join(filters),'-ac','1','-ar','44100','-codec:a','libmp3lame','-b:a','96k',str(dest)],check=True)
 decoded=subprocess.check_output(['ffmpeg','-v','error','-i',str(dest),'-ac','1','-ar','44100','-f','f32le','-'])
 size=len(decoded)/(4*44100)
 item={k:v for k,v in source.items() if k not in ['previousTakes']}
 item.update(id=plan['id'],name='Beast death · '+source['name']+' · fade',path=dest.relative_to(OUT).as_posix(),pack='audition-edits',packLabel='Audition edits',
             kind='processed-effect',originalKind=source['kind'],derivedFrom=source['id'],processingRecipe=recipe,preserveLevel=False,
             duration=round(size,4),takes=[{'start':0,'duration':round(size,4)}],takeMethod='selection-fade-room-v1',gainDb=0,
             notes='Current selection take with a 75% release fade and quiet early room reflections; no added voice modulation. Matched to normal reference level.')
 catalog=[entry for entry in catalog if entry["id"] != item["id"]]
 catalog.append(item);edits.append(recipe)
 print('Rendered',plan['id'],flush=True)
(OUT/'catalog.json').write_text(json.dumps(catalog,indent=2)+'\n')
prior_path.write_text(json.dumps({'method':__doc__.strip(),'edits':edits},indent=2)+'\n')
