"""Bounded native campaign PPO smoke runs from a preserved spatial policy."""

import argparse
import copy
import hashlib
import json
from pathlib import Path
import random
import time

import numpy as np
import torch
from torch import nn
from torch.nn import functional as F
import hex_command_training as native
from .spatial import SpatialPlanner
from .checkpoint import warm_start
from .recovery import atomic_torch_save, read_jsonl


def save_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def unpack(obs, spatial):
    features, offsets = obs[:2]
    boards, bounds, shapes, indices, values, metadata = spatial
    return {
        i: (
            features[offsets[i] : offsets[i + 1]].copy(),
            boards[bounds[i] : bounds[i + 1]]
            .reshape(len(native.SPATIAL_CHANNEL_NAMES), *shapes[i])
            .copy(),
            indices[offsets[i] : offsets[i + 1]].copy(),
            [
                (
                    int(site),
                    values[
                        offset : offset
                        + len(native.ACTOR_SPATIAL_CHANNELS) * int(np.prod(shapes[i]))
                    ]
                    .reshape(len(native.ACTOR_SPATIAL_CHANNELS), *shapes[i])
                    .copy(),
                )
                for owner, site, offset in metadata
                if owner == i
            ],
        )
        for i in range(len(shapes))
        if offsets[i + 1] > offsets[i]
    }


