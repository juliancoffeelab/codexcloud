"""Ordered teacher streams and truncated recurrent imitation, shared by validation."""

import numpy as np
import torch
from torch.nn import functional as F
from .spatial_data import batches


SEQUENCE_SCHEMA = 1


def decision_streams(contexts, count):
    """Partition decisions by match, observing faction and chapter generation.

    A map name alone cannot identify a new match or a repeated chapter.
    Never infer boundaries from changes in terrain, turns or tensor shapes.
    """
    if len(contexts) != count:
        raise ValueError("one sequence context is required per teacher decision")
    streams, previous, closed = {}, {}, set()
    active = {}
    for i, row in enumerate(contexts):
        c = row["context"]
        game = row["game"]
        if (type(game) is not int or game < 0
                or any(type(c[k]) is not int or c[k] < 0
                       for k in ("generation", "round", "decision"))
                or any(not isinstance(c[k], str) or not c[k]
                       for k in ("observer", "map"))):
            raise ValueError("invalid teacher sequence context")
        observer = (game, c["observer"])
        key = (*observer, c["generation"], c["map"])
        if key in closed:
            raise ValueError("teacher stream returned to an earlier chapter")
        old = active.get(observer)
        if old is not None and old != key:
            closed.add(old)
        active[observer] = key
        if key in previous:
            turn, decision = previous[key]
            if c["round"] < turn or c["decision"] <= decision:
                raise ValueError("teacher decisions are not in chronological order")
        previous[key] = c["round"], c["decision"]
        streams.setdefault(key, []).append(i)
    return [np.asarray(ids, dtype=np.int64) for ids in streams.values()]


def forward_window(model, data, streams, start, memory=None, length=8):
    """Advance parallel streams in order; gradients connect inside this window.

    Each stream starts at a real reset boundary. Callers carry detached memory
    to the following window; shorter streams leave the batch without affecting
    other streams. No candidate padding contributes to the loss.
    """
    if not streams or start < 0 or length < 1:
        raise ValueError("invalid imitation window")
    if memory is None:
        if start != 0:
            raise ValueError("continuing a sequence requires its previous memory")
        memory = next(model.parameters()).new_zeros((len(streams), 64))
    if tuple(memory.shape) != (len(streams), 64):
        raise ValueError("imitation memory shape differs from the stream batch")
    losses, correct, count = [], 0, 0
    for step in range(start, min(start + length, max(map(len, streams)))):
        active = [j for j, ids in enumerate(streams) if step < len(ids)]
        ids = np.asarray([streams[j][step] for j in active], dtype=np.int64)
        x, board, ix, mask, labels = next(batches(data, ids, len(ids)))
        owners = torch.tensor(active, dtype=torch.long, device=memory.device)
        scores, _, after = model.forward_recurrent(x, board, ix, memory[owners])
        scores = scores.masked_fill(~mask, -torch.inf)
        losses.append(F.cross_entropy(scores, labels, reduction="sum"))
        correct += int((scores.detach().argmax(-1) == labels).sum())
        count += len(labels)
        memory = memory.index_copy(0, owners, after)
    if not count:
        raise ValueError("imitation window contains no decisions")
    return torch.stack(losses).sum(), correct, count, memory
