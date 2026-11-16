"""Allocate fixed means quotas to row capacities without within-row repeats."""
import math
from collections import Counter


def assign_means(weights, row_counts, rng):
    n = len(row_counts)
    if not n or any(k < 0 or k > len(weights) for k in row_counts):
        raise ValueError('Each row must fit within the available distinct kinds')
    slots = sum(row_counts)
    expected = {kind: slots * weight / sum(weights.values()) for kind, weight in weights.items()}
    quotas = {kind: min(n, int(rng.random() < amount) if amount < 1 else math.floor(amount + .5))
              for kind, amount in expected.items()}
    priority = list(weights)
    rng.shuffle(priority)
    # Reconcile rounding against the actual slot budget, not an oversized deck.
    while sum(quotas.values()) < slots:
        kind = max((k for k in priority if quotas[k] < n), key=lambda k: expected[k] - quotas[k])
        quotas[kind] += 1
    while sum(quotas.values()) > slots:
        kind = max((k for k in priority if quotas[k]), key=lambda k: quotas[k] - expected[k])
        quotas[kind] -= 1

    def feasible():
        rows = sorted(row_counts, reverse=True)
        return all(sum(rows[:k]) <= sum(min(k, q) for q in quotas.values())
                   for k in range(1, n + 1))

    # Rare small batches may concentrate quotas too much for distinct assignment.
    repairs = []
    while not feasible():
        donor = max(priority, key=lambda k: quotas[k])
        recipient = min(priority, key=lambda k: quotas[k])
        if quotas[donor] - quotas[recipient] <= 1:
            raise ValueError('Cannot realize the requested row capacities')
        quotas[donor] -= 1
        quotas[recipient] += 1
        repairs.append({'from': donor, 'to': recipient})

    remaining = quotas.copy()
    order = list(range(n))
    rng.shuffle(order)
    order.sort(key=lambda i: -row_counts[i])
    selected = [[] for _ in row_counts]
    for i in order:
        kinds = list(weights)
        rng.shuffle(kinds)
        kinds.sort(key=lambda k: -remaining[k])
        for kind in kinds[:row_counts[i]]:
            if remaining[kind] <= 0:
                raise ValueError('Distinct assignment exhausted a quota')
            selected[i].append(kind)
            remaining[kind] -= 1
        rng.shuffle(selected[i])
    assert not any(remaining.values())
    assert Counter(k for row in selected for k in row) == Counter(quotas)
    return selected, {'slot_budget': slots, 'reference_counts': dict(weights),
                      'expected_counts': expected, 'assigned_counts': quotas,
                      'feasibility_adjustments': repairs}
