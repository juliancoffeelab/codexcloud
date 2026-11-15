#!/usr/bin/env python3
"""Emit a compile-checked bridge between the two independent frozen native models.

Known fields are visited explicitly, in both directions. Compiler warnings for
partial records and variants are errors, including when a new field is optional.
This bridge contains no JSON encode/decode and is not imported by application runtime.
"""
from pathlib import Path
import json,re,hashlib
root=Path(__file__).resolve().parents[1]
from historical_shapes import shapes,entries,name,part,union_tags as tags,field,snake,ctor
schema=json.loads((root/'packages/save-migrations/ocaml/native-v48.json').read_text())
def attr(text,key):
 m=re.search(r'(?:^|[ (,])'+key+r'\s*=\s*"([^"]+)"',text)
 return m.group(1) if m else None
def wire_name(f,parent):
 n=attr(f['serde'],'rename')
 if n:return n
 n=f['name'];rule=attr(parent,'rename_all')
 if rule=='camelCase':return re.sub(r'_([a-z])',lambda m:m[1].upper(),n)
 if rule=='lowercase':return n.lower()
 return n
def nfield(s):
 return 'unit' if s=='unit' else field(s)
def native_name(e):return 'N.'+(snake(e['name']).capitalize()+'.t' if e['kind']=='enum' else field(e['name']))
def native_ctor(e,v):return 'N.'+snake(e['name']).capitalize()+'.'+snake(v['name']).capitalize()
def members(i):return shapes[i]['members'] if shapes[i]['kind']=='union' else [i]
def effective(p):
 ms=[j for j in members(p['type']) if shapes[j]['kind']!='undefined']
 return ms[0] if p['optional'] and len(ms)==1 else p['type']
def unwrap(t):
 while t['kind']=='named' and t['name'] in ['Arc','Box','Rc']:t=t['args'][0]
 return t
def default(t):
 t=unwrap(t);n=t.get('name')
 if n=='Vec' or t['kind']=='array':return '[||]'
 if n=='Option':return 'None'
 if n=='BTreeMap':return 'N.String_map.empty'
 if n=='HashMap':return 'Hashtbl.create 0'
 if n=='String':return '""'
 if n=='bool':return 'false'
 if n in ['i32','isize']:return '0l'
 if n in ['u8','u16','i8','i16']:return '0'
 return '0.'
queue=[];seen={}
def named(n,i,d):
 key=(n,part[i],d)
 if key not in seen:
  seen[key]=('from_' if d else 'to_')+snake(n)+'_'+str(part[i]);queue.append((n,i,d,seen[key]))
 return seen[key]
def convert_option(inner,i,x,d):
 ss=shapes[i]
 if ss['kind']=='union':
  cases=[]
  for j,tag,payload in tags(i):
   k=shapes[j]['kind']
   if k in ['undefined','null']:
    if d:cases.append('`'+tag+' -> None')
   elif d:cases.append('`'+tag+(' v' if payload else '')+' -> Some ('+convert(inner,j,'v' if payload else '`'+tag,d)+')')
  if d:return '(match '+x+' with '+' | '.join(cases)+')'
  live=[(j,tag,p) for j,tag,p in tags(i) if shapes[j]['kind'] not in ['undefined','null']]
  if len(live)==1:
   j,tag,payload=live[0];present='`'+tag+(' ('+convert(inner,j,'v',False)+')' if payload else '')
  else:present=convert(inner,i,'v',False)
  absent=next(('`'+tag for j,tag,_ in tags(i) if shapes[j]['kind']=='null'),None)
  absent=absent or next(('`'+tag for j,tag,_ in tags(i) if shapes[j]['kind']=='undefined'),None)
  if absent is None:raise ValueError(('Option has no absence',i))
  return '(match '+x+' with None -> '+absent+' | Some v -> '+present+')'
 if ss['kind'] in ['null','undefined']:return 'None' if d else '`'+ss['kind'].capitalize()
 return '(Some ('+convert(inner,i,x,d)+'))' if d else '(match '+x+' with Some v -> '+convert(inner,i,'v',d)+' | None -> invalid_arg "Missing native optional value")'
