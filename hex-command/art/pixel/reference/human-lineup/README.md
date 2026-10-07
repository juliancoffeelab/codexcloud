# Current human artwork lineup

[Full teal board](full.png) · [Full red board](full-red.png).
Each proposed branch reads L1 → L2 → L3, left to right.
Repeated L1 sprites show shared parents; cavalry has two distinct starting poses.

- [Infantry](infantry.png)
- [Scouts and archers](ranged.png)
- [Mages and healers](magic.png)
- [Cavalry](cavalry.png)
- [Commander and existing game variants](existing.png)

Red sections: [Infantry](infantry-red.png), [Scouts and archers](ranged-red.png),
[Mages and healers](magic-red.png), [Cavalry](cavalry-red.png),
[Commander and existing game variants](existing-red.png).

The red board uses the same branch order. References that previously had only
teal matrices now also have red matrices and transparent PNGs. Black-clad L3
flankers retain their dark bodies and use muted crimson cloaks.

The board uses the latest canonical `art/pixel/units/` PNGs directly, preserving their palettes and
transparency. It excludes the earlier discarded human-branch transcriptions.
The proposed infantry tiers follow the approved leather → mixed armor → silver
plate sequence. These are artwork trees, not registered gameplay advancement
trees. Current Heavy infantry, Warden and Stone mage remain pictured separately.
Commander/Marshal now has the Grand Marshal L3 study. Other factions currently
have L1/L2 artwork and are not part of this human board.

Regenerate from the project root with:

```sh
node art/pixel/reference/human-lineup/create.cjs
```
