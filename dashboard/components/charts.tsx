"use client";

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

const COLORS = ["#4648d4", "#6366f1", "#818cf8", "#a5b4fc", "#c7d2fe"];

export function ProviderBarChart({
  data,
}: {
  data: { name: string; models: number }[];
}) {
  if (data.length === 0) {
    return (
      <div style={{ height: 160, display: "grid", placeItems: "center" }}>
        <p className="text-muted">No providers synced yet.</p>
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={160}>
      <BarChart data={data} margin={{ top: 4, right: 4, left: -20, bottom: 4 }}>
        <XAxis
          dataKey="name"
          tick={{ fontSize: 11, fill: "var(--text-faint)" }}
          axisLine={false}
          tickLine={false}
        />
        <YAxis
          tick={{ fontSize: 11, fill: "var(--text-faint)" }}
          axisLine={false}
          tickLine={false}
          allowDecimals={false}
        />
        <Tooltip
          cursor={{ fill: "var(--surface-hover)" }}
          contentStyle={{
            background: "var(--canvas)",
            border: "1.5px solid var(--line-strong)",
            borderRadius: "var(--radius-md)",
            fontSize: 12,
            boxShadow: "var(--shadow-md)",
          }}
          labelStyle={{ color: "var(--text)", fontWeight: 600 }}
          itemStyle={{ color: "var(--primary)" }}
        />
        <Bar dataKey="models" radius={[4, 4, 0, 0]}>
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
      ? ["var(--surface-inset)"]
      : ["#059669", "#dc2626", "#d97706"];

  return (
    <div style={{ position: "relative" }}>
      <ResponsiveContainer width="100%" height={160}>
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={48}
            outerRadius={72}
            strokeWidth={0}
            dataKey="value"
          >
            {data.map((_, index) => (
              <Cell key={index} fill={donutColors[index % donutColors.length]} />
            ))}
          </Pie>
          <Tooltip
            contentStyle={{
              background: "var(--canvas)",
              border: "1.5px solid var(--line-strong)",
              borderRadius: "var(--radius-md)",
              fontSize: 12,
              boxShadow: "var(--shadow-md)",
            }}
          />
        </PieChart>
      </ResponsiveContainer>
      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "grid",
          placeItems: "center",
          pointerEvents: "none",
        }}
      >
        <div style={{ textAlign: "center" }}>
          <strong style={{ fontSize: "1.5rem", fontWeight: 800, letterSpacing: "-0.03em", display: "block" }}>
            {completed}
          </strong>
          <span style={{ fontSize: "0.6875rem", color: "var(--text-faint)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 700 }}>
            synced
          </span>
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
    <div className="arc-meter">
      <svg viewBox="0 0 100 100" className="arc-meter__chart">
        <circle cx="50" cy="50" r="42" className="arc-meter__track" />
        <circle
          cx="50"
          cy="50"
          r="42"
          className="arc-meter__value"
          style={{
            strokeDasharray: circumference,
            strokeDashoffset: dashOffset,
          }}
        />
      </svg>
      <div className="arc-meter__text">
        <strong>{value}</strong>
        <span>{label}</span>
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
    <div className="spark-bars">
      {values.map((value, index) => (
        <div key={`${labels[index]}-${value}`} className="spark-bars__item">
          <div className="spark-bars__track">
            <span style={{ height: `${Math.max((value / max) * 100, 10)}%` }} />
          </div>
          <div className="spark-bars__meta">
            <strong>{value}</strong>
            <span>{labels[index]}</span>
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
    <div className="capability-matrix">
      {items.map((item) => (
        <div key={item.label} className="capability-matrix__item">
          <span>{item.label}</span>
          <strong>
            {typeof item.value === "boolean"
              ? item.value
                ? "Yes"
                : "No"
              : item.value ?? "n/a"}
          </strong>
        </div>
      ))}
    </div>
  );
}
