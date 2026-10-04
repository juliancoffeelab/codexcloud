# Campaign playtests

The overnight mechanical pass uses gameplay source `547718d`. Fresh linked
Luna 6 sessions receive the actual conversations throughout each story, alongside
briefings, hints and debriefs. Their results are recorded separately from earlier
cohorts. No historical completion rate estimates performance with the revised AI.

The previous Green Accord repair was tested at `81979dc`. The original campaign
completion rates describe `c3b2c72`, before refuge mediation and retreat changes.

Earlier refuge verification checked defended turns, medicine, escalation,
retreat/healing, saves, and both next-mission branches. Actual-engine scenario
probes completed the peaceful route after six enemy turns without medicine and
five with medicine. A deliberate pursuit probe completed the hostile route on
turn 10. These probes used three returning undead recruits and six carried gold;
they are controlled scenarios, not fresh linked Luna sessions or completion rates.

These are independent Luna 6 play sessions through `scripts/play-campaign.ts`,
which imports the production `Game` engine. The CLI exposes observations, legal
actions, exact combat forecasts, recruitment and conservative enemy threat
bounds. It does not choose moves. Enemy turns use the same action order and AI
as the browser controller. No browser or substitute combat model is involved.

Each player starts at mission one, earns survivor and gold carryover, and chooses
its own tactics. Each mission allows one retry; a second defeat ends that session.
A 40-turn limit per attempt is a test budget, not a game defeat condition. Players
cannot inspect implementation code, other trials, or edit saves. Dialogue is
acknowledged automatically, with its accumulated lines now exposed in observations.
The trials record decisions, retries, carryover and enemy responses.
`scripts/replay-campaign.ts` verifies these against the matching engine source.
Rules are frozen for each session and fingerprinted in new trials.

Combat is deterministic. Completion rates measure variation in independent
players' decisions, rather than combat dice or a guaranteed population win rate.
Five sessions per campaign are a small sample. Whole-campaign completion includes
the permitted retries; first-attempt mission wins are reported separately.

## Overnight mechanical verification

Gameplay source: `547718d47dfe0e045d1aa55d73590917d88dadfa`, held in an isolated
checkout while the main branch received review fixes. Enemy withdrawal now
checks actual available attacks and fatal retaliation, rather than a blanket
low-health or projected-threat threshold. A safe attack allows reengagement;
previously withdrawing troops can recover in safe owned villages. Cornered
troops may fight. Terrain base costs and explicit signed unit modifiers preserve
the existing numeric costs, including woodland movement for boars.

The later review repair at `fb1b896` changes dialogue persistence, terminal status,
ending speaker eligibility, tile accessibility and recall selection. It does not
change combat, movement, AI, maps or mission requirements. An independent reviewer
found five confirmed defects; each was reproduced before repair, and a separate
architecture reviewer checked the actual fixes and regression tests. A fixed
microtask-count wait in one test was replaced with an explicit controlled pause.

Verification: all 18 unit-test files passed in 4.46 seconds on the final repair;
strict TypeScript and
Svelte checks reported no errors or warnings; formatting and production build
passed. The same-map recall failure was reproduced with Svelte's Node reactivity
runtime and the production engine. No browser interaction or mobile visual
parity is claimed. Source-sheet inspection and focused asset checks cover the
four replacement Pixel skeleton/bone-archer frames; existing PNG assets and the
approved Low-poly artwork are unchanged. Story copy was not rewritten.

Five fresh sessions concluded. Lantern and Green each had one player; Ashen had
an initial player plus two independent repetitions after that player's two opener
losses. All three campaigns were completed by Luna 6. This unequal, small sample
is a sanity check, not an estimated population completion probability. Players
received action-schema clarification when needed, with no tactical coaching.

| Trial                                                            | Mission 1        | Mission 2 | Mission 3 | Complete |
| ---------------------------------------------------------------- | ---------------- | --------- | --------- | -------- |
| [Lantern](playtests/night-retaliation-1/night_lantern_luna.json) | Loss 3 → Win 6   | Win 19    | Win 17    | Yes      |
| [Green](playtests/night-retaliation-1/night-green-luna.json)     | Win 3            | Win 4     | Win 9     | Yes      |
| [Ashen A](playtests/night-retaliation-1/night-ashen-luna.json)   | Loss 14 → Loss 7 | —         | —         | No       |
| [Ashen B](playtests/night-retaliation-1/night-ashen-b-luna.json) | Win 7            | Win 9     | Win 9     | Yes      |
| [Ashen C](playtests/night-retaliation-1/night-ashen-c-luna.json) | Win 7            | Win 6     | Win 10    | Yes      |

Numbers are the recorded ending round. Unreached missions remain unreached. All
five full decision logs replay exactly against their recorded source; hashes,
attempts, survivors and fingerprints are in the machine-readable results.

