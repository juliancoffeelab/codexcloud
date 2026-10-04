# Remaining PR #5 tasks

Checked on 2026-10-04 against commit `b6f8ef5` for
[PR #5](https://github.com/juliancoffeelab/codexcloud/pull/5).
There are no submitted reviews or inline review threads. The outstanding requests
below come from the two discussion comments and the PR description, checked against
the current source, plus the user's subsequent woodland campaign feedback in chat.
Discussion comments do not have a resolved/unresolved flag.

Sources:

- [Original TODO and design followups](https://github.com/juliancoffeelab/codexcloud/pull/5#issuecomment-5973837666)
- [Playtest issues](https://github.com/juliancoffeelab/codexcloud/pull/5#issuecomment-5980397700)

## Gameplay and UI

- [ ] Fix the grove visit reward triggering during unrelated actions. The reported
      symptom was receiving the grove's six coins when visiting a village. In
      `lantern-bell`, an adept starts on the reward's trigger tile (`4,3`), and
      `engine.ts` treats a visit as current occupancy during a campaign update.
      Establish the intended visit timing and prevent starting occupancy or carryover
      from causing a delayed, unrelated trigger. The exact reported sequence has not
      been reproduced here.
- [ ] Require an adept to seal the Lantern Road finale's binding stones. Currently,
      `engine.ts` seals them for any player unit. Align the objective instructions and
      terrain help with the eventual rule.
- [ ] Stop showing completed objectives as active map targets. `battlefield-view.ts`
      adds every tile returned by `goalProgress`, including completed goals and sealed
      stones. Handle individual completed stones as well as whole objectives; preserve
      markers for objectives whose progress can still be lost during battle.
- [ ] Mark "Keep your leader alive" complete after victory. Its entry in
      `CampaignPanel.svelte` is static, unlike the other objective entries.

## Artwork

- [ ] Give villagers and other civilian speakers a worker portrait rather than the
      scout sprite. Reed gatherers and Villager Hesta currently use `scout`;
      `DialogueLine.portrait` is restricted to combat `UnitType`. Support civilian art
      without inventing a combat unit solely for its portrait. Agree on the design with
      the user before creating it.
- [x] Transcribe the remaining unit artwork into editable pixel matrices using the
      agreed references and fixed-resolution workflow. Soldier, archer, commander,
      warden, wildkeeper, thorncaller and enemy scout now join the existing unit
      matrices. Completed as 26 unit and 12 terrain matrices at 48×48, retaining
      existing terrain designs and the approved unit poses. Public atlases are
      rendered solely from these sources.
- [x] Fix the missing or inconsistent dark outlines on pixel wolves and skeleton
      warriors, for both factions. Match the clear silhouette and component outlines
      of the skeleton archer and boar so pale bones, grey fur, limbs and weapons
      remain readable against the terrain. The user's screenshots `15454.jpg` and
      `15456.jpg` show the discrepancy in the game. The existing matrices do not
      resolve this issue merely by being native pixel artwork.
- [x] Restore the boar's darker hide and the thorncaller's saturated orange-red fur.
- [x] Correct material artifacts in the new transcriptions: smooth gold trim and
      cloth regions, remove ivory/steel flecks from cloaks and gold from unrelated
      components, and use fur-colored thorncaller paws. Palette membership alone
      does not establish that a pixel belongs to its component.
- [ ] Review the remaining low-poly artwork for inconsistent style with the user.
      The original comment raised this separately from the cartoonish pixel designs;
      replacing approved designs without input is not authorized.

## Writing and quest design

The user authorized improving the woodland/Green Accord campaign: start with a
coherent story, then derive the mission goals and conflicts from it. Pixel
refactoring takes priority. Other creative changes still require the user's input.
Suggestions are welcome.

- [ ] Review campaign dialogue and chapter transitions for coherence, especially
      the Beastland/Green Accord witness, necromancer conflict and Dain's involvement.
      Explain how player actions lead to the next mission rather than relying on
      unexplained transitions.
- [ ] Resolve the first woodland mission's purpose and causal outcome with the user.
      In `green-border`, owning all three marked groves immediately wins the mission;
      the debrief then declares that cutting stopped and Dain accepted a truce.
      There is no separate woodcutting or surrender condition establishing why
      capturing those locations stops him. The user found the stage's purpose
      unclear. Agree on the relationship between the objective and the outcome,
      rather than just adding explanatory dialogue to justify the existing rule.
- [ ] Resolve the second woodland mission's truce and conflict with the user.
      `green-vigil` describes a personal truce with Dain, while the enemy units belong
      to the Overseer and continue attacking. This distinction was not clear in play.
      Dain is absent from the vigil map and its reinforcement waves: he exists only
      as a dialogue speaker claiming to watch from the treeline, then appears as a
      player warden in stage three. Account for his presence and role in stage two
      rather than implying an on-map truce participant who does not exist.
      The user understands the vigil's purpose somewhat, but asks why attacks occur
      under a truce and what challenge the stage would have if attacks ceased.
      Agree on who the truce covers and what threatens the vigil before changing
      story, factions or mission rules; do not simply remove attacks and leave an
      unchallenged waiting objective.
- [ ] Establish who the Overseer is and the woodland campaign's chain of authority
      and motives with the user. The text associates him with timber cutting,
      soldiers and Dain's employer, but does not explain his office or why the
      soldiers obey him when Dain makes a truce. It says he attacks the vigil to
      suppress evidence that the Rootbinder killed and raised the woodcutters for
      him, without establishing why he wanted those woodcutters killed. Introduce
      these relationships coherently; do not invent missing lore during bug fixes.
- [ ] Clarify Dain's allegiance and relationship to the necromancer in the skeleton
      campaign. Preserve the praised immortality/service exchange and the
      "we haven't decided what we'll do" conversation.
- [ ] Clarify why lighting the lanterns makes the necromancer withdraw. The user
      reported that the causal explanation was unclear or missing.
- [ ] Reconsider optional quests that feel like arbitrary six-gold rewards. Agree
      on meaningful purposes and rewards with the user; the grove trigger bug above is
      a separate implementation issue.

## Requested followups

- [ ] Design unit progression and branching advancement, starting with peasant →
      soldier → heavy infantry and extending to the other unit types.
- [ ] Review combat balance, including resistances, stronger attacks and abilities.
      These are design decisions requiring thought and user input, not blanket stat
      increases.

## Pending visual confirmation

- [ ] Confirm whether the reported mobile stray lines remain. The PR description
      explicitly leaves their visual correction unconfirmed; crop containment changes
      alone do not establish that the visible problem is gone.

## Requests already addressed in the current implementation

Excluded from the outstanding checklist: withdrawal based on lethal retaliation;
one-movement forest/grove travel for woodland beasts; numeric terrain base movement
costs plus unit modifiers; and dialogue back navigation with matching arrow
controls and no skip button. The approved pixel replacements and shared public
unit/terrain atlases are also present. This document does not claim a new playtest
or visual approval of those changes.
