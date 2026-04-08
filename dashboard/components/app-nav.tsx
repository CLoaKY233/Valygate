"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Server,
  KeyRound,
  Cpu,
  User,
} from "lucide-react";
import { cn } from "@/lib/utils";

const navItems = [
  { href: "/", label: "Overview", icon: LayoutDashboard },
  { href: "/providers", label: "Providers", icon: Server },
  { href: "/virtual-keys", label: "Virtual Keys", icon: KeyRound },
  { href: "/models", label: "Models", icon: Cpu },
  { href: "/profile", label: "Profile", icon: User },
];

export function AppNav() {
  const pathname = usePathname();

  return (
    <nav className="mt-5 flex flex-col gap-1 px-3">
      {navItems.map((item) => {
        const Icon = item.icon;
        const active =
          pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));

        return (
          <Link
            key={item.href}
            href={item.href}
            className={cn(
              "relative flex items-center gap-2.5 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground transition-all duration-150",
              active
                ? "bg-primary/5 text-foreground"
                : "hover:bg-muted/50 hover:text-foreground",
            )}
          >
            {active && (
              <span className="absolute left-0 top-1/2 -translate-y-1/2 h-5 w-[2px] rounded-full bg-primary" />
            )}
            <Icon
              size={16}
              className={cn("shrink-0 transition-opacity", active ? "opacity-100" : "opacity-60")}
            />
            <span>{item.label}</span>
          </Link>
        );
      })}
    </nav>
  );
}
