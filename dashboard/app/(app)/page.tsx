import Link from "next/link";
import {
  Server,
  KeyRound,
  Cpu,
  Activity,
  ArrowUpRight,
  TrendingUp,
  Shield,
} from "lucide-react";

import { listModels, listProviders, listVirtualKeys } from "@/lib/api";
import {
  compactNumber,
  computeOverviewStats,
  formatRelative,
  summarizeAllowedModels,
  summarizeRoutes,
} from "@/lib/format";

import { ProviderBarChart, SyncDonut } from "@/components/charts";
import { SectionHeader, StatusPill } from "@/components/ui";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

function toneForStatus(status: string) {
  switch (status) {
    case "completed":
      return "success" as const;
    case "failed":
      return "danger" as const;
    case "syncing":
    case "pending":
      return "warning" as const;
    default:
      return "neutral" as const;
  }
}

export default async function OverviewPage() {
  const [providers, keys, models] = await Promise.all([
    listProviders(),
    listVirtualKeys(),
    listModels(),
  ]);

  const stats = computeOverviewStats(providers, keys, models);
  const recentProviders = [...providers]
    .sort((a, b) => (b.updated_at > a.updated_at ? 1 : -1))
    .slice(0, 4);
  const recentKeys = [...keys]
    .sort((a, b) => (b.updated_at > a.updated_at ? 1 : -1))
    .slice(0, 4);

  const barData = Object.entries(stats.modelsByProvider).map(
    ([name, models]) => ({ name, models }),
  );

  const statCards = [
    {
      icon: Server,
      value: stats.providerCount,
      label: "Provider credentials",
      sub: `${stats.activeProviders} enabled`,
      accent: "text-indigo-600",
      bg: "bg-indigo-50",
    },
    {
      icon: KeyRound,
      value: stats.keyCount,
      label: "Virtual keys",
      sub: `${stats.activeKeys} live`,
      accent: "text-emerald-600",
      bg: "bg-emerald-50",
    },
    {
      icon: Cpu,
      value: stats.modelCount,
      label: "Usable models",
      sub: "From synced catalog",
      accent: "text-sky-600",
      bg: "bg-sky-50",
    },
    {
      icon: Activity,
      value: stats.syncingProviders,
      label: "In motion",
      sub: "Pending or syncing",
      accent: "text-amber-600",
      bg: "bg-amber-50",
    },
  ] as const;

  const scopedKeys = keys.filter((k) => k.allowed_models.length > 0).length;
  const unrestrictedKeys = keys.filter(
    (k) => k.allowed_models.length === 0,
  ).length;
  const routedKeys = keys.filter((k) => k.model_routes.length > 0).length;

  return (
    <div className="flex flex-col gap-8">
      {/* Header */}
      <SectionHeader
        title="Gateway overview"
        description="Live control-plane data from your ValyMux API."
        actions={
          <div className="flex items-center gap-2">
            <Link
              href="/providers"
              className="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-1.5 text-sm font-medium transition-colors hover:bg-muted/60"
            >
              <Server className="size-3.5" />
              Providers
            </Link>
            <Link
              href="/virtual-keys"
              className="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
            >
              <KeyRound className="size-3.5" />
              Virtual keys
            </Link>
          </div>
        }
      />

      {/* Stat cards */}
      <div className="grid grid-cols-2 gap-4 xl:grid-cols-4">
        {statCards.map((s) => (
          <Card key={s.label}>
            <CardContent className="flex items-center gap-4 px-5 py-5">
              <div
                className={cn(
                  "flex size-9 shrink-0 items-center justify-center rounded-md",
                  s.bg,
                )}
              >
                <s.icon className={cn("size-4", s.accent)} />
              </div>
              <div className="min-w-0">
                <p className="text-2xl font-bold tracking-tight">{s.value}</p>
                <p className="text-xs uppercase tracking-wide text-muted-foreground">
                  {s.label}
                </p>
                <p className="mt-0.5 text-xs text-muted-foreground/70">
                  {s.sub}
                </p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Charts row */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <p className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
              Catalog supply
            </p>
            <CardTitle>Models by provider</CardTitle>
          </CardHeader>
          <CardContent>
            <ProviderBarChart data={barData} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <p className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
              Sync health
            </p>
            <CardTitle>Provider readiness</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <SyncDonut
              completed={stats.completedProviders}
              failed={stats.failedProviders}
              syncing={stats.syncingProviders}
            />
            <Separator />
            <div className="grid grid-cols-3 gap-4">
              <div className="text-center">
                <p className="text-xs text-muted-foreground">Completed</p>
                <p className="text-lg font-bold tracking-tight">
                  {stats.completedProviders}
                </p>
              </div>
              <div className="text-center">
                <p className="text-xs text-muted-foreground">Failed</p>
                <p className="text-lg font-bold tracking-tight">
                  {stats.failedProviders}
                </p>
              </div>
              <div className="text-center">
                <p className="text-xs text-muted-foreground">Expiring keys</p>
                <p className="text-lg font-bold tracking-tight">
                  {stats.expiringKeys}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Recent activity */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {/* Recent providers */}
        <Card>
          <CardHeader className="flex-row items-center justify-between">
            <div>
              <p className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                Providers
              </p>
              <CardTitle>Recent activity</CardTitle>
            </div>
            <Link
              href="/providers"
              className="flex items-center gap-1 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground"
            >
              View all
              <ArrowUpRight className="size-3" />
            </Link>
          </CardHeader>
          <CardContent className="flex flex-col gap-1">
            {recentProviders.length === 0 ? (
              <p className="py-6 text-center text-sm text-muted-foreground">
                No provider credentials yet.
              </p>
            ) : (
              recentProviders.map((provider) => (
                <Link
                  key={provider.id}
                  href={`/providers/${provider.id}`}
                  className="flex items-start justify-between gap-4 rounded-md px-3 py-3 transition-colors hover:bg-muted/30"
                >
                  <div className="min-w-0">
                    <p className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                      {provider.provider}
                    </p>
                    <p className="truncate text-sm font-semibold tracking-tight text-foreground">
                      {provider.label}
                    </p>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                      Updated {formatRelative(provider.updated_at)} &middot;{" "}
                      {compactNumber(provider.model_count)} models
                    </p>
                  </div>
                  <StatusPill
                    label={provider.sync_status}
                    tone={toneForStatus(provider.sync_status)}
                    pulse={provider.sync_status === "syncing"}
                  />
                </Link>
              ))
            )}
          </CardContent>
        </Card>

        {/* Recent keys */}
        <Card>
          <CardHeader className="flex-row items-center justify-between">
            <div>
              <p className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                Virtual keys
              </p>
              <CardTitle>Recent keys</CardTitle>
            </div>
            <Link
              href="/virtual-keys"
              className="flex items-center gap-1 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground"
            >
              View all
              <ArrowUpRight className="size-3" />
            </Link>
          </CardHeader>
          <CardContent className="flex flex-col gap-1">
            {recentKeys.length === 0 ? (
              <p className="py-6 text-center text-sm text-muted-foreground">
                No virtual keys yet.
              </p>
            ) : (
              recentKeys.map((key) => (
                <Link
                  key={key.id}
                  href={`/virtual-keys/${key.id}`}
                  className="flex items-start justify-between gap-4 rounded-md px-3 py-3 transition-colors hover:bg-muted/30"
                >
                  <div className="min-w-0">
                    <p className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                      {key.key_prefix}
                    </p>
                    <p className="truncate text-sm font-semibold tracking-tight text-foreground">
                      {key.name}
                    </p>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                      {summarizeAllowedModels(key)} &middot;{" "}
                      {summarizeRoutes(key)} &middot;{" "}
                      {formatRelative(key.updated_at)}
                    </p>
                  </div>
                  <StatusPill
                    label={key.enabled ? "enabled" : "disabled"}
                    tone={key.enabled ? "success" : "neutral"}
                  />
                </Link>
              ))
            )}
          </CardContent>
        </Card>
      </div>

      {/* Access posture */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Shield className="size-4 text-muted-foreground" />
            <div>
              <p className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                Access posture
              </p>
              <CardTitle>Virtual key scope distribution</CardTitle>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-6">
            <div className="rounded-md bg-muted/30 px-5 py-4 text-center">
              <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                Scoped keys
              </p>
              <p className="mt-1 text-2xl font-bold tracking-tight">
                {scopedKeys}
              </p>
            </div>
            <div className="rounded-md bg-muted/30 px-5 py-4 text-center">
              <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                Unrestricted keys
              </p>
              <p className="mt-1 text-2xl font-bold tracking-tight">
                {unrestrictedKeys}
              </p>
            </div>
            <div className="rounded-md bg-muted/30 px-5 py-4 text-center">
              <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                Routed keys
              </p>
              <p className="mt-1 text-2xl font-bold tracking-tight">
                {routedKeys}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
