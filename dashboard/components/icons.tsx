import type { CSSProperties } from "react";

type IconProps = {
  className?: string;
  style?: CSSProperties;
};

function strokeProps(style?: CSSProperties) {
  return {
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.8,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    style,
  };
}

export function DashboardIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <path d="M3 13.2h8.4V3H3z" />
      <path d="M12.6 21H21v-11h-8.4z" />
      <path d="M12.6 10.8H21V3h-8.4z" />
      <path d="M3 21h8.4v-5.4H3z" />
    </svg>
  );
}

export function ProviderIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <path d="M5 7h14" />
      <path d="M5 12h14" />
      <path d="M5 17h14" />
      <path d="M7 5v14" />
      <path d="M17 5v14" />
    </svg>
  );
}

export function KeyIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <circle cx="8" cy="15" r="4" />
      <path d="M12 15h9" />
      <path d="M18 12v6" />
      <path d="M21 13.5v3" />
    </svg>
  );
}

export function ModelIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <path d="M12 3 4 7v10l8 4 8-4V7z" />
      <path d="M4 7l8 4 8-4" />
      <path d="M12 11v10" />
    </svg>
  );
}

export function ProfileIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <circle cx="12" cy="8" r="4" />
      <path d="M4 20c2.8-4.2 12.2-4.2 16 0" />
    </svg>
  );
}

export function ArrowTrendIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <path d="m4 16 5-5 4 4 7-8" />
      <path d="M15 7h5v5" />
    </svg>
  );
}

export function ShieldIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <path d="M12 3c3.2 2 5.9 3 8 3v6c0 4.8-3.2 7.9-8 9-4.8-1.1-8-4.2-8-9V6c2.1 0 4.8-1 8-3Z" />
    </svg>
  );
}

export function SparkIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <path d="M12 3v6" />
      <path d="M12 15v6" />
      <path d="M3 12h6" />
      <path d="M15 12h6" />
      <path d="m6.5 6.5 4 4" />
      <path d="m13.5 13.5 4 4" />
      <path d="m17.5 6.5-4 4" />
      <path d="m10.5 13.5-4 4" />
    </svg>
  );
}

export function CopyIcon({ className, style }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" className={className} {...strokeProps(style)}>
      <rect x="9" y="9" width="10" height="10" rx="2" />
      <path d="M6 15H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v1" />
    </svg>
  );
}
