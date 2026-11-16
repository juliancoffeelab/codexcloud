"""Correlated event-group helpers for the chapter generator.

The chapter CLI is generate.py. This module consumes its objective/result batch;
it does not start a separate objective walk or provide another chapter CLI.
The classified corpus and result references live beside this module.
"""
from collections import Counter, defaultdict
import json
import math
from pathlib import Path
import random

class Bags:
    """Every decision consumes a pre-shuffled finite weighted bag."""
    def __init__(self,seed,size):
        self.rng=random.Random(seed);self.size=size;self.rows={};self.bags={}
    def draw(self,key,weights):
        positive={k:float(weights[k]) for k in sorted(weights,key=str) if weights[k]>0}
        if not positive:raise ValueError(f'Empty sampling row: {key}')
        if key not in self.rows:
            self.rows[key]=positive
        elif self.rows[key]!=positive:raise ValueError(f'Changed distribution for {key}')
        if not self.bags.get(key):
            total=sum(positive.values());bag=[]
            for value,weight in positive.items():
                n=self.size*weight/total;integer=math.floor(n)
                integer+=self.rng.random()<n-integer
                bag.extend([value]*integer)
            if not bag:raise ValueError(f'Rounded empty bag: {key}')
            self.rng.shuffle(bag);self.bags[key]=bag
        return self.bags[key].pop()

def atoms(node,path='root'):
    if node['type']=='ATOM':yield path,node['objective']
    for i,child in enumerate(node.get('children',[])):
        yield from atoms(child,f'{path}.children[{i}]')

def group_key(e):
    return (e['chapter_id'],e.get('group_id') or 'UNGROUPED:'+e['event_id'])

def source_event(e):
    return {'event_id':e['event_id'],'kind':e['kind'],
            'actor_relation':e.get('actor',{}).get('relation','unknown'),
            'conditional':e.get('conditional',False),
            'source_event_ids':e['source_event_ids'],
            'source_objective_ids':[l['objective_id'] for l in e.get('objective_links',[])],
            'source_objective_links':e.get('objective_links',[]),
            'source_result_ids':e.get('result_ids',[]),
            'source_route_ids':e.get('route_ids',[])}

class Corpus:
    def __init__(self):
        data=json.loads(Path(__file__).with_name('event_corpus.json').read_text())
        self.events=data['events']
        self.objectives={o['objective_id']:o for o in data['objectives']}
        self.routes={r['route_id']:r for r in data['routes']}
        self.chapters=set(data['chapter_ids'])
        self.hashes=data['provenance']['classification_sha256']
        self.scope=data['scope']
        self.results=json.loads(Path(__file__).with_name('objective_results.json').read_text())['results']
        self.groups=defaultdict(list)
        self.by_objective=defaultdict(list)
        for e in self.events:
            self.groups[group_key(e)].append(e)
            if e['pool']=='objective_linked':
                for link in e['objective_links']:self.by_objective[link['objective_id']].append(e)
        self.main_profiles=defaultdict(list)
        for oid,o in self.objectives.items():
            if o['scope']!='main':continue
            source_routes=[r for r in o.get('route_ids',[]) if self.routes[r]['type']=='chapter_completion']
            contexts=source_routes or [None]
            for route in contexts:
                mapped=[e for e in self.by_objective[oid] if self.applies(e,route)]
                groups=[]
                for key in sorted({group_key(e) for e in mapped}):
                    # Keep connected linked effects together; independent and optional
                    # pools are sampled separately. No original story prose is output.
                    members=[e for e in self.groups[key] if e['pool']=='objective_linked' and self.applies(e,route)]
                    groups.append(self.pack(key,members))
                rids=self.routes[route].get('result_ids',[]) if route else []
                self.main_profiles[o['kind']].append({'objective_id':oid,'route_id':route,
                    'result_kinds':sorted({self.results[r]['result_kind'] for r in rids if r in self.results}),
                    'groups':groups,'weight':1/len(contexts)})
        self.pool_groups={p:[] for p in ['independent','optional_encounter']}
        for key,members in self.groups.items():
            for pool in self.pool_groups:
                selected=[e for e in members if e['pool']==pool]
                if selected:
                    variants=self.partition(key,selected)
                    self.pool_groups[pool].append({'source_group':list(key),'variants':variants})
        self.counts={}
        for pool,groups in self.pool_groups.items():
            counts=Counter(g['source_group'][0] for g in groups)
            self.counts[pool]=dict(Counter(counts[c] for c in self.chapters))
        self.main_kind_chapters=defaultdict(set)
        self.independent_kind_chapters=defaultdict(set)
        for o in self.objectives.values():
            if o['scope']=='main':self.main_kind_chapters[o['kind']].add(o['chapter_id'])
        for e in self.events:
            if e['pool']=='independent':self.independent_kind_chapters[e['kind']].add(e['chapter_id'])
    def applies(self,e,route):
        constraints={r for r in e.get('route_ids',[]) if self.routes[r]['type']=='chapter_completion'}
        return route is None or not constraints or route in constraints
    def pack(self,key,events):
        return {'source_group':list(key),'events':[source_event(e) for e in events],
                'source_result_kinds':sorted({self.results[r]['result_kind'] for e in events for r in e.get('result_ids',[]) if r in self.results})}
    def partition(self,key,members):
        # Split actual alternative memberships; shared effects occur in each
        # applicable variant. Encounter alternatives take precedence over chapter routes.
        used={r for e in members for r in e.get('route_ids',[])}
        encounter={r for r in used if self.routes[r]['type']=='encounter_alternative'}
        relevant=encounter or used
        signatures={tuple(sorted(set(e.get('route_ids',[]))&relevant)) for e in members}
        selectors=sorted(relevant) if len(signatures)>1 else [None]
        variants=[];seen=set()
        for route in selectors:
            selected=[e for e in members if route is None or not (set(e.get('route_ids',[]))&relevant) or route in e.get('route_ids',[])]
            signature=tuple(e['event_id'] for e in selected)
            if not signature or signature in seen:continue
            seen.add(signature)
            packed=self.pack(key,selected)
            packed['source_route_id']=route
            ids={l['objective_id'] for e in selected for l in e.get('objective_links',[])}
            packed['objectives']=[{'kind':self.objectives[o]['kind'],'source_objective_id':o} for o in sorted(ids)]
            variants.append(packed)
        return variants
    def bias(self,kinds,objective_kinds,strength):
        # Shrink small conditional chapter samples toward the global marginal.
        # This is a preference, not an asserted causal or joint distribution.
        lifts=[]
        for kind in sorted(set(kinds)):
            occurrences=self.independent_kind_chapters[kind]
            marginal=len(occurrences)/len(self.chapters)
            if not marginal:continue
            per_objective=[]
            for objective in sorted(set(objective_kinds)):
                cs=self.main_kind_chapters.get(objective,set())
                conditional=(len(cs&occurrences)+strength*marginal)/(len(cs)+strength)
                per_objective.append(conditional/marginal)
            lifts.append(sum(per_objective)/len(per_objective) if per_objective else 1)
        return math.exp(sum(math.log(v) for v in lifts)/len(lifts)) if lifts else 1

