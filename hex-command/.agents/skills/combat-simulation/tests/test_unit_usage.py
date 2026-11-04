"""Small recorded fixtures for presence denominators and support attribution."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('unit_usage', Path(__file__).parents[1] / 'scripts/unit_usage.py')
usage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(usage)


def unit(id, kind, side='player', tile='0,0'):
    return dict(id=id, type=kind, side=side, tile=tile, hp=10)


class UsageTests(unittest.TestCase):
    def test_damage_presence_ratio_weights_faction_totals_and_separates_thorns_and_tiers(self):
        profiles = {
            'boss': dict(family='boss', race='forest', leader=True),
            'tree': dict(name='Tree', level=1, family='tree', race='forest'),
            'sapling': dict(name='Sapling', level=2, family='tree', race='forest'),
            'unused': dict(name='Unused', level=3, family='tree', race='forest'),
            'foe': dict(family='foe', race='other'),
        }
        dims = dict(race='woodland', policy='balanced', opponentPolicy='balanced', map='all')
        presence = [dict(dims, unit='tree', unitTurns=10), dict(dims, unit='sapling', unitTurns=90),
                    dict(dims, unit='unused', unitTurns=0)]
        values = [dict(dims, kind='arena', unit=u, unitTurns=t, weaponHPDamageDealt=d,
                       thornHPDamageDealt=thorns) for u, t, d, thorns in
                  [('tree', 10, 500, 900), ('sapling', 90, 500, 0), ('boss', 999, 100000, 100000)]]
        rows = {r['unit']: r for r in usage.unit_damage(presence, values, profiles)}
        tree = rows['tree']
        self.assertEqual(tree['presenceSharePercent'], 10)
        self.assertEqual(tree['damageSharePercent'], 50)
        self.assertEqual(tree['damagePresenceRatio'], 5)
        self.assertEqual(tree['damagePerUnitTurn'], 50)
        self.assertEqual(tree['factionDamagePerUnitTurn'], 10)
        self.assertEqual(tree['thornDamagePerUnitTurn'], 90)
        self.assertAlmostEqual(rows['sapling']['damagePresenceRatio'], 5/9)
        self.assertEqual(rows['unused']['presenceSharePercent'], 0)
        self.assertIsNone(rows['unused']['damagePerUnitTurn'])
        self.assertIsNone(rows['unused']['damagePresenceRatio'])
        self.assertNotIn('boss', rows)
        self.assertNotIn('foe', rows)
        self.assertEqual(len(rows), 3, 'promotion tiers stay separate within a family')
        self.assertAlmostEqual(sum(r['presenceSharePercent'] * (r['damagePresenceRatio'] or 0)
                                   for r in rows.values()), 100)
        changed = usage.compare_unit_damage(list(rows.values()), list(rows.values()))
        self.assertEqual(changed[0]['damagePresenceRatioDelta'], 0)
        self.assertIsNone(next(r for r in changed if r['unit']=='unused')['damagePresenceRatioDelta'])
        text = usage.damage_markdown(changed, baseline=True)
        self.assertIn('10.00% → 10.00%', text)
        self.assertIn('5.00× → 5.00×', text)
        self.assertIn('90.00 → 90.00', text)
        self.assertIn('Unused (L3) | unknown | 0.00% → 0.00% | — → — | — → —', text)
        ordered = usage.unit_damage(presence, values, {
            k: dict(v, role='healer' if k == 'tree' else 'warrior') for k, v in profiles.items()})
        self.assertEqual([r['unitKind'] for r in ordered], ['infantry', 'infantry', 'support'])
        mixed = [dict(tree, unitKind=kind, name=kind) for kind in
                 ['support', 'unknown', 'ranger', 'flank', 'infantry']]
        rendered = usage.damage_markdown(mixed)
        self.assertEqual([line.split('|')[1].strip().split()[0] for line in rendered.splitlines()[2:]],
                         ['infantry', 'flank', 'ranger', 'support', 'unknown'])

    def test_damage_rates_keep_zero_damage_and_zero_exposure_distinct_and_do_not_pool_scopes(self):
        profiles = {u: dict(family=u, race='a') for u in ['foot', 'idle', 'unseen']}
        presence, values = [], []
        for landscape, policy, dealt in [('open', 'balanced', 20), ('wooded', 'balanced', 0),
                                          ('open', 'situational', 80)]:
            dims = dict(race='a', policy=policy, opponentPolicy=policy, map=landscape)
            for unit, turns in [('foot', 2), ('idle', 8), ('unseen', 0)]:
                presence.append(dict(dims, unit=unit, unitTurns=turns))
                values.append(dict(dims, kind='arena', unit=unit, unitTurns=turns,
                    weaponHPDamageDealt=dealt if unit=='foot' else 0, thornHPDamageDealt=0))
        rows = usage.unit_damage(presence, values, profiles)
        def row(unit, landscape='open', policy='balanced'):
            return next(r for r in rows if r['unit']==unit and r['map']==landscape and r['policy']==policy)
        self.assertEqual(row('idle')['damagePresenceRatio'], 0)
        self.assertEqual(row('idle')['damagePerUnitTurn'], 0)
        self.assertIsNone(row('unseen')['damagePresenceRatio'])
        self.assertIsNone(row('foot', 'wooded')['damagePresenceRatio'])
        self.assertEqual(row('foot', 'wooded')['damagePerUnitTurn'], 0)
        self.assertEqual(row('foot', policy='situational')['damagePerUnitTurn'], 40)
        before = [dict(row('foot'), damagePresenceRatio=2, damagePerUnitTurn=4)]
        comparison = usage.compare_unit_damage(rows, before)
        first = next(r for r in comparison if r['unit']=='foot' and r['map']=='open' and r['policy']=='balanced')
        self.assertEqual(first['damagePresenceRatioDelta'], 3)
        self.assertEqual(first['damagePerUnitTurnDelta'], 6)
        self.assertIsNone(next(r for r in comparison if r['unit']=='idle')['damagePresenceRatioBefore'])
        values[0]['unitTurns'] += 1
        with self.assertRaisesRegex(ValueError, 'census mismatch'):
            usage.unit_damage(presence, values, profiles)

    def test_presence_includes_starts_promotions_and_survival_but_excludes_leaders(self):
        profiles = {'boss':dict(family='boss',leader=True), 'foot':dict(family='foot'),
                    'heavy':dict(family='foot'), 'scout':dict(family='scout'), 'early':dict(family='early')}
        boss, foot, scout = unit('b','boss'), unit('f','foot'), unit('s','scout')
        heavy = unit('f','heavy')
        result = dict(config=dict(armies=dict(player='a',enemy='b'),player='situational',enemy='balanced',landscape='open'),
                      initial=dict(units=[boss,foot,scout,unit('d','early')]), phases=[
                          dict(side='player',before=[boss,foot,scout], units=[boss,foot,scout],losses=[]),
                          dict(side='player',before=[boss,foot],units=[boss,heavy],losses=[])])
        rows = self.run_record(profiles, result, support=False)['battlefield_presence']
        rows = {r['family']:r for r in rows if r['race']=='a' and r['map']=='all'}
        self.assertEqual(rows['foot']['unitTurns'],2)
        self.assertAlmostEqual(rows['foot']['sharePercent'],200/3)
        self.assertEqual(rows['scout']['unitTurns'],1)
        self.assertEqual(rows['scout']['battlePresencePercent'],100)
        self.assertNotIn('boss',rows)
        self.assertEqual(rows['early']['unitTurns'],0)
        self.assertEqual(rows['early']['battlePresencePercent'],100)

    def test_support_ties_split_credit_and_count_incoming_turns_for_thorns(self):
        profiles = {'medic':dict(family='mage',healing=3), 'aura':dict(family='mage',thornAura=2),
                    'foot':dict(family='foot',thorns=1),'enemy':dict(family='enemy')}
        m1,m2 = unit('m1','medic',tile='0,0'),unit('m2','medic',tile='2,0')
        target = unit('t','foot',tile='1,0')
        aura = unit('a','aura',tile='1,1')
        foe = unit('e','enemy',side='enemy',tile='3,0')
        roster=[m1,m2,target,aura,foe]
        phases=[dict(side='player',before=roster,units=roster,losses=[],combats=[],
                     healing=[dict(target='t',providers=['m1','m2'],amount=3)]),
                dict(side='enemy',before=roster,units=roster,losses=[],healing=[],combats=[dict(
                    target='t',thorns=dict(damage=6,auraDamage=4,effectiveDamage=3,auraSources=['a']))])]
        result=dict(config=dict(armies=dict(player='a',enemy='b'),player='situational',enemy='balanced',landscape='open'),initial=dict(units=roster),phases=phases)
        rows=self.run_record(profiles,result,presence=False)['support_effects']
        all_rows={r['ability']:r for r in rows if r['race']=='a' and r['map']=='all' and r['unit']=='all'}
        self.assertEqual(all_rows['healing']['providerTurns'],2)
        self.assertEqual(all_rows['healing']['healedHP'],3)
        self.assertEqual(all_rows['healing']['healedRecipients'],1)
        self.assertEqual(all_rows['thornAura']['providerTurns'],1)
        self.assertEqual(all_rows['thornAura']['reflectedDamage'],4)
        self.assertEqual(all_rows['thorns']['reflectedDamage'],2)
        self.assertEqual(all_rows['thornAura']['effectiveReflectedDamage'],2)
        self.assertEqual(all_rows['thorns']['effectiveReflectedDamage'],1)
        overlap = self.run_record(profiles,result,presence=False)['support_overlap']
        healing = [r for r in overlap if r['race']=='a' and r['map']=='all' and r['ability']=='healing'][0]
        self.assertEqual(healing['providerCount'],2)
        self.assertGreater(healing['overlappingRecipients'],0)


    def test_missing_support_telemetry_is_not_reported_as_zero_effect(self):
        result=dict(config=dict(armies=dict(player='a',enemy='b'),player='situational',enemy='balanced',landscape='open'),initial=dict(units=[]),phases=[])
        with self.assertRaisesRegex(ValueError,'fresh run'):
            self.run_record({},result)

    def test_arrow_usage_requests_existing_pyarrow_fallback_when_dependency_is_missing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            raw = b'{}'
            (root/'details.arrow').write_bytes(raw)
            path = root/'report.json'
            path.write_text(json.dumps(dict(status='complete',outcomes=[dict(kind='arena',config={},detail=dict(file='details.arrow',offset=0,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))])))
            with patch.object(usage.importlib.util,'find_spec',return_value=None):
                with self.assertRaisesRegex(ModuleNotFoundError,'pyarrow'):
                    list(usage.journals([path]))

    def run_record(self,profiles,result,**options):
        tiles = {u['tile'] for u in result['initial']['units']}
        for phase in result['phases']:
            tiles.update(u['tile'] for key in ('before', 'units', 'losses') for u in phase.get(key, []))
        result['initial']['tiles'] = [dict(id=t, col=int(t.split(',')[0]), row=int(t.split(',')[1])) for t in tiles]
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            raw=(json.dumps(result)+'\n').encode()
            (root/'details.jsonl').write_bytes(raw)
            report=dict(status='complete',unitProfiles=profiles,supportTelemetryVersion=1,outcomes=[dict(kind='arena',config=result['config'],detail=dict(file='details.jsonl',offset=0,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))])
            path=root/'report.json';path.write_text(json.dumps(report))
            tables,_=usage.analyze([path],**options)
            return tables


if __name__ == '__main__':
    unittest.main()
