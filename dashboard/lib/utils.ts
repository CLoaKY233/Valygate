import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** Strip SurrealDB table prefix from record IDs: "table:id" → "id" */
export function stripRecordPrefix(id: string): string {
  const colon = id.indexOf(":");
  return colon !== -1 ? id.slice(colon + 1) : id;
}
