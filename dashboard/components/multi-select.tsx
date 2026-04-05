"use client";

import { useState, useRef, useEffect } from "react";
import { ChevronDown, X } from "lucide-react";

export type SelectOption = {
  value: string;
  label: string;
  description?: string;
};

export function MultiSelect({
  name,
  options,
  defaultSelected = [],
  placeholder = "Select options…",
}: {
  name: string;
  options: SelectOption[];
  defaultSelected?: string[];
  placeholder?: string;
}) {
  const [selected, setSelected] = useState<string[]>(defaultSelected);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const filtered = options.filter((opt) =>
    search
      ? opt.label.toLowerCase().includes(search.toLowerCase()) ||
        opt.value.toLowerCase().includes(search.toLowerCase())
      : true,
  );

  function toggle(value: string) {
    setSelected((prev) =>
      prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value],
    );
  }

  function removeSelected(value: string) {
    setSelected((prev) => prev.filter((v) => v !== value));
  }

  function selectAll() {
    setSelected(options.map((o) => o.value));
  }

  function clearAll() {
    setSelected([]);
  }

  const selectedOptions = options.filter((o) => selected.includes(o.value));

  return (
    <div className="multi-select" ref={containerRef}>
      {/* Hidden inputs for form submission */}
      {selected.map((val) => (
        <input key={val} type="hidden" name={name} value={val} />
      ))}

      <button
        type="button"
        className="multi-select__trigger"
        onClick={() => setOpen((prev) => !prev)}
        aria-expanded={open}
      >
        <span style={{ color: selected.length === 0 ? "var(--text-faint)" : "var(--text)" }}>
          {selected.length === 0
            ? placeholder
            : `${selected.length} model${selected.length === 1 ? "" : "s"} selected`}
        </span>
        <ChevronDown
          size={14}
          style={{
            flexShrink: 0,
            color: "var(--text-faint)",
            transform: open ? "rotate(180deg)" : "rotate(0deg)",
            transition: "transform 150ms",
          }}
        />
      </button>

      {open && (
        <div className="multi-select__dropdown">
          <div className="multi-select__search">
            <input
              autoFocus
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search models…"
              onClick={(e) => e.stopPropagation()}
            />
          </div>

          <div className="multi-select__list">
            {filtered.length === 0 ? (
              <div style={{ padding: "1rem", color: "var(--text-faint)", fontSize: "0.875rem" }}>
                No models found
              </div>
            ) : (
              filtered.map((opt) => (
                <label key={opt.value} className="multi-select__item">
                  <input
                    type="checkbox"
                    checked={selected.includes(opt.value)}
                    onChange={() => toggle(opt.value)}
                  />
                  <div>
                    <div className="multi-select__item-label">{opt.label}</div>
                    {opt.description && (
                      <div className="multi-select__item-desc">{opt.description}</div>
                    )}
                  </div>
                </label>
              ))
            )}
          </div>

          <div className="multi-select__actions">
            <button type="button" className="multi-select__action-btn" onClick={selectAll}>
              Select all
            </button>
            <span style={{ color: "var(--text-faint)", fontSize: "0.75rem" }}>·</span>
            <button type="button" className="multi-select__action-btn" onClick={clearAll}>
              Clear all
            </button>
          </div>
        </div>
      )}

      {selectedOptions.length > 0 && (
        <div className="multi-select__pills">
          {selectedOptions.map((opt) => (
            <span key={opt.value} className="multi-select__pill">
              {opt.label}
              <button type="button" onClick={() => removeSelected(opt.value)} aria-label={`Remove ${opt.label}`}>
                <X size={10} />
              </button>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