def attach(batch,seed,strength=20):
    """Attach correlated event groups to an already sampled objective batch."""
    count=batch['count']
    if not math.isfinite(strength) or strength<=0:
        raise ValueError('event prior strength must be finite and positive')
    corpus=Corpus();bags=Bags(seed^0xEC03,max(256,count*32))
    output=[]
    for original in batch['objectives']:
        row={'id':original['id'],'objective_expression':original['expression'],
             'objective_flow':original['flow'],'outcomes':original['outcomes'],
             'objective_draws':original['draws'],
             'linked_events':[],'independent_groups':[],'optional_encounters':[]}
        kinds=[kind for _,kind in atoms(original['flow'])]
        used=set()
        for path,kind in atoms(original['flow']):
            results=sorted({o['result'] for o in original['outcomes'] if path in o['terminal_paths']})
            candidates=corpus.main_profiles.get(kind,[])
            # Result agreement is a soft preference, shrunk for sparse rows.
            # Explicitly incompatible result-delivery groups are still excluded.
            eligible=[]
            matching_mass=sum(p['weight'] for p in candidates if set(results)&set(p['result_kinds']))
            match_bonus=1+3*matching_mass/(matching_mass+strength) if results else 1
            for i,p in enumerate(candidates):
                groups=[g for g in p['groups'] if not results or not g['source_result_kinds']
                        or set(results)&set(g['source_result_kinds'])]
                if p['groups'] and not groups:continue
                eligible.append((i,p,groups))
            entry={'objective_path':path,'objective_kind':kind,'result_context':results,
                   'selection_context':'objective kind with shrunken result preference',
                   'result_match_bonus':match_bonus,'groups':[]}
            if not eligible:
                entry['note']='No compatible mapped source objective profile available.'
                row['linked_events'].append(entry);continue
            weights={i:p['weight']*(match_bonus if set(results)&set(p['result_kinds']) else 1)
                     for i,p,groups in eligible}
            i=bags.draw(('linked',kind,tuple(results),tuple(weights)),weights)
            profile=candidates[i]
            adapted_groups=next(groups for j,p,groups in eligible if j==i)
            entry['source_objective_id']=profile['objective_id']
            entry['source_route_id']=profile['route_id']
            for group in adapted_groups:
                key=tuple(group['source_group'])
                if key in used:continue
                # Never copy a known incompatible result-delivery group. An
                # unmapped group is usable, but its uncertainty remains explicit.
                if results and group['source_result_kinds'] and not set(results)&set(group['source_result_kinds']):continue
                used.add(key);entry['groups'].append(group)
            if not entry['groups']:entry['note']='No additional mapped event group drawn; the objective still happens.'
            row['linked_events'].append(entry)
        for pool,field in [('independent','independent_groups'),('optional_encounter','optional_encounters')]:
            n=bags.draw(('count',pool),corpus.counts[pool])
            candidates=corpus.pool_groups[pool]
            selected=set()
            for slot in range(n):
                weights={}
                for i,g in enumerate(candidates):
                    if i in selected or tuple(g['source_group']) in used:continue
                    event_kinds=[e['kind'] for v in g['variants'] for e in v['events']]
                    weights[i]=corpus.bias(event_kinds,kinds,strength) if pool=='independent' else 1
                index=bags.draw((pool,tuple(sorted(set(kinds))),tuple(sorted(selected)),tuple(sorted(used))),weights)
                selected.add(index);used.add(tuple(candidates[index]['source_group']));row[field].append(candidates[index])
        output.append(row)
    return {'seed':seed,'count':count,'prior_strength':strength,
            'method':{'draws':'Pre-shuffled weighted bags at every decision.',
                      'objectives_results':'Existing conditional objective walker, unchanged.',
                      'linked_events':'Sample recorded main-objective profiles with a soft result-match bonus (1 + 3*m/(m+prior_strength), m = matching source-case weight). Preserve connected groups; exclude explicitly incompatible result-delivery groups. Source groups cannot repeat within a chapter.',
                      'independent':'Draw chapter group count from recorded histogram; sample connected source groups without replacement, biased by shrunken chapter-level objective co-occurrence. Geometric mean over group event-kind lifts; arithmetic mean over objective kinds.',
                      'optional':'Draw recorded chapter encounter-group count, then whole encounter groups without replacement. Keep their action/event alternatives connected.',
                      'prior_strength':'20 pseudo-chapters toward global marginal by default; experimental design choice, not a measured frequency.',
                      'limitations':['All recorded alternatives contribute; counts are not per-playthrough frequencies.',
                                     'No mapped linked group is missing annotation or ordinary automatic resolution, not proof of no event.',
                                     'Conditional fields in source examples remain provenance, not mandatory casualties or plot conditions in the generated story.',
                                     'Source names and plot prose are withheld from the readable ingredients; source IDs remain in JSON.',
                                     'This does not fit chronology or guarantee narrative/engine compatibility.']},
            'scope':corpus.scope,'input_sha256':corpus.hashes,'objective_rejections':batch['rejected_attempts'],
            'chapters':output}

