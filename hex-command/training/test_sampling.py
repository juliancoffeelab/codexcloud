"""Small deterministic sampling streams, without running a training experiment."""
from copy import deepcopy
import json
import unittest
import numpy as np
from hex_command_rl.sampling import Sampler, mastery_multiplier


def fixture(**settings):
    adaptive = dict(enabled=True, completionThreshold=.8, minimumMultiplier=.1,
                    emaRate=.1, initialCompletionRate=.5, warmupEpisodes=10,
                    updateEveryEpisodes=20)
    adaptive.update(settings)
    spec = dict(samplingWeights=dict(arena=.25, campaign=.25, drill=.5),
                adaptiveSampling=adaptive,
                campaigns={cid: [dict(objective="authored", diplomacy="unrestricted",
                                     samplingGroup=bucket)]
                           for cid, bucket in [("broad", "campaign"), ("a", "drill"), ("b", "drill")]})
    base = dict(seed=1, player="balanced", enemy="classic", maxRounds=100,
                objective="authored", diplomacy="unrestricted")
    cases = [dict(base, campaign=cid) for cid in spec["campaigns"]]
    cases += [dict(base, landscape="arena", armies=dict(player=p, enemy=e), first=first)
              for p,e,first in [("human","skeletons","player"),
                                ("skeletons","human","player"),
                                ("human","skeletons","ai")]]
    return spec, cases


def episode(sampler, success, length=10, slot=0, truncated=False):
    c, _ = sampler.choose(slot)
    for _ in range(length):
        sampler.step(slot)
    sampler.finish(slot, length, success=success(c) if callable(success) else success,
                   truncated=truncated)
    return c


class SamplingContracts(unittest.TestCase):
    def test_threshold_linear_drop_and_floor(self):
        for p,w in [(0,1),(.5,1),(.8,1),(.9,.5),(.95,.25),(.98,.1),(1,.1)]:
            self.assertAlmostEqual(mastery_multiplier(p,.8,.1),w)

    def test_warmup_estimation_and_observation_outcomes(self):
        spec, cases = fixture(warmupEpisodes=10, updateEveryEpisodes=1)
        s = Sampler([cases[1]], np.random.default_rng(9), spec=spec)
        for _ in range(9): episode(s,True)
        key = next(iter(s.templates))
        self.assertEqual(s.multipliers[key],1)
        self.assertAlmostEqual(s.estimates[key]["completion"],1-.5*.9**9)
        episode(s,True)
        self.assertLess(s.multipliers[key],1)
        p = s.estimates[key]["completion"]
        episode(s,True,truncated=True)
        self.assertAlmostEqual(s.estimates[key]["completion"],p*.9)
        before = deepcopy(s.estimates)
        episode(s,None)
        self.assertEqual(s.estimates,before)

    def test_mastery_changes_actual_allocations_but_preserves_pool_budgets(self):
        spec, cases = fixture(emaRate=1,warmupEpisodes=1,updateEveryEpisodes=1)
        s = Sampler(cases,np.random.default_rng(19),spec=spec)
        for _ in range(600):
            episode(s,lambda c:c.get("campaign")=="a")
        a=next(k for k,c in s.templates.items() if c.get("campaign")=="a")
        b=next(k for k,c in s.templates.items() if c.get("campaign")=="b")
        self.assertAlmostEqual(s.weights[a]/s.weights[b],.1)
        self.assertLess(s.case_counts[a]/s.case_counts[b],.15)
        total=sum(s.case_counts.values())
        for pool,budget in spec["samplingWeights"].items():
            self.assertAlmostEqual(sum(w for k,w in s.weights.items() if s.buckets[k]==pool),budget)
            self.assertLess(abs(sum(v for k,v in s.case_counts.items() if s.buckets[k]==pool)/total-budget),.035)
        # Failures restore practice promptly, independent of old lifetime counts.
        for _ in range(300): episode(s,False)
        self.assertEqual(s.multipliers[a],1)
        self.assertAlmostEqual(s.weights[a],s.weights[b])

    def test_reweighting_keeps_other_pools_allocation_history(self):
        spec,cases=fixture(emaRate=1,warmupEpisodes=1,updateEveryEpisodes=1)
        s=Sampler(cases,np.random.default_rng(31),spec=spec)
        for _ in range(50):
            c,_=s.choose(0)
            for _ in range(10):s.step(0)
            before=deepcopy(s.allocation_counts)
            s.finish(0,10,success=c.get("campaign")=="a")
            if c.get("campaign")=="a":
                for key,value in before.items():
                    if s.buckets[key]!="drill":self.assertEqual(s.allocation_counts[key],value)
                break
        else:self.fail("mastered case was never scheduled")

    def test_armies_and_initiative_do_not_share_mastery(self):
        spec,cases=fixture(emaRate=1,warmupEpisodes=1,updateEveryEpisodes=1)
        s=Sampler(cases[3:],np.random.default_rng(3),spec=spec)
        for _ in range(180):
            episode(s,lambda c:c["armies"]["player"]=="human" and c["first"]=="player")
        self.assertEqual(len(s.estimates),3)
        for k,c in s.templates.items():
            expected=.1 if c["armies"]["player"]=="human" and c["first"]=="player" else 1
            self.assertEqual(s.multipliers[k],expected)

    def test_variable_lengths_balance_decisions_and_reserve_live_work(self):
        spec,cases=fixture(enabled=False)
        s=Sampler(cases[1:3],np.random.default_rng(4),spec=spec)
        first,_=s.choose(0)
        second,_=s.choose(1)
        self.assertNotEqual(first["campaign"],second["campaign"])
        for slot in (0,1):
            for _ in range(10):s.step(slot)
            s.finish(slot,10)
        for _ in range(300):
            c,_=s.choose(0);n=50 if c["campaign"]=="a" else 10
            for _ in range(n):s.step(0)
            s.finish(0,n,success=True)
        values=list(s.case_counts.values())
        self.assertLess(abs(values[0]/sum(values)-.5),.04)
        self.assertTrue(all(v==1 for v in s.multipliers.values()))

    def test_resume_preserves_frozen_weights_statistics_rng_and_next_choices(self):
        spec,cases=fixture(warmupEpisodes=1,updateEveryEpisodes=20)
        rng=np.random.default_rng(17)
        s=Sampler(cases,rng,spec=spec)
        for _ in range(37):episode(s,lambda c:c.get("campaign")=="a")
        # Sort/round-trip metadata as a checkpoint writer may do.
        state=json.loads(json.dumps(s.state(),sort_keys=True))
        other_rng=np.random.default_rng();other_rng.bit_generator.state=deepcopy(rng.bit_generator.state)
        other=Sampler(cases,other_rng,state,spec=spec)
        self.assertEqual(s.weights,other.weights)
        self.assertEqual(s.state(),other.state())
        for i in range(80):
            c,_=s.choose(0);d,_=other.choose(0);self.assertEqual(c,d)
            n=4+i%13
            for _ in range(n):s.step(0);other.step(0)
            s.finish(0,n,success=bool(i%2));other.finish(0,n,success=bool(i%2))
        self.assertEqual(s.state(),other.state())
        with self.assertRaises(ValueError):
            Sampler(cases,np.random.default_rng(0),dict(state,schema=1),spec=spec)
        # In-flight matches restart without being counted as a loss.
        s.choose(2);s.step(2)
        reset=Sampler(cases,np.random.default_rng(0),s.state(),spec=spec)
        self.assertFalse(reset.pending)
        self.assertEqual(s.estimates,reset.estimates)

if __name__=="__main__":unittest.main()
