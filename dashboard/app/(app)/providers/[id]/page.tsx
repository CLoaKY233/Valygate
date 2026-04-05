import Link from "next/link";

import { deleteProviderAction, syncProviderAction, updateProviderAction } from "@/app/actions";
import { EmptyState, MetaList, SectionHeader, StatusPill, Surface } from "@/components/ui";
import { getProvider, listProviderModels } from "@/lib/api";
import { formatDateTime, formatRelative } from "@/lib/format";

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

export default async function ProviderDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const [provider, models] = await Promise.all([getProvider(id), listProviderModels(id)]);

  return (
    <>
      <SectionHeader
        eyebrow={provider.provider}
        title={provider.label}
        description={`Encrypted credential with asynchronous model discovery. Sync status: ${provider.sync_status}.`}
        actions={
          <StatusPill
            label={provider.sync_status}
            tone={toneForStatus(provider.sync_status)}
            pulse={provider.sync_status === "syncing"}
          />
        }
      />

      <div className="panel-grid">
        <div className="stack">
          {/* Metadata */}
          <Surface>
            <div className="surface__title-row">
              <div>
                <p className="eyebrow">Metadata</p>
                <h2>Provider state</h2>
              </div>
            </div>
            <MetaList
              items={[
                { label: "Provider", value: provider.provider },
                { label: "Record ID", value: <span className="mono">{provider.id}</span> },
                { label: "Models synced", value: provider.model_count },
                {
                  label: "Status",
                  value: (
                    <StatusPill
                      label={provider.enabled ? "enabled" : "disabled"}
                      tone={provider.enabled ? "success" : "neutral"}
                    />
                  ),
                },
                { label: "Last synced", value: formatDateTime(provider.last_synced_at) },
                { label: "Last used", value: formatDateTime(provider.last_used_at) },
                { label: "Updated", value: formatRelative(provider.updated_at) },
                { label: "Created", value: formatDateTime(provider.created_at) },
              ]}
            />
            {provider.sync_error ? (
              <p className="form-message form-message--error" style={{ marginTop: "1rem" }}>
                {provider.sync_error}
              </p>
            ) : null}
          </Surface>

          {/* Synced models */}
          <Surface>
            <div className="surface__title-row">
              <div>
                <p className="eyebrow">Discovered models</p>
                <h2>Model inventory</h2>
              </div>
            </div>

            {models.length === 0 ? (
              <EmptyState
                title="No synced models yet"
                body="Trigger a manual sync or wait for the background discovery run to complete."
              />
            ) : (
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Model</th>
                    <th>Context</th>
                    <th>Capabilities</th>
                  </tr>
                </thead>
                <tbody>
                  {models.map((model) => (
                    <tr key={model.id}>
                      <td>
                        <div className="data-table__title">
                          <strong>{model.display_name}</strong>
                          <span className="mono">{model.alias}</span>
                        </div>
                      </td>
                      <td>
                        <div className="data-table__title">
                          <span>{model.context_window_tokens.toLocaleString()} ctx</span>
                          <span>{model.max_output_tokens.toLocaleString()} out</span>
                        </div>
                      </td>
                      <td>
                        <div className="chip-row">
                          {model.supports_streaming && <span className="capability-chip capability-chip--enabled">streaming</span>}
                          {model.supports_tools && <span className="capability-chip capability-chip--enabled">tools</span>}
                          {model.supports_vision && <span className="capability-chip capability-chip--enabled">vision</span>}
                          {model.supports_json_mode && <span className="capability-chip capability-chip--enabled">json</span>}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </Surface>
        </div>

        {/* Edit form */}
        <div>
          <Surface>
            <div className="surface__title-row">
              <div>
                <p className="eyebrow">Edit</p>
                <h2>Credential controls</h2>
              </div>
            </div>

            <form className="panel-form" action={updateProviderAction.bind(null, provider.id)}>
              <label className="field">
                <span>Label</span>
                <input name="label" defaultValue={provider.label} required />
              </label>
              <label className="field">
                <span>Replacement API key</span>
                <input
                  name="apiKey"
                  type="password"
                  placeholder="Leave blank to keep current secret"
                />
              </label>
              <label className="field">
                <span>Tags</span>
                <input name="tags" defaultValue={provider.tags.join(", ")} placeholder="prod, primary" />
              </label>
              <label className="field field--checkbox">
                <input name="enabled" type="checkbox" defaultChecked={provider.enabled} />
                <span>Credential enabled for routing and sync</span>
              </label>
              <button className="primary-button" type="submit">
                Save changes
              </button>
            </form>

            <div className="table-actions" style={{ marginTop: "1.25rem" }}>
              <form className="form-inline" action={syncProviderAction.bind(null, provider.id)}>
                <button className="secondary-button" type="submit">
                  Manual sync
                </button>
              </form>
              <form className="form-inline" action={deleteProviderAction.bind(null, provider.id)}>
                <button className="ghost-button" type="submit">
                  Delete credential
                </button>
              </form>
            </div>
          </Surface>
        </div>
      </div>
    </>
  );
}