def convert(t,i,x,d):
 t=unwrap(t);s=shapes[i];n=t.get('name')
 if t['kind']=='tuple':
  assert s['kind']=='tuple',(t,i,s)
  names=['v'+str(k) for k in range(len(t['args']))]
  return '(let ('+', '.join(names)+') = '+x+' in ('+', '.join(convert(a,j,v,d) for a,j,v in zip(t['args'],s['items'],names))+'))'
 if n=='Option':return convert_option(t['args'][0],i,x,d)
 if t['kind']=='array' and s['kind']=='tuple':
  inner=t['inner'];vs=['v'+str(k) for k in range(len(s['items']))]
  if d:return '(let ('+', '.join(vs)+') = '+x+' in [|'+'; '.join(convert(inner,j,v,True) for j,v in zip(s['items'],vs))+'|])'
  return '('+', '.join(convert(inner,j,x+'.('+str(k)+')',False) for k,j in enumerate(s['items']))+')'
 if t['kind']=='array' or n=='Vec':
  assert s['kind']=='array',(t,i,s)
  inner=t['inner'] if t['kind']=='array' else t['args'][0]
  return '(Array.map (fun v -> '+convert(inner,s['element'],'v',d)+') '+x+')'
 if n=='bool' and s['kind']=='literal':
  tag=ctor(s['value']);v='true' if s['value'] else 'false'
  return '(let (`'+tag+') = '+x+' in '+v+')' if d else '(if '+x+' = '+v+' then `'+tag+' else invalid_arg "Native boolean literal")'
 if n=='bool' and s['kind']=='union':
  if d:return '(match '+x+' with `True -> true | `False -> false | `Undefined -> invalid_arg "Absent native boolean")'
  return '(if '+x+' then `True else `False)'
 if n in ['String','str','bool','u32','usize','f32','f64']:return x
 if n in ['i32','isize']:return ('Int32.of_float' if d else 'Int32.to_float')+' ('+x+')'
 if n in ['u8','u16','i8','i16']:return ('int_of_float' if d else 'float_of_int')+' ('+x+')'
 if n in ['HashMap','BTreeMap']:
  assert s['kind']=='object' and not s.get('properties') and 'index'in s,(t,i,s)
  body=convert(t['args'][1],s['index'],'v',d)
  pairs='List.map (fun (k,v) -> (k,'+body+')) '
  if d:
   pairs+='(H.String_map.bindings '+x+')'
   return '(Hashtbl.of_seq (List.to_seq ('+pairs+')))' if n=='HashMap' else '(List.fold_left (fun m (k,v) -> N.String_map.add k v m) N.String_map.empty ('+pairs+'))'
  pairs+=('(List.of_seq (Hashtbl.to_seq '+x+'))' if n=='HashMap' else '(N.String_map.bindings '+x+')')
  return '(H.String_map.of_list ('+pairs+'))'
 if n in schema:
  e=schema[n]
  if e['kind']=='enum' and s['kind']=='literal' and d:
   v=next(v for v in e['variants'] if (v.get('serde_name') or wire_name({'name':v['name'],'serde':v['serde']},e['serde']))==s['value'])
   return '(let (`'+ctor(s['value'])+') = '+x+' in '+native_ctor(e,v)+')'
  return named(n,i,d)+' ('+x+')'
 raise ValueError(('unsupported native type',t,i))
def convert_field(f,p,x,d):
 t=unwrap(f['ty']);i=effective(p)
 if p['optional']:
  if t.get('name')=='Option':
   if d:return '(Option.bind '+x+' (fun v -> '+convert_option(t['args'][0],i,'v',True)+'))'
   return '(Option.map (fun v -> '+present(t['args'][0],i,'v')+') '+x+')'
  return '(Option.fold ~none:('+default(t)+') ~some:(fun v -> '+convert(t,i,'v',True)+') '+x+')' if d else '(Some ('+convert(t,i,x,False)+'))'
 return convert(t,i,x,d)
def present(t,i,x):
 if shapes[i]['kind']=='union':
  live=[(j,tag,p) for j,tag,p in tags(i) if shapes[j]['kind'] not in ['undefined','null']]
  if len(live)==1:
   j,tag,p=live[0];return '`'+tag+(' ('+convert(t,j,x,False)+')' if p else '')
 return convert(t,i,x,False)
