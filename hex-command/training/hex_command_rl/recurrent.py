"""Explicit rollout state and short, ordered PPO replay windows."""
import numpy as np
import torch
from torch.nn import functional as F


class MemoryBank:
    def __init__(self):
        self.states = {}

    def reset(self, slot):
        self.states = {k: v for k, v in self.states.items() if k[0] != slot}

    def inputs(self, slots, contexts, device=None):
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
        stacked = torch.stack(memories)
        return (stacked.to(device) if device is not None else stacked), resets

    def commit(self, slots, contexts, before, after):
        before_cpu = before.detach().cpu()
        after_cpu = after.detach().cpu()
        for i, (slot, context) in enumerate(zip(slots, contexts)):
            self.states[(slot, context[0])] = (tuple(context), before_cpu[i].clone(), after_cpu[i].clone())


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


def replay(ac, sequence, temperature, device=None):
    """TBPTT: detached window start; connected hidden states inside the window."""
    from .ppo import pack, categorical_kl
    from torch.distributions import Categorical
    device = next(ac.parameters()).device if device is None else torch.device(device)
    memory = torch.as_tensor(np.asarray(sequence[0]['memory']), device=device).unsqueeze(0)
    terms = []
    for s in sequence:
        tensors = pack([s['observation']], device=device)
        dist, value, memory = ac.forward_recurrent(*tensors, temperature, memory)
        action = torch.tensor([s['action']], dtype=torch.long, device=device)
        ratio = (dist.log_prob(action) - s['old_log_prob']).clamp(-20, 20).exp()
        adv = torch.tensor([s['advantage']], dtype=torch.float32, device=device)
        pg = -torch.minimum(ratio*adv, ratio.clamp(.8, 1.2)*adv).mean()
        vl = F.mse_loss(value, torch.tensor([s['target']], dtype=torch.float32, device=device))
        ref = Categorical(logits=torch.as_tensor(np.asarray(s['reference_logits']), device=device).unsqueeze(0))
        kl = categorical_kl(dist, ref).mean()
        terms.append(torch.stack([pg, vl, dist.entropy().mean(), kl, ((ratio-1).abs()>.2).float().mean()]))
    return torch.stack(terms).mean(0).unbind()


def group_windows(sequences, window_batch, rng):
    """Group windows of similar board size to minimize padding waste, then shuffle batches."""
    if window_batch <= 1:
        return [[seq] for seq in sequences]
    # Break ties randomly so equal-size maps mix across batches.
    ordered = sorted(
        sequences,
        key=lambda seq: (
            max(s['observation'][1].shape[1] * s['observation'][1].shape[2] for s in seq),
            float(rng.random()),
        ),
    )
    batches = [ordered[i : i + window_batch] for i in range(0, len(ordered), window_batch)]
    return [batches[i] for i in rng.permutation(len(batches))]


def prepare_replay_batch(sequences, device):
    """Pack and transfer a batch of windows once so multi-epoch PPO reuses device tensors."""
    from .ppo import pack
    if not sequences:
        raise ValueError("sequences must be nonempty")
    device = torch.device(device)
    batch_size = len(sequences)
    max_len = max(len(seq) for seq in sequences)
    init_memory = torch.as_tensor(
        np.stack([np.asarray(seq[0]['memory'], np.float32) for seq in sequences]),
        device=device,
    )
    pad_logit = float(np.finfo(np.float32).min)
    steps = []
    total_steps = 0
    for t in range(max_len):
        active = [i for i, seq in enumerate(sequences) if t < len(seq)]
        step_samples = [sequences[i][t] for i in active]
        tensors = pack([s['observation'] for s in step_samples], device=device)
        actions = torch.tensor([s['action'] for s in step_samples], dtype=torch.long, device=device)
        old_lp = torch.tensor([s['old_log_prob'] for s in step_samples], dtype=torch.float32, device=device)
        adv = torch.tensor([s['advantage'] for s in step_samples], dtype=torch.float32, device=device)
        target = torch.tensor([s['target'] for s in step_samples], dtype=torch.float32, device=device)
        width = tensors[0].shape[1]
        ref_logits = np.full((len(step_samples), width), pad_logit, dtype=np.float32)
        for j, s in enumerate(step_samples):
            rl = np.asarray(s['reference_logits'], dtype=np.float32)
            ref_logits[j, : len(rl)] = rl
        ref_logits_t = torch.from_numpy(ref_logits).to(device)
        steps.append((active, tensors, actions, old_lp, adv, target, ref_logits_t))
        total_steps += len(step_samples)
    return batch_size, init_memory, steps, total_steps


def replay_prepared_batch(ac, prepared, temperature):
    """Run TBPTT over a pre-packed batch of windows on-device."""
    from .ppo import categorical_kl
    from torch.distributions import Categorical
    batch_size, memory, steps, total_steps = prepared
    totals = None
    for active, tensors, actions, old_lp, adv, target, ref_logits in steps:
        mem_in = memory[active] if len(active) < batch_size else memory
        dist, value, mem_out = ac.forward_recurrent(*tensors, temperature, mem_in)
        if len(active) < batch_size:
            updated = memory.clone()
            updated[active] = mem_out
            memory = updated
        else:
            memory = mem_out
        ratio = (dist.log_prob(actions) - old_lp).clamp(-20, 20).exp()
        pg = -torch.minimum(ratio * adv, ratio.clamp(0.8, 1.2) * adv).sum()
        vl = F.mse_loss(value, target, reduction='sum')
        ref = Categorical(logits=ref_logits)
        kl = categorical_kl(dist, ref).sum()
        ent = dist.entropy().sum()
        clip_frac = ((ratio - 1).abs() > 0.2).float().sum()
        step_totals = torch.stack([pg, vl, ent, kl, clip_frac])
        totals = step_totals if totals is None else totals + step_totals
    return (totals / total_steps).unbind()


def replay_batch(ac, sequences, temperature, device=None):
    """Batched TBPTT across multiple windows, stepping all active windows together."""
    device = next(ac.parameters()).device if device is None else torch.device(device)
    return replay_prepared_batch(ac, prepare_replay_batch(sequences, device), temperature)