Green finished with eleven survivors and full-health Lark at the meeting, while
the Rootbinder was still alive. Lantern finished with five survivors after an
attritional village chapter and finale. Both successful Ashen players earned
medicine-assisted intervention after five defended turns. B lost and recaptured
the refuge before winning; C held it continuously. B lost Dain in the finale and
won with Orin at 1 HP; C kept Dain alive and won with Orin at 5 HP. Both had three
final survivors. B's frozen trace retains the old ending-speaker defect; the later
review repair suppresses dead Dain's lines and has separate death/save coverage.

A separate investigator replayed Ashen A's failed trace and verified avoidable
damage in its exact states. Its first attempt entered a ford at 4 HP within an
adept's 5-damage attack; staying on the previous hex survived that enemy turn.
Its retry exposed Orin ahead of his screens; a legal screened attack sequence
preserved all five allies and left Orin at 8 HP instead of 3. Neither attempt
recruited before leaving the keep. These branch probes demonstrate counterplay,
not additional independent campaign wins. No map or balance tuning followed.

Writing and creative changes remain for user input. One Lantern player read
Meri's grave-sealing line as an actor requirement, although soldiers can seal
graves. The same player noticed the reed-gatherer quest triggers from Meri's
starting tile. These observations are recorded as discussion points; dialogue,
hints and quest placement were preserved.

## Historical Green Accord repair

Gameplay trial source: `81979dce002f392cf18d0875806fcc03d9616de1`. Five fresh Luna 6 players
started at the opening mission, retaining their earned company and gold through
all reached chapters. Each had one retry per mission. The table records the
ending round of each attempt; unreached missions remain unreached.

The missions now connect grove recovery, Lark's vigil to uncover the murdered
woodcutters, and bringing that evidence to the villagers. The vigil requires
Lark to remain in the owned central grove through three completed enemy turns,
with progress resetting when she leaves or the grove is lost. Lark is required
throughout; her death ends the mission. Dain's death changes later dialogue and
reinforcements rather than restoring him, and the survivors' victory dialogue
does not require optional companions to remain alive.

Thorncaller's ranged Thorns deal 3 Pierce instead of 2; the weak melee reply, HP,
movement and cost are unchanged. Boars now pay 1 movement in forests and groves, matching
the other woodland beasts, while retaining their slower marsh and ford costs.
Both factions share these unit rules. Two opening guards and two vigil guards
start farther away, allowing an initial approach without immediately trapping
the required companion between two melee attackers.

The first repaired candidate, `a2a9bae`, completed 0/2 campaigns: A lost the
opening twice; B won the opening but lost the vigil twice. Those failures
prompted the guard placement and boar movement changes. Both failures remain
in the dataset under cohort `green-continuity-1`. The final five sessions use
cohort `green-continuity-2` and one frozen rules fingerprint.

Whole campaigns completed: **4/5**; first-attempt mission wins: **11/13**.
Three players completed all three chapters without a retry. C used its opening
retry successfully; D lost the opening twice and did not reach later chapters.

| Trial | Mission 1       | Mission 2 | Mission 3 | Complete |
| ----- | --------------- | --------- | --------- | -------- |
| C     | Loss 2 → Win 2  | Win 4     | Win 5     | Yes      |
| D     | Loss 1 → Loss 5 | —         | —         | No       |
| E     | Win 3           | Win 4     | Win 7     | Yes      |
| F     | Win 3           | Win 4     | Win 5     | Yes      |
| G     | Win 3           | Win 4     | Win 13    | Yes      |

These trials received briefings, hints and debriefs throughout. Conversation lines
were initially omitted by the CLI despite being automatically acknowledged; the
observation field was added during F and G's finale. Gameplay rules stayed frozen,
but these runs do not establish full-campaign narrative comprehension. Future
players receive the actual introduction, event and result lines as well.
A separate dialogue review by E found the three-mission motivation coherent,
but Tamsin arrived without an introduction on the Dain-alive path. After the
trials, her existing finale line was clarified to explain why she joined; this
copy change leaves unit stats, AI, maps and objectives unchanged.

All new decision traces replay against their recorded source. The
[machine-readable results](campaign-playtest-results.json) preserve both repair
cohorts, attempts, transcript hashes, source commits and rules fingerprints.
These are observed outcomes for five independent players, not a statistical
guarantee. Fragile Lark can still die when exposed to multiple attackers; the
objectives require protecting her rather than clearing every enemy.

Verification on that repair's production source: formatting, strict TypeScript and
Svelte checks, all 17 lightweight unit-test files (6.05 seconds), and production
build passed. Browser interaction and visual parity were not tested.

## Historical results

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

The overnight cohort's full decision logs are committed under
[`playtests/night-retaliation-1`](playtests/night-retaliation-1). Each result records
its log path, SHA-256, rules fingerprint and source commit. Replay these logs with
`scripts/replay-campaign.ts` from source `547718d`; later review repairs can change
terminal conversation state even though combat rules remain the same.

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
