import { getCurrentUser } from "@/lib/api";

import { AppNav } from "@/components/app-nav";
import { SignOutButton } from "@/components/signout-button";
import { BrandMark } from "@/components/ui";
import { Separator } from "@/components/ui/separator";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";

function initialFor(name: string) {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

export default async function AppLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const user = await getCurrentUser();

  return (
    <div className="flex h-screen bg-muted/30 p-3">
      {/* Sidebar */}
      <aside className="flex w-60 shrink-0 flex-col rounded-lg border border-border/50 bg-card shadow-[0_1px_3px_rgba(0,0,0,0.04)]">
        {/* Brand */}
        <div className="flex items-center gap-2.5 px-5 pt-5 pb-2">
          <BrandMark />
          <span className="text-sm font-semibold tracking-[-0.03em] text-foreground">
            ValyMux
          </span>
          <span className="ml-auto rounded-md bg-muted/60 px-1.5 py-0.5 text-[0.6rem] font-medium text-muted-foreground">
            v0.1
          </span>
        </div>

        {/* Navigation */}
        <AppNav />

        {/* Spacer */}
        <div className="flex-1" />

        {/* User session */}
        <div className="px-3 pb-3">
          <Separator className="mb-3 opacity-50" />
          <div className="flex items-center gap-2.5 rounded-md px-2 py-2">
            <Avatar className="size-7">
              <AvatarFallback className="bg-primary/10 text-[0.6rem] font-semibold text-primary">
                {initialFor(user.name)}
              </AvatarFallback>
            </Avatar>
            <div className="min-w-0 flex-1">
              <p className="truncate text-xs font-medium text-foreground">
                {user.name}
              </p>
              <p className="truncate text-[0.65rem] text-muted-foreground">
                {user.email}
              </p>
            </div>
            <SignOutButton />
          </div>
        </div>
      </aside>

      {/* Main content */}
      <div className="ml-3 flex flex-1 flex-col overflow-hidden">
        {/* Topbar */}
        <div className="flex h-14 shrink-0 items-center justify-between rounded-lg border border-border/50 bg-card px-5 shadow-[0_1px_3px_rgba(0,0,0,0.04)]">
          <div className="flex items-center gap-2 text-sm">
            <span className="text-muted-foreground">ValyMux</span>
            <span className="text-muted-foreground/40">/</span>
            <span className="font-medium text-foreground">Gateway operations</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="text-right">
              <p className="text-xs font-medium text-foreground">{user.name}</p>
              <p className="text-[0.65rem] text-muted-foreground">{user.email}</p>
            </div>
            <Avatar className="size-8">
              <AvatarFallback className="bg-primary/10 text-xs font-semibold text-primary">
                {initialFor(user.name)}
              </AvatarFallback>
            </Avatar>
          </div>
        </div>

        {/* Content area */}
        <div className="mt-3 flex-1 overflow-auto rounded-lg border border-border/50 bg-card p-6 shadow-[0_1px_3px_rgba(0,0,0,0.04)]">
          {children}
        </div>
      </div>
    </div>
  );
}
