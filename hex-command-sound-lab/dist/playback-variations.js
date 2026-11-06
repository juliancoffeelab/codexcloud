// Non-destructive variations for game-style triggers, never exact recording auditions.
export class PlaybackVariations {
  constructor(random = Math.random) { this.random = random; this.bags = new Map(); }
  draw(key, values) {
    let state = this.bags.get(key);
    const signature = JSON.stringify(values);
    if (!state || state.signature !== signature) state = { signature, bag: [], last: undefined };
    if (!state.bag.length) {
      state.bag = [...values];
      for (let i = state.bag.length - 1; i > 0; i--) {
        const j = Math.floor(this.random() * (i + 1));
        [state.bag[i], state.bag[j]] = [state.bag[j], state.bag[i]];
      }
      if (state.bag.length > 1 && state.bag.at(-1) === state.last)
        [state.bag[0], state.bag[state.bag.length - 1]] = [state.bag.at(-1), state.bag[0]];
    }
    state.last = state.bag.pop(); this.bags.set(key, state); return state.last;
  }
  plan(ref, action) {
    if (!ref || ref.take === null) return ref;
    const key = `${action.key}/${ref.id}/${ref.take ?? 0}`;
    const spread = action.key === 'death' ? 1.2 : action.key === 'move' ? .35 : ref.levelClass === 'terrain' ? .6 : .8;
    const semitones = this.draw(`${key}/pitch`, [-1, -2/3, -1/3, 0, 1/3, 2/3, 1]) * spread;
    return { ...ref, variationRate: 2 ** (semitones / 12), varied: true };
  }
}
