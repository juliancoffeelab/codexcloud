import type { ArenaConfig, ArmyId, CampaignConfig, Landscape } from "./simulation";
import type { PolicyId } from "./policies";
import type { DiplomacyId } from "./diplomacy";
import type { CampaignId } from "../types";

interface Limits {
  seed: number;
  maxRounds: number;
  recruitLimit: number;
}

/** Race × complete policy × terrain, with seats and initiative counterbalanced. */
export function arenaMatchups(
  options: Limits & {
    armies: readonly ArmyId[];
    policies: readonly PolicyId[];
    landscapes: readonly Landscape[];
  },
): ArenaConfig[] {
  const entrants = options.armies.flatMap((army) =>
    options.policies.map((policy) => ({ army, policy })),
  );
  const configs: ArenaConfig[] = [];
  for (const landscape of options.landscapes)
    for (let a = 0; a < entrants.length; a++)
      for (let b = a; b < entrants.length; b++) {
        const left = entrants[a]!,
          right = entrants[b]!;
        const seats =
          a === b
            ? [[left, right]]
            : [
                [left, right],
                [right, left],
              ];
        for (const [player, enemy] of seats)
          for (const first of ["player", "enemy"] as const)
            configs.push({
              seed: options.seed,
              maxRounds: options.maxRounds,
              recruitLimit: options.recruitLimit,
              landscape,
              first,
              player: player!.policy,
              enemy: enemy!.policy,
              armies: { player: player!.army, enemy: enemy!.army },
            });
      }
  return configs;
}

export function campaignMatchups(
  options: Limits & {
    campaigns: readonly CampaignId[];
    policies: readonly PolicyId[];
    diplomacy: readonly DiplomacyId[];
  },
): CampaignConfig[] {
  return options.campaigns.flatMap((campaign) =>
    options.policies.flatMap((player) =>
      options.policies.flatMap((enemy) =>
        options.diplomacy.map((diplomacy) => ({
          campaign,
          player,
          enemy,
          diplomacy,
          seed: options.seed,
          maxRounds: options.maxRounds,
          recruitLimit: options.recruitLimit,
        })),
      ),
    ),
  );
}
