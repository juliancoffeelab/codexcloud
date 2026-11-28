#!/usr/bin/env python3
"""Emit frozen historical OCaml records, variants and JSON codecs from readers.json."""
from pathlib import Path
import json,re,collections
from historical_shapes import (
 root,base,doc,shapes,entries,part,reps,names,paths,name,field,snake,ctor,
 union_members,union_tags,property_type,
)

def type_body(i):
 s=shapes[i];k=s['kind']
 if k=='number':return 'float'
 if k=='string':return 'string'
 if k=='boolean':return 'bool'
 if k=='unknown':return 'Js.Json.t'
 if k in ['null','undefined']:return '[ `'+k.capitalize()+' ]'
 if k=='literal':return '[ `'+ctor(s['value'])+' ]'
 if k=='array':return name(s['element'])+' array'
 if k=='tuple':return ' * '.join(name(j) for j in s['items'])
 if k=='union':return '[ '+' | '.join('`'+tag+(' of '+name(j) if payload else '') for j,tag,payload in union_tags(i))+' ]'
 if k=='object':
  props=s.get('properties',{})
  if not props and 'index'in s:return '('+name(s['index'])+') String_map.t'
  if not props:return 'empty_object'
  fields=['wire : wire']+[field(n)+' : '+property_type(p) for n,p in props.items()]
  if 'index'in s:fields.append('additional : ('+name(s['index'])+') String_map.t')
  return '{ '+ '; '.join(fields)+' }'
 raise ValueError(s)
lines=['(* Generated from frozen src/formats/readers.json. Do not edit. *)','[@@@warning "-30"]',"""module String_map = struct
  type 'a t = (string * 'a) list
  let empty = []
  let is_empty value = value = []
  let mem key value = List.mem_assoc key value
  let find key value = List.assoc key value
  let find_opt key value = List.assoc_opt key value
  let bindings value = value
  let cardinal = List.length
  let singleton key value = [key, value]
  let remove key value = List.filter (fun (k,_) -> k <> key) value
  let add key item value =
    if mem key value then List.map (fun (k,v) -> if k = key then (k,item) else (k,v)) value
    else value @ [key,item]
  let update key f value = match f (find_opt key value) with None -> remove key value | Some item -> add key item value
  let iter f value = List.iter (fun (k,v) -> f k v) value
  let fold f value init = List.fold_left (fun acc (k,v) -> f k v acc) init value
  let map f value = List.map (fun (k,v) -> (k,f v)) value
  let mapi f value = List.map (fun (k,v) -> (k,f k v)) value
  let filter f value = List.filter (fun (k,v) -> f k v) value
  let filter_map f value = List.filter_map (fun (k,v) -> Option.map (fun v -> (k,v)) (f k v)) value
  let exists f value = List.exists (fun (k,v) -> f k v) value
  let for_all f value = List.for_all (fun (k,v) -> f k v) value
  let of_list value = List.fold_left (fun map (k,v) -> add k v map) empty value
  let to_list value = value
end""",'type empty_object = Empty_object','type wire = { order : string list; extra : (string * Js.Json.t) list; undefined_fields : string list }','let empty_wire = { order = []; extra = []; undefined_fields = [] }','let wire_has value key = List.mem key value.order','let wire_remove value key = { order = List.filter ((<>) key) value.order; extra = List.filter (fun (k,_) -> k <> key) value.extra; undefined_fields = List.filter ((<>) key) value.undefined_fields }','let remove_fields wire keys = List.fold_left wire_remove wire keys','let merge_wire left right = { order = left.order @ List.filter (fun k -> not (List.mem k left.order)) right.order; extra = List.filter (fun (k,_) -> not (List.mem_assoc k right.extra)) left.extra @ right.extra; undefined_fields = List.filter (fun k -> not (List.mem k right.order)) left.undefined_fields @ right.undefined_fields }']
for n,(c,i) in enumerate(sorted(reps.items())):lines.append(('type ' if n==0 else 'and ')+names[c]+' = '+type_body(i))
# Aliases per named version/root; recursive graph stays frozen and shared only when identical.
def module_name(key):return re.sub(r'^Save(\d+)$',r'V\1',key)
def child(i,key):
 s=shapes[i]
 if s['kind']=='object':return s.get('properties',{}).get(key,{}).get('type')
 return None
