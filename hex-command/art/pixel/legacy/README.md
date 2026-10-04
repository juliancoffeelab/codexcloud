# Original pixel artwork

These original sheets and `frames.json` provide the artwork that has no editable
matrix yet. They are build inputs, kept outside `public/assets/`.

`scripts/render-pixel.py --atlas` copies the recorded crops without resampling.
Existing unit matrices replace their matching legacy frames. The output is one
unit atlas (`public/assets/units-pixel.png`) and one terrain atlas
(`public/assets/terrain-pixel.png`). Fractional legacy crop offsets are preserved.
