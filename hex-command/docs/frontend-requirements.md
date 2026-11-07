This document records the frontend’s product values, ownership boundaries, and requirements.

## Values

- **Interaction latency.** Minimize the time needed to perform an action and receive its response.
- **Simplicity.** Use the simplest code sufficient to achieve the other values.
- **Maintainability.** Prevent bugs and regressions by construction wherever practical. Make code easy to understand and change, with small, predictable implementation surfaces that don’t require knowledge of unrelated systems.
- **Performance and scalability.** Control algorithmic complexity, avoid unnecessary work, and keep resource consumption bounded as workloads grow.

## Architectural constraints

- **Svelte owns the HUD**, including surrounding menus, dialogs, and settings.
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