def optional_inner(i):
 s=shapes[i]
 if s['kind']=='union':
  ms=[j for j in s['members'] if shapes[j]['kind'] not in ['null','undefined']]
  if len(ms)==1:return ms[0]
 return i
def item(i):return shapes[i].get('element')
def discover_aliases(start):
 out={};seen=set();queue=collections.deque([(start,'t')])
 preferred={'checkpoint':'checkpoint','state':'state','units':'unit_','tiles':'tile','campaign':'campaign','scripts':'scripts','factions':'faction','condition':'condition','when':'rule','light':'light','veterans':'veteran','storyVeterans':'story_veteran','marks':'mark','pending':'pending','evacuated':'evacuated','entry':'entry','setup':'setup','enemy':'enemy','undo':'undo','progress':'progress','recruitment':'recruitment','flags':'flags','holds':'hold','connections':'connection','equipment':'equipment','side':'side','turn':'side','terrain':'terrain','role':'role','gold':'gold'}
 while queue:
  i,alias=queue.popleft();i=optional_inner(i);c=part[i]
  if alias not in out:out[alias]=i
  if c in seen:continue
  seen.add(c);s=shapes[i]
  if s['kind']=='object':
   for k,p in s.get('properties',{}).items():
    target=optional_inner(p['type']);j=item(target)
    if j is not None:target=j
    queue.append((target,preferred.get(k,field(k))))
  elif s['kind']=='union':
   for j in s['members']:
    if shapes[j]['kind']=='object':queue.append((j,alias+'_case'))
 return out
aliases={}
for key,i in entries.items():
 aliases[key]=discover_aliases(i)
 if key in ['Save39', 'Save40', 'Save41', 'Save42', 'Save43', 'Save44', 'Save45', 'Save46', 'Save47', 'Save48', 'Save49', 'Save50', 'Save51','Save52','Save53']:
  for j,tag,_ in union_tags(i):
   for a,k in discover_aliases(j).items():aliases[key][tag.lower() if a=='t' else tag.lower()+'_'+a]=k
 if key in ['Save49','Save50','Save51','Save52','Save53']:
  aliases[key]['current_changes_item']=aliases[key]['current_changes']
 if key=='Fragments':
  aliases[key].update({field(n):p['type'] for n,p in shapes[i]['properties'].items()})
  aliases[key].update({field(n)+'_item':shapes[p['type']]['element'] for n,p in shapes[i]['properties'].items() if shapes[p['type']]['kind']=='array'})
 if key in ['History4', 'History5']:
  aliases[key]['entry']=shapes[shapes[i]['properties']['entries']['type']]['element']
 lines.append('\nmodule '+module_name(key)+' = struct')
 for alias,j in aliases[key].items():lines.append('  type nonrec '+alias+' = '+name(j))
 lines.append('end')
