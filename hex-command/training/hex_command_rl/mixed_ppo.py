"""Balanced multi-task PPO with independent native slots and atomic recovery."""

import argparse
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import random
import resource
import time
import numpy as np
import torch
from torch.nn import functional as F
import hex_command_training as native
from .mixed_workload import group, mass
from .ppo import ActorCritic, pack, unpack, advantages, categorical, categorical_kl
from .spatial import SpatialPlanner
from .rollout import NativeRollout
from .recovery import save_bundle, atomic_json, load_resume, read_jsonl


def terminal_reward(config, reward, truncated):
    return -1.0 if truncated and "campaign" in config else float(reward)


class Sampler:
    """Track decision budgets, including work reserved for live episodes."""

    def __init__(self, configs, rng, state=None):
        self.rng = rng
        self.templates = defaultdict(list)
        self.weights = {}
        seen = set()
        for c in configs:
            k = json.dumps(group(c))
            base = dict(c)
            base.pop("seed")
            base["maxRounds"] = 100
            identity = json.dumps(base, sort_keys=True)
            if identity not in seen:
                self.templates[k].append(base)
                seen.add(identity)
            self.weights[k] = mass(c)
        self.counts = {k: 0 for k in self.templates}
        self.average = {
            k: 90.0 if json.loads(k)[0] == "arena" else 300.0 for k in self.templates
        }
        self.cursor = {k: 0 for k in self.templates}
        self.next_seed = 7000001
        self.pending = {}
        if state:
            self.counts = state["counts"]
            self.average = state["average"]
            self.cursor = state["cursor"]
            self.next_seed = state["next_seed"]

    def choose(self, slot):
        loads = {k: float(v) for k, v in self.counts.items()}
        for k, n in self.pending.values():
            loads[k] += n
        family = {
            f: sum(v for k, v in loads.items() if json.loads(k)[0] == f)
            for f in ["arena", "campaign"]
        }
        f = min(family, key=lambda x: family[x])
        keys = [k for k in loads if json.loads(k)[0] == f]
        # Within a family, retain equal task budgets and split variants inside them.
        total = sum(loads[k] for k in keys)
        k = max(
            keys,
            key=lambda k: (
                self.weights[k] / 0.5 * (total + 200)
                - loads[k]
                + float(self.rng.uniform(0, 1e-5))
            ),
        )
        templates = self.templates[k]
        i = self.cursor[k] % len(templates)
        self.cursor[k] += 1
        c = dict(templates[i], seed=self.next_seed)
        self.next_seed += 1
        self.pending[slot] = (k, self.average[k])
        return c, k

    def step(self, slot):
        k, n = self.pending[slot]
        self.counts[k] += 1
        self.pending[slot] = (k, max(0.0, n - 1))

    def finish(self, slot, length):
        k, _ = self.pending.pop(slot)
        self.average[k] = 0.9 * self.average[k] + 0.1 * length

    def state(self):
        return dict(
            counts=self.counts,
            average=self.average,
            cursor=self.cursor,
            next_seed=self.next_seed,
        )


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, required=True)
    p.add_argument("--seconds", type=float, default=21600)
    p.add_argument("--warm-start", type=Path)
    p.add_argument("--resume", type=Path)
    p.add_argument("--envs", type=int, default=4)
    p.add_argument("--critic-warmup-episodes", type=int, default=24)
    p.add_argument("--horizon", type=int, default=64)
    p.add_argument("--temperature", type=float)
    p.add_argument("--rollout-backend", choices=["torch", "native"])
    a = p.parse_args()
    if (
        not 0 < a.seconds <= 43200
        or not 1 <= a.envs <= 4
        or a.temperature is not None
        and not 0.05 <= a.temperature <= 2
        or a.horizon < 1
    ):
        p.error("invalid PPO budget")
    root = a.root
    run = root / "rl"
    run.mkdir(exist_ok=True)
    spec = json.loads((root / "workload.json").read_text())
    os.environ["HEX_COMMAND_DATA_ROOT"] = spec["catalog"]
    assert json.loads(native.fingerprints()) == spec["fingerprints"]
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)
    torch.manual_seed(271)
    random.seed(271)
    rng = np.random.default_rng(271)
    policy = SpatialPlanner().eval()
    old = load_resume(a.resume) if a.resume else None
    saved_backend = old.get("rollout_backend", "torch") if old else "torch"
    if old and a.rollout_backend is not None and a.rollout_backend != saved_backend:
        p.error("resume rollout backend must match; use a policy warm start to switch")
    a.rollout_backend = a.rollout_backend or saved_backend
    if old and a.temperature is not None and a.temperature != old["temperature"]:
        p.error("resume temperature must match the saved policy")
    a.temperature = (
        old["temperature"] if old else 0.35 if a.temperature is None else a.temperature
    )
    if old:
        assert (
            old["kind"] == "mixed_ppo"
            and old["workload_sha256"] == spec["workload_sha256"]
            and old["fingerprints"] == spec["fingerprints"]
        )
        policy.load_state_dict(old["policy"])
    else:
        warm = load_resume(a.warm_start or root / "posttune")
        policy.load_state_dict(warm["policy"])
    ac = ActorCritic(policy).eval()
    reference = SpatialPlanner().eval().requires_grad_(False)
    reference.load_state_dict(old["reference"] if old else policy.state_dict())
    actor_opt = torch.optim.Adam(policy.parameters(), lr=5e-6)
    critic_opt = torch.optim.Adam(ac.critic.parameters(), lr=1e-3)
    if old:
        ac.critic.load_state_dict(old["critic"])
        actor_opt.load_state_dict(old["actor_optimizer"])
        critic_opt.load_state_dict(old["critic_optimizer"])
        torch.set_rng_state(old["torch_rng"])
        rng.bit_generator.state = old["numpy_rng"]
        random.setstate(old["python_rng"])
    rollout = NativeRollout(ac) if a.rollout_backend == "native" else None
    sampler = Sampler(spec["train"], rng, old["sampler"] if old else None)
    update = old["update"] if old else 0
    episodes = old["episodes"] if old else 0
    decisions = old["decisions"] if old else 0
    elapsed = old["elapsed"] if old else 0.0
    start = time.monotonic()
    deadline = start + max(0, a.seconds - elapsed)
    read_jsonl(run / "metrics.jsonl", repair=True)
    log = (run / "metrics.jsonl").open("a")
    action_counts = defaultdict(int, old.get("action_counts", {}) if old else {})

    def record(event, **kw):
        row = dict(event=event, seconds=elapsed + time.monotonic() - start, **kw)
        log.write(json.dumps(row) + "\n")
        log.flush()
        if event != "episode" or episodes % 10 == 0:
            print(json.dumps(row), flush=True)
        atomic_json(
            run / "status.json",
            dict(
                event=event,
                seconds=row["seconds"],
                updates=update,
                episodes=episodes,
                decisions=decisions,
                rss_mib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024,
            ),
        )

    def checkpoint(label):
        state = dict(
            kind="mixed_ppo",
            policy=policy.state_dict(),
            critic=ac.critic.state_dict(),
            reference=reference.state_dict(),
            actor_optimizer=actor_opt.state_dict(),
            critic_optimizer=critic_opt.state_dict(),
            torch_rng=torch.get_rng_state(),
            numpy_rng=rng.bit_generator.state,
            python_rng=random.getstate(),
            sampler=sampler.state(),
            update=update,
            episodes=episodes,
            decisions=decisions,
            elapsed=elapsed + time.monotonic() - start,
            fingerprints=spec["fingerprints"],
            workload_sha256=spec["workload_sha256"],
            temperature=a.temperature,
            rollout_backend=a.rollout_backend,
            action_counts=dict(action_counts),
        )
        save_bundle(run, state, policy, label)

    atomic_json(
        run / "parameters.json",
        dict(
            seconds=a.seconds,
            temperature=a.temperature,
            envs=a.envs,
            horizon=a.horizon,
            rollout_backend=a.rollout_backend,
            actor_lr=5e-6,
            critic_lr=1e-3,
            gamma=0.999,
            gae_lambda=0.95,
            clip=0.2,
            entropy_coefficient=0.001,
            reference_kl_coefficient=0.02,
            critic_warmup_episodes=a.critic_warmup_episodes,
            rewards=dict(
                campaign_success=1,
                campaign_defeat=-1,
                campaign_cap=-1,
                arena_win=1,
                arena_defeat=-1,
                arena_cap=0,
            ),
        ),
    )
    checkpoint("initial")
    record("started", resumed=bool(old))
    envs = [native.BatchEnv(spatial=True) for _ in range(a.envs)]
    cases = {}
    keys = {}
    lengths = {i: 0 for i in range(a.envs)}
    observations = {}

    def reset(slot):
        c, k = sampler.choose(slot)
        cases[slot] = c
        keys[slot] = k
        lengths[slot] = 0
        return c

    try:
        with ThreadPoolExecutor(max_workers=a.envs) as pool:
            initial = {
                i: pool.submit(envs[i].reset, [json.dumps(reset(i))])
                for i in range(a.envs)
            }
            observations = {i: f.result() for i, f in initial.items()}
            last_save = time.monotonic()
            while time.monotonic() < deadline:
                sync_start = time.monotonic()
                if rollout is not None:
                    rollout.sync()
                rollout_sync_seconds = time.monotonic() - sync_start
                samples = []
                rew = []
                vals = []
                done = []
                collection_start = time.monotonic()
                for t in range(a.horizon):
                    while (
                        run / "evaluation-active.json"
                    ).exists() and time.monotonic() < deadline:
                        time.sleep(0.2)
                    if time.monotonic() >= deadline:
                        break
                    views = {
                        i: unpack(obs, envs[i].spatial_observation())[0]
                        for i, obs in observations.items()
                    }
                    ids = list(views)
                    with torch.no_grad():
                        dist, v = (
                            rollout.forward(list(views.values()), a.temperature)
                            if rollout is not None
                            else ac(*pack(list(views.values())), a.temperature)
                        )
                        acts = dist.sample()
                        old_lp = dist.log_prob(acts)
                    futures = {
                        i: pool.submit(envs[i].step, [int(acts[j])])
                        for j, i in enumerate(ids)
                    }
                    rewards = np.zeros(a.envs, np.float32)
                    values = np.zeros(a.envs, np.float32)
                    dones = np.zeros(a.envs, np.float32)
                    step = []
                    resets = {}
                    for j, i in enumerate(ids):
                        nxt = futures[i].result()
                        terminal = nxt[3][0] or nxt[4][0]
                        r = float(nxt[2][0])
                        c = cases[i]
                        r = terminal_reward(c, r, nxt[4][0])
                        rewards[i] = r
                        values[i] = float(v[j])
                        dones[i] = terminal
                        sampler.step(i)
                        lengths[i] += 1
                        decisions += 1
                        action_counts[
                            ["end_turn", "move", "attack", "advance"][
                                int(views[i][0][int(acts[j]), :4].argmax())
                            ]
                        ] += 1
                        step.append(
                            dict(
                                slot=i,
                                observation=views[i],
                                action=int(acts[j]),
                                old_log_prob=float(old_lp[j]),
                            )
                        )
                        if terminal:
                            episodes += 1
                            record(
                                "episode",
                                episode=episodes,
                                config=c,
                                group=json.loads(keys[i]),
                                reward=r,
                                success=r > 0,
                                truncated=bool(nxt[4][0]),
                                length=lengths[i],
                            )
                            sampler.finish(i, lengths[i])
                            resets[i] = pool.submit(
                                envs[i].reset, [json.dumps(reset(i))]
                            )
                        else:
                            observations[i] = nxt
                    for i, f in resets.items():
                        observations[i] = f.result()
                    samples.append(step)
                    rew.append(rewards)
                    vals.append(values)
                    done.append(dones)
                if not samples:
                    break
                views = {
                    i: unpack(obs, envs[i].spatial_observation())[0]
                    for i, obs in observations.items()
                }
                bootstrap = np.zeros(a.envs, np.float32)
                with torch.no_grad():
                    _, next_v = (
                        rollout.forward(list(views.values()), a.temperature)
                        if rollout is not None
                        else ac(*pack(list(views.values())), a.temperature)
                    )
                for j, i in enumerate(views):
                    bootstrap[i] = float(next_v[j])
                collection_seconds = time.monotonic() - collection_start
                optimization_start = time.monotonic()
                advantage, returns = advantages(
                    np.asarray(rew),
                    np.asarray(vals),
                    np.asarray(done),
                    bootstrap,
                    0.999,
                    0.95,
                )
                flat = [
                    dict(
                        s,
                        advantage=float(advantage[t, s["slot"]]),
                        target=float(returns[t, s["slot"]]),
                    )
                    for t, ss in enumerate(samples)
                    for s in ss
                ]
                aa = np.asarray([s["advantage"] for s in flat], np.float32)
                aa = (aa - aa.mean()) / max(float(aa.std()), 1e-6)
                for s, value in zip(flat, aa):
                    s["advantage"] = float(value)
                warmup = episodes < a.critic_warmup_episodes or update < 4
                update += 1
                stats = []
                for epoch in range(3):
                    order = rng.permutation(len(flat))
                    for first in range(0, len(flat), 64):
                        if time.monotonic() >= deadline:
                            break
                        batch = [flat[i] for i in order[first : first + 64]]
                        tensors = pack([s["observation"] for s in batch])
                        dist, value = ac(*tensors, a.temperature)
                        acts = torch.tensor([s["action"] for s in batch])
                        old_lp = torch.tensor([s["old_log_prob"] for s in batch])
                        adv = torch.tensor([s["advantage"] for s in batch])
                        target = torch.tensor([s["target"] for s in batch])
                        logp = dist.log_prob(acts)
                        ratio = (logp - old_lp).clamp(-20, 20).exp()
                        policy_loss = -torch.minimum(
                            ratio * adv, ratio.clamp(0.8, 1.2) * adv
                        ).mean()
                        value_loss = F.mse_loss(value, target)
                        with torch.no_grad():
                            ref = categorical(
                                reference(*tensors[:3]), tensors[3], a.temperature
                            )
                        kl = categorical_kl(dist, ref).mean()
                        entropy = dist.entropy().mean()
                        loss = value_loss + (
                            0 if warmup else policy_loss + 0.02 * kl - 0.001 * entropy
                        )
                        if not torch.isfinite(loss):
                            raise RuntimeError("nonfinite PPO objective")
                        actor_opt.zero_grad()
                        critic_opt.zero_grad()
                        loss.backward()
                        norm = torch.nn.utils.clip_grad_norm_(
                            ac.parameters(), 0.5, error_if_nonfinite=True
                        )
                        if not warmup:
                            actor_opt.step()
                        critic_opt.step()
                        stats.append(
                            [
                                float(z.detach())
                                for z in [
                                    policy_loss,
                                    value_loss,
                                    entropy,
                                    kl,
                                    ((ratio - 1).abs() > 0.2).float().mean(),
                                    norm,
                                ]
                            ]
                        )
                    if time.monotonic() >= deadline:
                        break
                if stats:
                    m = np.mean(stats, 0)
                    target_np = returns.flatten()
                    var = float(np.var(target_np))
                    ev = (
                        1 - float(np.var(returns - np.asarray(vals))) / var
                        if var > 1e-8
                        else 0.0
                    )
                    family = {
                        f: sum(
                            n
                            for k, n in sampler.counts.items()
                            if json.loads(k)[0] == f
                        )
                        for f in ["arena", "campaign"]
                    }
                    record(
                        "update",
                        update=update,
                        decisions=decisions,
                        episodes=episodes,
                        rollout_sync_seconds=rollout_sync_seconds,
                        collection_seconds=collection_seconds,
                        optimization_seconds=time.monotonic() - optimization_start,
                        policy_loss=m[0],
                        value_loss=m[1],
                        entropy=m[2],
                        reference_kl=m[3],
                        clip_fraction=m[4],
                        gradient_norm=m[5],
                        critic_explained_variance=ev,
                        warmup=warmup,
                        family_decisions=family,
                        action_counts=dict(action_counts),
                    )
                if time.monotonic() - last_save >= 120:
                    checkpoint(f"update-{update:08d}")
                    last_save = time.monotonic()
    finally:
        checkpoint("last")
        for env in envs:
            env.close()
        record("finished", updates=update, episodes=episodes, decisions=decisions)
        log.close()
    atomic_json(
        run / "complete.json",
        dict(
            elapsed=elapsed + time.monotonic() - start,
            updates=update,
            episodes=episodes,
            decisions=decisions,
        ),
    )


if __name__ == "__main__":
    main()
