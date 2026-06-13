import { For, createMemo, createResource, createSignal } from "solid-js";

import { api } from "../api";
import { PageHeader } from "../components/PageHeader";
import { formatTime, humanize, sampleJobs } from "../components/utils";

export function JobsPage() {
  const [sortKey, setSortKey] = createSignal<
    "status" | "created_at" | "updated_at"
  >("updated_at");
  const [jobs] = createResource(async () => {
    try {
      return await api.jobs();
    } catch {
      return sampleJobs();
    }
  });
  const sortedJobs = createMemo(() => {
    const key = sortKey();
    return [...(jobs() ?? [])].sort((a, b) =>
      String(b[key]).localeCompare(String(a[key])),
    );
  });

  return (
    <section class="page">
      <PageHeader
        title="Job Queue"
        subtitle="Active, in-progress, completed, and failed background work."
        right={
          <div class="segmented">
            <button
              classList={{ selected: sortKey() === "status" }}
              onClick={() => setSortKey("status")}
            >
              Status
            </button>
            <button
              classList={{ selected: sortKey() === "created_at" }}
              onClick={() => setSortKey("created_at")}
            >
              Created
            </button>
            <button
              classList={{ selected: sortKey() === "updated_at" }}
              onClick={() => setSortKey("updated_at")}
            >
              Updated
            </button>
          </div>
        }
      />
      <div class="job-table" role="table" aria-label="Job queue">
        <div class="job-row header" role="row">
          <span>Kind</span>
          <span>Status</span>
          <span>Attempts</span>
          <span>Updated</span>
          <span>Error</span>
        </div>
        <For each={sortedJobs()}>
          {(job) => (
            <div class="job-row" role="row">
              <span>{humanize(job.kind)}</span>
              <span class={`status-pill ${job.status}`}>
                {humanize(job.status)}
              </span>
              <span>
                {job.attempts}/{job.max_attempts}
              </span>
              <span>{formatTime(job.updated_at)}</span>
              <span class="error-text">{job.error ?? ""}</span>
            </div>
          )}
        </For>
      </div>
    </section>
  );
}
