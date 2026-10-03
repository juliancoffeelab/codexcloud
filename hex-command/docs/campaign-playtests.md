# Campaign playtests

The completion rates below describe the frozen balance pass at `c3b2c72`, before
the refuge mediation and enemy retreat changes. They are historical results,
not estimates for the current rules. Current refuge verification checks defended
turns, medicine, escalation, retreat/healing, saves, and both next-mission branches.
Actual-engine scenario probes also completed the peaceful route after six enemy
turns without medicine and five with medicine, using a returning company. Those
A deliberate pursuit probe also completed the hostile route on turn 10. The
probes used three returning undead recruits and six carried gold; these are
controlled scenarios, not fresh linked Luna sessions or new completion rates.

These are independent Luna 6 play sessions through `scripts/play-campaign.ts`,
which imports the production `Game` engine. The CLI exposes observations, legal
actions, exact combat forecasts, recruitment and conservative enemy threat
bounds. It does not choose moves. Enemy turns use the same action order and AI
as the browser controller. No browser or substitute combat model is involved.

Each player starts at mission one, earns survivor and gold carryover, and chooses
its own tactics. Each mission allows one retry; a second defeat ends that session.
A 40-turn limit per attempt is a test budget, not a game defeat condition. Players
cannot inspect implementation code, other trials, or edit saves. Dialogue is
acknowledged automatically. The trials record decisions, retries, carryover and
enemy responses. `scripts/replay-campaign.ts` verifies these against the matching
engine source. Rules are frozen for each session and fingerprinted in new trials.

Combat is deterministic. Completion rates measure variation in independent
players' decisions, rather than combat dice or a guaranteed population win rate.
Five sessions per campaign are a small sample. Whole-campaign completion includes
the permitted retries; first-attempt mission wins are reported separately.

## Final results

Five fresh sessions per story tested the final layouts. There were 40 independent
sessions across all versions, including unsuccessful tuning candidates.

| Campaign         | Whole campaigns completed | First-attempt mission wins |
| ---------------- | ------------------------: | -------------------------: |
| The Lantern Road |                 4/5 (80%) |                       9/13 |
| The Green Accord |                 4/5 (80%) |                       9/13 |
| The Ashen Oath   |                 4/5 (80%) |                       8/13 |

Mission wins count only missions actually reached. Unreached missions remain
unreached; they are not separately played with an invented recall army.
All surviving decision logs replay exactly against their frozen engine source.

| Trial     | Mission 1       | Mission 2       | Mission 3 | Complete |
| --------- | --------------- | --------------- | --------- | -------- |
| lantern-q | Loss 2 → Win 12 | Win 16          | Win 13    | Yes      |
| lantern-r | Win 5           | Loss 18 → Win 7 | Win 14    | Yes      |
| lantern-s | Win 3           | Win 12          | Win 14    | Yes      |
| lantern-t | Loss 2 → Loss 9 | —               | —         | No       |
| lantern-u | Loss 2 → Win 9  | Win 6           | Win 9     | Yes      |
| green-c   | Win 2           | Win 6           | Win 12    | Yes      |
| green-d   | Win 3           | Loss 7 → Win 6  | Win 12    | Yes      |
| green-e   | Loss 5 → Win 3  | Win 6           | Win 11    | Yes      |
| green-f   | Loss 3 → Win 2  | Win 6           | Win 11    | Yes      |
| green-g   | Loss 5 → Loss 5 | —               | —         | No       |
| ashen-h   | Win 7           | Loss 6 → Win 6  | Win 6     | Yes      |
| ashen-i   | Loss 7 → Loss 6 | —               | —         | No       |
| ashen-j   | Loss 8 → Win 6  | Win 6           | Win 6     | Yes      |
| ashen-k   | Loss 7 → Win 6  | Win 6           | Win 10    | Yes      |
| ashen-l   | Win 7           | Loss 10 → Win 6 | Win 10    | Yes      |

Numbers in the trial table are the ending round. An arrow separates the first
attempt and its single retry. The [machine-readable results](campaign-playtest-results.json)
include every cohort, recorded outcomes, rules fingerprints, transcript hashes,
and the one overwritten-trace exception described below.

These completion percentages are an empirical sanity check from five players
per story, not a statistical guarantee. Combat itself has no random rolls.

Verification: formatting, strict TypeScript/Svelte checks, all 15 lightweight
unit-test files (10.34 seconds under concurrent playtests), and the production
build passed. Browser interaction and visual parity were not tested.

## What changed after failures

The original six sessions completed no campaigns. Both human companies lost the
escort opener, both undead companies lost the quarry escape, and both woodland
companies reached the finale but lost the escort there.

The first tuning pass gave the human escort another soldier, a healing village
on the northern route, and a later southern patrol. The quarry's northern marsh
exit became ruins, its guards started farther away, and its warden joined an
announced patrol instead of the initial army. The woodland finale gained two
groves along its northern route, a less congested ford exit, and Tamsin, an adept
whose impact attack supplies a counter to skeleton armor. A forward wolf and
bone archer started farther from the crossing.

Human sessions still exposed pacing problems. The second pass moved the first
guards farther back and added ranged cover beyond the bridge. The village
mission now supplies a soldier even when little survives to recall. The finale
supplies a scout, and grave waves arrive on turns 6 and 9, allowing time to clear
the crossing and approach the seals. Briefings announce these timings.

Further escort failures exposed a mismatch between the first human mission's
promise (carry a message rather than conquer) and its destination beside the
enemy keep. The landing moved north, outside that immediate stronghold approach,
so the ford offers a real bypass. The briefing also explains the skeletons'
arrow resistance and mace counter. Existing trials finished on frozen source;
fresh players tested the changed destination.

The northern bypass still funneled the escort through marsh on both sides of
the ford. The next human adjustment removed the western marsh and replaced
the eastern one with ruins. The route now gives useful cover and enough movement
to cross without spending multiple turns exposed beside the bridge guards.
The southern marsh remains, and the ford retains its movement and exposure costs.

The human finale still defeated companies before they reached a grave. Its camp
now starts two columns nearer the crossings. Ruins beyond the flank bridges
replace marsh, and the two announced grave waves arrive on turns 8 and 12.
This creates time to fight for the seals before reinforcements multiply, while
keeping the defenders, combat rules, and all three sealing requirements. The
opening bone archer now starts on the southern guard road, farther from the
escort's northern bypass. Fresh companies tested the whole revised route.

The first quarry adjustment produced only two whole-campaign wins in five
sessions. All three unsuccessful companies lost Orin in the opener; the later
missions were completed by both companies that reached them. A second pass put
the guard keep and its starting patrol on the southern road, supplied a third
skeleton to screen the crossing, and added northern ruins and a healing village.
The escape now has a distinct route past the guarded bridge, while the southern
ledger side quest still invites a detour toward the fighting.

Unit prices, damage, armor, movement, healing, AI, objective requirements and
carryover rules are shared by both sides and were unchanged during tuning.
Later rounds remain recoverable: no artificial turn deadline was added.

## Reproducibility

Use a fresh trial path, the documented CLI in the README, and a fixed checkout.
`start` refuses to overwrite prior attempts; `next` and `retry` preserve the
earned company and history. A changed rules fingerprint rejects continuation.
Action batches stop at the first invalid action while retaining earlier legal
actions; the returned observation is authoritative.

One woodland session accidentally restarted its trial before this guard was
added. Its original failed opening attempt was reported, then its successful
two-turn retry was replayed to restore the earned company. That replay is not an
additional independent attempt. The restored victory path is replay-verifiable;
the overwritten defeat trace is unavailable. Results retain that failure and
identify the exception explicitly.