def flattened_record(e,b,i,d,parent):
 # Keep every native record pattern exhaustive, including the nested record
 # flattened on the wire. A missing optional field must still fail compilation.
 props=shapes[i].get('properties',{}); assigned={}
 def pattern(node,body,prefix,attrs):
  patterns=[]
  for f in body['fields']:
   nf=nfield(f['name']); key=wire_name(f,attrs)
   if re.search(r'\bskip\b',f['serde']):patterns.append(nf+' = _');continue
   if 'flatten' in f['serde']:
    t=unwrap(f['ty']);child=schema[t['name']]
    assert child['kind']=='struct' and child['body']['style']=='named',('unsupported flattened native type',t)
    nested=pattern(child,child['body'],prefix+nf+'_',child['serde'])
    patterns.append(nf+' = ({ N.'+'; '.join(nested)+' } : '+native_name(child)+')')
   else:
    assert key not in assigned,('overlapping flattened field',key)
    var='flat_'+prefix+nf;assigned[key]=(f,var)
    patterns.append(nf+' = '+var)
  return patterns
 np=pattern(e,b,'',parent or e['serde'])
 assert set(props)==set(assigned),('flattened field mismatch',e['name'],set(props)^set(assigned))
 def construct(node,body,attrs):
  result=[]
  for f in body['fields']:
   nf=nfield(f['name']);key=wire_name(f,attrs)
   if re.search(r'\bskip\b',f['serde']):value=default(f['ty'])
   elif 'flatten' in f['serde']:
    child=schema[unwrap(f['ty'])['name']]
    value='({ N.'+'; '.join(construct(child,child['body'],child['serde']))+' } : '+native_name(child)+')'
   else:value=convert_field(f,props[key],field(key),True)
   result.append(nf+' = '+value)
  return result
 hp=['wire = _']+list(field(k) for k in props)
 hs=['wire = H.empty_wire']+[field(k)+' = '+convert_field(f,props[k],var,False) for k,(f,var) in assigned.items()]
 return hp,np,hs,construct(e,b,parent or e['serde'])

def compatible_untagged(t,i):
 t=unwrap(t);n=t.get('name');shape=shapes[i];kind=shape['kind']
 if n in ['u8','u16','u32','u64','usize','i8','i16','i32','i64','isize','f32','f64']:return kind=='number'
 if n in ['String','str']:return kind=='string'
 if n=='bool':return kind=='boolean'
 if n in schema and schema[n]['kind']=='enum':
  e=schema[n]
  if all(v['body']['style']=='unit' for v in e['variants']):
   values={v['serde_name'] for v in e['variants']}
   return kind=='union' and {shapes[j].get('value') for j in shape['members']}==values
 raise ValueError(('unsupported untagged native variant',t,i))

def untagged_body(e,i,d):
 members=tags(i);cases=[];matched=set()
 for variant in e['variants']:
  assert variant['body']['style']=='tuple' and len(variant['body']['fields'])==1,('unsupported untagged variant',e['name'])
  t=variant['body']['fields'][0]['ty']
  choices=[(j,tag,payload) for j,tag,payload in members if compatible_untagged(t,j)]
  assert len(choices)==1,('ambiguous untagged contract',e['name'],choices)
  j,tag,payload=choices[0];assert j not in matched;matched.add(j)
  nc=native_ctor(e,variant)
  if d:cases.append('  | `'+tag+' v -> '+nc+' ('+convert(t,j,'v',True)+')')
  else:cases.append('  | '+nc+' v -> `'+tag+' ('+convert(t,j,'v',False)+')')
 assert len(matched)==len(members),('unmatched untagged contract',e['name'])
 return 'match value with\n'+'\n'.join(cases)

def record_body(e,b,i,d,parent=None,disc=None):
 if any('flatten' in f['serde'] for f in b['fields']):
  assert disc is None,'flattened tagged variants need an explicit contract implementation'
  return flattened_record(e,b,i,d,parent)
 s=shapes[i];assert s['kind']=='object',(e['name'],i,s)
 props=s.get('properties',{});fs=[f for f in b['fields'] if not re.search(r'\bskip\b',f['serde'])]
 assigned={wire_name(f,parent or e['serde']):f for f in fs}
 expected=set(assigned)|({disc[0]} if disc else set())
 assert set(props)==expected,('field mismatch',e['name'],i,set(props)-expected,expected-set(props))
 hp=['wire = _']+list(field(k) for k in props)
 np=[nfield(f['name'])+(' = _' if re.search(r'\bskip\b',f['serde']) else '') for f in b['fields']]
 hs=['wire = H.empty_wire'];ns=[nfield(f['name'])+' = '+default(f['ty']) for f in b['fields'] if re.search(r'\bskip\b',f['serde'])]
 for k,p in props.items():
  if disc and k==disc[0]:
   hs.append(field(k)+' = `'+ctor(disc[1]));continue
  f=assigned[k];hf=field(k);nf=nfield(f['name'])
  if d:ns.append(nf+' = '+convert_field(f,p,hf,True))
  else:hs.append(hf+' = '+convert_field(f,p,nf,False))
 return hp,np,hs,ns

