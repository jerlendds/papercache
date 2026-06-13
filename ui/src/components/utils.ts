import type { DocumentCard, Job, SearchResult } from "../api";
import type { ToastKind } from "./types";

export function eventText(event: {
  type?: string;
  error?: string;
}): { message: string; kind: ToastKind } | null {
  switch (event.type) {
    case "job_queued":
      return { message: "PDF ingestion queued!", kind: "info" };
    case "document_ready":
      return { message: "Indexing finished!", kind: "success" };
    case "document_discovered":
      return { message: "Import success", kind: "success" };
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

export const sampleDocuments: DocumentCard[] = [
  {
    id: "sample-1",
    path: "/Users/example/Papers/retrieval-augmented-generation.pdf",
    title: "Retrieval Augmented Generation for Knowledge Intensive Tasks",
    file_name: "retrieval-augmented-generation.pdf",
    status: "ready",
    page_count: 14,
    classification: {
      document_type: "research_paper",
      topics: ["retrieval", "rag", "machine learning"],
      confidence: 0.62,
      source: "rules-v1",
    },
    cover_url: "",
  },
  {
    id: "sample-2",
    path: "/Users/example/Papers/local-first-software.pdf",
    title: "Local-first software: You own your data",
    file_name: "local-first-software.pdf",
    status: "ready",
    page_count: 22,
    classification: {
      document_type: "research_paper",
      topics: ["systems", "databases"],
      confidence: 0.58,
      source: "rules-v1",
    },
    cover_url: "",
  },
];

export function sampleSearchResults(query: string): SearchResult[] {
  return sampleDocuments
    .filter((document) =>
      `${document.title} ${document.path}`
        .toLowerCase()
        .includes(query.toLowerCase()),
    )
    .map((document, index) => ({
      document_id: document.id,
      chunk_id: `${document.id}-chunk-${index}`,
      title: document.title ?? document.file_name,
      path: document.path,
      page_start: 1,
      page_end: document.page_count ?? 1,
      snippet: "Sample local result while the backend is unavailable.",
      score: 1,
    }));
}

export function sampleJobs(): Job[] {
  return [
    {
      id: "job-1",
      kind: "scan_folder",
      status: "succeeded",
      priority: 10,
      attempts: 1,
      max_attempts: 3,
      created_at: new Date(Date.now() - 120000).toISOString(),
      updated_at: new Date(Date.now() - 90000).toISOString(),
    },
    {
      id: "job-2",
      kind: "ingest_pdf",
      status: "running",
      priority: 0,
      attempts: 0,
      max_attempts: 3,
      created_at: new Date(Date.now() - 50000).toISOString(),
      updated_at: new Date(Date.now() - 10000).toISOString(),
    },
  ];
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
