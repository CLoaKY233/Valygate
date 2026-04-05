"use client";

import { useActionState } from "react";
import { KeyRound } from "lucide-react";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import type { FieldState } from "@/lib/types";

const initialState: FieldState = {};

export function ProviderCreateForm({
  action,
}: {
  action: (state: FieldState, formData: FormData) => Promise<FieldState>;
}) {
  const [state, formAction, pending] = useActionState(action, initialState);

  return (
    <form action={formAction} className="flex flex-col gap-5">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <Label
            htmlFor="provider-select"
            className="text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground"
          >
            Provider
          </Label>
          <Select name="provider" defaultValue="google-genai">
            <SelectTrigger id="provider-select" className="w-full">
              <SelectValue placeholder="Select a provider" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="google-genai">Google GenAI</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <Label
            htmlFor="provider-label"
            className="text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground"
          >
            Label
          </Label>
          <Input
            id="provider-label"
            name="label"
            type="text"
            placeholder="Production Gemini"
            required
          />
        </div>
      </div>

      <Separator />

      <div className="space-y-2">
        <Label
          htmlFor="provider-api-key"
          className="text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground"
        >
          API key
        </Label>
        <div className="relative">
          <Input
            id="provider-api-key"
            name="apiKey"
            type="password"
            placeholder="AIza..."
            required
            className="pr-9"
          />
          <KeyRound className="pointer-events-none absolute top-1/2 right-3 size-3.5 -translate-y-1/2 text-muted-foreground/50" />
        </div>
        <p className="text-[0.7rem] leading-relaxed text-muted-foreground/70">
          Encrypted at rest with AES-256-GCM. Never logged or exposed.
        </p>
      </div>

      <div className="space-y-2">
        <Label
          htmlFor="provider-tags"
          className="text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground"
        >
          Tags
        </Label>
        <Input
          id="provider-tags"
          name="tags"
          type="text"
          placeholder="prod, primary"
        />
        <p className="text-[0.7rem] leading-relaxed text-muted-foreground/70">
          Comma-separated. Used for filtering and organization.
        </p>
      </div>

      {state.error ? (
        <Alert variant="destructive">
          <AlertDescription>{state.error}</AlertDescription>
        </Alert>
      ) : null}
      {state.success ? (
        <Alert>
          <AlertDescription>{state.success}</AlertDescription>
        </Alert>
      ) : null}

      <Button type="submit" disabled={pending} className="self-start">
        {pending ? "Connecting..." : "Add provider"}
      </Button>
    </form>
  );
}