(base/'ocaml/Historical_types.ml').write_text('\n'.join(lines)+'\n')
# Emit exact readers and serializers. Decoder validates the source shape first.
C=['(* Generated from frozen src/formats/readers.json. Do not edit. *)','[@@@warning "-27"]','open Historical_types','open Migration_json','(* Melange 7.0.1 may merge null and undefined constant arms; keep null as an opaque, once-parsed value. *)','let json_null : Js.Json.t = Js.Json.parseExn "null"','let map_of_json decode value =\n  Array.fold_left (fun map (key, value) -> String_map.add key (decode value) map) String_map.empty (entries value)','let map_to_json encode value =\n  obj (List.map (fun (key, value) -> (key, encode value)) (String_map.bindings value))']
C.extend(['let wire_of_json known value =\n  let fields = Array.to_list (entries value) in\n  { order = List.map fst fields; extra = List.filter (fun (k,_) -> not (List.mem k known)) fields; undefined_fields = List.filter_map (fun (k,v) -> if is_undefined v then Some k else None) fields }', 'let wire_to_json ~known wire fields =\n  List.iter (fun (key, _) -> require (not (List.mem key known) || List.mem_assoc key fields) (\"Unpromoted historical field \" ^ key)) wire.extra;\n  let fields = fields @ List.filter (fun (k,_) -> not (List.mem_assoc k fields)) wire.extra in\n  let ordered = List.filter_map (fun key -> Option.map (fun v -> (key,v)) (List.assoc_opt key fields)) wire.order in\n  obj (ordered @ List.filter (fun (k,_) -> not (List.mem k wire.order)) fields)'])
def decode_expr(i,v):return 'decode_'+name(i)+' ('+v+')'
def encode_expr(i,v):return 'encode_'+name(i)+' ('+v+')'
def read_prop(key,p):
 v='field value '+json.dumps(key)
 if not p['optional']:return decode_expr(p['type'],v)
 ms=union_members(p['type'],True)
 dec='decode_'+name(ms[0]) if len(ms)==1 else 'decode_'+name(p['type'])
 return '(match get value '+json.dumps(key)+' with None -> None | Some x when is_undefined x -> None | Some x -> Some ('+dec+' x))'
def decode_body(i):
 s=shapes[i];k=s['kind']
 if k=='number':return 'number value'
 if k=='string':return 'string value'
 if k=='boolean':return 'boolean value'
 if k=='unknown':return 'value'
 if k in ['null','undefined']:return '`'+k.capitalize()
 if k=='literal':return '`'+ctor(s['value'])
 if k=='array':return 'Array.map decode_'+name(s['element'])+' (array value)'
 if k=='tuple':return 'let xs = array value in ('+', '.join(decode_expr(j,f'xs.({pos})') for pos,j in enumerate(s['items']))+')'
 if k=='union':
  tags=union_tags(i);expr='invalid "Invalid frozen union"'
  for j,tag,payload in reversed(tags):
   result='`'+tag+(' ('+decode_expr(j,'value')+')' if payload else '')
   expr='if Migration_reader.matches value '+str(j)+' then '+result+' else '+expr
  return expr
 if k=='object':
  props=s.get('properties',{})
  if not props and 'index'in s:return 'map_of_json decode_'+name(s['index'])+' value'
  if not props:return 'Empty_object'
  fields=['wire = wire_of_json ['+'; '.join(json.dumps(n) for n in props)+'] value']+[field(n)+' = '+read_prop(n,p) for n,p in props.items()]
  if 'index'in s:fields.append('additional = map_of_json decode_'+name(s['index'])+' value')
  return '{ '+ '; '.join(fields)+' }'
 raise ValueError(s)
