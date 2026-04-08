import Link from "next/link";
import {
  ArrowLeft,
  RefreshCw,
  Trash2,
  Shield,
  Cpu,
  Settings,
} from "lucide-react";

import {
  deleteProviderAction,
  syncProviderAction,
  updateProviderAction,
} from "@/app/actions";
import { EmptyState, MetaList, SectionHeader, StatusPill } from "@/components/ui";
import { getProvider, listProviderModels } from "@/lib/api";
import { formatDateTime, formatRelative } from "@/lib/format";
import { stripRecordPrefix } from "@/lib/utils";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
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
import { Alert, AlertDescription } from "@/components/ui/alert";

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
  const [provider, models] = await Promise.all([
    getProvider(id),
    listProviderModels(id),
  ]);

  return (
    <div className="flex flex-col gap-6">
      {/* Back link */}
      <div>
        <Link
          href="/providers"
          className="inline-flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
        >
          <ArrowLeft className="size-3" />
          All providers
        </Link>
      </div>

      <SectionHeader
        eyebrow={provider.provider}
        title={provider.label}
        description={`Encrypted credential with asynchronous model discovery.`}
        actions={
          <div className="flex items-center gap-2">
            <StatusPill
              label={provider.sync_status}
              tone={toneForStatus(provider.sync_status)}
              pulse={provider.sync_status === "syncing"}
            />
            <StatusPill
              label={provider.enabled ? "enabled" : "disabled"}
              tone={provider.enabled ? "success" : "neutral"}
            />
          </div>
        }
      />

      <div className="grid gap-6 lg:grid-cols-[1fr_360px]">
        {/* Left column: metadata + models */}
        <div className="flex flex-col gap-6">
          {/* Metadata card */}
          <Card className="border-border/50 shadow-[0_1px_3px_rgba(0,0,0,0.04),0_1px_2px_rgba(0,0,0,0.02)]">
            <CardHeader className="pb-4">
              <div className="flex items-center gap-2">
                <Shield className="size-3.5 text-muted-foreground/60" />
                <CardTitle className="text-[0.8rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  Provider state
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              <MetaList
                items={[
                  { label: "Provider", value: provider.provider },
                  {
                    label: "Record ID",
                    value: (
                      <span className="font-mono text-[0.78rem]">
                        {stripRecordPrefix(provider.id)}
                      </span>
                    ),
                  },
                  {
                    label: "Models synced",
                    value: (
                      <span className="tabular-nums">{provider.model_count}</span>
                    ),
                  },
                  {
                    label: "Status",
                    value: (
                      <StatusPill
                        label={provider.enabled ? "enabled" : "disabled"}
                        tone={provider.enabled ? "success" : "neutral"}
                      />
                    ),
                  },
                  {
                    label: "Last synced",
                    value: formatDateTime(provider.last_synced_at),
                  },
                  {
                    label: "Last used",
                    value: formatDateTime(provider.last_used_at),
                  },
                  {
                    label: "Updated",
                    value: formatRelative(provider.updated_at),
                  },
                  {
                    label: "Created",
                    value: formatDateTime(provider.created_at),
                  },
                ]}
              />

              {provider.sync_error ? (
                <div className="mt-4">
                  <Alert variant="destructive">
                    <AlertDescription>{provider.sync_error}</AlertDescription>
                  </Alert>
                </div>
              ) : null}
            </CardContent>
          </Card>

          {/* Models card */}
          <Card className="border-border/50 shadow-[0_1px_3px_rgba(0,0,0,0.04),0_1px_2px_rgba(0,0,0,0.02)]">
            <CardHeader className="pb-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Cpu className="size-3.5 text-muted-foreground/60" />
                  <CardTitle className="text-[0.8rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                    Discovered models
                  </CardTitle>
                </div>
                <Badge
                  variant="secondary"
                  className="rounded-md text-[0.65rem] font-medium tabular-nums"
                >
                  {models.length} model{models.length !== 1 ? "s" : ""}
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="p-0 pt-0">
              {models.length === 0 ? (
                <div className="p-6 pt-0">
                  <EmptyState
                    title="No synced models yet"
                    body="Trigger a manual sync or wait for the background discovery run to complete."
                  />
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow className="hover:bg-transparent">
                      <TableHead className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                        Model
                      </TableHead>
                      <TableHead className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                        Context
                      </TableHead>
                      <TableHead className="text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                        Capabilities
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {models.map((model) => (
                      <TableRow key={model.id}>
                        <TableCell>
                          <div className="flex flex-col gap-0.5">
                            <span className="text-[0.84rem] font-semibold tracking-[-0.01em] text-foreground">
                              {model.display_name}
                            </span>
                            <span className="font-mono text-[0.7rem] text-muted-foreground/70">
                              {model.alias}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex flex-col gap-0.5 tabular-nums">
                            <span className="text-[0.8rem] text-foreground">
                              {model.context_window_tokens.toLocaleString()} ctx
                            </span>
                            <span className="text-[0.7rem] text-muted-foreground">
                              {model.max_output_tokens.toLocaleString()} out
                            </span>
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex flex-wrap gap-1">
                            {model.supports_streaming && (
                              <Badge
                                variant="secondary"
                                className="rounded-md px-1.5 py-0 text-[0.6rem] font-medium"
                              >
                                streaming
                              </Badge>
                            )}
                            {model.supports_tools && (
                              <Badge
                                variant="secondary"
                                className="rounded-md px-1.5 py-0 text-[0.6rem] font-medium"
                              >
                                tools
                              </Badge>
                            )}
                            {model.supports_vision && (
                              <Badge
                                variant="secondary"
                                className="rounded-md px-1.5 py-0 text-[0.6rem] font-medium"
                              >
                                vision
                              </Badge>
                            )}
                            {model.supports_json_mode && (
                              <Badge
                                variant="secondary"
                                className="rounded-md px-1.5 py-0 text-[0.6rem] font-medium"
                              >
                                json
                              </Badge>
                            )}
                            {model.supports_thinking && (
                              <Badge
                                variant="secondary"
                                className="rounded-md px-1.5 py-0 text-[0.6rem] font-medium"
                              >
                                thinking
                              </Badge>
                            )}
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

        {/* Right column: settings */}
        <div className="flex flex-col gap-6">
          <Card className="border-border/50 shadow-[0_1px_3px_rgba(0,0,0,0.04),0_1px_2px_rgba(0,0,0,0.02)]">
            <CardHeader className="pb-4">
              <div className="flex items-center gap-2">
                <Settings className="size-3.5 text-muted-foreground/60" />
                <CardTitle className="text-[0.8rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  Credential settings
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              <form
                className="flex flex-col gap-5"
                action={updateProviderAction.bind(null, provider.id)}
              >
                <div className="space-y-2">
                  <Label
                    htmlFor="edit-label"
                    className="text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground"
                  >
                    Label
                  </Label>
                  <Input
                    id="edit-label"
                    name="label"
                    defaultValue={provider.label}
                    required
                  />
                </div>

                <div className="space-y-2">
                  <Label
                    htmlFor="edit-api-key"
                    className="text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground"
                  >
                    Replacement API key
                  </Label>
                  <Input
                    id="edit-api-key"
                    name="apiKey"
                    type="password"
                    placeholder="Leave blank to keep current secret"
                  />
                </div>

                <div className="space-y-2">
                  <Label
                    htmlFor="edit-tags"
                    className="text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground"
                  >
                    Tags
                  </Label>
                  <Input
                    id="edit-tags"
                    name="tags"
                    defaultValue={provider.tags.join(", ")}
                    placeholder="prod, primary"
                  />
                </div>

                <div className="flex items-center justify-between rounded-md bg-muted/30 px-4 py-3">
                  <div className="space-y-0.5">
                    <Label
                      htmlFor="edit-enabled"
                      className="text-[0.8rem] font-medium text-foreground"
                    >
                      Enabled
                    </Label>
                    <p className="text-[0.7rem] text-muted-foreground">
                      Available for routing and sync
                    </p>
                  </div>
                  <Switch
                    id="edit-enabled"
                    name="enabled"
                    defaultChecked={provider.enabled}
                    value="on"
                  />
                </div>

                <Button type="submit" className="self-start">
                  Save changes
                </Button>
              </form>

              <Separator className="my-5" />

              {/* Actions */}
              <div className="flex flex-col gap-2">
                <p className="mb-1 text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
                  Actions
                </p>
                <form action={syncProviderAction.bind(null, provider.id)}>
                  <Button
                    variant="outline"
                    size="sm"
                    type="submit"
                    className="w-full justify-start"
                  >
                    <RefreshCw className="size-3" />
                    Manual sync
                  </Button>
                </form>
                <form action={deleteProviderAction.bind(null, provider.id)}>
                  <Button
                    variant="destructive"
                    size="sm"
                    type="submit"
                    className="w-full justify-start"
                  >
                    <Trash2 className="size-3" />
                    Delete credential
                  </Button>
                </form>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
