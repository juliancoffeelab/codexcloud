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
