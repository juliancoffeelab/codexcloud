import Phaser from "phaser";
import { SpatialIndex, type PaintBounds } from "./spatial-index";
/** Container.remove restores exclusive children to the Scene display list.
 * Retained offscreen roots must leave both lists, and remove Phaser 3.90's
 * onChildDestroyed listener before a later Container.add registers it again. */
export function detachRoot(
  parent: Phaser.GameObjects.Container,
  root: Phaser.GameObjects.GameObject,
) {
  if (!root.scene) return;
  parent.remove(root, false);
  root.removeFromDisplayList();
  const handler = (
    parent as Phaser.GameObjects.Container & {
      onChildDestroyed: (object: Phaser.GameObjects.GameObject) => void;
    }
  ).onChildDestroyed;
  root.off(Phaser.GameObjects.Events.DESTROY, handler, parent);
}
type RecordEntry = {
  key: string;
  parent: Phaser.GameObjects.Container;
  root: Phaser.GameObjects.GameObject;
  order: number;
  tie: number;
};
/** Detaching offscreen roots removes hidden-child walks from Phaser Containers. */
export class BattlefieldVisibility {
  private index = new SpatialIndex<RecordEntry>(256);
  private records = new Map<string, RecordEntry>();
  private active = new Set<RecordEntry>();
  private tie = 0;
  private dirty = true;
  private signature = "";
  set(
    key: string,
    parent: Phaser.GameObjects.Container,
    root: Phaser.GameObjects.GameObject,
    bounds: PaintBounds,
    order: number,
  ) {
    let entry = this.records.get(key);
    if (!entry) {
      entry = { key, parent, root, order, tie: this.tie++ };
      this.records.set(key, entry);
      if (root.parentContainer === parent) detachRoot(parent, root);
    }
    entry.order = order;
    this.index.set(entry, bounds);
    this.dirty = true;
  }
  delete(key: string) {
    const entry = this.records.get(key);
    if (!entry) return;
    this.index.delete(entry);
    this.records.delete(key);
    this.active.delete(entry);
    this.dirty = true;
  }
  update(bounds: PaintBounds) {
    const signature = `${bounds.left}:${bounds.top}:${bounds.right}:${bounds.bottom}`;
    if (!this.dirty && signature === this.signature) return;
    this.signature = signature;
    this.dirty = false;
    const entries = this.index.query(bounds),
      visible = new Set(entries);
    const parents = new Set<Phaser.GameObjects.Container>();
    for (const entry of this.active)
      if (!visible.has(entry)) {
        detachRoot(entry.parent, entry.root);
        parents.add(entry.parent);
      }
    entries.sort((a, b) => a.order - b.order || a.tie - b.tie);
    for (const entry of entries) {
      if (!entry.root.parentContainer) entry.parent.add(entry.root);
      parents.add(entry.parent);
    }
    // Compare only active lists. Each parent retains its own global ordinal order.
    for (const parent of parents) {
      const ordered = entries.filter((entry) => entry.parent === parent).map((entry) => entry.root);
      for (let i = 0; i < ordered.length; i++)
        if (parent.list[i] !== ordered[i]) parent.moveTo(ordered[i]!, i);
    }
    this.active = visible;
  }
  clear() {
    for (const entry of this.active) detachRoot(entry.parent, entry.root);
    this.records.clear();
    this.active.clear();
    this.index.clear();
    this.dirty = true;
  }
}
