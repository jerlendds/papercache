import type { ToastKind } from "./types";

export function eventText(event: {
  type?: string;
  error?: string;
}): { message: string; kind: ToastKind } | null {
  switch (event.type) {
    case "job_queued":
      return null;
    case "document_ready":
      return null;
    case "document_discovered":
      return null;
    case "job_failed":
      return {
        message: event.error
          ? `Import failed: ${event.error}`
          : "Import failed!",
        kind: "error",
      };
    default:
      return null;
  }
}

export function humanize(value: string) {
  return value
    .replaceAll("_", " ")
    .replace(/^\w/, (char) => char.toUpperCase());
}

export function formatTime(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export function initials(value: string) {
  return value
    .split(/\s|-/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}
