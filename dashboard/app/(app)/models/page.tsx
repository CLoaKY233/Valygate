import Link from "next/link";

import { EmptyState, SectionHeader, Surface } from "@/components/ui";
import { listModels } from "@/lib/api";

const capabilityFilters = [
  { key: "supports_streaming", label: "Streaming" },
  { key: "supports_tools", label: "Tools" },
  { key: "supports_vision", label: "Vision" },
  { key: "supports_json_mode", label: "JSON mode" },
  { key: "supports_thinking", label: "Thinking" },
] as const;

const capabilityBadges = [
  { key: "supports_streaming", label: "streaming" },
  { key: "supports_tools", label: "tools" },
  { key: "supports_vision", label: "vision" },
  { key: "supports_json_mode", label: "json" },
] as const;

export default async function ModelsPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const params = await searchParams;
  const q = String(params.q ?? "").toLowerCase();
  const providerFilter = String(params.provider ?? "").toLowerCase();
  const capFilter = String(params.cap ?? "").toLowerCase();

  const models = await listModels();

  const filtered = models.filter((model) => {
    if (providerFilter && model.provider !== providerFilter) return false;

    if (capFilter) {
      const caps = capFilter.split(",").filter(Boolean);
      for (const cap of caps) {
        if (!model[cap as keyof typeof model]) return false;
      }
    }

    if (!q) return true;

    return [model.alias, model.display_name, model.provider, model.description ?? ""]
      .join(" ")
      .toLowerCase()
      .includes(q);
  });

  const providers = [...new Set(models.map((m) => m.provider))];

  return (
    <>
      <SectionHeader
        title="Model catalog"
        description="Synced models from your provider credentials. Sync a provider first to populate this catalog."
      />

      <div className="models-layout">
        {/* Filter sidebar */}
        <aside>
          <div className="models-filter-panel">
            {/* Search */}
            <div>
              <h4>Search</h4>
              <form>
                <input
                  name="q"
                  placeholder="Search models…"
                  defaultValue={q}
                  style={{ fontSize: "0.875rem" }}
                />
                {/* preserve other filters */}
                {providerFilter && <input type="hidden" name="provider" value={providerFilter} />}
                {capFilter && <input type="hidden" name="cap" value={capFilter} />}
              </form>
            </div>

            {/* Provider filter */}
            {providers.length > 0 && (
              <div className="models-filter-section">
                <h4>Provider</h4>
                {providers.map((provider) => (
                  <label key={provider}>
                    <input
                      type="checkbox"
                      readOnly
                      checked={providerFilter === provider}
                    />
                    {provider}
                  </label>
                ))}
                <form style={{ marginTop: "0.5rem" }}>
                  {q && <input type="hidden" name="q" value={q} />}
                  {capFilter && <input type="hidden" name="cap" value={capFilter} />}
                  {providerFilter ? (
                    <button className="ghost-button" type="submit" style={{ fontSize: "0.75rem", padding: "0.25rem 0" }}>
                      Clear filter
                    </button>
                  ) : null}
                  {providers.map((p) => (
                    <Link
                      key={p}
                      href={`/models?${new URLSearchParams({
                        ...(q ? { q } : {}),
                        provider: p,
                        ...(capFilter ? { cap: capFilter } : {}),
                      })}`}
                      style={{
                        display: "block",
                        padding: "0.375rem 0",
                        fontSize: "0.875rem",
                        color: providerFilter === p ? "var(--primary)" : "var(--text-soft)",
                        fontWeight: providerFilter === p ? 600 : 400,
                      }}
                    >
                      {p}
                    </Link>
                  ))}
                </form>
              </div>
            )}

            {/* Capability filter */}
            <div className="models-filter-section">
              <h4>Capabilities</h4>
              {capabilityFilters.map(({ key, label }) => (
                <Link
                  key={key}
                  href={`/models?${new URLSearchParams({
                    ...(q ? { q } : {}),
                    ...(providerFilter ? { provider: providerFilter } : {}),
                    cap: capFilter === key ? "" : key,
                  })}`}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "0.5rem",
                    padding: "0.375rem 0",
                    fontSize: "0.875rem",
                    color: capFilter === key ? "var(--primary)" : "var(--text-soft)",
                    fontWeight: capFilter === key ? 600 : 400,
                  }}
                >
                  <span style={{
                    width: "0.875rem",
                    height: "0.875rem",
                    borderRadius: "3px",
                    border: `1.5px solid ${capFilter === key ? "var(--primary)" : "var(--line-strong)"}`,
                    background: capFilter === key ? "var(--primary)" : "transparent",
                    display: "grid",
                    placeItems: "center",
                    flexShrink: 0,
                  }}>
                    {capFilter === key && (
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                        <path d="M2 5l2.5 2.5L8 3" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
                      </svg>
                    )}
                  </span>
                  {label}
                </Link>
              ))}
            </div>

            {(q || providerFilter || capFilter) && (
              <Link
                href="/models"
                className="ghost-button"
                style={{ fontSize: "0.8125rem", justifyContent: "flex-start" }}
              >
                Clear all filters
              </Link>
            )}
          </div>
        </aside>

        {/* Model grid */}
        <div>
          {filtered.length === 0 ? (
            <Surface>
              <EmptyState
                title="No models found"
                body={q || providerFilter || capFilter
                  ? "Try different filters."
                  : "Sync a provider credential first. The catalog only contains models reachable through your providers."}
              />
            </Surface>
          ) : (
            <>
              <p style={{ margin: "0 0 1rem", fontSize: "0.8125rem", color: "var(--text-faint)" }}>
                {filtered.length} model{filtered.length === 1 ? "" : "s"}
              </p>
              <div className="model-card-grid">
                {filtered.map((model) => (
                  <Link key={model.id} href={`/models/${model.alias}`} className="model-card">
                    <div>
                      <span className="model-card__provider">{model.provider}</span>
                      <h3 className="model-card__name">{model.display_name}</h3>
                      <span className="model-card__alias">{model.alias}</span>
                    </div>

                    <div className="chip-row">
                      {capabilityBadges.map((cap) => (
                        <span
                          key={cap.key}
                          className={`capability-chip ${
                            model[cap.key] ? "capability-chip--enabled" : "capability-chip--disabled"
                          }`}
                        >
                          {cap.label}
                        </span>
                      ))}
                    </div>

                    <div className="model-card__footer">
                      <span>{model.context_window_tokens.toLocaleString()} ctx</span>
                      <span>{model.max_output_tokens.toLocaleString()} out</span>
                    </div>
                  </Link>
                ))}
              </div>
            </>
          )}
        </div>
      </div>
    </>
  );
}
