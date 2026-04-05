import Link from "next/link";

import { CapabilityMatrix } from "@/components/charts";
import { SectionHeader, StatusPill, Surface } from "@/components/ui";
import { getModel } from "@/lib/api";

export default async function ModelDetailPage({
  params,
}: {
  params: Promise<{ alias: string[] }>;
}) {
  const { alias } = await params;
  const model = await getModel(alias.join("/"));

  return (
    <>
      <SectionHeader
        eyebrow={model.provider}
        title={model.display_name}
        description={model.description ?? "Capability-aware model definition from your ValyMux catalog."}
        actions={
          <StatusPill
            label={model.enabled ? "enabled" : "disabled"}
            tone={model.enabled ? "success" : "neutral"}
          />
        }
      />

      <div className="split-grid">
        {/* Identity */}
        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Identity</p>
              <h2>Model metadata</h2>
            </div>
          </div>
          <dl className="meta-list">
            <div>
              <dt>Alias</dt>
              <dd className="mono">{model.alias}</dd>
            </div>
            <div>
              <dt>Upstream model</dt>
              <dd className="mono">{model.upstream_model}</dd>
            </div>
            <div>
              <dt>Context window</dt>
              <dd>{model.context_window_tokens.toLocaleString()} tokens</dd>
            </div>
            <div>
              <dt>Max output</dt>
              <dd>{model.max_output_tokens.toLocaleString()} tokens</dd>
            </div>
            {model.temperature_min !== null && (
              <div>
                <dt>Temp range</dt>
                <dd>{model.temperature_min} – {model.temperature_max}</dd>
              </div>
            )}
            {model.temperature_fixed_to !== null && (
              <div>
                <dt>Fixed temp</dt>
                <dd>{model.temperature_fixed_to}</dd>
              </div>
            )}
          </dl>

          <div style={{ marginTop: "1.25rem" }}>
            <Link href="/models" className="ghost-button" style={{ fontSize: "0.8125rem" }}>
              ← Back to catalog
            </Link>
          </div>
        </Surface>

        {/* Capabilities */}
        <Surface>
          <div className="surface__title-row">
            <div>
              <p className="eyebrow">Capabilities</p>
              <h2>Feature matrix</h2>
            </div>
          </div>
          <CapabilityMatrix
            items={[
              { label: "Streaming", value: model.supports_streaming },
              { label: "Thinking", value: model.supports_thinking },
              { label: "Thinking required", value: model.thinking_required },
              { label: "Temperature", value: model.supports_temperature },
              { label: "Top P", value: model.supports_top_p },
              { label: "System messages", value: model.supports_system_messages },
              { label: "Tools", value: model.supports_tools },
              { label: "Vision", value: model.supports_vision },
              { label: "JSON mode", value: model.supports_json_mode },
              { label: "Parallel tools", value: model.supports_parallel_tool_calls },
            ]}
          />
        </Surface>
      </div>
    </>
  );
}
