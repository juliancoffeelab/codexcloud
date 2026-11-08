import { stringPayloads } from "./strings.mjs";

export function operation(adapter, variant, c) {
  const { B, host } = adapter;
  const ours = ["native", "js", "js-record"].includes(variant);
  const wire = variant === "game-wire";
  const fn = (name) => B[(wire ? "wire_" : "") + name];
  const native = (text) => (ours ? adapter.nativeString(text) : text);
  const selected = (text) => (variant === "native" ? native(text) : text);
  let store, counter, call;
  const dispose = () => {
    if (store) {
      ours ? adapter.disposeStore(store) : store.free();
      store = undefined;
    }
    if (counter) {
      ours ? adapter.disposeCounter(counter) : counter.free();
      counter = undefined;
    }
  };
  if (c.kind === "string-control") {
    const text = stringPayloads[c.payload],
      alias = selected(text);
    if (c.operation.startsWith("throws-")) {
      store = B.string_store_new(alias);
      call =
        c.operation === "throws-native-out"
          ? () => B.text_throws_native_out(store, 7, 0)
          : c.operation === "throws-error"
            ? () => B.text_throws_error_code(store)
            : () => B.text_throws_out(store, 7, 0);
    } else if (c.operation.startsWith("result-")) {
      store = B.string_store_new(alias);
      call =
        c.operation === "result-error"
          ? () => B.text_result_error_code(store)
          : c.operation === "result-host"
            ? () => host(B.text_result_unwrap(store))
            : () => B.text_result_unwrap(store);
    } else if (c.operation.startsWith("envelope-")) {
      const labels =
        c.labels === undefined
          ? ["unit_archer_0123", "café"]
          : Array.from({ length: c.labels }, (_, id) =>
              c.labelUnicode ? `弓兵_${id}_café` : `unit_${id}`,
            );
      const value = { revision: 7, body: alias, labels: labels.map(selected) };
      const toHost =
        !ours || variant === "js-record"
          ? (value) => value
          : (value) => ({
              revision: value.revision,
              body: host(value.body),
              labels: value.labels.map(host),
            });
      if (c.operation === "envelope-in") call = () => B.text_envelope_in(value);
      else if (c.operation === "envelope-echo")
        call = () => B.text_envelope_echo(value);
      else {
        store =
          c.labels === undefined
            ? B.string_store_new(alias)
            : B.string_envelope_store_new(alias, c.labels, c.labelUnicode);
        call =
          c.operation === "envelope-host"
            ? () => toHost(B.text_envelope_out(store))
            : () => B.text_envelope_out(store);
      }
    } else if (c.operation === "alias-in") call = () => B.text_in(alias);
    else if (c.operation === "native-literal-in") {
      const value = native(text);
      call = () => B.string_literal_in(value);
    } else if (c.operation === "native-borrowed-in") {
      const value = native(text);
      call = () => B.string_borrowed_in(value);
    } else if (c.operation === "mixed-in") {
      const value = native(text),
        borrowed = native(text + "λ");
      call = () => B.string_mixed(value, borrowed, alias, 7);
    } else if (c.operation === "echo") call = () => B.text_echo(alias);
    else {
      store = B.string_store_new(alias);
      call =
        c.operation === "host"
          ? () => host(B.text_out(store))
          : () => B.text_out(store);
    }
  } else if (["noop", "add", "eight"].includes(c.kind)) {
    const f = fn(c.kind);
    call =
      c.kind === "noop"
        ? () => f()
        : c.kind === "add"
          ? () => f(3, 5)
          : () => f(1, 2, 3, 4, 5, 6, 7, 8);
  } else if (c.kind.startsWith("counter")) {
    const get = fn("counter_get"),
      add = fn("counter_add");
    if (c.kind === "counter-lifecycle")
      call = () => {
        const value = B.counter_new(1);
        const result = add(value, 1);
        ours ? adapter.disposeCounter(value) : value.free();
        return result;
      };
    else {
      counter = B.counter_new(1);
      call =
        c.kind === "counter-get" ? () => get(counter) : () => add(counter, 1);
    }
  } else if (c.kind.startsWith("numeric-")) {
    const arrays = Array.from({ length: 8 }, (_, j) => {
      const values = Array.from({ length: c.n }, (_, i) => i + 0.25 + j * 0.5);
      return (ours && variant === "native") || wire
        ? values
        : Float64Array.from(values);
    });
    const bytes = Array.from({ length: c.n }, (_, i) => i % 256);
    const byteInput =
      (ours && variant === "native") || wire ? bytes : Uint8Array.from(bytes);
    const f = fn(c.kind.replace("-", "_"));
    call =
      c.kind === "numeric-heterogeneous"
        ? () => f(arrays[0], byteInput, 2)
        : c.kind === "numeric-slice"
          ? () => f(arrays[0])
          : c.kind === "numeric-pair"
            ? () => f(arrays[0], arrays[1])
            : c.kind === "numeric-mixed"
              ? () => f(arrays[0], 3.5, arrays[1], 2)
              : () => f(...arrays);
  } else {
    store = B.store_new(
      c.n ?? 1,
      c.depth ?? 0,
      c.width ?? 0,
      c.units ?? 0,
      c.unicode ?? false,
    );
    const [type, direction] = c.kind.split("-");
    const output = fn(type + "_out");
    if (direction === "out") call = () => output(store);
    else if (direction === "host") call = () => host(output(store));
    else {
      const value = output(store),
        f = fn(type + (direction === "in" ? "_in" : "_echo"));
      call = () => f(value);
    }
  }
  return { call, dispose };
}
