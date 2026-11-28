"""Audit matching and consolidate the visibility evidence and native scores."""
from collections import Counter
from pathlib import Path

from fog_eval import CLEAR, IDS, OUT, SCOPES, catalog_for, digest, read, write


def results(scope, cohort):
    folder = OUT/scope/cohort
    protocol = read(folder/'protocol.json')
    cohorts = read(folder/'summary.json')
    for entry in cohorts:
        assert entry['successes']+entry['defeats']+entry['caps'] == protocol['seeds'][1]-protocol['seeds'][0]+1
        assert entry['caps'] == 0
    return dict(protocol=str(folder/'protocol.json'), seeds=protocol['seeds'], raw=str(folder/'results.jsonl'), count=sum(
        x['successes']+x['defeats']+x['caps'] for x in cohorts), byFixture={cid:{
        row['player']:{k:v for k,v in row.items() if k not in ('campaign','player')}
        for row in cohorts if row['campaign']==cid} for cid in IDS})


def verify_files(scope, catalog, hashes):
    for name, fingerprint in hashes.items():
        assert digest(catalog/name) == fingerprint
        if name.split('/')[-1] in ('roster.json','campaign.json'):
            continue
        assert digest(CLEAR/name) == fingerprint, name
    for cid in IDS:
        path = Path('game/campaigns')/cid/'roster.json'
        frozen, candidate = read(CLEAR/path), read(catalog/path)
        for group in candidate['visionGroups']:
            group['sight']['penetrates'] = ['fog']
        assert frozen == candidate


def variant(scope):
    catalog = catalog_for(scope)
    confirm = OUT/scope/'confirmation'
    proto = read(confirm/'protocol.json')
    audits = read(confirm/'evidence/fog-scope-audit.json')
    views = read(confirm/'evidence/native-visible-information.json')
    verify_files(scope,catalog,proto['files'])
    stats = {}
    for cid, raw in audits.items():
        phase = raw['phaseWitnesses'][0]['engineInitial']
        assert all(w['engineInitial']==phase for w in raw['phaseWitnesses'])
        assert phase['stateRound'] == 1 and phase['stateTurn']=='player'
        by_pilot = {pilot: Counter() for pilot in proto['cases']}
        for w in raw['phaseWitnesses']:
            by_pilot[w['pilot']]['sampled'] += 1
            by_pilot[w['pilot']]['captainBypass' if w['captainUsedBypass'] else 'captainOffBypass'] += 1
        sampled_views = [row for row in views if row['config']['campaign']==cid]
        counts = {view['observer']:view['visibleTiles'] for view in sampled_views if view['turn']==1}
        assert counts == {'player':198 if scope=='clear' else 65, 'enemy':37 if scope=='both-fog' else 198}, counts
        stats[cid] = dict(qualification='validated training candidate' if cid==IDS[0] else 'evaluation control',
                         routeEvent=raw['routeEvent'], firstAppliedPlayerPhase=phase,
                         verifiedReplays=len(raw['phaseWitnesses']), captainRouteSamples=by_pilot,
                         initialReferenceObservations={row['observer']:dict(visibleTiles=row['visibleTiles'],
                           visibleHostile=row['visibleHostile']) for row in sampled_views if row['turn']==1},
                         returnedFogSeenBy={side:any(row['newlyFoggedTiles'] for row in sampled_views
                            if row['observer']==side) for side in ('player','enemy')},
                         files={kind:str(catalog/'game/campaigns'/cid/f'{kind}.json') for kind in
                                ('map','roster','scripts','campaign','stage','story')})
    discovery = OUT/scope/'discovery/summary.json'
    return dict(catalog=str(catalog),compiledFingerprints=proto['fingerprints'],exactFileDigests=proto['files'],
                objectiveRoute='authored' if scope=='clear' else scope, fixtures=stats,
                discovery=results(scope,'discovery') if discovery.exists() else None,
                confirmation=results(scope,'confirmation'), scopeEvidence=str(confirm/'evidence/fog-scope-audit.json'),
                observedByFaction=str(confirm/'evidence/native-visible-information.json'),
                rawTraceDirectory=str(confirm/'evidence'))


def main():
    finished = {scope:variant(scope) for scope in SCOPES}
    assert {tuple(data['confirmation']['seeds']) for data in finished.values()} == {(56181201,56181220)}
    counts = {scope:{'discovery':data['discovery']['count'] if data['discovery'] else 0,
                     'confirmation':data['confirmation']['count']} for scope,data in finished.items()}
    write(OUT/'final-summary.json',dict(opponent='Classic', recruitsPerPhase=0, roundCap=100,
         goal='Bring named captain to (16,4) alive by the end of turn 7. Authored defeat on turn 8.',
         scope=finished, counts=counts, builtFromClearCatalog=str(CLEAR),
         renderedMap=str(CLEAR.parents[1]/'scree-relay.png'),
         sourceComparison=str(CLEAR.parent/'confirmation/results.jsonl')))
    print(counts)


if __name__ == '__main__':
    main()
