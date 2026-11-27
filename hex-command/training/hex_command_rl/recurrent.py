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


def prepare_replay_batch(sequences, device, fp16=False):
    """Pack all TBPTT steps in a window batch into one flat CNN batch on-device."""
    from .ppo import pack
    if not sequences:
        raise ValueError("sequences must be nonempty")
    device = torch.device(device)
    if fp16 and device.type == "cpu":
        raise ValueError("fp16 mixed precision is only supported on MPS/CUDA")
    batch_size = len(sequences)
    max_len = max(len(seq) for seq in sequences)
    init_memory = torch.as_tensor(
        np.stack([np.asarray(seq[0]['memory'], np.float32) for seq in sequences]),
        device=device,
    )
    flat_samples = []
    step_slices = []
    for t in range(max_len):
        active = [i for i, seq in enumerate(sequences) if t < len(seq)]
        start = len(flat_samples)
        for i in active:
            flat_samples.append(sequences[i][t])
        step_slices.append((active, start, len(flat_samples)))
    real_n = len(flat_samples)
    pack_obs = [s['observation'] for s in flat_samples]
    if device.type == "mps":
        # Pad batch dim to power-of-2 (min 32) so uneven episode-boundary batches reuse MPSGraph.
        padded_n = 32
        while padded_n < real_n:
            padded_n *= 2
        if padded_n > real_n:
            pack_obs = pack_obs + [pack_obs[0]] * (padded_n - real_n)
    tensors = pack(pack_obs, device=device)
    if fp16:
        x, (board, contexts, owners, sites), pos, mask = tensors
        tensors = (x.half(), (board.half(), contexts.half(), owners, sites), pos, mask)
    actions = torch.tensor([s['action'] for s in flat_samples], dtype=torch.long, device=device)
    old_lp = torch.tensor([s['old_log_prob'] for s in flat_samples], dtype=torch.float32, device=device)
    adv = torch.tensor([s['advantage'] for s in flat_samples], dtype=torch.float32, device=device)
    target = torch.tensor([s['target'] for s in flat_samples], dtype=torch.float32, device=device)
    width = tensors[0].shape[1]
    pad_logit = float(np.finfo(np.float32).min)
    ref_logits = np.full((real_n, width), pad_logit, dtype=np.float32)
    for j, s in enumerate(flat_samples):
        rl = np.asarray(s['reference_logits'], dtype=np.float32)
        ref_logits[j, : len(rl)] = rl
    ref_logits_t = torch.from_numpy(ref_logits).to(device)
    return (
        device.type,
        fp16,
        batch_size,
        real_n,
        init_memory,
        step_slices,
        tensors,
        actions,
        old_lp,
        adv,
        target,
        ref_logits_t,
    )


def replay_prepared_batch(ac, prepared, temperature):
    """Run flattened CNN + sequential GRU TBPTT over a pre-packed batch of windows."""
    from .ppo import categorical, categorical_kl
    from torch.distributions import Categorical
    (
        device_type,
        fp16,
        batch_size,
        real_n,
        memory,
        step_slices,
        tensors,
        actions,
        old_lp,
        adv,
        target,
        ref_logits,
    ) = prepared
    x, boards, positions, mask = tensors
    with torch.amp.autocast(device_type, dtype=torch.float16, enabled=fp16):
        features, parts, pooled = ac.policy.encode_spatial(x, boards, positions)
    pooled_f32 = pooled[:real_n].float()
    mem_steps = []
    for active, start, end in step_slices:
        mem_in = memory[active] if len(active) < batch_size else memory
        mem_out = ac.policy.gru(pooled_f32[start:end], mem_in)
        mem_steps.append(mem_out)
        if len(active) < batch_size:
            updated = memory.clone()
            updated[active] = mem_out
            memory = updated
        else:
            memory = mem_out
    all_mem = torch.cat(mem_steps, dim=0)
    if len(features) > real_n:
        pad_mem = all_mem.new_zeros((len(features) - real_n, all_mem.shape[1]))
        head_mem = torch.cat([all_mem, pad_mem], dim=0)
    else:
        head_mem = all_mem
    with torch.amp.autocast(device_type, dtype=torch.float16, enabled=fp16):
        mem_exp = head_mem.to(features.dtype)[:, None, :].expand(-1, features.shape[1], -1)
        scores = ac.policy.head(torch.cat([features, *parts, mem_exp], dim=-1)).squeeze(-1)
    scores = scores[:real_n].float()
    real_mask = mask[:real_n]
    dist = categorical(scores, real_mask, temperature)
    with torch.no_grad():
        x_f32 = x[:real_n].float()
        mean = (x_f32 * real_mask[:, :, None]).sum(1) / real_mask.sum(1, keepdim=True)
    value = ac.critic(torch.cat([pooled_f32.detach(), all_mem.detach(), mean], -1)).squeeze(-1)
    ratio = (dist.log_prob(actions) - old_lp).clamp(-20, 20).exp()
    pg = -torch.minimum(ratio * adv, ratio.clamp(0.8, 1.2) * adv).mean()
    vl = F.mse_loss(value, target)
    ref = Categorical(logits=ref_logits)
    kl = categorical_kl(dist, ref).mean()
    ent = dist.entropy().mean()
    clip_frac = ((ratio - 1).abs() > 0.2).float().mean()
    return pg, vl, ent, kl, clip_frac


def replay_batch(ac, sequences, temperature, device=None, fp16=False):
    """Batched TBPTT across multiple windows, stepping all active windows together."""
    device = next(ac.parameters()).device if device is None else torch.device(device)
    return replay_prepared_batch(
        ac, prepare_replay_batch(sequences, device, fp16=fp16), temperature
    )
