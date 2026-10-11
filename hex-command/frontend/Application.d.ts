/** Mounts the actual React application after native initialization. */
import type { Root } from "react-dom/client";
export function mount(root: Root, target: Element, current: () => boolean): Promise<() => void>;
