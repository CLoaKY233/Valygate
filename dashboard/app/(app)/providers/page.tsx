import Link from "next/link";
import { Eye, RefreshCw, Search, Server } from "lucide-react";

import { createProviderAction, syncProviderAction } from "@/app/actions";
import { CreateProviderSheet } from "@/components/create-provider-sheet";
import { EmptyState, SectionHeader, StatusPill } from "@/components/ui";
import { formatRelative } from "@/lib/format";
import { stripRecordPrefix } from "@/lib/utils";
import { listProviders } from "@/lib/api";

import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Separator } from "@/components/ui/separator";

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
    <div className="flex flex-col gap-6">
      <SectionHeader
        title="Providers"
        description="Encrypted provider credentials for model discovery and proxy routing."
        actions={<CreateProviderSheet action={createProviderAction} />}
      />

      <Card className="border-border/50 shadow-[0_1px_3px_rgba(0,0,0,0.04),0_1px_2px_rgba(0,0,0,0.02)]">
        <CardContent className="p-0">
          {/* Search bar */}
          <div className="flex items-center gap-3 border-b border-border/40 px-4 py-3">
            <form className="relative flex-1">
              <Search className="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
              <Input
                name="q"
                placeholder="Search providers, labels, tags..."
                defaultValue={q}
                className="h-8 pl-9 text-[0.8rem]"
              />
            </form>
            <div className="flex items-center gap-1.5 text-[0.7rem] text-muted-foreground">
              <Server className="size-3" />
              <span>
                {filtered.length} credential{filtered.length !== 1 ? "s" : ""}
              </span>
            </div>
          </div>

          {filtered.length === 0 ? (
            <div className="p-6">
              <EmptyState
                title={q ? "No results" : "No provider credentials yet"}
                body={
                  q
                    ? "Try a different search term."
                    : "Add a Google GenAI credential to start syncing models into your catalog."
                }
              />
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow className="hover:bg-transparent">
                  <TableHead className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                    Credential
                  </TableHead>
                  <TableHead className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                    Sync Status
                  </TableHead>
                  <TableHead className="text-center text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                    Models
                  </TableHead>
                  <TableHead className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                    Last Synced
                  </TableHead>
                  <TableHead className="text-right text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                    Actions
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filtered.map((provider) => (
                  <TableRow
                    key={provider.id}
                    className="group transition-colors"
                  >
                    <TableCell>
                      <div className="flex flex-col gap-1.5">
                        <div className="flex items-center gap-2">
                          <span className="text-[0.84rem] font-semibold tracking-[-0.01em] text-foreground">
                            {provider.label}
                          </span>
                          <StatusPill
                            label={provider.enabled ? "enabled" : "disabled"}
                            tone={provider.enabled ? "success" : "neutral"}
                          />
                        </div>
                        <span className="font-mono text-[0.7rem] text-muted-foreground/70">
                          {provider.provider} &middot; {stripRecordPrefix(provider.id)}
                        </span>
                        {provider.tags.length > 0 && (
                          <div className="flex flex-wrap gap-1">
                            {provider.tags.map((tag) => (
                              <Badge
                                key={tag}
                                variant="secondary"
                                className="rounded-md px-1.5 py-0 text-[0.65rem] font-medium"
                              >
                                {tag}
                              </Badge>
                            ))}
                          </div>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-col gap-1">
                        <StatusPill
                          label={provider.sync_status}
                          tone={toneForStatus(provider.sync_status)}
                          pulse={provider.sync_status === "syncing"}
                        />
                        {provider.sync_error && (
                          <p className="max-w-[200px] truncate text-[0.7rem] text-rose-600">
                            {provider.sync_error}
                          </p>
                        )}
                      </div>
                    </TableCell>
                    <TableCell className="text-center">
                      <span className="text-[0.84rem] font-semibold tabular-nums text-foreground">
                        {provider.model_count}
                      </span>
                    </TableCell>
                    <TableCell>
                      <span className="text-[0.8rem] text-muted-foreground">
                        {formatRelative(provider.last_synced_at)}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Link
                          href={`/providers/${provider.id}`}
                          className="inline-flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
                        >
                          <Eye className="size-3" />
                          Inspect
                        </Link>
                        <form
                          action={syncProviderAction.bind(null, provider.id)}
                        >
                          <Button variant="ghost" size="sm" type="submit">
                            <RefreshCw className="size-3" />
                            Sync
                          </Button>
                        </form>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
