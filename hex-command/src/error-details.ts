/** Preserve the original asynchronous failure and its cause chain for recovery UI. */
export function errorDetails(error: unknown): string {
  if (error instanceof Error) {
    const detail = error.stack ?? error.message;
    return error.cause ? `${detail}\nCaused by: ${errorDetails(error.cause)}` : detail;
  }
  return typeof error === "object" && error !== null ? JSON.stringify(error) : String(error);
}
