# Independent mechanics review

Reviewed source: `6e1f5c3`, 2026-10-05. Gameplay changes are those of `efe3ec5`;
later changes at that point add trial documentation and formatting. The reviewer
worked independently, did not edit implementation and did not replay active
player files. Its full `npm test` run passed all nineteen files in 6.80 seconds.

Independent probes exercised capture and ledger-event undo, three defended
phases completed before the minimum high-tide turn, completed legacy victory
migration, enemy recapture and subsequent player recapture, and a pending enemy
turn across reload. Held duration remained zero after recapture until a new
enemy phase completed. All seventeen promotion families retained three wounds,
identity and name, consumed their actions and exhausted the required XP.

Source inspection confirmed rookie swordsmen in recurring Ashen purchases and
the turn-three reinforcement, with the initially authored armored patrol
retained. Enemy promotions use the same XP and advancement rules. No actionable
mechanics blocker was found in the reviewed changes.

Both finished Green records replayed independently against frozen `a8ec024`.
Mixed recruitment completed in 11 / 4 / 6 turns with two troop losses and Dain
alive. Boar-only recruitment completed in 17 / 4 / 6 with three losses and Dain
dead; the death was first recorded in opening-chapter turn fourteen. The
reviewer judged that the recorded mixed strategy had the better campaign
outcome, while explicitly limiting that conclusion to one pair with different
decisions, earned rosters and companion survival.

Coastal and Ashen balance was not approved by this review: those comparisons
were still running. The earlier absolute-round evacuation shortcut was an
actual finding of the preceding review and prompted the duration-state change.
Mechanical checks and an absence of a newly found blocker do not establish
optimal unit balance or browser appearance. Distinct veteran artwork remains
deferred.

## Old Ashen follow-up

The reviewer independently replayed both completed old Ashen records on
`8259b39`. The mixed player's first refuge attempt bought no troops despite
24–30 available gold, sent Orin north and left healthy recalled skeletons near
their starting positions. The isolated refuge holder died immediately. The
retry made only one skeleton attack in ten turns and earned no promotions;
the successful skeleton-only retry maintained ownership, rotated screens and
promoted Orin and a skeleton. It still used the authored bow and Orin's Impact.
Mixed's quarry retry was faster and less costly than mono's successful quarry
attempt: six turns and one casualty versus eleven and five.

There was also an actual encounter bias: recurring heavy infantry took two
bone-bow damage versus three from skeleton clubs. Rookie recurring swordsmen
improve that matchup to three bow damage while retaining the initial armored
patrol. The reviewer recommends this targeted change rather than a further
global bow or movement buff before the fresh matched records finish. Tactical
differences and unequal mediation clarification limit the old comparison.

The finished coastal mixed record also replayed independently on `deae30c`:
8 / 6 / 5 turns, two troop losses and no retry. The reviewer did not replay its
still-active restricted companion trial or infer the final paired result.

## Completed eighth coastal pair

Both records subsequently replayed exactly on `deae30c`: 135 mixed decisions
and 156 restricted decisions. Mixed won 8 / 6 / 5 with two troop losses and no
retry. Reef-only purchases won 9 / 6 / 7 after an opening defeat on turn two:
three troop losses on successful attempts, plus Edda's death in that failed
opening. Both finally retained Mira, Edda and Tomas. Mixed Edda was a tide
hunter; restricted Edda remained a harpooner. Authored support roles and their
promotions remained usable in the restricted policy.

The reviewer judges the recorded mixed result better in turns, casualties and
retries, unlike the older speed/attrition tradeoff. The control's avoidable
Edda exposure limits attribution to recruitment. One pair cannot isolate the
HP change, establish optimal balance or prove the revision caused improvement.
No further coastal change is justified by this pair alone.

## Quarry post and ninth-cohort follow-up

The reviewer inspected actual `3956f5c`. The quarry's authored leader post
changes only that leader's destination policy. Movement remains legal,
displaced leaders return toward their post, retreat executes first and adjacent
intrusions can still be attacked. Ordinary patrols and other missions keep
their prior policy. No implementation blocker was found. A bounded local
counterfactual restored surviving retreats for Orin; it was not a complete
campaign result. Fresh tenth-cohort players are testing the revised encounter.

Lantern ninth-cohort mixed replayed exactly: the ferry won at four with no
losses; the second chapter capped at thirty with a null winner and six losses.
It started with 26 gold. The enemy bought four troops during rounds two through
five, then stopped recruiting; its army stabilized at seven. From rounds eleven
through twenty-nine the player submitted no actions. Three surviving Impact
casters remained west of the river while player gold reached 89. Earlier
isolated advances are difficulty evidence, but the nineteen inactive turns
substantially confound this result. It establishes neither an overwhelming
enemy economy nor balanced Lantern play and alone justifies no stat buff or
economy reduction.

## Deadline checkpoint

