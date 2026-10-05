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
