import Link from "next/link";

import { createVirtualKeyAction } from "@/app/actions";
import { VirtualKeyEditor } from "@/components/virtual-key-form";
import { EmptyState, SectionHeader, StatusPill, Surface } from "@/components/ui";
import { listProviders, listVirtualKeys } from "@/lib/api";
import { formatDateTime, summarizeAllowedModels, summarizeRoutes } from "@/lib/format";

export default async function VirtualKeysPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const params = await searchParams;
  const q = String(params.q ?? "").toLowerCase();
  const [keys, providers] = await Promise.all([listVirtualKeys(), listProviders()]);
  const filtered = keys.filter((key) => {
    if (!q) return true;
    return [key.name, key.key_prefix, key.tags.join(" "), key.allowed_models.join(" ")]
      .join(" ")
      .toLowerCase()
      .includes(q);
  });

  return (
    <>
      <SectionHeader
        title="Virtual keys"
        description="Scoped gateway credentials. Create a key, copy the raw value, then configure model scopes from the detail page."
      />

      <div className="panel-grid">
        {/* Keys list */}
        <div className="stack">
          <form>
            <input
              name="q"
              placeholder="Search keys, prefixes, tags…"
              defaultValue={q}
              style={{ maxWidth: 320 }}
            />
          </form>

          <Surface>
            {filtered.length === 0 ? (
              <EmptyState
                title={q ? "No results" : "No virtual keys yet"}
                body={
                  q
                    ? "Try a different search term."
                    : "Create a key to get started. The raw key is shown once at creation."
                }
              />
            ) : (
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Key</th>
                    <th>Scope</th>
                    <th>Routes</th>
                    <th>Expiry</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((key) => (
                    <tr key={key.id}>
                      <td>
                        <div className="data-table__title">
                          <strong>{key.name}</strong>
                          <span>
                            <span className="inline-code">{key.key_prefix}</span>
                          </span>
                          <div className="tag-row" style={{ marginTop: "0.25rem" }}>
                            {key.tags.map((tag) => (
                              <span className="tag" key={tag}>{tag}</span>
                            ))}
                            <StatusPill
                              label={key.enabled ? "enabled" : "disabled"}
                              tone={key.enabled ? "success" : "neutral"}
                            />
                          </div>
                        </div>
                      </td>
                      <td>
                        <span className="text-muted">{summarizeAllowedModels(key)}</span>
                      </td>
                      <td>
                        <span className="text-muted">{summarizeRoutes(key)}</span>
                      </td>
                      <td>
                        <span className="text-muted">{formatDateTime(key.expires_at)}</span>
                      </td>
                      <td>
                        <Link className="secondary-button" href={`/virtual-keys/${key.id}`}>
                          Inspect
                        </Link>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </Surface>
        </div>

        {/* Create key form — simplified: no model scoping at creation */}
        <div>
          <Surface>
            <div className="surface__title-row">
              <div>
                <p className="eyebrow">Create key</p>
                <h2>New virtual credential</h2>
              </div>
            </div>
            <p style={{ margin: "0 0 1.25rem", fontSize: "0.875rem", color: "var(--text-soft)" }}>
              Configure model scopes from the key detail page after your providers are synced.
            </p>
            <VirtualKeyEditor providers={providers} action={createVirtualKeyAction} mode="create" />
          </Surface>
        </div>
      </div>
    </>
  );
}
