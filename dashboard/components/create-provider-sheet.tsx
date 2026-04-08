"use client";

import { Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetTrigger,
} from "@/components/ui/sheet";
import { ProviderCreateForm } from "./provider-create-form";
import type { FieldState } from "@/lib/types";

export function CreateProviderSheet({
  action,
}: {
  action: (state: FieldState, formData: FormData) => Promise<FieldState>;
}) {
  return (
    <Sheet>
      <SheetTrigger render={<Button size="sm" />}>
        <Plus className="size-3.5" />
        Add provider
      </SheetTrigger>
      <SheetContent className="sm:max-w-md overflow-y-auto">
        <SheetHeader>
          <SheetTitle>New provider credential</SheetTitle>
          <SheetDescription>
            Connect a Google GenAI API key. Model sync starts automatically
            after creation.
          </SheetDescription>
        </SheetHeader>
        <div className="px-4 pb-4">
          <ProviderCreateForm action={action} />
        </div>
      </SheetContent>
    </Sheet>
  );
}
