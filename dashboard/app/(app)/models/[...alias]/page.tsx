import Link from "next/link";
import { ArrowLeft } from "lucide-react";

import { getModel } from "@/lib/api";
import { SectionHeader, StatusPill } from "@/components/ui";
import { CapabilityMatrix } from "@/components/charts";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

export default async function ModelDetailPage({
  params,
}: {
  params: Promise<{ alias: string[] }>;
}) {
  const { alias } = await params;
  const model = await getModel(alias.join("/"));

  const metaItems: { label: string; value: string }[] = [
    { label: "Alias", value: model.alias },
    { label: "Upstream model", value: model.upstream_model },
    { label: "Context window", value: `${model.context_window_tokens.toLocaleString()} tokens` },
    { label: "Max output", value: `${model.max_output_tokens.toLocaleString()} tokens` },
  ];

  if (model.temperature_min !== null && model.temperature_max !== null) {
    metaItems.push({ label: "Temp range", value: `${model.temperature_min} - ${model.temperature_max}` });
  }

  if (model.temperature_fixed_to !== null) {
    metaItems.push({ label: "Fixed temp", value: String(model.temperature_fixed_to) });
  }

  return (
    <div className="flex flex-col gap-6">
      <SectionHeader
        eyebrow={model.provider}
        title={model.display_name}
        description={model.description ?? "Capability-aware model definition from your ValyMux catalog."}
        actions={
          <StatusPill
            label={model.enabled ? "enabled" : "disabled"}
            tone={model.enabled ? "success" : "neutral"}
            pulse={model.enabled}
          />
        }
      />

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Model metadata */}
        <Card className="border-border/50">
          <CardHeader className="pb-4">
            <CardTitle className="text-sm font-semibold tracking-tight">Model metadata</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-5">
            <dl className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              {metaItems.map((item) => (
                <div key={item.label} className="rounded-md bg-muted/30 px-4 py-3">
                  <dt className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                    {item.label}
                  </dt>
                  <dd
                    className={`mt-1 text-sm font-medium ${
                      item.label === "Alias" || item.label === "Upstream model" ? "font-mono" : ""
                    }`}
                  >
                    {item.value}
                  </dd>
                </div>
              ))}
            </dl>

            <Separator />

            <Link href="/models">
              <Button variant="ghost" size="sm" className="gap-1.5 text-xs text-muted-foreground">
                <ArrowLeft className="size-3" />
                Back to catalog
              </Button>
            </Link>
          </CardContent>
        </Card>

        {/* Feature matrix */}
        <Card className="border-border/50">
          <CardHeader className="pb-4">
            <CardTitle className="text-sm font-semibold tracking-tight">Feature matrix</CardTitle>
          </CardHeader>
          <CardContent>
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
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