def encode_body(i):
 s=shapes[i];k=s['kind']
 if k=='number':return 'n value'
 if k=='string':return 's value'
 if k=='boolean':return 'b value'
 if k=='unknown':return 'value'
 if k=='null':return 'json_null'
 if k=='undefined':return 'undefined'
 if k=='literal':return 'Js.Json.parseExn '+json.dumps(json.dumps(s['value']))
 if k=='array':return 'Js.Json.array (Array.map encode_'+name(s['element'])+' value)'
 if k=='tuple':
  vs=['v'+str(n) for n in range(len(s['items']))]
  return 'let ('+', '.join(vs)+') = value in a ['+'; '.join(encode_expr(j,v) for j,v in zip(s['items'],vs))+']'
 if k=='union':
  cases=[]
  for j,tag,payload in union_tags(i):
   member=shapes[j]
   if payload:body=encode_expr(j,'x')
   elif member['kind']=='null':body='json_null'
   elif member['kind']=='undefined':body='undefined'
   elif member['kind']=='literal':body='Js.Json.parseExn '+json.dumps(json.dumps(member['value']))
   else:raise ValueError(member)
   cases.append('| `'+tag+(' x' if payload else '')+' -> '+body)
  return 'match value with '+' '.join(cases)
 if k=='object':
  props=s.get('properties',{})
  if not props and 'index'in s:return 'map_to_json encode_'+name(s['index'])+' value'
  if not props:return 'obj []'
  required=[];optional=[]
  for key,p in props.items():
   val='value.'+field(key)
   if p['optional']:
    ms=union_members(p['type'],True);fn=name(ms[0]) if len(ms)==1 else name(p['type'])
    optional.append('(match '+val+' with None -> (if List.mem '+json.dumps(key)+' value.wire.undefined_fields then [('+json.dumps(key)+', undefined)] else []) | Some x -> [('+json.dumps(key)+', encode_'+fn+' x)])')
   else:required.append('('+json.dumps(key)+', '+encode_expr(p['type'],val)+')')
  all='['+'; '.join(required)+']'
  for o in optional:all+=' @ '+o
  if 'index'in s:all+=' @ List.map (fun (k,v) -> (k,encode_'+name(s['index'])+' v)) (String_map.bindings value.additional)'
  return 'wire_to_json ~known:['+'; '.join(json.dumps(n) for n in props)+'] value.wire ('+all+')'
 raise ValueError(s)
for n,(c,i) in enumerate(sorted(reps.items())):C.append(('let rec 'if n==0 else 'and ')+'decode_'+names[c]+' value : '+names[c]+' =\n  '+decode_body(i))
for n,(c,i) in enumerate(sorted(reps.items())):C.append(('let rec 'if n==0 else 'and ')+'encode_'+names[c]+' (value : '+names[c]+') : Js.Json.t =\n  '+encode_body(i))
for c,i in sorted(reps.items()):
 C.extend(['let read_'+names[c]+' value : '+names[c]+' =','  require (Migration_reader.matches value '+str(i)+') '+json.dumps('Invalid frozen '+names[c])+';','  decode_'+names[c]+' value'])
for key,i in entries.items():
 if key in ['Save49','Save50','Save51','Save52','Save53','History4','History5']:continue
 mod=module_name(key) if key!='Fragments' else 'Fragments_document';C.extend(['\nmodule '+mod+' = struct','  let of_json value : Historical_types.'+module_name(key)+'.t =','    let _ = Migration_reader.read value '+json.dumps(key)+' in','    decode_'+name(i)+' value','  let to_json (value : Historical_types.'+module_name(key)+'.t) = encode_'+name(i)+' value','end'])
# Direct fragment APIs so no JSON survives a validated fixture boundary.
frag=shapes[entries['Fragments']]['properties']
C.append('\nmodule Fragment = struct')
for key,p in frag.items():
 C.extend(['  let '+field(key)+' value =','    let _ = Migration_reader.fragment value '+json.dumps(key)+' in','    decode_'+name(p['type'])+' value'])
C.append('end')
C.append('module Fragments = struct')
for key,p in frag.items():
 C.append('  let '+field(key)+'_of_json = Fragment.'+field(key))
 C.append('  let '+field(key)+'_to_json = encode_'+name(p['type']))
C.append('end')
# Normalize the final V49 save and V4 -> V5 history boundary to host text.
# Unknown JSON payloads are already host values and must remain untouched.
seen_text = set()
def text_visit(i):
 c = part[i]
 if c in seen_text: return
 seen_text.add(c)
 s = shapes[i]
 if s['kind']=='object':
  for p in s.get('properties',{}).values():
   ms=union_members(p['type'],True) if p['optional'] else []
   text_visit(ms[0] if len(ms)==1 else p['type'])
  if 'index' in s: text_visit(s['index'])
 elif s['kind']=='array': text_visit(s['element'])
 elif s['kind']=='tuple':
  for j in s['items']: text_visit(j)
 elif s['kind']=='union':
  for j,_,payload in union_tags(i):
   if payload: text_visit(j)
