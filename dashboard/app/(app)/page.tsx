import Link from "next/link";
import { Server, KeyRound, Cpu, Activity } from "lucide-react";

import { listModels, listProviders, listVirtualKeys } from "@/lib/api";
import {
  compactNumber,
  computeOverviewStats,
  formatRelative,
  summarizeAllowedModels,
  summarizeRoutes,
} from "@/lib/format";

import { ProviderBarChart, SyncDonut } from "@/components/charts";
import { ResourceLinkCard, SectionHeader, StatusPill, Surface } from "@/components/ui";

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
  const recentKeys = [...keys].sort((a, b) => (b.updated_at > a.updated_at ? 1 : -1)).slice(0, 4);

  const barData = Object.entries(stats.modelsByProvider).map(([name, models]) => ({
    name,
    models,
  }));

  return (
    <>
      <SectionHeader
        title="Gateway overview"
        description="Live control-plane data from your ValyMux API."
        actions={
          <>
            <Link className="secondary-button" href="/providers">
              Providers
            </Link>
            <Link className="primary-button" href="/virtual-keys">
              Virtual keys
            </Link>
          </>
        }
      />

      {/* Stat cards */}
      <div className="stat-cards">
        <div className="stat-card">
          <div className="stat-card__icon">
            <Server size={16} />
          </div>
          <div className="stat-card__body">
            <div className="stat-card__value">{stats.providerCount}</div>
            <div className="stat-card__label">Provider credentials</div>
            <div className="stat-card__sub">{stats.activeProviders} enabled</div>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-card__icon">
            <KeyRound size={16} />
          </div>
          <div className="stat-card__body">
            <div className="stat-card__value">{stats.keyCount}</div>
            <div className="stat-card__label">Virtual keys</div>
            <div className="stat-card__sub">{stats.activeKeys} live</div>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-card__icon">
            <Cpu size={16} />
          </div>
          <div className="stat-card__body">
            <div className="stat-card__value">{stats.modelCount}</div>
            <div className="stat-card__label">Usable models</div>
            <div className="stat-card__sub">From synced catalog</div>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-card__icon">
            <Activity size={16} />
          </div>
          <div className="stat-card__body">
            <div className="stat-card__value">{stats.syncingProviders}</div>
            <div className="stat-card__label">In motion</div>
            <div className="stat-card__sub">Pending or syncing</div>
          </div>
        </div>
      </div>

      {/* Charts row */}
      <div className="split-grid">
        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Catalog supply</p>
              <h2>Models by provider</h2>
            </div>
          </div>
          <ProviderBarChart data={barData} />
        </Surface>

        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Sync health</p>
              <h2>Provider readiness</h2>
            </div>
          </div>
          <SyncDonut
            completed={stats.completedProviders}
            failed={stats.failedProviders}
            syncing={stats.syncingProviders}
          />
          <div className="stat-strip" style={{ marginTop: "1rem" }}>
            <div className="stat-strip__item">
              <span>Completed</span>
              <strong>{stats.completedProviders}</strong>
            </div>
            <div className="stat-strip__item">
              <span>Failed</span>
              <strong>{stats.failedProviders}</strong>
            </div>
            <div className="stat-strip__item">
              <span>Expiring keys</span>
              <strong>{stats.expiringKeys}</strong>
            </div>
          </div>
        </Surface>
      </div>

      {/* Recent activity */}
      <div className="split-grid">
        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Providers</p>
              <h2>Recent activity</h2>
            </div>
            <Link href="/providers" className="ghost-button" style={{ fontSize: "0.8125rem" }}>
              View all
            </Link>
          </div>
          <div className="stack">
            {recentProviders.length === 0 ? (
              <p className="text-muted">No provider credentials yet.</p>
            ) : (
              recentProviders.map((provider) => (
                <ResourceLinkCard
                  key={provider.id}
                  href={`/providers/${provider.id}`}
                  kicker={provider.provider}
                  title={provider.label}
                  body={`Updated ${formatRelative(provider.updated_at)} · ${compactNumber(provider.model_count)} models`}
                  right={
                    <StatusPill
                      label={provider.sync_status}
                      tone={toneForStatus(provider.sync_status)}
                      pulse={provider.sync_status === "syncing"}
                    />
                  }
                />
              ))
            )}
          </div>
        </Surface>

        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Virtual keys</p>
              <h2>Recent keys</h2>
            </div>
            <Link href="/virtual-keys" className="ghost-button" style={{ fontSize: "0.8125rem" }}>
              View all
            </Link>
          </div>
          <div className="stack">
            {recentKeys.length === 0 ? (
              <p className="text-muted">No virtual keys yet.</p>
            ) : (
              recentKeys.map((key) => (
                <ResourceLinkCard
                  key={key.id}
                  href={`/virtual-keys/${key.id}`}
                  kicker={key.key_prefix}
                  title={key.name}
                  body={`${summarizeAllowedModels(key)} · ${summarizeRoutes(key)} · ${formatRelative(key.updated_at)}`}
                  right={
                    <StatusPill
                      label={key.enabled ? "enabled" : "disabled"}
                      tone={key.enabled ? "success" : "neutral"}
                    />
                  }
                />
              ))
            )}
          </div>
        </Surface>
      </div>

      {/* Key scope overview */}
      <Surface>
        <div className="surface__title-row">
          <div>
            <p className="eyebrow">Access posture</p>
            <h2>Virtual key scope distribution</h2>
          </div>
        </div>
        <div className="stat-strip">
          <div className="stat-strip__item">
            <span>Scoped keys</span>
            <strong>{keys.filter((key) => key.allowed_models.length > 0).length}</strong>
          </div>
          <div className="stat-strip__item">
            <span>Unrestricted keys</span>
            <strong>{keys.filter((key) => key.allowed_models.length === 0).length}</strong>
          </div>
          <div className="stat-strip__item">
            <span>Routed keys</span>
            <strong>{keys.filter((key) => key.model_routes.length > 0).length}</strong>
          </div>
        </div>
      </Surface>
    </>
  );
}
