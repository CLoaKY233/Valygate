import { deleteVirtualKeyAction, updateVirtualKeyFormAction } from "@/app/actions";
import { VirtualKeyEditor } from "@/components/virtual-key-form";
import { MetaList, SectionHeader, StatusPill, EmptyState } from "@/components/ui";
import { getVirtualKey, listModels, listProviders } from "@/lib/api";
import { formatDateTime, summarizeAllowedModels, summarizeRoutes } from "@/lib/format";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
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
import { Route, Trash2 } from "lucide-react";

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
        description={`Prefix: ${key.key_prefix}`}
        actions={
          <StatusPill
            label={key.enabled ? "enabled" : "disabled"}
            tone={key.enabled ? "success" : "neutral"}
            pulse={key.enabled}
          />
        }
      />

      <div className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-[1fr_400px]">
        {/* Left column */}
        <div className="flex flex-col gap-6">
          {/* Metadata */}
          <Card className="border-border/50">
            <CardHeader className="pb-4">
              <CardTitle className="text-[0.88rem] font-semibold tracking-[-0.02em]">
                Credential posture
              </CardTitle>
            </CardHeader>
            <Separator />
            <CardContent className="pt-5">
              <MetaList
                items={[
                  {
                    label: "Record ID",
                    value: <span className="font-mono text-[0.78rem]">{key.id}</span>,
                  },
                  {
                    label: "Key prefix",
                    value: (
                      <Badge
                        variant="outline"
                        className="rounded-md font-mono text-[0.7rem] tracking-tight"
                      >
                        {key.key_prefix}
                      </Badge>
                    ),
                  },
                  { label: "Allowed models", value: summarizeAllowedModels(key) },
                  { label: "Routes", value: summarizeRoutes(key) },
                  { label: "Expires", value: formatDateTime(key.expires_at) },
                  { label: "Last used", value: formatDateTime(key.last_used_at) },
                  { label: "Updated", value: formatDateTime(key.updated_at) },
                  { label: "Created", value: formatDateTime(key.created_at) },
                ]}
              />
            </CardContent>
          </Card>

          {/* Routes */}
          <Card className="border-border/50">
            <CardHeader className="pb-4">
              <div className="flex items-center gap-2">
                <Route className="size-4 text-muted-foreground" />
                <CardTitle className="text-[0.88rem] font-semibold tracking-[-0.02em]">
                  Resolved provider paths
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent className="px-0 pb-0">
              {key.model_routes.length === 0 ? (
                <div className="px-6 pb-6">
                  <EmptyState
                    title="No explicit routes"
                    body="Add routes below to map specific model aliases to provider credentials."
                  />
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow className="hover:bg-transparent">
                      <TableHead className="pl-6 text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
                        Model alias
                      </TableHead>
                      <TableHead className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
                        Provider
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {key.model_routes.map((route) => (
                      <TableRow key={`${route.model_alias}-${route.provider_credential_id}`}>
                        <TableCell className="pl-6">
                          <span className="font-mono text-[0.8rem] font-medium tracking-tight text-foreground">
                            {route.model_alias}
                          </span>
                        </TableCell>
                        <TableCell>
                          <div className="flex flex-col">
                            <span className="text-[0.8rem] font-medium text-foreground">
                              {route.provider_label}
                            </span>
                            <span className="text-[0.72rem] text-muted-foreground">
                              {route.provider}
                            </span>
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

        {/* Right column - Edit form */}
        <Card className="border-border/50 self-start">
          <CardHeader>
            <CardTitle className="text-[0.88rem] font-semibold tracking-[-0.02em]">
              Scope & routing
            </CardTitle>
            <p className="mt-1 text-[0.78rem] leading-relaxed text-muted-foreground">
              Configure allowed models and provider routing for this key.
            </p>
          </CardHeader>
          <Separator />
          <CardContent className="pt-5">
            <VirtualKeyEditor
              providers={providers}
              action={updateVirtualKeyFormAction.bind(null, key.id)}
              mode="edit"
              keyRecord={key}
              models={models}
            />

            <Separator className="my-5" />

            <form action={deleteVirtualKeyAction.bind(null, key.id)}>
              <Button variant="destructive" size="sm" type="submit" className="gap-1.5">
                <Trash2 className="size-3.5" />
                Delete key
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
