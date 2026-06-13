import { createMemo, createResource, createSignal } from "solid-js";

import type { Classification, DocumentCard, ViewMode } from "../api";
import { api } from "../api";
import { PageHeader } from "../components/PageHeader";
import { ViewToggle } from "../components/ViewToggle";
import { VirtualDocumentList } from "../components/VirtualDocumentList";
import type { Notify } from "../components/types";
import { sampleDocuments } from "../components/utils";

export function LibraryPage(props: { notify: Notify }) {
  const [mode, setMode] = createSignal<ViewMode>("grid");
  const [query, setQuery] = createSignal("");
  const [documents, { refetch, mutate }] = createResource(
    () => query(),
    async (q) => {
      try {
        const items = await api.documents(500, 0, q);
        return items.length ? items : sampleDocuments;
      } catch {
        return sampleDocuments;
      }
    },
  );
  const items = createMemo(() => documents() ?? sampleDocuments);

  const addTopic = async (document: DocumentCard, topic: string) => {
    const value = topic.trim();
    if (!value) return;
    const nextClassification: Classification = {
      ...(document.classification ?? {}),
      topics: [...(document.classification?.topics ?? []), value],
    };
    mutate((current) =>
      (current ?? []).map((item) =>
        item.id === document.id
          ? { ...item, classification: nextClassification }
          : item,
      ),
    );
    try {
      await api.updateClassification(document.id, nextClassification);
      props.notify("Classification updated", "success");
      refetch();
    } catch {
      props.notify("Classification update failed", "error");
    }
  };

  return (
    <section class="page">
      <PageHeader
        title="Library"
        subtitle="Indexed papers, metadata, and classifications."
        right={<ViewToggle mode={mode()} onMode={setMode} />}
      />
      <div class="library-toolbar">
        <input
          aria-label="Filter library"
          placeholder="Filter by title, file name, or path"
          value={query()}
          onInput={(event) => setQuery(event.currentTarget.value)}
        />
      </div>
      <VirtualDocumentList
        mode={mode()}
        documents={items()}
        onAddTopic={addTopic}
        emptyLabel={
          documents.loading
            ? "Loading indexed papers..."
            : "No indexed papers yet"
        }
      />
    </section>
  );
}
