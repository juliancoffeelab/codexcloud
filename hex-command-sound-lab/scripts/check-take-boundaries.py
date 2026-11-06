#!/usr/bin/env python3
"""Validate authored source-time boundaries and retention of source audio."""
import hashlib,json,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[1]
catalog=json.loads((root/'dist/catalog.json').read_text())
by_id={x['id']:x for x in catalog}
review=json.loads((root/'take-boundaries.json').read_text())
expected={x['id'] for x in catalog if x['pack']=='still-north-media/weapon-textures' and any(s in x['name'] for s in ['Arrow','Bolt','Crossbow','Longbow','Recurve'])}
assert {x['id'] for x in review['recordings']}==expected
for entry in review['recordings']:
 item=by_id[entry['id']]
 audit_path=root/'dist/timing-audit.json'
 audit={x['id']:x for x in json.loads(audit_path.read_text())['entries']} if audit_path.exists() else {}
 if entry['id'] in audit:
  assert audit[entry['id']]['originalRanges']==entry['takes']
  assert item['takes']==audit[entry['id']]['appliedRanges']
 else:assert item['takes']==entry['takes']
 assert item['sourceHash']==entry['sourceHash']
 assert hashlib.sha256((root/'vendor/sfx'/item['sourcePath']).read_bytes()).hexdigest()==entry['sourceHash']
 end=0
 for take in entry['takes']:
  assert take['start']>=end-.0001 and take['duration']>0
  end=take['start']+take['duration']
  assert end<=item['duration']+.0001
 # Both original and browser listening copy must remain byte-for-byte unchanged.
 old=json.loads(subprocess.check_output(['git','show','6cb90a8:hex-command-sound-lab/dist/catalog.json']))
 previous=next(x for x in old if x['id']==item['id'])
 assert item['path']==previous['path'] and item['duration']==previous['duration']
 blob=subprocess.check_output(['git','show','6cb90a8:hex-command-sound-lab/dist/'+item['path']])
 assert (root/'dist'/item['path']).read_bytes()==blob
shoot=next(x for x in catalog if x['name']=='Scythian Recurve Shoot')
assert len(shoot['takes'])==8
nock=next(x for x in catalog if x['name']=='English Longbow Nock Arrow')
assert {'start':3.6439,'duration':.2744} in nock['takes']
assert len(nock['takes'])==9
print(f"Reviewed boundaries valid for {len(expected)} sessions; original audio unchanged; clean nock and eight recurve shots retained.")
