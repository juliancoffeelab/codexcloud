import * as Session from "@frontend/Session";
import * as Action from "@frontend/Action_presentation";
// Explicit silent renderer for controller-only tests. Browser probes verify
// production frame, transition and actual audio completion before acknowledgments.
export function attachCompletedRenderer(session, sessionApi = Session, actionApi = Action) {
  return sessionApi.subscribe(session, (view) =>
    queueMicrotask(() => {
      view.combat_playback?.skip();
      if (view.action) actionApi.complete(actionApi.claim(view.action));
    }),
  );
}
export async function createHeadlessSession(...args) {
  const session = await Session.create_async(...args);
  attachCompletedRenderer(session);
  return session;
}
