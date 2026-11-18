#!/usr/bin/env python3
"""Shared frozen wire-shape graph and OCaml naming model for migration generators.

Importing this module analyzes readers.json but never emits or rewrites files.
"""
from pathlib import Path
import json,re,collections
root=Path(__file__).resolve().parents[1]
base=root/'packages/save-migrations'
doc=json.loads((base/'src/formats/readers.json').read_text())
shapes=doc['shapes'];entries=doc['entries']
def references(s):
 k=s['kind']
 if k=='array':return [s['element']]
 if k=='tuple':return s['items']
 if k=='union':return s['members']
 if k=='object':return [v['type'] for v in s.get('properties',{}).values()]+([s['index']] if 'index'in s else [])
 return []
def signature(s,part):
 k=s['kind']
 if k=='object':return (k,tuple((n,v['optional'],part[v['type']]) for n,v in s.get('properties',{}).items()),part[s['index']] if 'index'in s else None)
 if k in ['array','tuple','union']:return (k,tuple(part[x] for x in references(s)))
 return (k,json.dumps(s.get('value'),sort_keys=True))
part=[0]*len(shapes)
while True:
 seen={};nxt=[]
 for s in shapes:
  key=signature(s,part)
  if key not in seen:seen[key]=len(seen)
  nxt.append(seen[key])
 if nxt==part:break
 part=nxt
reps={}
for i,c in enumerate(part):reps.setdefault(c,i)
def snake(s):
 s=re.sub(r'([A-Z]+)([A-Z][a-z])',r'\1_\2',s);s=re.sub(r'([a-z0-9])([A-Z])',r'\1_\2',s)
 return re.sub(r'[^a-zA-Z0-9_]', '_',s).lower()
reserved=set('effect and as assert asr begin class constraint do done downto else end exception external false for fun function functor if in include inherit initializer land lazy let lor lsl lsr lxor match method mod module mutable new nonrec object of open or private rec sig struct then to true try type val virtual when while with unit'.split())
def field(s):
 n=snake(s)
 if n in reserved:n+='_' 
 if n[:1].isdigit():n='n_'+n
 return n
def ctor(x):
 if isinstance(x,bool):return 'True' if x else 'False'
 if isinstance(x,(int,float)):return 'V'+str(x).replace('-','minus_').replace('.','_')
 n=snake(str(x))
 if not n:n='Empty'
 if n[0].isdigit():n='N_'+n
 return n[0].upper()+n[1:]
# Preserve names of frozen declarations when later versions add shorter paths
# to an already shared shape. Naming changes must not rewrite historical code.
names={part[int(i)]: n for i,n in doc.get('typeNames', {}).items()};paths=dict(names);q=collections.deque()
for key,i in entries.items():q.append((i,snake(key)))
while q:
 i,path=q.popleft();c=part[i]
 if c in names:continue
 names[c]=path;paths[c]=path
 s=shapes[i];k=s['kind']
 if k=='object':
  for n,p in s.get('properties',{}).items():q.append((p['type'],path+'_'+field(n)))
  if 'index'in s:q.append((s['index'],path+'_value'))
 elif k=='array':q.append((s['element'],path+'_item'))
 elif k=='tuple':
  for n,j in enumerate(s['items']):q.append((j,path+'_item'+str(n)))
 elif k=='union':
  for n,j in enumerate(s['members']):q.append((j,path+'_member'+str(n)))
for c in reps:names.setdefault(c,'shape_'+str(c))
# Short shared leaf names improve the public interface.
for c,i in reps.items():
 k=shapes[i]['kind']
 if k in ['number','string','boolean','unknown','null','undefined']:names[c]='json_'+k
 if k=='literal':names[c]='literal_'+snake(str(shapes[i]['value']))
# Ensure names unique.
used={}
for c,n in list(names.items()):
 if n in used and used[n]!=c:n+='_'+str(c)
 names[c]=n;used[n]=c
def name(i):return names[part[i]]
def union_members(i,strip_undefined=False):
 s=shapes[i]
 members=s['members'] if s['kind']=='union' else [i]
 if strip_undefined:members=[j for j in members if shapes[j]['kind']!='undefined']
 return members
def property_type(p):
 i=p['type'];ms=union_members(i,True) if p['optional'] else [i]
 t=name(ms[0]) if len(ms)==1 else name(i)
 return '('+t+') option' if p['optional'] else t

def union_tags(i):
 used={};out=[]
 for pos,j in enumerate(shapes[i]['members']):
  s=shapes[j];kind=s['kind'];payload=True
  if kind=='literal':tag=ctor(s['value']);payload=False
  elif kind=='null':tag='Null';payload=False
  elif kind=='undefined':tag='Undefined';payload=False
  elif kind=='object':
   discriminants=[(k,p) for k,p in s.get('properties',{}).items() if shapes[p['type']]['kind']=='literal']
   preferred=next(((k,p) for k,p in discriminants if k in ['kind','op','type']),None)
   if preferred:tag=ctor(shapes[preferred[1]['type']]['value'])
   elif 'pendingEnemy'in s.get('properties',{}):tag='Current' if 'undo'in s['properties'] and shapes[s['properties']['undo']['type']]['kind']=='array' else 'Turn'
   elif 'col'in s.get('properties',{}):tag='Hex'
   elif 'location'in s.get('properties',{}):tag='Location'
   else:tag='Object'+str(pos+1)
  else:tag={'string':'String','number':'Number','boolean':'Boolean','array':'Array','tuple':'Tuple'}.get(kind,'Member'+str(pos+1))
  if tag in used:tag+=str(pos+1)
  used[tag]=True;out.append((j,tag,payload))
 return out

