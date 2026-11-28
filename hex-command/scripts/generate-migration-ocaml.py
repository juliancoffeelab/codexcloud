#!/usr/bin/env python3
"""Regenerate frozen OCaml types/codecs without reading live application definitions."""
from pathlib import Path
import json, re, subprocess, tempfile, hashlib, shutil, os
r=Path(__file__).resolve().parents[1];base=r/'packages/save-migrations/ocaml'
os.chdir(r)
# The SDK emits type declarations from frozen metadata. The existing Wasm supplies
# ABI scaffolding only; its live schema is replaced in a disposable copy.
def leb(n):
 out=[]
 while n>127:out.append((n&127)|128);n>>=7
 return bytes(out+[n])
def read_leb(data,i):
 n=shift=0
 while True:
  c=data[i];i+=1;n|=(c&127)<<shift
  if c<128:return n,i
  shift+=7
wasm=(r/'rust/pkg/api-input.wasm').read_bytes();scaffold=bytearray(wasm[:8]);i=8
while i<len(wasm):
 start=i;kind=wasm[i];length,j=read_leb(wasm,i+1);end=j+length
 drop=False
 if kind==0:
  length,k=read_leb(wasm,j);drop=wasm[k:k+length]==b'ocamljs.schema'
 if not drop:scaffold.extend(wasm[start:end])
 i=end
work=Path(tempfile.mkdtemp(prefix='migration-ocaml-',dir=r/'tmp'))
version = 53
schema = json.loads((base/'native-v53.json').read_text())
name = b'ocamljs.schema'
data = leb(len(name)) + name + '\n'.join(json.dumps(e) for e in schema.values()).encode()
binary = work/'v53.wasm'
binary.write_bytes(scaffold + b'\0' + leb(len(data)) + data)
subprocess.run([str(r/'rust/pkg/ocamljs_pack'), 'generate', '--wasm', str(binary), '--out-dir', str(work/'v53')], check=True)

def snake(s):
 return re.sub(r'([a-z0-9])([A-Z])', r'\1_\2', re.sub(r'([A-Z]+)([A-Z][a-z])', r'\1_\2', s)).lower()

# The independent native signature is used only for the application's complete
# compile-time contract comparison. Historical conversions use Historical_types.
impl = (work/'v53/Bindings.ml').read_text()
intf = (work/'v53/Bindings.mli').read_text()
types = impl[:impl.index('module Internal = struct')]
# Frozen save types cannot depend on the live binding runtime's text comparator.
# Polymorphic comparison of host strings uses their JavaScript code-unit order.
types = re.sub(r'let compare \(left : t\) \(right : t\) =\s*let module Compare = struct.*?end in\s*Compare.apply left right',
               'let compare (left : t) (right : t) = Stdlib.compare left right', types, flags=re.S)

enums = []
for n, entry in schema.items():
 if entry['kind'] == 'enum':
  name = snake(n).capitalize()
  match = re.search(r'^module ' + name + r' = struct\n.*?^end\n', impl, re.M | re.S)
  if match:
   enums.append(match[0])
(base/'Native_v53.ml').write_text(types + '\n' + ''.join(enums))
frozen_intf = intf[:intf.index('module Wasm : sig')]
frozen_intf = re.sub(r'^module \w+ : sig\n(?:(?!^end$).)*^end\n', lambda m: '' if re.search(r'^val ',m[0],re.M) else m[0], frozen_intf, flags=re.M|re.S)
(base/'Native_v53.mli').write_text(frozen_intf)

subprocess.run(['python3', 'scripts/generate-historical-types.py'], check=True)
subprocess.run(['python3', 'scripts/generate-migration-native-contract.py'], check=True)
formatter = 'ocamlformat'
outputs = [base/'Native_v41.ml', base/'Native_v41.mli', base/'Native_v43.ml', base/'Native_v43.mli', base/'Native_v44.ml', base/'Native_v44.mli', base/'Native_v46.ml', base/'Native_v46.mli', base/'Native_v47.ml', base/'Native_v47.mli', base/'Native_v48.ml', base/'Native_v48.mli', base/'Native_v50.ml', base/'Native_v50.mli', base/'Native_v51.ml', base/'Native_v51.mli', base/'Native_v52.ml', base/'Native_v52.mli', base/'Native_v53.ml', base/'Native_v53.mli', base/'Historical_types.ml',
           base/'Historical_codec.ml', base/'Historical_codec.mli']
for path in outputs + [r/'frontend/persistence/Migration_native_contract.ml']:
 subprocess.run([formatter, '--inplace', str(path)], check=True)
inputs = [base/'native-v38.json', base/'native-v39.json', base/'native-v40.json', base/'native-v41.json', base/'native-v43.json', base/'native-v44.json', base/'native-v46.json', base/'native-v47.json', base/'native-v48.json', base/'native-v50.json', base/'native-v51.json', base/'native-v52.json', base/'native-v53.json', base.parent/'src/formats/readers.json']
def hashes(paths):
 return {str(p.relative_to(base.parent)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
(base/'contracts.json').write_text(json.dumps({'inputs': hashes(inputs), 'outputs': hashes(outputs)}, indent=2) + '\n')
shutil.rmtree(work)
