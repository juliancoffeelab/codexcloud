export interface SaveFailure {
  kind: "full" | "blocked" | "internal";
  detail: string;
}

/** Capacity errors have cleanup actions; access and game errors need different guidance. */
export function saveFailure(error: unknown): SaveFailure {
  const detail = error instanceof Error ? (error.stack ?? error.message) : String(error);
  const name = error instanceof Error ? error.name : "";
  const kind =
    ["QuotaExceededError", "NS_ERROR_DOM_QUOTA_REACHED"].includes(name) ||
    /QuotaExceededError|NS_ERROR_DOM_QUOTA_REACHED/.test(detail)
      ? "full"
      : name === "SecurityError" || /SecurityError|access.*denied|storage.*disabled/i.test(detail)
        ? "blocked"
        : "internal";
  return { kind, detail };
}