def pack(samples):
    count = max(len(s[0]) for s in samples)
    height = max(s[1].shape[1] for s in samples)
    width = max(s[1].shape[2] for s in samples)
    x = np.zeros((len(samples), count, len(native.FEATURE_NAMES)), np.float32)
    board = np.zeros(
        (len(samples), len(native.SPATIAL_CHANNEL_NAMES), height, width), np.float32
    )
    positions = np.full((len(samples), count, 3), -1, np.int64)
    mask = np.zeros((len(samples), count), bool)
    context_values, owners, sites = [], [], []
    for row, sample in enumerate(samples):
        features, b, ix = sample[:3]
        contexts = sample[3] if len(sample) > 3 else []
        n = len(features)
        h, w = b.shape[1:]
        x[row, :n, : features.shape[1]] = features
        board[row, : b.shape[0], :h, :w] = b
        positions[row, :n] = np.where(ix < 0, -1, (ix // w) * width + ix % w)
        mask[row, :n] = True
        # Context zero is the actor-free board for EndTurn and critic pooling.
        for site, planes in [(-1, None), *contexts]:
            padded = np.zeros(
                (len(native.ACTOR_SPATIAL_CHANNELS), height, width), np.float32
            )
            if planes is not None:
                padded[:, :h, :w] = planes
            context_values.append(padded)
            owners.append(row)
            sites.append(-1 if site < 0 else (site // w) * width + site % w)
    bundle = tuple(
        torch.from_numpy(v)
        for v in (
            board,
            np.stack(context_values),
            np.asarray(owners, np.int64),
            np.asarray(sites, np.int64),
        )
    )
    return (
        torch.from_numpy(x),
        bundle,
        torch.from_numpy(positions),
        torch.from_numpy(mask),
    )


def categorical(scores, mask, temperature):
    # Finite padding avoids NaN gradients in KL's 0 * (-inf - -inf).
    logits = (scores / temperature).masked_fill(~mask, torch.finfo(scores.dtype).min)
    return torch.distributions.Categorical(logits=logits)


def categorical_kl(policy, reference):
    # Both distributions have identical legal actions and finite log probabilities.
    # PyTorch's generic categorical KL treats underflowed reference probabilities
    # as absent support, producing infinity even though their logits are finite.
    return (policy.probs * (policy.logits - reference.logits)).sum(-1)


class ActorCritic(nn.Module):
    def __init__(self, policy):
        super().__init__()
        self.policy = policy
        self.critic = nn.Sequential(
            nn.Linear(16 + len(native.FEATURE_NAMES), 64), nn.Tanh(), nn.Linear(64, 1)
        )
        nn.init.zeros_(self.critic[-1].weight)
        nn.init.zeros_(self.critic[-1].bias)

    def forward(self, x, board, positions, mask, temperature):
        scores, pooled = self.policy.forward_with_pool(x, board, positions)
        dist = categorical(scores, mask, temperature)
        # Critic training initially leaves the pretrained policy encoder untouched.
        with torch.no_grad():
            mean = (x * mask[:, :, None]).sum(1) / mask.sum(1, keepdim=True)
        value = self.critic(torch.cat([pooled.detach(), mean], -1)).squeeze(-1)
        return dist, value


def advantages(rewards, values, dones, bootstrap, gamma, lam):
    result = np.zeros_like(values, dtype=np.float32)
    carry = np.zeros(values.shape[1], np.float32)
    next_value = np.asarray(bootstrap, np.float32)
    for t in reversed(range(len(values))):
        live = 1.0 - dones[t]
        delta = rewards[t] + gamma * next_value * live - values[t]
        carry = delta + gamma * lam * live * carry
        result[t] = carry
        next_value = values[t]
    return result, result + values


def configuration(seed, recruiter, args):
    return dict(
        seed=seed,
        campaign=args.campaign,
        diplomacy="unrestricted",
        objective="authored",
        player="balanced",
        enemy=args.opponent,
        maxRounds=args.rounds,
        recruitLimit=None,
        playerBlocks=dict(recruiter=recruiter, campaign="mission", planner="queued"),
    )


def plot_metrics(directory):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    rows = read_jsonl(directory / "metrics.jsonl")
    updates = [r for r in rows if r["event"] == "update"]
    evaluations = [r for r in rows if r["event"] == "evaluation"]
    episodes = [r for r in rows if r["event"] == "episode"]
    fig, axes = plt.subplots(2, 3, figsize=(14, 7))
    for a in axes.flat:
        a.grid(alpha=0.2)
        a.set_xlabel("Minutes")
    x = [r["seconds"] / 60 for r in updates]
    for key in ["policy_loss", "value_loss"]:
        axes[0, 0].plot(x, [r[key] for r in updates], label=key)
    axes[0, 0].set_title("Optimization losses")
    axes[0, 0].legend()
    axes[0, 1].plot(x, [r["entropy"] for r in updates])
    axes[0, 1].set_title("Policy entropy")
    axes[0, 2].plot(x, [r["reference_kl"] for r in updates])
    axes[0, 2].set_title("KL from starting policy")
    if episodes:
        ex = [r["seconds"] / 60 for r in episodes]
        axes[1, 0].plot(
            ex,
            [
                np.mean([v["reward"] for v in episodes[max(0, i - 19) : i + 1]])
                for i in range(len(episodes))
            ],
        )
        axes[1, 1].plot(
            ex,
            [
                np.mean([v["success"] for v in episodes[max(0, i - 19) : i + 1]]) * 100
                for i in range(len(episodes))
            ],
        )
    axes[1, 0].set_title("Episode reward (last 20)")
    axes[1, 1].set_title("Exploratory completions (last 20, %)")
    axes[1, 2].plot(
        [r["seconds"] / 60 for r in evaluations],
        [r["completions"] for r in evaluations],
        marker="o",
    )
    axes[1, 2].set(title="Greedy validation completions / 10", ylim=(-0.3, 10.3))
    fig.suptitle(
        "Lantern campaign PPO smoke run · fixed opponent · terminal outcome rewards"
    )
    fig.tight_layout()
    fig.savefig(directory / "curves.png", dpi=140)
    plt.close(fig)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--out", type=Path, required=True)
    p.add_argument(
        "--warm-start",
        type=Path,
        default=Path("training/checkpoints/campaign-cnn-imitation-v1/weights.pt"),
    )
    p.add_argument(
        "--resume",
        type=Path,
        help="Restore policy, critic, optimizers and RNGs; rollouts restart at fresh seeds",
    )
    p.add_argument("--seconds", type=float, default=300)
    p.add_argument("--campaign", default="lantern-road")
    p.add_argument("--opponent", choices=["classic", "predictor"], default="classic")
    p.add_argument("--rounds", type=int, default=100)
    p.add_argument("--envs", type=int, default=4)
    p.add_argument("--horizon", type=int, default=64)
    p.add_argument("--threads", type=int, default=1)
    p.add_argument("--seed", type=int, default=173)
    args = p.parse_args()
    if (
        not 0 < args.seconds <= 28800
        or not 1 <= args.envs <= 4
        or not 1 <= args.threads <= 2
        or args.horizon < 1
        or not 1 <= args.rounds <= 100
    ):
        p.error("invalid time, environment, thread, horizon or round budget")
    resume = (
        torch.load(args.resume, map_location="cpu", weights_only=True)
        if args.resume
        else None
    )
    if resume:
        prior = resume["manifest"]
        prior_rounds = prior.get("rounds")
        if prior_rounds is None:
            # Older smoke checkpoints recorded the cap only in their cases.
            caps = {c.get("maxRounds") for c in prior.get("validation_configs", [])}
            prior_rounds = caps.pop() if len(caps) == 1 else None
        if (
            prior["fingerprints"] != json.loads(native.fingerprints())
            or prior["campaign"] != args.campaign
            or prior["opponent"] != args.opponent
            or prior_rounds != args.rounds
        ):
            p.error(
                "resume catalog, engine, campaign, opponent and round cap must match; use --warm-start for a new workload"
            )
        args.warm_start = Path(prior["warm_start"])
        if (
            hashlib.sha256(args.warm_start.read_bytes()).hexdigest()
            != prior["warm_start_sha256"]
        ):
            p.error("resume reference policy changed")
    args.out.mkdir(parents=True, exist_ok=False)
    torch.set_num_threads(args.threads)
    torch.set_num_interop_threads(1)
    torch.manual_seed(args.seed)
    random.seed(args.seed)
    rng = np.random.default_rng(args.seed)
    policy = SpatialPlanner().eval()
    warm_start(policy, args.warm_start)
    ac = ActorCritic(policy).eval()
    reference = copy.deepcopy(policy).eval().requires_grad_(False)
    actor_opt = torch.optim.Adam(policy.parameters(), lr=1e-5)
    critic_opt = torch.optim.Adam(ac.critic.parameters(), lr=1e-3)
    if resume:
        policy.load_state_dict(resume["policy"])
        ac.critic.load_state_dict(resume["critic"])
        actor_opt.load_state_dict(resume["actor_optimizer"])
        critic_opt.load_state_dict(resume["critic_optimizer"])
        torch.set_rng_state(resume["torch_rng"])
        rng.bit_generator.state = resume["numpy_rng"]
        random.setstate(resume["python_rng"])
    gamma, lam, temperature = 0.999, 0.95, 1.25
    start = time.monotonic()
    deadline = start + args.seconds
    log = (args.out / "metrics.jsonl").open("w")

    def record(event, **values):
        row = dict(event=event, seconds=time.monotonic() - start, **values)
        log.write(json.dumps(row) + "\n")
        log.flush()
        print(json.dumps(row), flush=True)

    valid = [
        configuration(s, r, args)
        for r in ["situational", "predictor"]
        for s in range(221001, 221006)
    ]
    manifest = dict(
        algorithm="PPO",
        fingerprints=json.loads(native.fingerprints()),
        warm_start=str(args.warm_start),
        warm_start_sha256=hashlib.sha256(args.warm_start.read_bytes()).hexdigest(),
        campaign=args.campaign,
        opponent=args.opponent,
        rounds=args.rounds,
        seconds=args.seconds,
        envs=args.envs,
        horizon=args.horizon,
        gamma=gamma,
        gae_lambda=lam,
        temperature=temperature,
        clip=0.2,
        reference_kl_coefficient=0.02,
        entropy_coefficient=0.005,
        actor_learning_rate=1e-5,
        critic_learning_rate=1e-3,
        critic_warmup_updates=2,
        rewards=dict(success=1, defeat=-1, truncation=-1),
        reward_shaping=False,
        validation_configs=valid,
        training_first_seed=201001,
        random_seed=args.seed,
        optimizer_state_in_checkpoints=True,
        resume_source=None
        if not args.resume
        else dict(
            path=str(args.resume),
            sha256=hashlib.sha256(args.resume.read_bytes()).hexdigest(),
        ),
    )
    save_json(args.out / "manifest.json", manifest)
    best = (-1, -100)
    selected = None
    update = resume["update"] if resume else 0
    episode_count = resume["episodes"] if resume else 0
    decisions = resume["decisions"] if resume else 0
    next_seed = resume["next_training_seed"] if resume else 201001

    def checkpoint(label):
        directory = args.out / label
        directory.mkdir(exist_ok=True)
        atomic_torch_save(policy.state_dict(), directory / "weights.pt")
        save_json(directory / "planner.json", json.loads(policy.export()))
        atomic_torch_save(
            dict(
                policy=policy.state_dict(),
                critic=ac.critic.state_dict(),
                actor_optimizer=actor_opt.state_dict(),
                critic_optimizer=critic_opt.state_dict(),
                torch_rng=torch.get_rng_state(),
                numpy_rng=rng.bit_generator.state,
                python_rng=random.getstate(),
                update=update,
                episodes=episode_count,
                decisions=decisions,
                next_training_seed=next_seed,
                manifest=manifest,
            ),
            directory / "resume.pt",
        )
        return directory

    def evaluate(label):
        nonlocal best, selected
        directory = checkpoint(label)
        results = [
            json.loads(r)
            for r in native.evaluate(
                [json.dumps(c) for c in valid], (directory / "planner.json").read_text()
            )
        ]
        wins = sum(r["success"] for r in results)
        caps = sum(r["truncated"] for r in results)
        save_json(directory / "validation.json", dict(configs=valid, results=results))
        score = (wins, -caps)
        if score > best:
            best = score
            selected = label
        record(
            "evaluation",
            checkpoint=label,
            completions=wins,
            defeats=10 - wins - caps,
            caps=caps,
            selected=selected,
        )
        plot_metrics(args.out)

    env = native.BatchEnv(spatial=True)
    try:
        evaluate("baseline")

        def reset():
            nonlocal next_seed
            configs = [
                configuration(next_seed + i, ["situational", "predictor"][i % 2], args)
                for i in range(args.envs)
            ]
            next_seed += args.envs
            return env.reset([json.dumps(c) for c in configs])

        obs = reset()
        next_eval = 60.0
        while time.monotonic() < deadline:
            samples = []
            rewards = []
            values = []
            dones = []
            lengths = 0
            for t in range(args.horizon):
                if time.monotonic() >= deadline:
                    break
                observations = unpack(obs, env.spatial_observation())
                if not observations:
                    break
                ids = list(observations)
                tensors = pack(list(observations.values()))
                with torch.no_grad():
                    dist, v = ac(*tensors, temperature)
                    acts = dist.sample()
                    lp = dist.log_prob(acts)
                actions = [0] * args.envs
                step_samples = {}
                for j, slot in enumerate(ids):
                    actions[slot] = int(acts[j])
                    step_samples[slot] = dict(
                        observation=observations[slot],
                        action=int(acts[j]),
                        old_log_prob=float(lp[j]),
                    )
                following = env.step(actions)
                reward = np.zeros(args.envs, np.float32)
                value = np.zeros(args.envs, np.float32)
                done = np.ones(args.envs, np.float32)
                for j, slot in enumerate(ids):
                    terminal = following[3][slot] or following[4][slot]
                    reward[slot] = -1.0 if following[4][slot] else following[2][slot]
                    value[slot] = float(v[j])
                    done[slot] = terminal
                    if terminal:
                        episode_count += 1
                        record(
                            "episode",
                            episode=episode_count,
                            reward=float(reward[slot]),
                            success=bool(reward[slot] > 0),
                            truncated=bool(following[4][slot]),
                        )
                samples.append(step_samples)
                rewards.append(reward)
                values.append(value)
                dones.append(done)
                decisions += len(ids)
                obs = following
                lengths += 1
                if all(t or u for t, u in zip(obs[3], obs[4])):
                    break
            if not lengths:
                break
            bootstrap = np.zeros(args.envs, np.float32)
            live = unpack(obs, env.spatial_observation())
            if live:
                with torch.no_grad():
                    _, next_v = ac(*pack(list(live.values())), temperature)
                for j, slot in enumerate(live):
                    bootstrap[slot] = float(next_v[j])
            adv, returns = advantages(
                np.asarray(rewards),
                np.asarray(values),
                np.asarray(dones),
                bootstrap,
                gamma,
                lam,
            )
            flat = [
                dict(s, advantage=float(adv[t, slot]), target=float(returns[t, slot]))
                for t, step in enumerate(samples)
                for slot, s in step.items()
            ]
            a = np.asarray([s["advantage"] for s in flat], np.float32)
            a = (a - a.mean()) / max(float(a.std()), 1e-6)
            for s, normalized in zip(flat, a):
                s["advantage"] = float(normalized)
            update += 1
            stats = []
            for epoch in range(3):
                for first in range(0, len(flat), 64):
                    if time.monotonic() >= deadline:
                        break
                    # Shuffle once per PPO pass, retaining each sample exactly once.
                    if first == 0:
                        order = rng.permutation(len(flat))
                    batch = [flat[i] for i in order[first : first + 64]]
                    tensors = pack([s["observation"] for s in batch])
                    dist, value = ac(*tensors, temperature)
                    acts = torch.tensor([s["action"] for s in batch])
                    old = torch.tensor([s["old_log_prob"] for s in batch])
                    advantage = torch.tensor([s["advantage"] for s in batch])
                    target = torch.tensor([s["target"] for s in batch])
                    logp = dist.log_prob(acts)
                    ratio = (logp - old).exp()
                    policy_loss = -torch.minimum(
                        ratio * advantage, ratio.clamp(0.8, 1.2) * advantage
                    ).mean()
                    value_loss = F.mse_loss(value, target)
                    with torch.no_grad():
                        ref_dist = categorical(
                            reference(*tensors[:3]), tensors[3], temperature
                        )
                    kl = categorical_kl(dist, ref_dist).mean()
                    entropy = dist.entropy().mean()
                    loss = value_loss + (
                        policy_loss + 0.02 * kl - 0.005 * entropy if update > 2 else 0
                    )
                    if not torch.isfinite(loss):
                        raise RuntimeError("nonfinite PPO loss")
                    actor_opt.zero_grad()
                    critic_opt.zero_grad()
                    loss.backward()
                    nn.utils.clip_grad_norm_(
                        ac.parameters(), 0.5, error_if_nonfinite=True
                    )
                    if update > 2:
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
                            ]
                        ]
                    )
                if time.monotonic() >= deadline:
                    break
            if stats:
                means = np.mean(stats, 0)
                record(
                    "update",
                    update=update,
                    decisions=decisions,
                    episodes=episode_count,
                    samples=len(flat),
                    policy_loss=means[0],
                    value_loss=means[1],
                    entropy=means[2],
                    reference_kl=means[3],
                    clip_fraction=means[4],
                )
            if time.monotonic() - start >= next_eval and time.monotonic() < deadline:
                evaluate(f"update-{update:05d}")
                next_eval += 60
            if (
                all(t or u for t, u in zip(obs[3], obs[4]))
                and time.monotonic() < deadline
            ):
                obs = reset()
        training_seconds = time.monotonic() - start
        evaluate("last")
        save_json(
            args.out / "selection.json",
            dict(
                selected=selected,
                criterion="Most validation campaign completions; fewer caps breaks ties; earliest checkpoint retained on ties",
                validation_score=best,
                final_test_seeds_used=False,
            ),
        )
        record(
            "finished",
            training_seconds=training_seconds,
            updates=update,
            decisions=decisions,
            episodes=episode_count,
            selected=selected,
        )
    finally:
        env.close()
        log.close()
    plot_metrics(args.out)


if __name__ == "__main__":
    main()
