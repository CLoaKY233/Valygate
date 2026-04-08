import { Model, Provider, VirtualKey } from "./types";

export function formatDateTime(value: string | null) {
  if (!value) {
    return "Never";
  }

  return new Intl.DateTimeFormat("en", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

export function formatRelative(value: string | null) {
  if (!value) {
    return "No activity yet";
  }

  const now = Date.now();
  const then = new Date(value).getTime();
  const minutes = Math.round((then - now) / 60000);
  const absMinutes = Math.abs(minutes);

  if (absMinutes < 1) {
    return "just now";
  }

  if (absMinutes < 60) {
    return new Intl.RelativeTimeFormat("en", { numeric: "auto" }).format(
      minutes,
      "minute",
    );
  }

  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 24) {
    return new Intl.RelativeTimeFormat("en", { numeric: "auto" }).format(
      hours,
      "hour",
    );
  }

  const days = Math.round(hours / 24);
  return new Intl.RelativeTimeFormat("en", { numeric: "auto" }).format(days, "day");
}

export function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

export function compactNumber(value: number) {
  return new Intl.NumberFormat("en", {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(value);
}

export function providerAccent(provider: string) {
  switch (provider) {
    case "google-genai":
      return "var(--accent-amber)";
    default:
      return "var(--accent-blue)";
  }
}

export function summarizeAllowedModels(key: VirtualKey) {
  if (key.allowed_models.length === 0) {
    return "All synced models";
  }

  if (key.allowed_models.length === 1) {
    return key.allowed_models[0];
  }

  return `${key.allowed_models.length} scoped aliases`;
}

export function summarizeRoutes(key: VirtualKey) {
  if (key.model_routes.length === 0) {
    return "Open routing";
  }

  return `${key.model_routes.length} route${key.model_routes.length === 1 ? "" : "s"}`;
}

export function computeOverviewStats(
  providers: Provider[],
  keys: VirtualKey[],
  models: Model[],
) {
  const activeProviders = providers.filter((provider) => provider.enabled).length;
  const syncingProviders = providers.filter(
    (provider) => provider.sync_status === "syncing" || provider.sync_status === "pending",
  ).length;
  const failedProviders = providers.filter((provider) => provider.sync_status === "failed").length;
  const completedProviders = providers.filter(
    (provider) => provider.sync_status === "completed",
  ).length;

  const activeKeys = keys.filter((key) => key.enabled).length;
  const expiringKeys = keys.filter((key) => {
    if (!key.expires_at) {
      return false;
    }

    const diff = new Date(key.expires_at).getTime() - Date.now();
    return diff > 0 && diff < 1000 * 60 * 60 * 24 * 14;
  }).length;

  const modelsByProvider = providers.reduce<Record<string, number>>((acc, provider) => {
    acc[provider.provider] = (acc[provider.provider] ?? 0) + provider.model_count;
    return acc;
  }, {});

  return {
    providerCount: providers.length,
    activeProviders,
    syncingProviders,
    failedProviders,
    completedProviders,
    keyCount: keys.length,
    activeKeys,
    expiringKeys,
    modelCount: models.length,
    modelsByProvider,
  };
}
