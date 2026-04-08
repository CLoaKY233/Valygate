import Link from "next/link";
import { Search } from "lucide-react";

import { listModels } from "@/lib/api";
import { EmptyState, SectionHeader } from "@/components/ui";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

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
  { key: "supports_thinking", label: "thinking" },
] as const;

function buildParams(overrides: Record<string, string>, base: { q: string; provider: string; cap: string }) {
  const out: Record<string, string> = {};
  if (base.q) out.q = base.q;
  if (base.provider) out.provider = base.provider;
  if (base.cap) out.cap = base.cap;
  return new URLSearchParams({ ...out, ...overrides }).toString();
}

function toggleCap(current: string, key: string): string {
  const active = current ? current.split(",").filter(Boolean) : [];
  const idx = active.indexOf(key);
  if (idx === -1) return [...active, key].join(",");
  return active.filter((k) => k !== key).join(",");
}

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

  const providers = [...new Set(models.map((m) => m.provider))].sort();
  const hasFilters = !!(q || providerFilter || capFilter);
  const base = { q, provider: providerFilter, cap: capFilter };

  return (
    <div className="flex flex-col gap-6">
      <SectionHeader
        title="Model catalog"
        description="Synced models from your provider credentials. Sync a provider first to populate this catalog."
      />

      <div className="flex flex-col gap-6 lg:flex-row">
        {/* Filter sidebar */}
        <aside className="w-full shrink-0 lg:w-56">
          <Card className="border-border/50">
            <CardContent className="flex flex-col gap-4 p-4">
              {/* Search */}
              <form>
                <div className="relative">
                  <Search className="absolute left-2.5 top-2.5 size-3.5 text-muted-foreground" />
                  <Input
                    name="q"
                    placeholder="Search models..."
                    defaultValue={q}
                    className="h-8 pl-8 text-xs"
                  />
                </div>
                {providerFilter && <input type="hidden" name="provider" value={providerFilter} />}
                {capFilter && <input type="hidden" name="cap" value={capFilter} />}
              </form>

              <Separator />

              {/* Provider filter */}
              {providers.length > 0 && (
                <div className="flex flex-col gap-1.5">
                  <div className="flex items-center justify-between">
                    <h4 className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                      Provider
                    </h4>
                    {providerFilter && (
                      <Link
                        href={`/models?${buildParams({ provider: "" }, base)}`}
                        className="text-[0.65rem] text-primary hover:underline"
                      >
                        Clear
                      </Link>
                    )}
                  </div>
                  {providers.map((p) => (
                    <Link
                      key={p}
                      href={`/models?${buildParams({ provider: providerFilter === p ? "" : p }, base)}`}
                    >
                      <Button
                        variant="ghost"
                        size="sm"
                        className={cn(
                          "h-7 w-full justify-start px-2 text-xs font-normal",
                          providerFilter === p && "bg-primary/10 text-primary font-medium",
                        )}
                      >
                        {p}
                      </Button>
                    </Link>
                  ))}
                </div>
              )}

              <Separator />

              {/* Capability filter */}
              <div className="flex flex-col gap-1.5">
                <h4 className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  Capabilities
                </h4>
                {capabilityFilters.map(({ key, label }) => {
                  const activeCaps = capFilter ? capFilter.split(",").filter(Boolean) : [];
                  const isActive = activeCaps.includes(key);
                  return (
                    <Link
                      key={key}
                      href={`/models?${buildParams({ cap: toggleCap(capFilter, key) }, base)}`}
                    >
                      <Button
                        variant="ghost"
                        size="sm"
                        className={cn(
                          "h-7 w-full justify-start gap-2 px-2 text-xs font-normal",
                          isActive && "bg-primary/10 text-primary font-medium",
                        )}
                      >
                        <span
                          className={cn(
                            "flex size-3.5 shrink-0 items-center justify-center rounded-sm border",
                            isActive
                              ? "border-primary bg-primary text-primary-foreground"
                              : "border-muted-foreground/30",
                          )}
                        >
                          {isActive && (
                            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
                              <path
                                d="M2 5l2.5 2.5L8 3"
                                stroke="currentColor"
                                strokeWidth="1.5"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                              />
                            </svg>
                          )}
                        </span>
                        {label}
                      </Button>
                    </Link>
                  );
                })}
              </div>

              {hasFilters && (
                <>
                  <Separator />
                  <Link href="/models">
                    <Button variant="ghost" size="sm" className="h-7 w-full text-xs text-muted-foreground">
                      Clear all filters
                    </Button>
                  </Link>
                </>
              )}
            </CardContent>
          </Card>
        </aside>

        {/* Model grid */}
        <div className="min-w-0 flex-1">
          {filtered.length === 0 ? (
            <EmptyState
              title="No models found"
              body={
                hasFilters
                  ? "Try different filters or search terms."
                  : "Sync a provider credential first. The catalog only contains models reachable through your providers."
              }
            />
          ) : (
            <div className="flex flex-col gap-4">
              <p className="text-xs text-muted-foreground">
                {filtered.length} model{filtered.length === 1 ? "" : "s"}
              </p>
              <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                {filtered.map((model) => (
                  <Link key={model.id} href={`/models/${model.alias}`} className="group">
                    <Card className="h-full border-border/50 transition-all duration-150 group-hover:border-border group-hover:shadow-md group-hover:scale-[0.995]">
                      <CardContent className="flex flex-col gap-3 p-4">
                        <div className="flex flex-col gap-1.5">
                          <Badge variant="secondary" className="w-fit text-[0.6rem] uppercase tracking-wider">
                            {model.provider}
                          </Badge>
                          <h3 className="text-sm font-semibold tracking-tight">{model.display_name}</h3>
                          <span className="font-mono text-xs text-muted-foreground">{model.alias}</span>
                        </div>

                        <div className="flex flex-wrap gap-1">
                          {capabilityBadges.map((cap) => (
                            <Badge
                              key={cap.key}
                              variant={model[cap.key] ? "outline" : "secondary"}
                              className={cn(
                                "text-[0.6rem] px-1.5 py-0",
                                !model[cap.key] && "text-muted-foreground/50",
                              )}
                            >
                              {cap.label}
                            </Badge>
                          ))}
                        </div>

                        <div className="flex items-center gap-3 border-t border-border/40 pt-2 text-[0.65rem] text-muted-foreground">
                          <span>{model.context_window_tokens.toLocaleString()} ctx</span>
                          <span>{model.max_output_tokens.toLocaleString()} out</span>
                        </div>
                      </CardContent>
                    </Card>
                  </Link>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