def body(n,i,d):
 e=schema[n]
 if e['kind']=='alias':return convert(e['ty'],i,'value',d)
 if e['kind']=='enum' and 'untagged' in e['serde']:return untagged_body(e,i,d)
 if e.get('transparent'):return convert(e['body']['fields'][0]['ty'],i,'value',d)
 if e['kind']=='struct':
  hp,np,hs,ns=record_body(e,e['body'],i,d)
  return 'let ({ '+('H.'+'; '.join(hp) if d else 'N.'+'; '.join(np))+' } : '+('H.'+name(i) if d else native_name(e))+') = value in\n  { '+'; '.join(ns if d else hs)+' }'
 # Variant matches inspect both independent declarations exhaustively.
 hi=tags(i) if shapes[i]['kind']=='union' else [(i,ctor(shapes[i].get('value','Single')),False)]
 cases=[];tagfield=attr(e['serde'],'tag')
 for v in e['variants']:
  vn=v.get('serde_name') or wire_name({'name':v['name'],'serde':v['serde']},e['serde'])
  chosen=[]
  for j,tag,payload in hi:
   sh=shapes[j]
   if tagfield:
    pp=sh.get('properties',{}).get(tagfield);ok=pp is not None and shapes[pp['type']].get('value')==vn
   elif v['body']['style']=='unit':ok=sh.get('value')==vn
   else:ok=vn in sh.get('properties',{})
   if ok:chosen.append((j,tag,payload))
  if d and not chosen:continue
  assert len(chosen)==1,('variant mismatch',n,vn,i,chosen)
  j,tag,payload=chosen[0];nc=native_ctor(e,v);style=v['body']['style']
  hpat='`'+tag if shapes[i]['kind']=='union' else ''
  if tagfield:
   hp,np,hs,ns=record_body(e,v['body'],j,d,e['serde'],(tagfield,vn))
   if d:
    hpat+=' ({ H.'+'; '.join(hp)+' } : H.'+name(j)+')'
    out=nc+(' { '+'; '.join(ns)+' }' if style!='unit' else '')
    out='let (`'+ctor(vn)+') = '+field(tagfield)+' in '+out
   else:
    nc+=(' { '+'; '.join(np)+' }' if style!='unit' else '')
    out='`'+tag+' { '+'; '.join(hs)+' }'
  elif style=='unit':out=nc if d else '`'+tag
  else:
   p=shapes[j]['properties'][vn];assert len(v['body']['fields'])==1
   f=v['body']['fields'][0];hf=field(vn)
   if d:hpat+=' ({ H.wire = _; '+hf+' } : H.'+name(j)+')';out=nc+' ('+convert_field(f,p,hf,True)+')'
   else:nc+=' v';out='`'+tag+' { wire = H.empty_wire; '+hf+' = '+convert_field(f,p,'v',False)+' }'
  cases.append('  | '+(hpat if d else nc)+' -> '+out)
 absent=[(j,tag) for j,tag,_ in hi if shapes[j]['kind'] in ['null','undefined']]
 if d:
  cases += ['  | `'+tag+' -> invalid_arg "Absent native enum"' for j,tag in absent]
 assert len(cases)==len(hi)-(0 if d else len(absent)),('unmatched historical variant',n,i,len(cases),len(hi))
 return 'match value with\n'+'\n'.join(cases)

checkpoint=shapes[entries['Save49']]['members'][0]
checkpoint=shapes[checkpoint]['properties']['checkpoint']['type']
for d in [True,False]:named('SavedGame',checkpoint,d)
functions=[];cursor=0
while cursor<len(queue):
 n,i,d,fn=queue[cursor];cursor+=1
 b=body(n,i,d);src='H.'+name(i) if d else native_name(schema[n]);dst=native_name(schema[n]) if d else 'H.'+name(i)
 functions.append((fn,src,dst,b))
contract_inputs = [
 'packages/save-migrations/ocaml/native-v48.json',
 'packages/save-migrations/src/formats/readers.json',
 'scripts/generate-migration-native-contract.py',
 'scripts/historical_shapes.py',
]
digest = hashlib.sha256()
for relative in sorted(contract_inputs):
 digest.update(relative.encode() + b'\0')
 digest.update((root/relative).read_bytes())
 digest.update(b'\0')
out=['(* Generated by scripts/generate-migration-native-contract.py. *)', '(* Sources SHA256: '+digest.hexdigest()+' *)','[@@@warning "+8+9-30-39"]','[@@@warnerror "+8+9"]','module H = Historical_types','module Make (N : module type of Native_v48) = struct','']
for k,(fn,src,dst,b) in enumerate(functions):out.append(('let rec 'if k==0 else'and ')+fn+' (value : '+src+') : '+dst+' =\n  '+b)
out += ['','let to_native = '+seen[('SavedGame',part[checkpoint],True)],'let of_native = '+seen[('SavedGame',part[checkpoint],False)],'end','']
target=root/'frontend/persistence/Migration_native_contract.ml';target.write_text('\n\n'.join(out))
print('Generated',len(functions),'typed native contract functions')
