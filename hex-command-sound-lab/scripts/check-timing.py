#!/usr/bin/env python3
"""Verify timing ranges, immutable files, safe bounds and the active palette."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parents[1];out=root/'dist'
raw=(out/'catalog.json').read_bytes();catalog=json.loads(raw);audit=json.loads((out/'timing-audit.json').read_text())
assert audit['applied'] and audit['catalogSha256']==hashlib.sha256(raw).hexdigest()
assert len(catalog)==audit['recordings']
assert not audit['paletteInternalQuietIssues'],audit['paletteInternalQuietIssues']
by_id={e['id']:e for e in audit['entries']}
for item in catalog:
 e=by_id[item['id']];assert item['takes']==e['appliedRanges']
 assert hashlib.sha256((out/item['path']).read_bytes()).hexdigest()==e['listeningSha256']
 assert len(item['takes'])==len(e['originalRanges'])
 for take,original,details in zip(item['takes'],e['originalRanges'],e['takes']):
  assert not details.get('silent'),(item['id'],details)
  assert take['duration']>0 and take['start']>=original['start']-.0002
  assert take['start']+take['duration']<=original['start']+original['duration']+.0002
  assert take['start']+take['duration']<=item['duration']+.001,(item['id'],take)
grove=next(x for x in catalog if x['id']=='edits/grove-short-magic-v2');assert grove['takes'][0]['duration']<=.31
print(f"Timing valid for {len(catalog)} recordings / {audit['takes']} takes; no silent take or internal quiet >=100ms in {audit['paletteReferences']} suggested references; files unchanged.")
