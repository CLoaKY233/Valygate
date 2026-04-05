import Link from "next/link";
import type { ReactNode } from "react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";

export function BrandMark() {
  return (
    <div className="relative flex size-6 shrink-0 items-center justify-center">
      <span className="absolute inset-0 rotate-45 rounded-[3px] border border-foreground/10 bg-gradient-to-br from-foreground/[0.03] to-transparent" />
      <span className="absolute inset-[5px] rotate-45 rounded-[2px] bg-gradient-to-br from-slate-600 to-slate-950 shadow-[0_2px_8px_rgba(15,23,42,0.25)]" />
      <span className="absolute inset-[8px] rotate-45 rounded-[1px] bg-gradient-to-br from-slate-400/30 to-transparent" />
    </div>
  );
}

export function SectionHeader({
  eyebrow,
  title,
  description,
  actions,
}: {
  eyebrow?: string;
  title: string;
  description?: string;
  actions?: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
      <div className="min-w-0">
        {eyebrow ? (
          <p className="mb-1.5 text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
            {eyebrow}
          </p>
        ) : null}
        <h1 className="text-[1.4rem] font-semibold tracking-[-0.03em] text-foreground">
          {title}
        </h1>
        {description ? (
          <p className="mt-1.5 max-w-2xl text-[0.84rem] leading-relaxed text-muted-foreground">
            {description}
          </p>
        ) : null}
      </div>
      {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
    </div>
  );
}

export function Surface({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) {
  return (
    <Card
      className={cn(
        "border-border/50 bg-card shadow-[0_1px_3px_rgba(0,0,0,0.04),0_1px_2px_rgba(0,0,0,0.02)]",
        className,
      )}
    >
      <CardContent className="p-6">{children}</CardContent>
    </Card>
  );
}

export function StatusPill({
  label,
  tone,
  pulse,
}: {
  label: string;
  tone: "neutral" | "success" | "warning" | "danger" | "info";
  pulse?: boolean;
}) {
  const toneClass = {
    neutral: "border-border/60 bg-secondary/50 text-secondary-foreground",
    success: "border-emerald-200/60 bg-emerald-50/80 text-emerald-700",
    warning: "border-amber-200/60 bg-amber-50/80 text-amber-700",
    danger: "border-rose-200/60 bg-rose-50/80 text-rose-700",
    info: "border-sky-200/60 bg-sky-50/80 text-sky-700",
  }[tone];

  return (
    <Badge variant="outline" className={cn("gap-1.5 rounded-md px-2 py-0.5 text-[0.7rem] font-medium", toneClass)}>
      {pulse ? <span className="size-1.5 rounded-full bg-current animate-pulse" /> : null}
      {label}
    </Badge>
  );
}

export function MetaList({ items }: { items: Array<{ label: string; value: ReactNode }> }) {
  return (
    <dl className="grid grid-cols-1 gap-3 sm:grid-cols-2">
      {items.map((item) => (
        <div
          key={item.label}
          className="rounded-md bg-muted/30 px-4 py-3.5"
        >
          <dt className="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
            {item.label}
          </dt>
          <dd className="mt-1.5 text-[0.84rem] font-medium tracking-[-0.01em] text-foreground">
            {item.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}

export function EmptyState({
  title,
  body,
  action,
}: {
  title: string;
  body: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 rounded-md border border-dashed border-border/60 bg-muted/10 px-6 py-10 text-center">
      <h3 className="text-[0.95rem] font-semibold tracking-[-0.03em] text-foreground">
        {title}
      </h3>
      <p className="max-w-md text-[0.84rem] leading-relaxed text-muted-foreground">
        {body}
      </p>
      {action ? <div className="mt-1">{action}</div> : null}
    </div>
  );
}

export function ResourceLinkCard({
  href,
  title,
  body,
  kicker,
  right,
}: {
  href: string;
  title: string;
  body: string;
  kicker?: string;
  right?: ReactNode;
}) {
  return (
    <Link
      className="group flex items-start justify-between gap-4 rounded-md border border-border/50 bg-card px-4 py-4 shadow-[0_1px_2px_rgba(0,0,0,0.03)] transition-all duration-150 hover:border-border hover:bg-accent/50 hover:shadow-[0_2px_4px_rgba(0,0,0,0.05)] hover:scale-[0.995] active:scale-[0.99]"
      href={href}
    >
      <div className="min-w-0">
        {kicker ? (
          <p className="mb-1.5 text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
            {kicker}
          </p>
        ) : null}
        <h3 className="truncate text-[0.88rem] font-semibold tracking-[-0.03em] text-foreground">
          {title}
        </h3>
        <p className="mt-1 text-[0.8rem] leading-relaxed text-muted-foreground">
          {body}
        </p>
      </div>
      {right ? <div className="shrink-0">{right}</div> : null}
    </Link>
  );
}