text_visit(entries['Save49'])
text_visit(entries['History4'])
host_text=set(seen_text)
seen_text.clear()
text_visit(entries['Save49'])
text_visit(entries['History5'])
text_visit(entries['Save50'])
text_visit(entries['Save51'])
text_visit(entries['Save52'])
text_visit(entries['Save53'])
js_text=set(seen_text)
def text_call(i,v):return 'host_'+name(i)+' ('+v+')'
def text_body(i):
 s=shapes[i];k=s['kind']
 if k=='string':return 'Migration_utf8.to_js value'
 if k=='array':return 'Array.map host_'+name(s['element'])+' value'
 if k=='tuple':
  vs=['v'+str(n) for n in range(len(s['items']))]
  return 'let ('+', '.join(vs)+') = value in ('+', '.join(text_call(j,v) for j,v in zip(s['items'],vs))+')'
 if k=='union':
  return 'match value with '+' '.join('| `'+tag+(' x -> `'+tag+' ('+text_call(j,'x')+')' if payload else ' -> `'+tag) for j,tag,payload in union_tags(i))
 if k=='object':
  props=s.get('properties',{})
  def map_index(v):return 'String_map.of_list (List.map (fun (key,v) -> (Migration_utf8.to_js key, '+text_call(s['index'],'v')+')) (String_map.bindings '+v+'))'
  if not props and 'index' in s:return map_index('value')
  if not props:return 'value'
  fields=['wire = host_wire value.wire']
  for n,p in props.items():
   v='value.'+field(n)
   if p['optional']:
    ms=union_members(p['type'],True);fn=name(ms[0]) if len(ms)==1 else name(p['type'])
    v='Option.map host_'+fn+' '+v
   else:v=text_call(p['type'],v)
   fields.append(field(n)+' = '+v)
  if 'index' in s:fields.append('additional = '+map_index('value.additional'))
  return '{ '+'; '.join(fields)+' }'
 return 'value'
C.append('let host_wire value = { order = List.map Migration_utf8.to_js value.order; extra = List.map (fun (k,v) -> (Migration_utf8.to_js k,v)) value.extra; undefined_fields = List.map Migration_utf8.to_js value.undefined_fields }')
for n,c in enumerate(sorted(host_text)):
 i=reps[c]
 C.append(('let rec ' if n==0 else 'and ')+'host_'+names[c]+' (value : '+names[c]+') : '+names[c]+' = '+text_body(i))



C.extend([
 'let js_map_of_json decode value = Array.fold_left (fun map (key,v) -> String_map.add key (decode v) map) String_map.empty (Js.Dict.entries (object_ value))',
 'let js_map_to_json encode value = object_from_entries (Array.of_list (List.map (fun (k,v) -> (k,encode v)) (String_map.bindings value)))',
 'let js_wire_of_json known value = let fields = Array.to_list (Js.Dict.entries (object_ value)) in { order = List.map fst fields; extra = List.filter (fun (k,_) -> not (List.mem k known)) fields; undefined_fields = List.filter_map (fun (k,v) -> if is_undefined v then Some k else None) fields }',
 'let js_wire_to_json ~known wire fields = List.iter (fun (key,_) -> require (not (List.mem key known) || List.mem_assoc key fields) ("Unpromoted historical field " ^ key)) wire.extra; let fields = fields @ List.filter (fun (k,_) -> not (List.mem_assoc k fields)) wire.extra in let ordered = List.filter_map (fun k -> Option.map (fun v -> (k,v)) (List.assoc_opt k fields)) wire.order in object_from_entries (Array.of_list (ordered @ List.filter (fun (k,_) -> not (List.mem k wire.order)) fields))'
])
for n,c in enumerate(sorted(js_text)):
 i=reps[c]
 body=decode_body(i).replace('decode_', 'js_decode_').replace('map_of_json ', 'js_map_of_json ').replace('wire_of_json ', 'js_wire_of_json ')
 if shapes[i]['kind']=='string':body='js_string value'
 C.append(('let rec ' if n==0 else 'and ')+'js_decode_'+names[c]+' value : '+names[c]+' = '+body)
