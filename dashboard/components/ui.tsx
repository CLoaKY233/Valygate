import Link from "next/link";
import type { ReactNode } from "react";

export function BrandMark() {
  return (
    <div className="brand-mark">
      <span className="brand-mark__core" />
      <span className="brand-mark__ring" />
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
    <div className="section-header">
      <div>
        {eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}
        <h1 className="page-title">{title}</h1>
        {description ? <p className="page-description">{description}</p> : null}
      </div>
      {actions ? <div className="section-actions">{actions}</div> : null}
    </div>
  );
}

export function Surface({
  className = "",
  children,
}: {
  className?: string;
  children: ReactNode;
}) {
  return <section className={`surface ${className}`.trim()}>{children}</section>;
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
  return (
    <span className={`status-pill status-pill--${tone}`}>
      {pulse ? <span className={`status-pill__dot ${pulse ? "status-pill__dot--pulse" : ""}`} /> : null}
      {label}
    </span>
  );
}

export function MetaList({ items }: { items: Array<{ label: string; value: ReactNode }> }) {
  return (
    <dl className="meta-list">
      {items.map((item) => (
        <div key={item.label}>
          <dt>{item.label}</dt>
          <dd>{item.value}</dd>
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
    <div className="empty-state">
      <h3>{title}</h3>
      <p>{body}</p>
      {action}
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
    <Link className="resource-link-card" href={href}>
      <div>
        {kicker ? <p className="eyebrow">{kicker}</p> : null}
        <h3>{title}</h3>
        <p>{body}</p>
      </div>
      {right ? <div className="resource-link-card__right">{right}</div> : null}
    </Link>
  );
}
