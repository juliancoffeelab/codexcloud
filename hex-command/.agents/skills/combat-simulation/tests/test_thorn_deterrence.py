"""Saved-phase reconstruction and paired aura comparison helpers."""
import importlib.util
from pathlib import Path
import sys
import unittest

SCRIPTS = Path(__file__).resolve().parents[1] / 'scripts'
sys.path.insert(0, str(SCRIPTS))
SPEC = importlib.util.spec_from_file_location('thorn_deterrence', SCRIPTS / 'measure-thorn-deterrence.py')
audit = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(audit)


class ThornDeterrenceTests(unittest.TestCase):
    def test_before_state_restores_spending_and_ownership_without_turn_start(self):
        actor = dict(id='u8', type='foot', side='enemy', tile='0,0', hp=7, moved=False)
        phase = dict(side='enemy', round=3, before=[actor], gold=dict(player=4, enemy=2),
                     actions=[dict(kind='recruit', type='foot'), dict(kind='advance', type='aura')],
                     ownership=[dict(tile='0,0', owner='enemy')], combats=[])
        previous = dict(ownership=[dict(tile='0,0', owner='player')],
                        before=[dict(id='u12')], units=[], losses=[dict(id='u15')], combats=[])
        detail = dict(initial=dict(nextId=3, tiles=[dict(id='0,0', owner=None)], combatRng=7),
                      final=dict(combatRng=7), phases=[previous, phase])
        state = audit.before_state(detail, 1, {'foot': dict(cost=6)})
        self.assertEqual(state['gold'], dict(player=4, enemy=8))
        self.assertEqual(state['tiles'][0]['owner'], 'player')
        self.assertEqual(state['nextId'], 16)
        self.assertEqual(state['units'], [actor])
        self.assertEqual(state['turn'], 'enemy')
        self.assertEqual(state['combatRng'], 7)
        self.assertEqual(phase['gold']['enemy'], 2)

    def test_phase_rng_includes_attack_retaliation_and_zero_spread_strikes(self):
        # Two identical fixed-damage strikes still consume two draws; the reply
        # consumes a third. The next phase must start after all three draws.
        previous = dict(combats=[dict(rawRolls=[5, 5], responseRawRolls=[0])],
                        ownership=[], before=[], units=[], losses=[])
        phase = dict(side='player', round=2, before=[], gold=dict(player=0, enemy=0),
                     actions=[], combats=[dict(rawRolls=[2], responseRawRolls=[])])
        detail = dict(initial=dict(combatRng=7, nextId=1, tiles=[]),
                      final=dict(combatRng=3981355051), phases=[previous, phase])
        self.assertEqual(audit.before_state(detail, 1, {})['combatRng'], 2630631676)
        self.assertEqual(detail['initial']['combatRng'], 7)

    def test_rng_reconstruction_rejects_missing_draws_or_unsupported_history(self):
        detail = dict(initial=dict(combatRng=7), final=dict(combatRng=1025555898),
                      phases=[dict(combats=[dict(rawRolls=[5], responseRawRolls=[])])])
        self.assertEqual(audit.combat_rng_states(detail), [7])
        detail['final']['combatRng'] = 3923423697
        with self.assertRaisesRegex(ValueError, 'rejected random samples'):
            audit.combat_rng_states(detail)
        detail['final']['combatRng'] = 1025555898
        del detail['phases'][0]['combats'][0]['responseRawRolls']
        with self.assertRaisesRegex(ValueError, 'responseRawRolls'):
            audit.combat_rng_states(detail)
        del detail['initial']['combatRng']
        with self.assertRaisesRegex(ValueError, 'initial.combatRng'):
            audit.combat_rng_states(detail)

    def test_aura_covers_intrinsic_thorns_but_not_itself(self):
        state = dict(turn='enemy', units=[
            dict(id='u1', type='aura', side='player', tile='0,0'),
            dict(id='u2', type='shield', side='player', tile='1,0'),
            dict(id='u3', type='foot', side='enemy', tile='0,1')])
        state['tiles'] = [dict(id=u['tile'], col=int(u['tile'].split(',')[0]), row=int(u['tile'].split(',')[1])) for u in state['units']]
        providers, covered = audit.coverage(state, {'aura': dict(thornAura=2), 'shield': dict(thorns=1), 'foot': {}})
        self.assertEqual(len(providers), 1)
        self.assertEqual(covered, {'u2'})

    def test_unchanged_replay_must_reproduce_recorded_phase(self):
        phase = dict(actions=[dict(kind='move', unit='u1', tile='1,0')],
                     units=[dict(id='u1', hp=7)], gold=dict(player=2, enemy=0))
        response = dict(records=dict(actions=phase['actions']),
                        final=dict(units=phase['units'], gold=phase['gold']))
        audit.verify_original_phase(response, phase)
        response['final']['units'] = [dict(id='u1', hp=6)]
        with self.assertRaisesRegex(ValueError, 'does not reproduce'):
            audit.verify_original_phase(response, phase)

    def test_switches_distinguish_missing_attackers_and_weapon_changes(self):
        def response(combats):
            return dict(records=dict(combats=[dict(unit=u, target=t, weapon=w) for u, t, w in combats]))
        on = response([('a', 'x', 'bow'), ('c', 'x', 'axe')])
        off = response([('a', 'y', 'sword'), ('b', 'x', 'axe')])
        self.assertEqual(audit.switches(on, off), dict(attackersSkippingWithEffect=1,
                         attackersAttackingOnlyWithEffect=1, targetSwitches=1, weaponSwitches=1))

    def test_effect_selection_preserves_other_thorns_and_selects_affected_recipients(self):
        rules = {'both': dict(thorns=2, thornAura=4), 'shield': dict(thorns=1), 'foot': {}}
        state = dict(turn='enemy', units=[
            dict(id='a', type='both', side='player', tile='0,0'),
            dict(id='b', type='foot', side='player', tile='1,0'),
            dict(id='c', type='shield', side='player', tile='4,0'),
            dict(id='d', type='shield', side='enemy', tile='0,1')])
        state['tiles'] = [dict(id=u['tile'], col=int(u['tile'].split(',')[0]), row=int(u['tile'].split(',')[1])) for u in state['units']]
        providers, covered = audit.coverage(state, rules, 'intrinsic')
        self.assertEqual({u['id'] for u in providers}, {'a', 'c'})
        self.assertEqual(covered, {'a', 'c'})
        self.assertEqual(audit.coverage(state, rules, 'all')[1], {'a', 'b', 'c'})
        self.assertEqual(audit.without_effect(rules['both'], 'intrinsic'), dict(thorns=0, thornAura=4))
        self.assertEqual(audit.without_effect(rules['both'], 'aura'), dict(thorns=2, thornAura=0))
        self.assertEqual(audit.without_effect(rules['both'], 'all'), dict(thorns=0, thornAura=0))
        self.assertEqual(rules['both'], dict(thorns=2, thornAura=4))

    def test_threat_filter_uses_melee_reach_and_offset_hex_coordinates(self):
        rules = {'foot': dict(move=1, attacks=[dict(category='melee')]),
                 'bow': dict(move=5, attacks=[dict(category='ranged')])}
        target = dict(id='a', type='foot', side='player', tile='1,1')
        enemy = dict(id='b', type='foot', side='enemy', tile='2,2')
        state = dict(turn='enemy', units=[target, enemy])
        state['tiles'] = [dict(id=u['tile'], col=int(u['tile'].split(',')[0]), row=int(u['tile'].split(',')[1])) for u in state['units']]
        self.assertTrue(audit.threatened(state, rules, {'a'}))
        self.assertFalse(audit.threatened(state, rules, set()))
        state['tiles'].append(dict(id='9,9', col=9, row=9))
        enemy['tile'] = '9,9'
        self.assertFalse(audit.threatened(state, rules, {'a'}))
        enemy.update(type='bow', tile='2,2')
        self.assertFalse(audit.threatened(state, rules, {'a'}))

    def test_profile_mismatch_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'thornAura'):
            audit.validate_profiles({'a': dict(thornAura=3)}, {'a': dict(thornAura=2)})


if __name__ == '__main__':
    unittest.main()
