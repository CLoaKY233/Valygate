"use client";

import { useActionState, useState } from "react";
import { Plus, X, AlertCircle, CheckCircle2, KeyRound } from "lucide-react";
import Link from "next/link";

import type { FieldState, Provider, VirtualKey, Model } from "@/lib/types";
import type { SelectOption } from "./multi-select";
import { CopyButton } from "./copy-button";
import { cn } from "@/lib/utils";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

const initialState: FieldState = {};

type RouteRow = {
  modelAlias: string;
  providerCredentialId: string;
};

function buildInitialRoutes(key?: VirtualKey): RouteRow[] {
  if (!key || key.model_routes.length === 0) {
    return [];
  }

  return key.model_routes.map((route) => ({
    modelAlias: route.model_alias,
    providerCredentialId: route.provider_credential_id,
  }));
}

export function VirtualKeyEditor({
  providers,
  action,
  mode,
  keyRecord,
  models = [],
}: {
  providers: Provider[];
  action: (state: FieldState, formData: FormData) => Promise<FieldState>;
  mode: "create" | "edit";
  keyRecord?: VirtualKey;
  models?: Model[];
}) {
  const [state, formAction, pending] = useActionState(action, initialState);
  const [routes, setRoutes] = useState<RouteRow[]>(buildInitialRoutes(keyRecord));
  const [enabled, setEnabled] = useState(keyRecord?.enabled ?? true);

  const modelOptions: SelectOption[] = models.map((m) => ({
    value: m.alias,
    label: m.display_name,
    description: m.alias,
  }));

  function addRoute() {
    setRoutes((current) => [...current, { modelAlias: "", providerCredentialId: "" }]);
  }

  function removeRoute(index: number) {
    setRoutes((current) =>
      current.length === 1 ? [] : current.filter((_, i) => i !== index),
    );
  }

  function updateRoute(index: number, field: keyof RouteRow, value: string) {
    setRoutes((current) =>
      current.map((row, i) => (i === index ? { ...row, [field]: value } : row)),
    );
  }

  return (
    <form action={formAction} className="flex flex-col gap-4">
      {keyRecord ? <input type="hidden" name="keyId" value={keyRecord.id} /> : null}

      {/* Hidden inputs for routes */}
      {routes.map((route, i) => (
        <div key={`hidden-${i}`}>
          <input type="hidden" name="routeModelAlias" value={route.modelAlias} />
          <input type="hidden" name="routeProviderCredentialId" value={route.providerCredentialId} />
        </div>
      ))}

      {/* Hidden input for enabled */}
      {mode === "edit" ? (
        <input type="hidden" name="enabled" value={enabled ? "on" : ""} />
      ) : null}

      {/* Name + Expires */}
      <div className={cn("grid gap-4", mode === "edit" ? "grid-cols-2" : "grid-cols-1")}>
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="name" className="text-xs font-medium">
            Name
          </Label>
          <Input
            id="name"
            name="name"
            type="text"
            placeholder="Production key"
            defaultValue={keyRecord?.name ?? ""}
            required
            className="h-8 text-sm"
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <Label htmlFor="expiresAt" className="text-xs font-medium">
            Expires at
          </Label>
          <Input
            id="expiresAt"
            name="expiresAt"
            type="datetime-local"
            defaultValue={keyRecord?.expires_at?.slice(0, 16) ?? ""}
            className="h-8 text-sm"
          />
        </div>
      </div>

      {/* Tags */}
      <div className="flex flex-col gap-1.5">
        <Label htmlFor="tags" className="text-xs font-medium">
          Tags
        </Label>
        <Input
          id="tags"
          name="tags"
          type="text"
          defaultValue={keyRecord?.tags.join(", ") ?? ""}
          placeholder="prod, analytics, backend"
          className="h-8 text-sm"
        />
      </div>

      {/* Enabled switch */}
      {mode === "edit" ? (
        <div className="flex items-center justify-between rounded-md border border-border/50 bg-muted/20 px-3 py-2.5">
          <div className="flex flex-col">
            <span className="text-[0.8rem] font-medium text-foreground">Gateway traffic</span>
            <span className="text-[0.7rem] text-muted-foreground">
              {enabled ? "Key accepts proxy requests" : "Key is disabled"}
            </span>
          </div>
          <Switch
            checked={enabled}
            onCheckedChange={setEnabled}
          />
        </div>
      ) : null}

      {/* Route editor - edit mode only */}
      {mode === "edit" ? (
        <>
          <Separator className="my-1" />

          <div className="flex flex-col gap-3">
            <div className="flex items-start justify-between gap-2">
              <div>
                <h3 className="text-[0.84rem] font-semibold tracking-[-0.01em] text-foreground">
                  Model scope & routing
                </h3>
                <p className="mt-0.5 text-[0.72rem] leading-relaxed text-muted-foreground">
                  Each route scopes the key to that model and routes it to a provider.
                </p>
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-7 gap-1 text-xs shrink-0"
                onClick={addRoute}
              >
                <Plus className="size-3" />
                Add route
              </Button>
            </div>

            {routes.length === 0 ? (
              <div className="rounded-md border border-dashed border-border/60 bg-muted/10 px-4 py-6 text-center">
                <p className="text-[0.78rem] text-muted-foreground">
                  No routes configured. This key has unrestricted model access.
                </p>
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                {routes.map((route, index) => (
                  <div
                    key={`route-${index}`}
                    className="group/route flex items-end gap-2 rounded-md border border-border/40 bg-muted/10 p-3"
                  >
                    <div className="flex flex-1 flex-col gap-3">
                      <div className="flex flex-col gap-1.5">
                        <Label className="text-[0.68rem] font-medium text-muted-foreground">
                          Model alias
                        </Label>
                        {modelOptions.length > 0 ? (
                          <Select
                            value={route.modelAlias || undefined}
                            onValueChange={(val) => updateRoute(index, "modelAlias", val as string)}
                          >
                            <SelectTrigger className="h-8 w-full text-sm">
                              <SelectValue placeholder="Select model" />
                            </SelectTrigger>
                            <SelectContent>
                              {modelOptions.map((opt) => (
                                <SelectItem key={opt.value} value={opt.value}>
                                  {opt.label}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : (
                          <Input
                            type="text"
                            value={route.modelAlias}
                            onChange={(e) =>
                              updateRoute(index, "modelAlias", e.target.value)
                            }
                            placeholder="google-genai/gemini-2.5-flash"
                            className="h-8 font-mono text-sm"
                          />
                        )}
                      </div>

                      <div className="flex flex-col gap-1.5">
                        <Label className="text-[0.68rem] font-medium text-muted-foreground">
                          Provider credential
                        </Label>
                        <Select
                          value={route.providerCredentialId || undefined}
                          onValueChange={(val) =>
                            updateRoute(index, "providerCredentialId", val as string)
                          }
                        >
                          <SelectTrigger className="h-8 w-full text-sm">
                            <SelectValue placeholder="Select credential" />
                          </SelectTrigger>
                          <SelectContent>
                            {providers.map((provider) => (
                              <SelectItem key={provider.id} value={provider.id}>
                                {provider.label} · {provider.provider}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    </div>

                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="mb-0.5 h-7 w-7 shrink-0 p-0 text-muted-foreground opacity-0 transition-opacity group-hover/route:opacity-100"
                      onClick={() => removeRoute(index)}
                    >
                      <X className="size-3.5" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </>
      ) : null}

      {/* Messages */}
      {state.error ? (
        <Alert variant="destructive">
          <AlertCircle className="size-4" />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{state.error}</AlertDescription>
        </Alert>
      ) : null}

      {state.success && !state.rawKey ? (
        <Alert>
          <CheckCircle2 className="size-4 text-emerald-600" />
          <AlertTitle>Saved</AlertTitle>
          <AlertDescription>{state.success}</AlertDescription>
        </Alert>
      ) : null}

      {/* Raw key display */}
      {state.rawKey ? (
        <div className="flex flex-col gap-3 rounded-md border border-emerald-200/60 bg-emerald-50/50 p-4">
          <div className="flex items-start gap-2">
            <KeyRound className="mt-0.5 size-4 shrink-0 text-emerald-600" />
            <div>
              <h3 className="text-[0.84rem] font-semibold tracking-[-0.01em] text-foreground">
                Store this key — shown once only
              </h3>
              <p className="mt-0.5 text-[0.72rem] leading-relaxed text-muted-foreground">
                The backend only returns the raw key once. Copy it now before navigating away.
              </p>
            </div>
          </div>
          <code className="block break-all rounded-md bg-foreground/5 px-3 py-2 font-mono text-[0.78rem] tracking-tight text-foreground">
            {state.rawKey}
          </code>
          <div className="flex items-center gap-2">
            <CopyButton value={state.rawKey} />
            {state.createdKeyId ? (
              <Link href={`/virtual-keys/${state.createdKeyId}`}>
                <Button variant="outline" size="sm">
                  Configure model scopes
                </Button>
              </Link>
            ) : null}
          </div>
        </div>
      ) : null}

      <Button type="submit" size="sm" className="mt-1 w-full" disabled={pending}>
        {pending
          ? "Saving..."
          : mode === "create"
            ? "Create virtual key"
            : "Save changes"}
      </Button>
    </form>
  );
}
