This document records the frontend’s product values, ownership boundaries, and requirements.

## Values

- **Interaction latency.** Minimize the time needed to perform an action and receive its response.
- **Simplicity.** Use the simplest code sufficient to achieve the other values.
- **Maintainability.** Prevent bugs and regressions by construction wherever practical. Make code easy to understand and change, with small, predictable implementation surfaces that don’t require knowledge of unrelated systems.
- **Performance and scalability.** Control algorithmic complexity, avoid unnecessary work, and keep resource consumption bounded as workloads grow.

## Architectural constraints

- **Preact owns the HUD**, including surrounding menus, dialogs, and settings.
- **Rust owns authoritative game data and logic**, including commands, legality, visibility, and published state.
- **Phaser owns everything else.** An exception requires demonstrated necessity and evidence that the alternative achieves parity or a significant advantage against the values above.
- **HUD audio is an authorized exception:** a lightweight application-owned service may prepare and play HUD cues independently of Phaser. Battlefield audio remains Phaser-owned; both follow the user's sound preference.

## Non-functional requirements

### Readiness and early usability

- Users must not wait for predictably needed resources during an interaction.
- **Loaded means ready to use immediately**, including any required decoding, preparation, or GPU upload—not merely downloaded.
- Starting a map must prepare its predictably needed audio, unit sprites, terrain, items, and other resources before they are needed.
- Playing a campaign must preload upcoming maps and their resources before the player reaches them.
- Show meaningful, working functionality as early as possible. Where functionality cannot yet be available, show stable visuals.
- Independently ready functionality must remain usable while unrelated work loads.

### Correctness and consistency

- Displayed state, selection, available actions, and submitted commands must agree with authoritative game state.
- Obsolete asynchronous work must never overwrite newer state or affect a different session.
- Concurrent operations must not leave partially applied or contradictory state.

### Continuity and stability

- Loading, resizing, opening panels, and receiving updates must preserve the user’s view and ongoing interaction unless the action deliberately requires a change.
- Avoid unintended camera movement, world-scale changes, layout shifts, flicker, and interrupted gestures.

### Explicit failures and recovery

- Failures must be visible and explain what failed. An impossible operation must not appear to succeed.
- Missing resources must produce an explicit failure. A missing texture must not silently become a similar-looking substitute.
- Expected cancellation and ordinary pending work must be distinguished from failures.
- Provide a clean recovery or restart path, preserving progress wherever possible.
- Loading failures must include full backtraces and useful diagnostic context.

### Input, accessibility, and compatibility

- Phone touch, macOS trackpad gestures, and mouse controls are hard requirements and must remain fully supported.
- Preserve keyboard controls for HUD, menus, and dialogs, plus battlefield zoom, zoom reset, and Escape to clear selection. Hex traversal and tile selection through the keyboard are removed by product decision.
- Apply practical accessibility measures to the surrounding HUD, menus, and dialogs: semantic controls, labels, visible keyboard focus, and dialog focus handling.
- Define the supported browser and device scope explicitly.
- Respect accessibility preferences, including reduced motion.
- Full battlefield screen-reader support is outside the current scope. Any future commitment requires an explicit assessment of usability, architectural complexity, and maintenance cost.
- Per-tile semantic controls and equivalent battlefield access through assistive technology are not requirements for this migration. Identify and document any existing accessibility capabilities it removes.

### Behavioral and visual fidelity

- Preserve existing gameplay, artwork, world scale, and interaction behavior unless a change is explicitly approved.
- Preserve native 64×64 artwork and the common terrain/unit world scale.
- Performance improvements must not quietly remove functionality or reduce visual fidelity.

### Resource efficiency

- Bound CPU work, memory, GPU allocations, and network traffic.
- Avoid unnecessary work when the game is stationary or in the background.
- Increasing map, actor, or light counts must increase managed workload without relying on ever-larger individual resources.
- Prefetching and resource retention must satisfy readiness requirements within an explicit supported workload and resource envelope.

## Current rendering bounds and validation scope

The intended input scope is desktop mouse and macOS trackpad, Android touch,
and iOS touch in browsers with WebGL and Web Audio. Automated validation currently
covers Chromium on Linux, including 390×844 at device-pixel ratio 1.25 and
1440×1000 at device-pixel ratio 3. Other browser engines and physical devices
require their own runs; this evidence does not certify them or establish 60 FPS
on hardware. The 60 FPS requirement above remains the target.
The Phaser canvas replaces the earlier labelled SVG tile buttons; individual
hexes no longer expose separate assistive-technology targets or keyboard
traversal. HUD controls and dialogs retain their semantic controls and keyboard
handling.

The renderer checks device limits and requires at least seven vertex attributes,
six varying vectors and three texture units. The main drawing surface is limited to
4,194,304 pixels before integer rounding and to the smaller of 4096 pixels per
dimension and the device's texture, renderbuffer and viewport limits. Terrain
uses 1020-pixel pages with two-pixel gutters instead of a map-sized texture.
The loader downloads at most six concurrent resources. Non-required decoded
images enter GPU/adoption preparation at most once per frame. These bounds
preserve the upstream policy.

The former browser benchmark exercised authored maps and separate scaling axes up
to 40×40 tiles, 96 actors and 48 lights, plus a combined workload that replaces
200 enemies with 30 friendly reinforcements and 200 illuminated death marks,
producing 300 lights. It recorded managed resource
estimates and their peaks; these estimates are not total browser or GPU memory.
These fixtures define the current measured workload envelope, not a rule that
larger games may drop features or miss the frame target. The benchmark is retained
only as a historical source reference in the performance skill; its page and
supporting fixtures are absent from the game. Future device measurements need a
runnable workload using the current game, extending this envelope as content grows.
Linux software-GPU
results can establish workload validity and translation parity, but cannot stand
in for physical-device frame-rate acceptance.
An additional simultaneous-peak workload repopulated the vacated cells with 200
new enemies through the Rust script engine, retaining all 200 death marks and
300 lights while measuring fierce navigation and fixed-viewport redraws.