for n,c in enumerate(sorted(js_text)):
 i=reps[c]
 body=encode_body(i).replace('encode_', 'js_encode_').replace('map_to_json ', 'js_map_to_json ').replace('wire_to_json ', 'js_wire_to_json ')
 if shapes[i]['kind']=='string':body='Js.Json.string value'
 C.append(('let rec ' if n==0 else 'and ')+'js_encode_'+names[c]+' (value : '+names[c]+') : Js.Json.t = '+body)
for key in ['Save49', 'Save50', 'Save51', 'Save52', 'Save53', 'History4', 'History5']:
 mod=module_name(key)
 prefix='' if key=='History4' else 'js_'
 C.extend(['module '+mod+' = struct', 'let of_json value : Historical_types.'+mod+'.t = let _ = Migration_reader.read value "'+key+'" in '+prefix+'decode_'+name(entries[key])+' value', 'let to_json = '+prefix+'encode_'+name(entries[key])])
 if key in ['Save49', 'History4']: C.append('let of_legacy = host_'+name(entries[key]))
 if key in ['Save49','Save50','Save51','Save52','Save53']:
  for alias in ['current_entry','turn_entry','current_state','current_changes_item']:
   C.append('let '+alias+'_to_json = js_encode_'+name(aliases[key][alias]))
   C.append('let '+alias+'_of_json value = require (Migration_reader.matches value '+str(aliases[key][alias])+') "Invalid frozen state or change"; js_decode_'+name(aliases[key][alias])+' value')
 C.append('end')
(base/'ocaml/Historical_codec.ml').write_text('\n'.join(C)+'\n')
I=['(* Generated from frozen src/formats/readers.json. Unchecked decoder helpers are private. *)','open Historical_types','val wire_to_json : known:string list -> wire -> (string * Js.Json.t) list -> Js.Json.t']
for c,i in sorted(reps.items()):
 I.append('val read_'+names[c]+' : Js.Json.t -> '+names[c])
 I.append('val encode_'+names[c]+' : '+names[c]+' -> Js.Json.t')
for key,i in entries.items():
 mod=module_name(key) if key!='Fragments' else 'Fragments_document'
 extras=[]
 if key in ['Save49','History4']: extras.append('  val of_legacy : Historical_types.'+mod+'.t -> Historical_types.'+mod+'.t')
 if key in ['Save49','Save50','Save51','Save52','Save53']:
  for alias in ['current_entry','turn_entry','current_state','current_changes_item']:
   extras.append('  val '+alias+'_to_json : Historical_types.'+mod+'.'+alias+' -> Js.Json.t')
   extras.append('  val '+alias+'_of_json : Js.Json.t -> Historical_types.'+mod+'.'+alias)
 I.extend(['module '+mod+' : sig','  val of_json : Js.Json.t -> Historical_types.'+module_name(key)+'.t','  val to_json : Historical_types.'+module_name(key)+'.t -> Js.Json.t']+extras+['end'])
I.append('module Fragment : sig')
for key,p in frag.items():I.append('  val '+field(key)+' : Js.Json.t -> '+name(p['type']))
I.extend(['end','module Fragments : sig'])
for key,p in frag.items():
 I.append('  val '+field(key)+'_of_json : Js.Json.t -> '+name(p['type']))
 I.append('  val '+field(key)+'_to_json : '+name(p['type'])+' -> Js.Json.t')
I.append('end')
(base/'ocaml/Historical_codec.mli').write_text('\n'.join(I)+'\n')
print('Generated',len(reps),'shared frozen types from',len(shapes),'wire shapes')
