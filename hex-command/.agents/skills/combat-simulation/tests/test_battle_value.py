"""Small fixtures for protection, objective attribution and death evidence."""
import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('battle_value', Path(__file__).parents[1] / 'scripts/battle_value.py')
value = importlib.util.module_from_spec(spec)
spec.loader.exec_module(value)


def unit(id, kind, side='player', tile='0,0'):
    return dict(id=id, type=kind, side=side, tile=tile, hp=10)


def fixture():
    foot, foe = unit('f', 'foot'), unit('e', 'foe', 'enemy', '2,0')
    report = dict(combatTelemetryVersion=1, leaderAuditVersion=1, unitProfiles={
        'foot': dict(family='foot', cost=7, race='human'),
        'heavy': dict(family='foot', cost=12, race='human'),
        'foe': dict(family='foe', cost=6, race='skeletons')})
    initial = dict(units=[foot, foe], tiles=[dict(id='0,0', terrain='keep', owner='player'),
        dict(id='1,0', terrain='village', owner=None), dict(id='2,0', terrain='plains', owner=None)])
    result = dict(kind='arena', config=dict(armies=dict(player='human', enemy='skeletons'),
        player='balanced', enemy='balanced', landscape='open', seed=1, first='player'),
        initial=initial, phases=[])
    return report, result, foot, foe


def analyze(report, result, **options):
    def journals(inputs, kinds):
        yield report, result
    return value.analyze([], journals, **options)


