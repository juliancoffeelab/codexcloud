"""Explicit rollout state and short, ordered PPO replay windows."""
import numpy as np
import torch
from torch.nn import functional as F


class MemoryBank:
    def __init__(self):
        self.states = {}

    def reset(self, slot):
        self.states = {k: v for k, v in self.states.items() if k[0] != slot}

    def inputs(self, slots, contexts):
        memories, resets = [], []
        for slot, context in zip(slots, contexts):
            observer, map_id, round_no, decision, generation = context
            old = self.states.get((slot, observer))
            reset = (old is None or old[0][1] != map_id or old[0][4] != generation
                     or round_no < old[0][2] or decision < old[0][3])
            if reset:
                memory = torch.zeros(64)
            elif tuple(context) == old[0]:
                memory = old[1]  # Re-evaluation uses the same input state.
            else:
                memory = old[2]
            memories.append(memory)
            resets.append(reset)
        return torch.stack(memories), resets

    def commit(self, slots, contexts, before, after):
        for i, (slot, context) in enumerate(zip(slots, contexts)):
            self.states[(slot, context[0])] = (tuple(context), before[i].detach().clone(), after[i].detach().clone())


def windows(samples, rng, length=8):
    """Shuffle windows, never individual decisions within a window."""
    streams, sequences = {}, []
    for sample in samples:
        key = (sample['slot'], sample['context'][0])
        stream = streams.setdefault(key, [])
        if stream and (sample['memory_reset'] or len(stream) == length):
            sequences.append(stream)
            stream = streams[key] = []
        stream.append(sample)
    sequences.extend(s for s in streams.values() if s)
    return [sequences[i] for i in rng.permutation(len(sequences))]


def replay(ac, sequence, temperature):
    """TBPTT: detached window start; connected hidden states inside the window."""
    from .ppo import pack, categorical_kl
    from torch.distributions import Categorical
    memory = torch.as_tensor(np.asarray(sequence[0]['memory'])).unsqueeze(0)
    terms = []
    for s in sequence:
        tensors = pack([s['observation']])
        dist, value, memory = ac.forward_recurrent(*tensors, temperature, memory)
        action = torch.tensor([s['action']])
        ratio = (dist.log_prob(action) - s['old_log_prob']).clamp(-20, 20).exp()
        adv = torch.tensor([s['advantage']])
        pg = -torch.minimum(ratio*adv, ratio.clamp(.8, 1.2)*adv).mean()
        vl = F.mse_loss(value, torch.tensor([s['target']]))
        ref = Categorical(logits=torch.as_tensor(np.asarray(s['reference_logits'])).unsqueeze(0))
        kl = categorical_kl(dist, ref).mean()
        terms.append(torch.stack([pg, vl, dist.entropy().mean(), kl, ((ratio-1).abs()>.2).float().mean()]))
    return torch.stack(terms).mean(0).unbind()
