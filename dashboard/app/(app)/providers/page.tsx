import Link from "next/link";

import { createProviderAction, syncProviderAction } from "@/app/actions";
import { ProviderCreateForm } from "@/components/provider-create-form";
import { EmptyState, SectionHeader, StatusPill, Surface } from "@/components/ui";
import { formatRelative } from "@/lib/format";
import { listProviders } from "@/lib/api";

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

export default async function ProvidersPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const params = await searchParams;
  const q = String(params.q ?? "").toLowerCase();
  const providers = await listProviders();
  const filtered = providers.filter((provider) => {
    if (!q) return true;
    return [provider.label, provider.provider, provider.tags.join(" ")]
      .join(" ")
      .toLowerCase()
      .includes(q);
  });

  return (
    <>
      <SectionHeader
        title="Providers"
        description="Encrypted provider credentials for model discovery and proxy routing."
      />

      <div className="panel-grid">
        {/* Table */}
        <div className="stack">
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "1rem" }}>
            <form>
              <input
                name="q"
                placeholder="Search providers, labels, tags…"
                defaultValue={q}
                style={{ maxWidth: 320 }}
              />
            </form>
          </div>

          <Surface>
            {filtered.length === 0 ? (
              <EmptyState
                title={q ? "No results" : "No provider credentials yet"}
                body={
                  q
                    ? "Try a different search term."
                    : "Add a Google GenAI credential to start syncing models into your catalog."
                }
              />
            ) : (
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Credential</th>
                    <th>Status</th>
                    <th>Models</th>
                    <th>Last sync</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((provider) => (
                    <tr key={provider.id}>
                      <td>
                        <div className="data-table__title">
                          <strong>{provider.label}</strong>
                          <span>{provider.provider} · {provider.id}</span>
                          <div className="tag-row" style={{ marginTop: "0.25rem" }}>
                            {provider.tags.map((tag) => (
                              <span className="tag" key={tag}>{tag}</span>
                            ))}
                            <StatusPill
                              label={provider.enabled ? "enabled" : "disabled"}
                              tone={provider.enabled ? "success" : "neutral"}
                            />
                          </div>
                        </div>
                      </td>
                      <td>
                        <StatusPill
                          label={provider.sync_status}
                          tone={toneForStatus(provider.sync_status)}
                          pulse={provider.sync_status === "syncing"}
                        />
                        {provider.sync_error && (
                          <p className="text-muted" style={{ marginTop: "0.25rem", fontSize: "0.75rem" }}>
                            {provider.sync_error}
                          </p>
                        )}
                      </td>
                      <td>{provider.model_count}</td>
                      <td>
                        <span className="text-muted">
                          {formatRelative(provider.last_synced_at)}
                        </span>
                      </td>
                      <td>
                        <div className="table-actions">
                          <Link className="secondary-button" href={`/providers/${provider.id}`}>
                            Inspect
                          </Link>
                          <form className="form-inline" action={syncProviderAction.bind(null, provider.id)}>
                            <button className="ghost-button" type="submit">
                              Sync
                            </button>
                          </form>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </Surface>
        </div>

        {/* Create form */}
        <div>
          <Surface>
            <div className="surface__title-row">
              <div>
                <p className="eyebrow">Add credential</p>
                <h2>New provider</h2>
              </div>
            </div>
            <p style={{ margin: "0 0 1.25rem", fontSize: "0.875rem", color: "var(--text-soft)" }}>
              ValyMux currently supports Google GenAI. Model sync starts automatically after creation.
            </p>
            <ProviderCreateForm action={createProviderAction} />
          </Surface>
        </div>
      </div>
    </>
  );
}
