"""Balanced multi-task PPO with independent native slots and atomic recovery."""

import argparse
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
import json
import math
import os
from pathlib import Path
import random
import resource
import time
import numpy as np
import torch
from torch.nn import functional as F
import hex_command_training as native
from .sampling import Sampler
from .ppo import ActorCritic, pack, unpack, advantages, categorical, resolve_device
from .spatial import SpatialPlanner
from .recurrent import (
    MemoryBank,
    windows,
    replay,
    group_windows,
    prepare_replay_batch,
    replay_prepared_batch,
)
from .recovery import save_bundle, atomic_json, load_resume, read_jsonl
from .mixed_workload import reward_rules, sampling_group, task, definition


def terminal_reward(config, reward, truncated, result=None, campaign_cap=-2.0):
    if "campaign" not in config:
        return -1.0 if truncated else float(reward)
    if truncated:
        return campaign_cap
    if reward < 0 and result:
        return {"escort_loss": -2.0, "protected_loss": -1.5, "leader_loss": -1.0}.get(
            result.get("failure"), -1.0
        )
    return float(reward)


def step_reward(config, reward, truncated, result, chapter_clears, campaign_cap):
    reward = terminal_reward(config, reward, truncated, result, campaign_cap)
    rules = reward_rules(config)
    if result and result["success"] and not truncated:
        reward = rules["win"]
    reward += rules["chapter"] * chapter_clears
    return reward


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, required=True)
    p.add_argument("--seconds", type=float, default=21600)
    p.add_argument("--warm-start", type=Path)
    p.add_argument("--resume", type=Path)
    p.add_argument(
        "--seed-start",
        type=int,
        help="fresh rollout seeds for a warm start (at least 7000001)",
    )
    p.add_argument("--envs", type=int, default=4)
    p.add_argument("--device", choices=["auto", "cpu", "mps", "cuda"], default="cpu")
    p.add_argument("--precision", choices=["auto", "fp32", "fp16"], default="auto")
    p.add_argument("--window-batch", type=int, default=None)
    p.add_argument("--actor-lr", type=float, default=None)
    p.add_argument("--checkpoint-seconds", type=float, default=120.0)
    p.add_argument("--critic-warmup-episodes", type=int, default=24)
    p.add_argument("--horizon", type=int, default=64)
    p.add_argument("--temperature", type=float)
    p.add_argument(
        "--campaign-cap", type=float, choices=[-1.0, -1.5, -2.0], default=-2.0
    )
    a = p.parse_args()
    if (
        not 0 < a.seconds <= 43200
        or not 1 <= a.envs <= 16
        or a.window_batch is not None
        and a.window_batch < 1
        or a.temperature is not None
        and not 0.05 <= a.temperature <= 2
        or a.horizon < 1
        or a.checkpoint_seconds <= 0
    ):
        p.error("invalid PPO budget")
    try:
        device = resolve_device(a.device)
    except ValueError as err:
        p.error(str(err))
    window_batch = a.window_batch or (1 if device.type == "cpu" else 16)
    actor_lr = (
        a.actor_lr
        if a.actor_lr is not None
        else 5e-6 * math.sqrt(window_batch)
    )
    if not actor_lr > 0:
        p.error("--actor-lr must be positive")
    use_fp16 = (
        device.type != "cpu"
        if a.precision == "auto"
        else a.precision == "fp16"
    )
    if use_fp16 and device.type == "cpu":
        p.error("--precision fp16 requires --device mps or cuda")
    root = a.root
    run = root / "rl"
    run.mkdir(exist_ok=True)
    spec = json.loads((root / "workload.json").read_text())
    os.environ["HEX_COMMAND_DATA_ROOT"] = spec["catalog"]
    assert json.loads(native.fingerprints()) == spec["fingerprints"]
    training_spec = definition()
    profiles = training_spec["rewardProfiles"]
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)
    torch.manual_seed(271)
    random.seed(271)
    rng = np.random.default_rng(271)
    policy = SpatialPlanner().eval()
    old = load_resume(a.resume) if a.resume else None
    if a.seed_start is not None and (old or not 7000001 <= a.seed_start <= 4000000000):
        p.error(
            "seed-start is only for warm starts and must avoid held-out seed ranges"
        )
    seed_start = old.get("seed_start", 7000001) if old else a.seed_start or 7000001
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
            and old.get("campaign_cap", -1.0) == a.campaign_cap
        )
        policy.load_state_dict(old["policy"])
    else:
        warm = load_resume(a.warm_start or root / "posttune")
        from .checkpoint import expand_state

        expand_state(policy, warm["policy"])
    policy = policy.to(device)
    ac = ActorCritic(policy).eval().to(device)
    reference = SpatialPlanner().eval().requires_grad_(False)
    reference.load_state_dict(old["reference"] if old else policy.state_dict())
    reference = reference.to(device)
    actor_opt = torch.optim.Adam(policy.parameters(), lr=actor_lr)
    critic_opt = torch.optim.Adam(ac.critic.parameters(), lr=1e-3)
    if old:
        ac.critic.load_state_dict(old["critic"])
        actor_opt.load_state_dict(old["actor_optimizer"])
        critic_opt.load_state_dict(old["critic_optimizer"])
        torch.set_rng_state(old["torch_rng"])
        rng.bit_generator.state = old["numpy_rng"]
        random.setstate(old["python_rng"])
    sampler = Sampler(spec["train"], rng, old["sampler"] if old else None)
    if not old:
        sampler.next_seed = seed_start
    atomic_json(run / "sampler-status.json", sampler.report())
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
            campaign_cap=a.campaign_cap,
            seed_start=seed_start,
            action_counts=dict(action_counts),
        )
        save_bundle(run, state, policy, label)

    atomic_json(
        run / "parameters.json",
        dict(
            seconds=a.seconds,
            seed_start=seed_start,
            temperature=a.temperature,
            envs=a.envs,
            device=str(device),
            precision="fp16" if use_fp16 else "fp32",
            window_batch=window_batch,
            horizon=a.horizon,
            minibatch_size=8,
            recurrent_window=8,
            reference_outputs_cached=True,
            optimization_epochs=3,
            max_gradient_norm=0.5,
            actor_lr=actor_lr,
            checkpoint_seconds=a.checkpoint_seconds,
            critic_lr=1e-3,
            gamma=0.999,
            gae_lambda=0.95,
            clip=0.2,
            entropy_coefficient=0.001,
            reference_kl_coefficient=0.02,
            critic_warmup_episodes=a.critic_warmup_episodes,
            rewards=dict(
                profiles=profiles,
                campaign_chapter=profiles["campaign"]["chapter"],
                campaign_finale=profiles["campaign"]["win"],
                campaign_defeat=-1,
                campaign_cap=a.campaign_cap,
                escort_loss=-2,
                protected_loss=-1.5,
                leader_loss=-1,
                arena_win=profiles["arena"]["win"],
                arena_defeat=-1,
                arena_cap=-1,
            ),
        ),
    )
    checkpoint("initial")
    record("started", resumed=bool(old))
    envs = [native.BatchEnv(spatial=True) for _ in range(a.envs)]
    cases = {}
    keys = {}
    lengths = {i: 0 for i in range(a.envs)}
    chapters_cleared = {}
    episode_returns = {}
    observations = {}

    memory_bank, reference_bank = MemoryBank(), MemoryBank()

    def reset(slot):
        memory_bank.reset(slot)
        reference_bank.reset(slot)
        c, k = sampler.choose(slot)
        cases[slot] = c
        keys[slot] = k
        lengths[slot] = 0
        chapters_cleared[slot] = 0
        episode_returns[slot] = 0.0
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
                samples = []
                rew = []
                vals = []
                done = []
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
                    tensors = pack(list(views.values()), device=device)
                    contexts = [envs[i].decision_contexts()[0] for i in ids]
                    memory, memory_resets = memory_bank.inputs(ids, contexts, device=device)
                    ref_memory, _ = reference_bank.inputs(ids, contexts, device=device)
                    with torch.no_grad(), torch.amp.autocast(
                        device.type, dtype=torch.float16, enabled=use_fp16
                    ):
                        dist, v, next_memory = ac.forward_recurrent(*tensors, a.temperature, memory)
                        acts = dist.sample()
                        old_lp = dist.log_prob(acts)
                    acts_cpu = acts.cpu().tolist()
                    # Submit Rust env steps before running reference so CPU stepping overlaps GPU inference.
                    futures = {
                        i: pool.submit(envs[i].step, [int(acts_cpu[j])])
                        for j, i in enumerate(ids)
                    }
                    with torch.no_grad(), torch.amp.autocast(
                        device.type, dtype=torch.float16, enabled=use_fp16
                    ):
                        ref_scores, _, next_ref = reference.forward_recurrent(*tensors[:3], ref_memory)
                        ref_dist = categorical(ref_scores, tensors[3], a.temperature)
                    memory_bank.commit(ids, contexts, memory, next_memory)
                    reference_bank.commit(ids, contexts, ref_memory, next_ref)
                    vals_cpu = v.cpu().tolist()
                    old_lp_cpu = old_lp.cpu().tolist()
                    mem_cpu = memory.detach().cpu().numpy()
                    ref_logits_cpu = ref_dist.logits.detach().cpu().numpy()
                    rewards = np.zeros(a.envs, np.float32)
                    values = np.zeros(a.envs, np.float32)
                    dones = np.zeros(a.envs, np.float32)
                    step = []
                    resets = {}
                    for j, i in enumerate(ids):
                        nxt = futures[i].result()
                        terminal = nxt[3][0] or nxt[4][0]
                        c = cases[i]
                        result = json.loads(envs[i].results()[0]) if terminal else None
                        cleared = envs[i].chapters_cleared()[0]
                        r = step_reward(
                            c,
                            nxt[2][0],
                            nxt[4][0],
                            result,
                            cleared - chapters_cleared[i],
                            a.campaign_cap,
                        )
                        chapters_cleared[i] = cleared
                        episode_returns[i] += r
                        rewards[i] = r
                        values[i] = float(vals_cpu[j])
                        dones[i] = terminal
                        sampler.step(i)
                        lengths[i] += 1
                        decisions += 1
                        action_counts[
                            ["end_turn", "move", "attack", "advance"][
                                int(views[i][0][int(acts_cpu[j]), :4].argmax())
                            ]
                        ] += 1
                        step.append(
                            dict(
                                slot=i,
                                context=contexts[j], memory=mem_cpu[j].copy(),
                                memory_reset=memory_resets[j],
                                reference_logits=ref_logits_cpu[j, :len(views[i][0])].copy(),
                                observation=views[i],
                                action=int(acts_cpu[j]),
                                old_log_prob=float(old_lp_cpu[j]),
                            )
                        )
                        if terminal:
                            episodes += 1
                            record(
                                "episode",
                                episode=episodes,
                                config=c,
                                group=json.loads(keys[i]),
                                sampling_group=sampling_group(c, training_spec),
                                sampling_map=task(c, training_spec),
                                reward_profile=reward_rules(c, training_spec),
                                reward=r,
                                episode_return=episode_returns[i],
                                success=result["success"],
                                truncated=bool(nxt[4][0]),
                                result=result,
                                length=lengths[i],
                            )
                            sampling = sampler.finish(
                                i, lengths[i], success=result["success"],
                                truncated=bool(nxt[4][0]),
                            )
                            if sampling is not None:
                                record("sampling", **sampling)
                                atomic_json(run / "sampler-status.json", sampler.report())
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
                with torch.no_grad(), torch.amp.autocast(
                    device.type, dtype=torch.float16, enabled=use_fp16
                ):
                    ids = list(views)
                    contexts = [envs[i].decision_contexts()[0] for i in ids]
                    memory, _ = memory_bank.inputs(ids, contexts, device=device)
                    _, next_v, _ = ac.forward_recurrent(
                        *pack(list(views.values()), device=device), a.temperature, memory
                    )
                next_v_cpu = next_v.cpu().tolist()
                for j, i in enumerate(views):
                    bootstrap[i] = float(next_v_cpu[j])
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
                prepared_batches = (
                    [
                        prepare_replay_batch(batch, device, fp16=use_fp16)
                        for batch in group_windows(windows(flat, rng), window_batch, rng)
                    ]
                    if window_batch > 1
                    else None
                )
                for epoch in range(3):
                    batches = (
                        windows(flat, rng)
                        if window_batch == 1
                        else [
                            prepared_batches[i]
                            for i in rng.permutation(len(prepared_batches))
                        ]
                    )
                    for batch in batches:
                        if time.monotonic() >= deadline:
                            break
                        policy_loss, value_loss, entropy, kl, clip_fraction = (
                            replay(ac, batch, a.temperature, device=device)
                            if window_batch == 1
                            else replay_prepared_batch(ac, batch, a.temperature)
                        )
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
                                    clip_fraction,
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
                if time.monotonic() - last_save >= a.checkpoint_seconds:
                    checkpoint(f"update-{update:08d}")
                    last_save = time.monotonic()
    finally:
        atomic_json(run / "sampler-status.json", sampler.report())
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
