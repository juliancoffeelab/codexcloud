# Race and progression balance trials

The 2026-10-05 balance pass uses Luna players choosing actual legal actions in
the game engine. It does not substitute a combat simulation or a developer's
winning route for an independent campaign. Players start at chapter one and
retain only earned units, XP and gold. Each receives one retry after an actual
defeat and a thirty-turn limit per chapter. Incomplete attempts retain a null
winner. Both policies may use their authored starting company and earned
promotions; only **new purchases** differ.

The headless interface supplies legal-action options and can forecast the next
enemy phase on an exact cloned state. Both policies have that capability. Such
forecasts are predictions, not recorded victories; subsequent actions invalidate
them. This gives the agents more information than ordinary browser play and
limits conclusions about human difficulty. Actual submitted actions and enemy
responses, including mistakes, remain the evidence used for comparison.

One run per policy is evidence about those recorded decisions, not an estimate
of optimal play. Completion and retries, companion survival, casualties and
turns all matter. A faster opening alone does not establish a better campaign.
Repeatedly weaker mixed results prompt changes and a fresh matched comparison.

Each cohort records its frozen source commit, full decisions and exact enemy
responses. `scripts/replay-campaign.ts` verifies those records against that
source. Old failures remain in the repository; later wins do not overwrite
them. A developer route study is identified separately.

| Cohort                           | Scope                             | What it established                                                                                                |
| -------------------------------- | --------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| [1](playtests/tides-1/README.md) | Initial four-campaign comparisons | Baseline failures and dominant recruitment prompted revision.                                                      |
| [2](playtests/tides-2/README.md) | Revised four-campaign comparisons | Failed and interrupted runs remain explicitly recorded.                                                            |
| [3](playtests/tides-3/README.md) | Woodland and coast                | Boar and reef recruitment still outperformed mixed armies; a developer woodland route is separate.                 |
| [4](playtests/tides-4/README.md) | Ashen opening                     | Both independent players failed; this did not establish balance.                                                   |
| [5](playtests/tides-5/README.md) | Woodland and Lantern              | Woodland mixed attrition remained worse; Lantern mixed completed, while swordsman-only lost its sole sealing mage. |
| [6](playtests/tides-6/README.md) | Coast and Ashen                   | Mixed coast opened faster but lost six troops versus three; independent Ashen mixed failed the refuge.             |

The [seventh woodland pair](playtests/tides-7/README.md) completed on the revised
source. Mixed recruitment won in 11 / 4 / 6 turns, lost two troops and retained
Dain. Boar-only recruitment needed 17 / 4 / 6, lost three troops and killed Dain
in the opening. Neither needed a retry. This supports the woodland revision in
its recorded pair.

Subsequent revisions add woodland ranged cover for thorncallers, one HP for
coastal healers, actual defended-duration tracking for evacuation, and rookie
Ashen reinforcements that earn heavy armor. Fresh coastal and Ashen
players are testing these revisions. The [eighth coastal pair](playtests/tides-8/README.md)
also favors mixed recruitment: 8 / 6 / 5 turns, two troop losses and no retry,
versus 9 / 6 / 7 successful turns, three troop losses and an opening defeat for
reef-only purchases. Both ultimately kept the named companions. Another
independent coastal pair checks variation between players. Final comparisons will be added
when actual campaigns finish. The older Lantern pair used the former ten-gold
adept price; the current price is eight. That difference is disclosed rather
than claiming its records replay a later version.

The [ninth records](playtests/tides-9/README.md) retain two failed Ashen mixed
openings, the restricted player's costly retry and explicitly paused refuge,
and an unfinished Lantern mixed attempt. The Lantern player stopped acting for
nineteen turns; enemy recruitment had already stopped. That capped result is
not a basis for an economy nerf or a claim of balance. Fresh current-source
players are still required.

The quarry now assigns the Ledgerkeeper a post at his southern office. He
guards its immediate approaches, returns by legal movement if displaced and
can still attack or retreat. Regular patrols and reinforcements remain mobile.
An independent review found no implementation blocker, but its bounded local
escape probe is not a whole-campaign win. The tenth cohort tests the encounter
from chapter one.

The independent mechanics review found that the old evacuation objective could
borrow elapsed global turns after a late capture. The current rule counts three
consecutive defended enemy phases after capture, resets on enemy recapture and
waits until at least turn six. Focused checks cover late arrival, recapture,
save/load and repeated turn initialization. Mechanical checks support this rule;
they do not replace the player comparisons.

Distinct veteran sprites remain deferred at the user's request. Browser and
phone appearance are not claimed as verified by these headless trials.
