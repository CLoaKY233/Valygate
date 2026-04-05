import { deleteVirtualKeyAction, updateVirtualKeyFormAction } from "@/app/actions";
import { VirtualKeyEditor } from "@/components/virtual-key-form";
import { EmptyState, MetaList, SectionHeader, StatusPill, Surface } from "@/components/ui";
import { getVirtualKey, listModels, listProviders } from "@/lib/api";
import { formatDateTime, summarizeAllowedModels, summarizeRoutes } from "@/lib/format";

export default async function VirtualKeyDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const [key, providers, models] = await Promise.all([
    getVirtualKey(id),
    listProviders(),
    listModels(),
  ]);

  return (
    <>
      <SectionHeader
        eyebrow="Virtual key"
        title={key.name}
        description={`Prefix: ${key.key_prefix} — Configure allowed models and routing below.`}
        actions={
          <StatusPill
            label={key.enabled ? "enabled" : "disabled"}
            tone={key.enabled ? "success" : "neutral"}
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
                <h2>Credential posture</h2>
              </div>
            </div>
            <MetaList
              items={[
                { label: "Record ID", value: <span className="mono">{key.id}</span> },
                { label: "Key prefix", value: <span className="inline-code">{key.key_prefix}</span> },
                { label: "Allowed models", value: summarizeAllowedModels(key) },
                { label: "Routes", value: summarizeRoutes(key) },
                { label: "Expires", value: formatDateTime(key.expires_at) },
                { label: "Last used", value: formatDateTime(key.last_used_at) },
                { label: "Updated", value: formatDateTime(key.updated_at) },
                { label: "Created", value: formatDateTime(key.created_at) },
              ]}
            />
          </Surface>

          {/* Routes */}
          <Surface>
            <div className="surface__title-row">
              <div>
                <p className="eyebrow">Routing</p>
                <h2>Resolved provider paths</h2>
              </div>
            </div>

            {key.model_routes.length === 0 ? (
              <EmptyState
                title="No explicit routes"
                body="Add routes below to map specific model aliases to provider credentials."
              />
            ) : (
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Model alias</th>
                    <th>Provider</th>
                  </tr>
                </thead>
                <tbody>
                  {key.model_routes.map((route) => (
                    <tr key={`${route.model_alias}-${route.provider_credential_id}`}>
                      <td>
                        <div className="data-table__title">
                          <strong className="mono">{route.model_alias}</strong>
                        </div>
                      </td>
                      <td>
                        <div className="data-table__title">
                          <strong>{route.provider_label}</strong>
                          <span>{route.provider}</span>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </Surface>
        </div>

        {/* Edit form with MultiSelect */}
        <div>
          <Surface>
            <div className="surface__title-row">
              <div>
                <p className="eyebrow">Edit</p>
                <h2>Scope & routing</h2>
              </div>
            </div>
            <VirtualKeyEditor
              providers={providers}
              action={updateVirtualKeyFormAction.bind(null, key.id)}
              mode="edit"
              keyRecord={key}
              models={models}
            />

            <div className="table-actions" style={{ marginTop: "1.25rem" }}>
              <form className="form-inline" action={deleteVirtualKeyAction.bind(null, key.id)}>
                <button className="ghost-button" type="submit">
                  Delete key
                </button>
              </form>
            </div>
          </Surface>
        </div>
      </div>
    </>
  );
}