The reviewer independently replayed the completed ninth coastal reef record:
9 / 6 / 6, two ordinary casualties, no retries, eleven reef purchases and three
earned trident promotions. Mira, Edda and Tomas survive; Edda finishes at one
HP. Wreck ownership accumulates three actual phases and waits for high tide.
Types and XP carry forward normally. Its mixed counterpart paused unfinished
at the user's deadline, so superiority is not established.

Ashen tenth mixed independently replayed 180 decisions to wins 8 / 6 / 9 with
ten casualties and no retries. Dain survived peaceful mediation but fell on
finale turn eight. Both binding stones were sealed and the Master defeated.
This establishes independent mixed viability on the revised encounter, with
substantial attrition and a named loss. It proves neither a clean victory nor
causal improvement or superiority over the paused incomplete control.

The reviewer inspected the Lantern route-transfer procedure and independently
replayed both outputs. Every original player choice and enemy-phase marker
is preserved. Mixed still wins 5 / 5 / 7; swordsman still ends at the finale's
turn fifteen after losing its sole mage. These are explicitly scripted
compatibility checks. Fresh mixed exposed its courier in its first attempt
and delayed near its start on retry; the old successful route used a scout's
Impact sling and coordinated support. No relevant opening stat/map/AI regression
was found. The failed fresh run remains adverse evidence; current matched
results remain incomplete and cannot be replaced by compatibility checks.

## Extended review: progression artwork and current balance evidence

An independent reviewer inspected gameplay/art source `9ef4a38` and the later
documentation updates on 2026-10-05. All nineteen test files passed in 7.17
seconds. The native matrix/atlas check passed. It inspected the complete roster
and verified all seventeen promotion families, thirty-four ranks and sixty-eight
faction frames, wound preservation, maximum weapon-range summaries and terrain
access precedence. No additional source or visible artwork blocker was found.
Published legacy deployment migration preserves saved armies and suppresses
duplicate replacement waves. This does not claim phone/browser inspection.

All eight eleventh-cohort records and both archived twelfth-cohort Ashen records
replayed independently against their frozen sources. The updated reports retain
adverse and interrupted results. Ashen mixed balance remains unproven: eighteen
losses versus twelve in cohort eleven, then two quarry defeats in cohort twelve
while its control remains unfinished. No enemy bone bows appear in the quarry,
so their increased damage cannot explain those opening defeats.

Both eleventh-cohort refuge policies fetch medicine by turn three. Mixed loses
ownership, recaptures in turn four and accumulates five defended phases only by
turn nine; the control never resets ownership and wins at six. The three extra
pressure phases and different retained bow support confound recruit efficiency.
The control recalls its surviving authored bow, whereas mixed lost that bow.
Restrictions concern new purchases, not the absence of all auxiliary troops.

The reviewer recommends no additional global skeleton nerf or auxiliary buff
from these records alone. Raising skeleton cost would also break the fourteen-
gold skeleton-plus-bow opening. Stronger bows and reduced quarry pressure address
specific deficiencies but still need current comparison evidence. Further
refuge difficulty changes should follow actual competent deployment and pressure
patterns, rather than treating mechanical correctness as balance approval.

## Final guided comparisons and deployment confounds

Independent replay confirms cohort fourteen's mixed Ashen campaign wins
8 / 6 / 8 without a retry, with eight losses including Dain. Its matching
restricted control later stops at the deadline with fourteen all-attempt
losses and its finale unfinished. This favors the recorded mixed play but
does not produce two completed campaigns. Cohort twelve's Lantern control ends at
the village turn cap with a null winner. It makes only five attacks in the
entire second chapter despite buying seventeen swordsmen and retaining
twenty-one troops against nine enemies. A larger idle army does not establish
that enemy recruitment overwhelms an actively deployed company.

The current thirteenth woodland mixed player has adverse opening and vigil
results and an actual escort defeat. Independent replay of its recorded
phases confirms the fatal escort move sends two-HP Lark from `9,0` to `13,0`,
where a wolf at `11,2` can reach and kill her for three damage. Its earlier
escort casualties are boars; no auxiliary HP threshold explains that final
death. During the failed vigil, a promoted healer and three boars remain at
their original western positions while Dain and Lark advance. The successful
retry loses Dain and carries rookie Lark with five XP into the escort; the
control retains Dain and promotes Lark at the escort's start.

Those adverse results remain valid records of player decisions. The reviewer
finds no causal basis here for claiming that an auxiliary HP buff would repair
the observed defeats. Current woodland and Ashen comparisons remain unfinished
until their actual traces stop. No global stat change or universal balance
approval follows from these deployment-confounded records. Final exact replay
archives the deadline checkpoints: both Lantern policies cap with null winners;
woodland boar completes while mixed is unfinished after two retries; both
restricted Ashen finales are unfinished. A reported forecast mismatch was
checked against the exact preceding state and matched the actual response;
the player had compared it with a later phase. No source change was made.
