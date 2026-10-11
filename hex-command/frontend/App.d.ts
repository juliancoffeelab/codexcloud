/** React embedding of the immutable OCaml battle owner. */
import type { ComponentType } from "react";
import type { t as Session } from "./Session";
export const make: ComponentType<{ session: Session; onMount: () => () => void }>;
