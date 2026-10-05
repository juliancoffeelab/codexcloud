# PR followups

Updated on 2026-10-04 for [PR #6](https://github.com/juliancoffeelab/codexcloud/pull/6).
The original requests came from [PR #5](https://github.com/juliancoffeelab/codexcloud/pull/5)
and subsequent playtest feedback. Discussion comments do not have a resolution flag.

Sources:

- [Original TODO and design followups](https://github.com/juliancoffeelab/codexcloud/pull/5#issuecomment-5973837666)
- [Playtest issues](https://github.com/juliancoffeelab/codexcloud/pull/5#issuecomment-5980397700)

## Completed gameplay and UI fixes

- [x] Visit rewards fire on a player entering the named hex, once per mission.
      Starting occupancy, unrelated moves, turn changes and reloads do not count.
      The Lantern finale's reward grove is now at unoccupied `5,1`, beside the
      northern bridge, rather than underneath the starting adept at `4,3`.
      Old saves keep units, gold and earned rewards when the site moves. Checks
      cover visit placement, arrival, starting occupancy, undo and repeat visits.
- [x] Lantern Road's final binding stones require adepts. Other campaigns retain
      their existing sealing rule. Objective text and contextual terrain help
      describe the required unit and sealed state.
- [x] Permanently completed stones and ledger objectives lose their active map
      markers. Reversible control/escort/hold goals remain marked until battle ends.
- [x] Leader and required-companion survival objectives show completion at victory.
- [x] Dain cannot be recalled as a duplicate unnamed recruit when the following
      chapter already includes him as a story companion.

## Completed pixel work

- [x] All 68 faction combat-unit frames and 12 terrain frames have editable 48×48 matrices.
      One unit atlas and one terrain atlas in `public/assets` are rendered from
      those sources and used by the ordinary game renderer.
- [x] Wolves and skeleton warriors have dark silhouette outlines, retaining their
      approved poses. The boar has darker hide; thorncaller fur has saturated orange.
- [x] Correct material artifacts in the transcriptions, including smooth gold trim,
      cloak/cloth planes, thorncaller paws and armored boots. Remove the green flecks
      inside the keep's stonework and grey specks from the fort's wooden fence.
      Plains grass tufts use coherent flat regions. Latest renders were inspected
      on dark backgrounds and on the map; matrix/palette equality alone is not
      evidence of correct material placement.
- [x] Civilian speakers use a neutral worker portrait, stored as an editable matrix
      and packed in the shared unit atlas. It is a portrait, not a combat unit.
      Reed gatherers and the village elder no longer borrow the scout design.
- [x] Replace runtime atlas cropping for every art style with source-resolution
      frame extraction at load time. Units, terrain and dialogue portraits receive
      standalone images containing only that frame's pixels. Remove the offset SVG
      atlas images and HTML/foreignObject cropping paths entirely.
- [x] Pixel terrain uses its palette's actual ground colour beneath the transparent
      raster edges. The legacy grey/dark underlay no longer creates a stepped fringe
      around ruins and other hexes. All twelve terrain backgrounds are checked
      against their source pixels; phone confirmation remains a separate check.

## Completed woodland campaign rewrite

Story and mechanics are specified in [design/green-accord.md](design/green-accord.md).
The user explicitly authorized rewriting this campaign; that authorization does
not extend to rewriting the other campaigns.

- [x] The first mission captures both logging supply camps and recovers the missing
      workers' pay ledger. Merely standing at cutting markers no longer wins.
- [x] The Overseer is the council's timber/toll officer. He diverts wages and hands
      protesting workers to the Rootbinder, who kills and binds them. The bargain
      gives the Overseer unpaid labor and the Rootbinder bodies and river access.
- [x] Dain's personal truce covers his patrol. Other charter guards attack the vigil
      to suppress evidence. Living Dain is present as a playable ally in stage two;
      his death persists into later stages and changes the story.
- [x] The vigil produces the workers' memories in a living branch. Stage three
      secures a crossing and escorts Lark to the council, which revokes the charter.
      Tamsin joins here. The ending does not make an undefeated Rootbinder disappear.
- [x] Old conversations remain recognizable to the save loader; earned victories
      remain earned. Old vigil saves gain living Dain without resurrecting a Dain
      whose death was recorded. Current-stage deaths and retries are checked.

## Still requiring design input

- [ ] Review inconsistent low-poly artwork with the user before replacing designs.
- [ ] Clarify Dain's allegiance in the skeleton campaign, preserving the praised
      immortality/service exchange and the undecided future conversation.
- [ ] Clarify why lighting the Lantern Road lanterns forces the necromancer away.
- [ ] Review the remaining optional quest rewards with the user. The woodland pay
      ledger now serves a required evidence objective and has no arbitrary gold bonus.
- [x] Implement the subsequently authorized progression task: all 17 combat-unit
      families have beginner and veteran ranks, experience, promotion actions and
      retained campaign advancement. Swordsmen and spearmen earn heavy armor and
      blunt weapons. Branching alternatives are not part of this first ladder.
- [ ] Finish the subsequently authorized balance pass with matched Luna campaign
      trials, preserving failed attempts and replaying actual commands. Mechanical
      correctness alone does not establish that diverse armies pay off.
      The user extended work to 13:00 Kyiv. Eleventh-cohort coast and woodland
      favor the recorded mixed armies overall; Lantern control loses its sole
      caster, and Ashen mixed still has worse attrition. Twelfth-cohort matched
      Ashen/Lantern trials test revised bone bows. Actual outcomes and limits are
      preserved in [the balance report](docs/balance-playtests.md).
      Guided fourteenth-cohort Ashen mixed finishes 8/6/8 without retries, with
      eight losses including Dain; its matching control remains in progress.
      The twelfth Lantern control reaches the village turn cap without capturing
      a village. Neither unfinished pairs nor known lethal escort moves are
      claimed as completed evidence that recruitment diversity is balanced.

## Overnight race and coastal work

Specified in [design/tides-and-progression.md](design/tides-and-progression.md).

- [x] Finite terrain costs and explicit passability, with race then unit overrides,
      shared by movement, recruitment, recall, waves and save validation.
- [x] Race resistance defaults and race/unit terrain damage modifiers. Merfolk swim,
      retain ordinary plains footing, suffer exposure in woods/hills and gain marsh
      protection. Ground races are exposed in marsh as in shallow water.
- [x] Three authored coastal chapters with a diversion, worker evacuation and sluice
      restoration story. Named casualties persist, retries restore checkpoints,
      and old coastal saves retain their units while migrating officer identities.
- [x] Occupying woodland and Lantern village guards defend their mission objectives
      instead of pursuing the starting company across the entire map. They retake
      captured posts and use the same combat, movement and retreat rules.
- [x] Give every advancement distinct pixel artwork and inspect the full roster for
      consistent humanoid scale and material detail. The user explicitly placed this
      after balance. All seventeen family edits are rendered; independent visual
      review caught and verified the corrected second harpoon hook, crystal
      alignment and Lich chest plate. Both factions retain matching anatomy,
      approved skull pixels and connected limbs/fangs. Published in the ordinary
      game on 2026-10-05; the developer map includes both ranks of every family.

## Pending device confirmation

- [x] Fix the blank-page deployment package: Git-ignored generated JavaScript was
      absent from the uploaded archive despite being referenced by its HTML.
      Package the complete build and verify the actual archive's client references
      and bytes. A startup panel also displays backtraces until the app mounts,
      with a reload action that retains the saved battle.
- [ ] Confirm the reported mobile stray lines and portrait leakage on the user's
      phone after replacing atlas cropping with isolated frame images. Source
      extraction is verified; phone appearance has not yet been confirmed.

## Earlier fixes retained

Withdrawal based on lethal retaliation; one-movement forest/grove travel for
woodland beasts; numeric terrain movement costs plus unit modifiers; matching
dialogue navigation arrows with no skip button; developer showcase as an ordinary
map listed last in the picker. These are separate from the fixes above.