class ValueTests(unittest.TestCase):
    def test_effective_protection_capture_control_and_promoted_casualty_cost(self):
        report, result, foot, foe = fixture()
        heavy = unit('f', 'heavy', tile='1,0')
        mitigation = dict(rawDamage=12, afterArmourDamage=8, afterTerrainDamage=7,
                          availableHp=10, effectiveArmourPrevented=2, effectiveTerrainPrevented=1)
        retaliation = dict(rawDamage=4, afterArmourDamage=6, afterTerrainDamage=6,
                           availableHp=4, effectiveArmourPrevented=0, effectiveTerrainPrevented=0)
        owners = [dict(tile=t['id'], owner=('player' if t['id']=='1,0' else t.get('owner')))
                  for t in result['initial']['tiles']]
        result['initial']['units'] = [foe]
        result['phases'] = [dict(side='player', round=1, before=[foe], units=[foot, foe], losses=[],
            actions=[dict(kind='recruit', unit='leader', type='foot', tile='0,0')],
            combats=[], ownership=[dict(tile=t['id'], owner=t.get('owner')) for t in result['initial']['tiles']]),
            dict(side='player', round=2, before=[foot, foe], units=[heavy, foe],
            losses=[], actions=[dict(kind='advance', unit='f', type='heavy'), dict(kind='move', unit='f', tile='1,0')],
            combats=[dict(unit='e', target='f', mitigation=dict(attack=mitigation, retaliation=retaliation))],
            ownership=owners, objectives=[dict(kind='capture', tiles=['1,0'])]),
            dict(side='enemy', round=1, before=[heavy, foe], units=[foe], losses=[heavy],
                 actions=[], combats=[], ownership=owners)]
        tables = analyze(report, result, leaders=False)
        rows = tables['battlefield_value']
        row = next(r for r in rows if r['family']=='foot' and r['map']=='all')
        self.assertEqual(row['hpDamageTaken'], 7)
        self.assertEqual(row['hpDamageDealt'], 4)
        self.assertEqual(row['armourDamagePrevented'], 2)
        self.assertEqual(row['terrainDamagePrevented'], 1)
        self.assertEqual(row['rawArmourDelta'], 4)
        self.assertEqual(row['objectiveCaptures'], 1)
        self.assertEqual(row['objectiveOccupationTurns'], 1)
        self.assertEqual(row['ownedTileTurns'], 1)
        self.assertEqual(row['recruitGoldLost'], 7)
        self.assertEqual(row['casualtyReplacementValue'], 12)
        promoted = next(r for r in tables['battlefield_unit_value'] if r['unit']=='heavy' and r['map']=='all')
        self.assertEqual(promoted['recruitGoldLost'], 7)
        enemy = next(r for r in rows if r['family']=='foe' and r['map']=='all')
        self.assertEqual(enemy['rawArmourDelta'], -2)
        self.assertEqual(enemy['retaliationsTaken'], 1)

    def test_fresh_telemetry_is_required_not_silently_zero(self):
        report, result, _, _ = fixture()
        report.pop('combatTelemetryVersion')
        with self.assertRaisesRegex(ValueError, 'fresh run'):
            analyze(report, result, leaders=False)

    def test_effective_thorns_credit_intrinsic_and_aura_sources_without_overkill(self):
        report, result, foot, foe = fixture()
        aura = unit('a', 'heavy', tile='1,0')
        stage = dict(rawDamage=2, afterArmourDamage=2, afterTerrainDamage=2,
                     availableHp=10, effectiveArmourPrevented=0, effectiveTerrainPrevented=0)
        result['phases'] = [dict(side='enemy', round=1, before=[foot,foe,aura], units=[foot,foe,aura],
            losses=[], actions=[], ownership=[dict(tile=t['id'], owner=t.get('owner')) for t in result['initial']['tiles']],
            combats=[dict(unit='e', target='f', mitigation=dict(attack=stage, retaliation=None),
                          thorns=dict(damage=6, auraDamage=4, effectiveDamage=3, auraSources=['a']))])]
        rows = analyze(report, result, leaders=False)['battlefield_unit_value']
        rows = {r['unit']:r for r in rows if r['map']=='all'}
        self.assertEqual(rows['foot']['thornHPDamageDealt'], 1)
        self.assertEqual(rows['heavy']['thornHPDamageDealt'], 2)
        self.assertEqual(rows['foe']['thornHPDamageTaken'], 3)
        self.assertEqual(rows['foe']['weaponHPDamageDealt'], 2)

    def test_leader_death_links_only_its_own_match_and_retains_uncertainty(self):
        report, result, foot, foe = fixture()
        army = dict(units=3, cost=20, hp=30, maxHp=40, hpValue=15.0)
        audit = dict(leader='f', side='player', before=dict(tile='0,0', hp=10, maxHp=10),
            after=dict(tile='1,0', hp=10, maxHp=10), armyBefore=army, armyAfter=army,
            opponentBefore=army, opponentAfter=army,
            recruitmentBefore=dict(gold=7, onOwnedKeep=True, legalOptions=1, affordableOptions=1),
            exposure=dict(beforeDamage=0, chosenTileDamage=12, minDamage=0, minTile='0,0',
                          reachableTiles=2, safeTiles=1, chosenTileReachable=True, afterDamage=12))
        dead = copy.deepcopy(audit)
        dead.update(after=None, exposure=None)
        dead['armyAfter'] = dict(army, units=0, hpValue=0)
        result['phases'] = [dict(side='player', round=1, before=[foot,foe], units=[foot,foe],
            losses=[], leaderAudit=[audit]), dict(side='enemy', round=1, before=[foot,foe],
            units=[foe], losses=[foot], leaderAudit=[dead])]
        tables = analyze(report, result, value=False)
        death = tables['leader_deaths'][0]
        self.assertTrue(death['lastDecisionForecastLethalWithSafeAlternative'])
        self.assertEqual(death['armyUnitsAfter'], 0)
        self.assertNotIn('reckless', death)
        second = copy.deepcopy(result)
        second['config']['armies']['enemy'] = 'woodland'
        second['phases'] = [second['phases'][1]]
        def journals(inputs, kinds):
            yield report, result
            yield report, second
        deaths = value.analyze([], journals, value=False)['leader_deaths']
        self.assertIsNone(deaths[1]['lastDecisionRound'])

