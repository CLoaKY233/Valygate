import { revalidatePath } from "next/cache";

import { getCurrentUser } from "@/lib/api";
import { SectionHeader, StatusPill } from "@/components/ui";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";

async function updateProfileAction(formData: FormData) {
  "use server";

  const name = String(formData.get("name") ?? "").trim();
  const { getApiBaseUrl } = await import("@/lib/config");
  const { getSessionToken } = await import("@/lib/session");
  const token = await getSessionToken();

  if (!token) return;

  await fetch(`${getApiBaseUrl()}/me`, {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ name }),
    cache: "no-store",
  });

  revalidatePath("/profile");
}

export default async function ProfilePage() {
  const user = await getCurrentUser();

  return (
    <div className="flex flex-col gap-6">
      <SectionHeader
        title="Profile"
        description="Manage your account identity and settings."
        actions={
          <StatusPill
            label={user.enabled ? "active" : "disabled"}
            tone={user.enabled ? "success" : "neutral"}
            pulse={user.enabled}
          />
        }
      />

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Account identity */}
        <Card className="border-border/50">
          <CardHeader className="pb-4">
            <CardTitle className="text-sm font-semibold tracking-tight">Account identity</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <div className="rounded-md bg-muted/30 px-4 py-3">
                <p className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  User ID
                </p>
                <p className="mt-1 font-mono text-sm font-medium">{user.id}</p>
              </div>
              <div className="rounded-md bg-muted/30 px-4 py-3">
                <p className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  Email
                </p>
                <p className="mt-1 text-sm font-medium">{user.email}</p>
              </div>
              <div className="rounded-md bg-muted/30 px-4 py-3">
                <p className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  Display name
                </p>
                <p className="mt-1 text-sm font-medium">{user.name}</p>
              </div>
              <div className="rounded-md bg-muted/30 px-4 py-3">
                <p className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  Status
                </p>
                <div className="mt-1">
                  <StatusPill
                    label={user.enabled ? "active" : "disabled"}
                    tone={user.enabled ? "success" : "neutral"}
                  />
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Update profile */}
        <Card className="border-border/50">
          <CardHeader className="pb-4">
            <CardTitle className="text-sm font-semibold tracking-tight">Update profile</CardTitle>
          </CardHeader>
          <CardContent>
            <form action={updateProfileAction} className="flex flex-col gap-4">
              <div className="flex flex-col gap-2">
                <Label htmlFor="name" className="text-xs">
                  Display name
                </Label>
                <Input
                  id="name"
                  name="name"
                  defaultValue={user.name}
                  required
                  className="h-9"
                />
              </div>
              <Separator />
              <Button type="submit" size="sm" className="w-fit">
                Save changes
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
