import { beforeEach, vi } from "vitest";
// Asset upload/rendering has a browser gate. Node suites do not bake PNG pages.
vi.mock("virtual:terrain-prebuilt", () => ({ default: {} }));
beforeEach(() => {
  // Node controller suites have no browser decoder; native renderer loading is
  // exercised by the browser probes rather than this terrain-prefetch hint.
  if (typeof Image === "undefined")
    vi.stubGlobal(
      "Image",
      class {
        set src(value) {
          this.url = value;
        }
      },
    );
});
