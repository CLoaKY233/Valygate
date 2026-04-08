"use client";

import { useState, useRef, useEffect, useCallback } from "react";
import { Check, ChevronsUpDown, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export type SelectOption = {
  value: string;
  label: string;
  description?: string;
};

export function MultiSelect({
  name,
  options,
  defaultSelected = [],
  placeholder = "Select options...",
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

  const handleClickOutside = useCallback((e: MouseEvent) => {
    if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
      setOpen(false);
    }
  }, []);

  useEffect(() => {
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [handleClickOutside]);

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

  const selectedOptions = options.filter((o) => selected.includes(o.value));

  return (
    <div className="relative" ref={containerRef}>
      {selected.map((val) => (
        <input key={val} type="hidden" name={name} value={val} />
      ))}

      <Button
        type="button"
        variant="outline"
        role="combobox"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
        className="h-8 w-full justify-between font-normal"
      >
        <span className={cn("text-sm", selected.length === 0 && "text-muted-foreground")}>
          {selected.length === 0
            ? placeholder
            : `${selected.length} model${selected.length === 1 ? "" : "s"} selected`}
        </span>
        <ChevronsUpDown className="size-3.5 shrink-0 text-muted-foreground" />
      </Button>

      <div
        className={cn(
          "absolute inset-x-0 top-[calc(100%+4px)] z-50 rounded-md border border-border bg-popover shadow-md transition-all duration-150",
          open
            ? "pointer-events-auto translate-y-0 opacity-100"
            : "pointer-events-none -translate-y-1 opacity-0",
        )}
      >
        <div className="border-b border-border p-2">
          <input
            autoFocus={open}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search models..."
            className="h-7 w-full rounded-md border border-input bg-transparent px-2 text-sm outline-none transition-colors focus:border-ring focus:ring-2 focus:ring-ring/20"
            onClick={(e) => e.stopPropagation()}
          />
        </div>

        <div className="max-h-52 overflow-y-auto p-1">
          {filtered.length === 0 ? (
            <div className="py-3 text-center text-sm text-muted-foreground">
              No models found
            </div>
          ) : (
            filtered.map((opt) => {
              const isSelected = selected.includes(opt.value);
              return (
                <button
                  key={opt.value}
                  type="button"
                  onClick={() => toggle(opt.value)}
                  className={cn(
                    "flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-sm transition-colors",
                    isSelected ? "bg-accent/60" : "hover:bg-accent/40",
                  )}
                >
                  <span
                    className={cn(
                      "flex size-4 shrink-0 items-center justify-center rounded-[3px] border transition-colors",
                      isSelected
                        ? "border-primary bg-primary text-primary-foreground"
                        : "border-input bg-transparent",
                    )}
                  >
                    {isSelected && <Check className="size-3" strokeWidth={2.5} />}
                  </span>
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-[0.8rem] font-medium tracking-[-0.01em]">
                      {opt.label}
                    </div>
                    {opt.description && (
                      <div className="truncate font-mono text-[0.68rem] text-muted-foreground">
                        {opt.description}
                      </div>
                    )}
                  </div>
                </button>
              );
            })
          )}
        </div>

        <div className="flex items-center gap-2 border-t border-border px-3 py-1.5">
          <button
            type="button"
            className="text-xs font-medium text-foreground transition-colors hover:text-primary hover:underline"
            onClick={() => setSelected(options.map((o) => o.value))}
          >
            Select all
          </button>
          <span className="text-xs text-muted-foreground/50">|</span>
          <button
            type="button"
            className="text-xs font-medium text-foreground transition-colors hover:text-primary hover:underline"
            onClick={() => setSelected([])}
          >
            Clear
          </button>
          <span className="ml-auto text-[0.65rem] tabular-nums text-muted-foreground">
            {selected.length}/{options.length}
          </span>
        </div>
      </div>

      {selectedOptions.length > 0 && (
        <div className="mt-2 flex flex-wrap gap-1">
          {selectedOptions.map((opt) => (
            <Badge
              key={opt.value}
              variant="secondary"
              className="gap-1 rounded-md pr-1 text-[0.68rem]"
            >
              {opt.label}
              <button
                type="button"
                onClick={() => setSelected((prev) => prev.filter((v) => v !== opt.value))}
                className="rounded-[3px] p-0.5 transition-colors hover:bg-muted-foreground/20"
              >
                <X className="size-2.5" />
              </button>
            </Badge>
          ))}
        </div>
      )}
    </div>
  );
}
