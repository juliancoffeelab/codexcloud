import tempfile
import unittest
from pathlib import Path
import torch
from hex_command_rl.recovery import save_bundle,resolve,load_resume
from hex_command_rl.spatial import SpatialPlanner
from hex_command_rl.mixed_workload import configs,group,mass

class RecoveryContracts(unittest.TestCase):
    def test_bundle_and_corruption_detection(self):
        with tempfile.TemporaryDirectory() as d:
            policy=SpatialPlanner();save_bundle(d,dict(value=torch.tensor([3.])),policy,'test',best=True)
            self.assertEqual(float(load_resume(d)['value'][0]),3.)
            folder=resolve(d,'best.json');(folder/'resume.pt').write_bytes(b'broken')
            with self.assertRaises(ValueError):resolve(d)

    def test_workload_disjoint_and_hierarchically_balanced(self):
        partitions=[configs(s) for s in ['train','validation','test']]
        seeds=[{c['seed'] for c in cases} for cases in partitions]
        for i in range(3):
            for j in range(i):self.assertFalse(seeds[i]&seeds[j])
        unique={group(c):c for c in partitions[0]}
        self.assertAlmostEqual(sum(mass(c) for c in unique.values()),1.)
        self.assertAlmostEqual(sum(mass(c) for c in unique.values() if 'campaign' in c),.5)
