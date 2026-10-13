# TODO

## Design

- [ ] Clarify Dain's allegiance in the skeleton campaign, preserving the
      immortality/service exchange and the undecided future conversation.
- [ ] Clarify why lighting the Lantern Road lanterns forces the necromancer away.
- [ ] Review remaining optional quest rewards with the user.
- [ ] Add the woodland Thorncaller gameplay branch for the Thorner after designing
      its abilities. Approved caster artwork is integrated in `pixel/art/units/woodland/`
      and `pixel/art/units/lineups/woodland.json`: shared Thorncaller L1; healer
      Herbkeeper L2 / Bloomweaver L3; Thorner Bramble warden L2 / Thornbinder L3.
      Their matched 45px bodies and the L2 Thorner eye/muzzle fix are preserved.
      Gameplay abilities remain separate from this artwork pass.

## Balance and simulations

- [x] Separate Recruiter, Campaign and Planner blocks; share exhaustive actor
      routing for Vigil, Escort, Escape, typed Seal and Capture objectives, and
      remove stale shelter targets when Refuge encounters escalate.
- [ ] Investigate coordinated attacks when individually unfavorable trades can
      finish a defended enemy; campaign route fixes do not solve this combat
      stalemate.

- [ ] Align simulation-player roles with infantry-only, fair use of every roster
      unit with a stable recruitment pool, and situational unit selection. All
      three must intelligently complete campaigns and objectives; current
      Rush/Mixed/Priority/Battler do not yet fully implement these distinctions.
- [ ] Validate those players against the same Classic enemy in matched arenas
      and complete campaigns: fair-roster and situational should outperform
      infantry-only, with fair-roster remaining reasonably close to situational.
      Keep these balance checks and campaign competence when reducing compute
      for larger maps, longer campaigns and more units.
- [ ] Finish the balance pass with matched campaign and arena trials, preserving
      failed attempts and recording actual commands.
- [ ] Run updated comparisons as needed for balance work. Keep campaign and arena
      rosters, scenarios and results separate.
- [ ] Review veteran boar attacks separately after the rookie balance changes.

## Merfolk campaign

- [ ] Consider terrain-aware shoreline sprites.

## Saves

- [ ] Simplify saves to snapshots of the resolved game state after completed
      actions. Do not save unexecuted AI commands, queued moves or planning
      workspaces. Rebuild AI plans after loading. Keep turn progress and any
      remaining recruitment quota in the game state so completed actions are
      neither repeated nor undone.

## Mobile rendering

- [ ] Investigate and fix mobile stray lines and portrait leakage that remain
      after replacing atlas cropping with isolated frame images.
