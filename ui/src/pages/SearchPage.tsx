import { For, Show, createMemo, createResource, createSignal } from "solid-js";

import type { DocumentCard, SearchResult, ViewMode } from "../api";
import { api } from "../api";
import { ViewToggle } from "../components/ViewToggle";
import { DocumentResults } from "../components/DocumentResults";
import { Icon } from "../components/Icon";
import type { Notify } from "../components/types";
import { sampleDocuments, sampleSearchResults } from "../components/utils";

export function SearchPage(props: { notify: Notify }) {
  const [query, setQuery] = createSignal("");
  const [submitted, setSubmitted] = createSignal("");
  const [mode, setMode] = createSignal<ViewMode>("grid");
  const [search] = createResource(submitted, async (value) => {
    if (!value.trim()) return { results: [] };
    try {
      return await api.search(value.trim(), 36, 0);
    } catch {
      props.notify("Search failed", "error");
      return { results: sampleSearchResults(value) };
    }
  });
  const suggestions = createMemo(() => {
    const value = query().trim().toLowerCase();
    if (!value || submitted() === query()) return [];
    return sampleDocuments
      .filter((document) =>
        `${document.title} ${document.path}`.toLowerCase().includes(value),
      )
      .slice(0, 5);
  });

  const runSearch = () => {
    const value = query().trim();
    if (value) setSubmitted(value);
  };

  return (
    <section class="page search-page">
      <ViewToggle mode={mode()} onMode={setMode} />
      <div class="search-center">
        <h1>Search papers</h1>
        <div class="search-box">
          <Icon name="search" />
          <input
            aria-label="Search papers"
            placeholder="retrieval augmented generation"
            value={query()}
            onInput={(event) => setQuery(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") runSearch();
            }}
          />
        </div>
        <Show when={suggestions().length > 0}>
          <div class="suggestions" role="listbox">
            <For each={suggestions()}>
              {(item) => (
                <button
                  role="option"
                  onClick={() => {
                    setQuery(item.title ?? item.file_name);
                    setSubmitted(item.title ?? item.file_name);
                  }}
                >
                  <strong>{item.title ?? item.file_name}</strong>
                  <span>{item.path}</span>
                </button>
              )}
            </For>
          </div>
        </Show>
      </div>
      <Show when={submitted()}>
        <DocumentResults
          mode={mode()}
          documents={(search()?.results ?? []).map(searchResultToDocument)}
          emptyLabel={
            search.loading ? "Searching..." : "No matching indexed chunks found"
          }
        />
      </Show>
    </section>
  );
}

function searchResultToDocument(result: SearchResult): DocumentCard {
  return {
    id: result.document_id,
    path: result.path,
    title: result.title,
    file_name: result.path.split("/").at(-1) ?? result.title,
    status: "ready",
    page_count: result.page_end,
    classification: { topics: [], source: `score ${result.score.toFixed(2)}` },
    cover_url: `/api/documents/${result.document_id}/cover`,
  };
}
