import "server-only";

export function getApiBaseUrl() {
  const value =
    process.env.VALYMUX_API_BASE_URL ?? process.env.NEXT_PUBLIC_VALYMUX_API_BASE_URL;

  if (!value) {
    throw new Error(
      "Missing VALYMUX_API_BASE_URL. Set it to the Rust API origin, for example http://127.0.0.1:3000.",
    );
  }

  return value.replace(/\/$/, "");
}