def event_label(e,objective_id=None):
    kind=e['kind'];relation=e['actor_relation']
    if kind in ['Retreat','Departure','Surrender']:
        kind = f'{kind} ({relation} actor)' if relation!='unknown' else kind
    if e.get('conditional'):
        kind += ' [conditional]'
    if objective_id is not None:
        roles=sorted({l['relationship'] for l in e['source_objective_links'] if l['objective_id']==objective_id})
        kind += ' (' + '/'.join(roles or ['connected effect']) + ')'
    return kind

def group_label(group,objective_id=None):
    return ' + '.join(dict.fromkeys(event_label(e,objective_id) for e in group['events']))

def render_events(row):
    """Render connected event ingredients without assigning their chronology."""
    lines=['**Linked events:**']
    for entry in row['linked_events']:
        value='; '.join(group_label(g,entry.get('source_objective_id')) for g in entry['groups']) or 'No additional mapped event'
        lines.append(f"- {entry['objective_kind']} → {value}.")
    lines.extend(['','**Independent developments (unordered):**'])
    if not row['independent_groups']:
        lines.append('- None drawn.')
    for number,group in enumerate(row['independent_groups'],1):
        for i,variant in enumerate(group['variants'],1):
            prefix=f'Group {number}, alternative {i}: ' if len(group['variants'])>1 else ''
            lines.append('- '+prefix+group_label(variant)+'.')
    lines.extend(['','**Optional encounters:**'])
    if not row['optional_encounters']:
        lines.append('- None drawn.')
    for number,group in enumerate(row['optional_encounters'],1):
        for i,variant in enumerate(group['variants'],1):
            action=' AND '.join(dict.fromkeys(o['kind'] for o in variant['objectives'])) or 'Action not classified'
            prefix=f'Encounter {number}, alternative {i}: ' if len(group['variants'])>1 else f'Encounter {number}: '
            lines.append('- '+prefix+action+' → '+group_label(variant)+'.')
    return '\n'.join(lines)
