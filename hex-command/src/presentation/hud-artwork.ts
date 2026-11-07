import type { FrameImage } from "./art";

type ImageRequest = (frame: Readonly<FrameImage>) => Promise<HTMLImageElement>;

/** Shared decoded HUD sources, supplied by Phaser's loader rather than a competing batch. */
export class HudArtworkCache {
  private images = new Map<string, HTMLImageElement>();
  private request?: ImageRequest;

  bind(request: ImageRequest) {
    this.request = request;
    return () => {
      if (this.request === request) this.request = undefined;
    };
  }
  adopt(url: string, image: HTMLImageElement) {
    this.images.set(url, image);
  }
  ready(frame: Readonly<FrameImage>) {
    return this.images.get(frame.url);
  }
  load(frame: Readonly<FrameImage>): Promise<HTMLImageElement> {
    const image = this.ready(frame);
    if (image) return Promise.resolve(image);
    if (!this.request)
      return Promise.reject(new Error(`Artwork loader unavailable: ${frame.key ?? frame.url}`));
    return this.request(frame);
  }
  retry(frame: Readonly<FrameImage>) {
    return this.load(frame);
  }
  dispose() {
    this.request = undefined;
    this.images.clear();
  }
}
