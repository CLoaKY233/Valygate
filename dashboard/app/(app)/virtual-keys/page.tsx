import Link from "next/link";
import { Plus, Search, KeyRound, Eye } from "lucide-react";

import { createVirtualKeyAction } from "@/app/actions";
import { VirtualKeyEditor } from "@/components/virtual-key-form";
import { SectionHeader, StatusPill, EmptyState } from "@/components/ui";
import { listProviders, listVirtualKeys } from "@/lib/api";
import { formatDateTime, summarizeAllowedModels, summarizeRoutes } from "@/lib/format";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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

      <div className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-[1fr_380px]">
        {/* Key list */}
        <Card className="border-border/50">
          <CardHeader className="pb-4">
            <div className="flex items-center justify-between gap-4">
              <CardTitle className="text-[0.95rem] font-semibold tracking-[-0.02em]">
                All keys
              </CardTitle>
              <Badge variant="secondary" className="rounded-md text-xs tabular-nums">
                {keys.length}
              </Badge>
            </div>
            <form className="mt-3">
              <div className="relative">
                <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
                <Input
                  name="q"
                  placeholder="Search keys, prefixes, tags..."
                  defaultValue={q}
                  className="h-8 pl-8 text-sm"
                />
              </div>
            </form>
          </CardHeader>
          <CardContent className="px-0 pb-0">
            {filtered.length === 0 ? (
              <div className="px-6 pb-6">
                <EmptyState
                  title={q ? "No results" : "No virtual keys yet"}
                  body={
                    q
                      ? "Try a different search term."
                      : "Create a key to get started. The raw key is shown once at creation."
                  }
                  action={
                    !q ? (
                      <KeyRound className="size-8 text-muted-foreground/40" />
                    ) : undefined
                  }
                />
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow className="hover:bg-transparent">
                    <TableHead className="pl-6 text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
                      Key
                    </TableHead>
                    <TableHead className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
                      Scope
                    </TableHead>
                    <TableHead className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
                      Routes
                    </TableHead>
                    <TableHead className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
                      Expiry
                    </TableHead>
                    <TableHead className="w-[100px]" />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filtered.map((key) => (
                    <TableRow key={key.id} className="group">
                      <TableCell className="pl-6">
                        <div className="flex flex-col gap-1.5">
                          <div className="flex items-center gap-2">
                            <span className="text-[0.84rem] font-semibold tracking-[-0.01em] text-foreground">
                              {key.name}
                            </span>
                            <Badge
                              variant="outline"
                              className="rounded-md font-mono text-[0.65rem] tracking-tight text-muted-foreground"
                            >
                              {key.key_prefix}
                            </Badge>
                          </div>
                          <div className="flex flex-wrap items-center gap-1">
                            {key.tags.map((tag) => (
                              <Badge
                                key={tag}
                                variant="secondary"
                                className="rounded-md px-1.5 py-0 text-[0.6rem] font-medium"
                              >
                                {tag}
                              </Badge>
                            ))}
                            <StatusPill
                              label={key.enabled ? "enabled" : "disabled"}
                              tone={key.enabled ? "success" : "neutral"}
                            />
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <span className="text-[0.8rem] text-muted-foreground">
                          {summarizeAllowedModels(key)}
                        </span>
                      </TableCell>
                      <TableCell>
                        <span className="text-[0.8rem] text-muted-foreground">
                          {summarizeRoutes(key)}
                        </span>
                      </TableCell>
                      <TableCell>
                        <span className="text-[0.8rem] text-muted-foreground">
                          {formatDateTime(key.expires_at)}
                        </span>
                      </TableCell>
                      <TableCell>
                        <Link
                          href={`/virtual-keys/${key.id}`}
                          className="inline-flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
                        >
                          <Eye className="size-3" />
                          Inspect
                        </Link>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        {/* Create form */}
        <Card className="border-border/50 self-start">
          <CardHeader>
            <div className="flex items-center gap-2">
              <div className="flex size-7 items-center justify-center rounded-md bg-primary/10">
                <Plus className="size-3.5 text-primary" />
              </div>
              <div>
                <CardTitle className="text-[0.88rem] font-semibold tracking-[-0.02em]">
                  New virtual key
                </CardTitle>
              </div>
            </div>
            <p className="mt-1 text-[0.78rem] leading-relaxed text-muted-foreground">
              Configure model scopes from the key detail page after your providers are synced.
            </p>
          </CardHeader>
          <Separator />
          <CardContent className="pt-5">
            <VirtualKeyEditor providers={providers} action={createVirtualKeyAction} mode="create" />
          </CardContent>
        </Card>
      </div>
    </>
  );
}
