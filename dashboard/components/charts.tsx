"use client";

import React from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";

import { cn } from "@/lib/utils";

const COLORS = ["#4338ca", "#6366f1", "#818cf8", "#a5b4fc", "#c7d2fe"];

const tooltipStyle: React.CSSProperties = {
  background: "#ffffff",
  border: "1px solid oklch(0.929 0.013 255)",
  borderRadius: "6px",
  fontSize: 12,
  boxShadow: "0 4px 12px rgba(0,0,0,0.08), 0 1px 3px rgba(0,0,0,0.06)",
  padding: "6px 10px",
  color: "oklch(0.141 0.033 264)",
};

const tooltipLabelStyle: React.CSSProperties = {
  fontWeight: 600,
  fontSize: 11,
  color: "oklch(0.554 0.046 257)",
  textTransform: "uppercase",
  letterSpacing: "0.05em",
  marginBottom: 2,
};

export function ProviderBarChart({
  data,
}: {
  data: { name: string; models: number }[];
}) {
  if (data.length === 0) {
    return (
      <div className="flex h-[180px] items-center justify-center">
        <p className="text-sm text-muted-foreground">
          No providers synced yet.
        </p>
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={180}>
      <BarChart data={data} margin={{ top: 4, right: 4, left: -20, bottom: 4 }}>
        <XAxis
          dataKey="name"
          tick={{ fontSize: 11, fill: "hsl(var(--muted-foreground))" }}
          axisLine={false}
          tickLine={false}
        />
        <YAxis
          tick={{ fontSize: 11, fill: "hsl(var(--muted-foreground))" }}
          axisLine={false}
          tickLine={false}
          allowDecimals={false}
        />
        <Tooltip
          cursor={{ fill: "oklch(0.968 0.007 264 / 0.5)", radius: 4 }}
          contentStyle={tooltipStyle}
          labelStyle={tooltipLabelStyle}
          itemStyle={{ color: "#6366f1", fontSize: 12 }}
        />
        <Bar dataKey="models" radius={[3, 3, 0, 0]}>
          {data.map((_, index) => (
            <Cell key={index} fill={COLORS[index % COLORS.length]} />
          ))}
        </Bar>
      </BarChart>
    </ResponsiveContainer>
  );
}

export function SyncDonut({
  completed,
  failed,
  syncing,
}: {
  completed: number;
  failed: number;
  syncing: number;
}) {
  const total = completed + failed + syncing;
  const data =
    total === 0
      ? [{ name: "None", value: 1 }]
      : [
          { name: "Completed", value: completed },
          { name: "Failed", value: failed },
          { name: "In progress", value: syncing },
        ].filter((d) => d.value > 0);

  const donutColors =
    total === 0
      ? ["hsl(var(--muted))"]
      : ["#059669", "#e11d48", "#d97706"];

  return (
    <div className="relative">
      <ResponsiveContainer width="100%" height={180}>
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={50}
            outerRadius={75}
            strokeWidth={0}
            dataKey="value"
          >
            {data.map((_, index) => (
              <Cell
                key={index}
                fill={donutColors[index % donutColors.length]}
              />
            ))}
          </Pie>
          <Tooltip
            contentStyle={tooltipStyle}
            labelStyle={tooltipLabelStyle}
            itemStyle={{ fontSize: 12 }}
          />
        </PieChart>
      </ResponsiveContainer>
      <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
        <div className="text-center">
          <p className="text-2xl font-bold tracking-tight">{completed}</p>
          <p className="text-xs text-muted-foreground">synced</p>
        </div>
      </div>
    </div>
  );
}

// Legacy components kept for model detail page
export function ArcMeter({
  value,
  total,
  label,
}: {
  value: number;
  total: number;
  label: string;
}) {
  const ratio = total === 0 ? 0 : Math.min(value / total, 1);
  const circumference = 2 * Math.PI * 42;
  const dashOffset = circumference * (1 - ratio);

  return (
    <div className="relative flex size-[100px] items-center justify-center">
      <svg viewBox="0 0 100 100" className="size-full -rotate-90">
        <circle
          cx="50"
          cy="50"
          r="42"
          fill="none"
          strokeWidth="6"
          className="stroke-muted"
        />
        <circle
          cx="50"
          cy="50"
          r="42"
          fill="none"
          strokeWidth="6"
          strokeLinecap="round"
          className="stroke-indigo-500"
          style={{
            strokeDasharray: circumference,
            strokeDashoffset: dashOffset,
            transition: "stroke-dashoffset 0.6s ease",
          }}
        />
      </svg>
      <div className="absolute text-center">
        <p className="text-lg font-bold tracking-tight">{value}</p>
        <p className="text-[0.6rem] text-muted-foreground">{label}</p>
      </div>
    </div>
  );
}

export function SparkBars({
  values,
  labels,
}: {
  values: number[];
  labels: string[];
}) {
  const max = Math.max(...values, 1);

  return (
    <div className="flex items-end gap-2">
      {values.map((value, index) => (
        <div
          key={`${labels[index]}-${value}`}
          className="flex flex-col items-center gap-1"
        >
          <div className="flex h-16 w-6 items-end overflow-hidden rounded-sm bg-muted/30">
            <span
              className="w-full rounded-sm bg-indigo-500 transition-all duration-300"
              style={{
                height: `${Math.max((value / max) * 100, 10)}%`,
              }}
            />
          </div>
          <div className="text-center">
            <p className="text-xs font-bold tracking-tight">{value}</p>
            <p className="text-[0.6rem] text-muted-foreground">
              {labels[index]}
            </p>
          </div>
        </div>
      ))}
    </div>
  );
}

export function CapabilityMatrix({
  items,
}: {
  items: Array<{ label: string; value: boolean | string | number | null }>;
}) {
  return (
    <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
      {items.map((item) => (
        <div
          key={item.label}
          className="flex items-center justify-between rounded-md border border-border/50 bg-muted/30 px-3 py-2.5"
        >
          <span className="text-xs text-muted-foreground">{item.label}</span>
          <span className="text-xs font-semibold tracking-tight">
            {typeof item.value === "boolean"
              ? item.value
                ? "Yes"
                : "No"
              : item.value ?? "n/a"}
          </span>
        </div>
      ))}
    </div>
  );
}
