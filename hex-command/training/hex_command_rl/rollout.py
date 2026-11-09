"""No-gradient native policy inference; optimization stays in PyTorch."""

import numpy as np
import torch
import hex_command_training as native
from .ppo import categorical


class NativeRollout:
    def __init__(self, actor_critic):
        self.ac = actor_critic
        self.sync()

    def sync(self):
        # One export per rollout, never per decision. Critic weights stay in Torch.
        self.evaluator = native.SpatialEvaluator(self.ac.policy.export())

    @torch.no_grad()
    def forward(self, samples, temperature):
        sizes = np.asarray([len(x) for x, _, _ in samples], dtype=np.int64)
        if not len(sizes) or np.any(sizes == 0):
            raise ValueError("rollout samples must contain candidates")
        offsets = np.concatenate([np.zeros(1, np.int64), sizes.cumsum()])
        board_sizes = np.asarray([b.size for _, b, _ in samples], np.int64)
        bounds = np.concatenate([np.zeros(1, np.int64), board_sizes.cumsum()])
        scores, pooled = self.evaluator.score(
            np.concatenate([x for x, _, _ in samples]),
            offsets,
            np.concatenate([b.ravel() for _, b, _ in samples]),
            bounds,
            np.asarray([b.shape[1:] for _, b, _ in samples], np.int64),
            np.concatenate([ix for _, _, ix in samples]),
        )
        logits = torch.zeros((len(samples), int(sizes.max())), dtype=torch.float32)
        mask = torch.zeros_like(logits, dtype=torch.bool)
        for i, n in enumerate(sizes):
            logits[i, :n] = torch.from_numpy(scores[offsets[i] : offsets[i + 1]])
            mask[i, :n] = True
        mean = torch.stack([torch.from_numpy(x).mean(0) for x, _, _ in samples])
        value = self.ac.critic(torch.cat([torch.from_numpy(pooled), mean], -1))
        return categorical(logits, mask, temperature), value.squeeze(-1)
