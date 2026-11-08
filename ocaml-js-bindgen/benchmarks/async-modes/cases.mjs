export const cases = [];
for (const mode of [
  "plain-melange-promise",
  "borrowed-melange-promise",
  "owned-melange-promise",
  "owned-js_of_ocaml-promise",
  "owned-js_of_ocaml-lwt",
  "owned-rescript-promise",
  "completion-melange-promise",
  "completion-js_of_ocaml-promise",
  "completion-js_of_ocaml-lwt",
  "completion-rescript-promise",
  "callbacks-melange-promise",
  "callbacks-js_of_ocaml-promise",
  "callbacks-rescript-promise",
]) {
  const operations = mode.startsWith("callbacks")
    ? ["callback_once"]
    : mode.startsWith("completion")
      ? [
          "text_native",
          "text_js",
          "point",
          "text_result",
          "text_error",
          "boolean",
          "wide",
          "unit",
          "host_echo",
        ]
      : [
          "ready",
          "suspended",
          ...(mode.startsWith("plain") ? [] : ["borrowed"]),
          ...(mode.startsWith("owned") ? ["owned", "nested"] : []),
        ];
  for (const op of operations) cases.push({ mode, op });
}

// These functions are the calls being measured, including the client-side
// decoding/ownership needed to use the generated runtime's private entrypoints.
export function operation(api, op) {
  const owner = op === "borrowed" ? api.call_counter(7) : undefined;
  const host = { tag: 7 };
  const decode = (r) => {
    if (r && typeof r === "object" && "scalarError" in r)
      return { error: r.scalarError };
    if (!r || typeof r !== "object" || !("position" in r)) return r;
    let value;
    if (op === "text_native") value = api.r_string(r);
    else if (op === "text_js") value = api.r_js_string(r);
    else if (op === "text_result" || op === "text_error")
      value = api.r_u8(r) === 0 ? api.r_js_string(r) : { error: api.r_u32(r) };
    else if (op === "point") {
      const at = r.take(16),
        d = r.view();
      value = [d.getFloat64(at, true), d.getFloat64(at + 8, true)];
    } else if (op === "boolean") value = api.r_bool(r);
    else if (op === "wide") value = api.r_u64(r);
    else if (op === "unit") value = undefined;
    else if (op === "host_echo") value = api.r_host(r);
    else if (op === "owned") value = api.r_handle_counter(r);
    else if (op === "nested") {
      api.r_u8(r);
      value = { value: api.r_u32(r), owner: api.r_handle_counter(r) };
    } else value = api.r_u32(r);
    api.finish?.(r);
    return value;
  };
  if (op === "callback_once") {
    const callback = (r) => {
      const value = r.view().getUint32(r.take(4), true);
      api.finish?.(r);
      return value;
    };
    return {
      call: () => api.call_callback_once(7, callback),
      dispose: () => {},
    };
  }
  const awaitTask =
    api.task_await ??
    ((token) =>
      new Promise((resolve, reject) => {
        const ok = (value) => {
          resolve(value);
          return true;
        };
        const error = (value) => {
          reject(value);
          return true;
        };
        const boundary = (value) => {
          reject(Error(value));
          return true;
        };
        if (api.task_cancel)
          api.task_receive(
            token,
            ok,
            () => {
              reject(Error("cancelled"));
              return true;
            },
            error,
            boundary,
          );
        else api.task_receive(token, ok, error, boundary);
      }));
  const fn = api[`call_${op}`];
  return {
    call: () => {
      const token =
        op === "borrowed"
          ? fn(owner)
          : op === "host_echo"
            ? fn(host)
            : ["ready", "suspended", "owned", "nested"].includes(op)
              ? fn(7)
              : fn();
      const pending = awaitTask(api.task_map(token, decode));
      if (op === "owned")
        return pending.then((value) => api.dispose_Counter(value));
      if (op === "nested")
        return pending.then((value) => api.dispose_Counter(value.owner));
      return pending;
    },
    dispose: () => {
      if (owner) api.dispose_Counter(owner);
    },
  };
}
