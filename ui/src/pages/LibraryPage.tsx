import {
  For,
  Show,
  createMemo,
  createResource,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";

import { ApiError, type Classification, type DocumentCard, type ViewMode } from "../api";
import { api } from "../api";
import { Icon } from "../components/Icon";
import { Modal } from "../components/Modal";
import { ViewToggle } from "../components/ViewToggle";
import { VirtualDocumentList } from "../components/VirtualDocumentList";
import type { Notify } from "../components/types";
import { formatTime } from "../components/utils";

type LibraryRefreshEvent = {
  type?: string;
  document_id?: string;
  folder_id?: string;
};

export function LibraryPage(props: { notify: Notify }) {
  const [mode, setMode] = createSignal<ViewMode>("grid");
  const [query, setQuery] = createSignal("");
  const [importsOpen, setImportsOpen] = createSignal(false);
  const [folderPath, setFolderPath] = createSignal("");
  const [recursive, setRecursive] = createSignal(true);
  const [isImporting, setIsImporting] = createSignal(false);
  const [documents, { mutate }] = createResource(
    () => query(),
    async (q) => {
      try {
        return await api.documents(500, 0, q);
      } catch {
        props.notify("Library load failed", "error");
        return [];
      }
    },
  );
  const [folders, { refetch: refetchFolders }] = createResource(async () => {
    try {
      return await api.folders();
    } catch {
      props.notify("Imported folders failed to load", "error");
      return [];
    }
  });
  const [documentCount, { refetch: refetchDocumentCount }] =
    createResource(async () => {
      try {
        return await api.documentCount();
      } catch {
        props.notify("Paper count failed to load", "error");
        return { total: 0 };
      }
    });
  const items = createMemo(() => documents() ?? []);
  const totalPapers = createMemo(() => documentCount()?.total ?? items().length);
  const pendingDocumentIds = new Set<string>();
  let patchTimer: number | undefined;

  const matchesQuery = (document: DocumentCard) => {
    const value = query().trim().toLocaleLowerCase();
    if (!value) return true;
    return [
      document.title,
      document.file_name,
      document.path,
      document.doi,
      document.arxiv_id,
      ...(document.authors ?? []),
    ].some((field) => field?.toLocaleLowerCase().includes(value));
  };

  const patchDocument = (nextDocument: DocumentCard) => {
    mutate((current) => {
      const currentDocuments = current ?? [];
      const index = currentDocuments.findIndex((item) => item.id === nextDocument.id);
      if (!matchesQuery(nextDocument)) {
        return index === -1
          ? currentDocuments
          : currentDocuments.filter((item) => item.id !== nextDocument.id);
      }
      if (index === -1) return [...currentDocuments, nextDocument];
      return currentDocuments.map((item) =>
        item.id === nextDocument.id ? nextDocument : item,
      );
    });
  };

  const flushDocumentPatches = async () => {
    const ids = [...pendingDocumentIds];
    pendingDocumentIds.clear();
    await Promise.all(
      ids.map(async (id) => {
        try {
          patchDocument(await api.document(id));
        } catch {
          // Ignore stale events for documents that were deleted or are not readable yet.
        }
      }),
    );
  };

  const scheduleDocumentPatch = (documentId: string) => {
    pendingDocumentIds.add(documentId);
    window.clearTimeout(patchTimer);
    patchTimer = window.setTimeout(() => {
      void flushDocumentPatches();
    }, 150);
  };

  onMount(() => {
    const onAppEvent = (event: Event) => {
      const detail = (event as CustomEvent<LibraryRefreshEvent>).detail;
      if (detail?.type === "folder_scan_completed") {
        refetchFolders();
        refetchDocumentCount();
        return;
      }
      if (
        (detail?.type === "document_discovered" ||
          detail?.type === "document_ready") &&
        detail.document_id
      ) {
        if (detail.type === "document_discovered") refetchDocumentCount();
        scheduleDocumentPatch(detail.document_id);
      }
    };

    window.addEventListener("papercache:event", onAppEvent);
    onCleanup(() => {
      window.removeEventListener("papercache:event", onAppEvent);
      window.clearTimeout(patchTimer);
    });
  });

  const importFolder = async (event: SubmitEvent) => {
    event.preventDefault();
    const path = folderPath().trim();
    if (!path || isImporting()) return;
    setIsImporting(true);
    try {
      await api.importFolder(path, recursive());
      setFolderPath("");
      props.notify("Folder scan queued", "success");
      refetchFolders();
      refetchDocumentCount();
    } catch (error) {
      console.error("Folder import failed", {
        path,
        recursive: recursive(),
        error,
      });
      const detail =
        error instanceof ApiError ? `: ${error.message}` : "";
      props.notify(`Folder import failed${detail}`, "error");
    } finally {
      setIsImporting(false);
    }
  };

  const disableFolder = async (folderId: string) => {
    try {
      await api.disableFolder(folderId);
      props.notify("Folder import disabled", "success");
      refetchFolders();
      refetchDocumentCount();
      mutate((current) =>
        (current ?? []).map((document) =>
          document.folder_id === folderId
            ? { ...document, status: "missing" }
            : document,
        ),
      );
    } catch {
      props.notify("Folder disable failed", "error");
    }
  };

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
    } catch {
      props.notify("Classification update failed", "error");
    }
  };

  const toggleDocumentFlag = async (
    document: DocumentCard,
    flag: "is_favorite" | "is_bookmarked" | "is_pinned",
  ) => {
    const nextValue = !Boolean(document[flag]);
    mutate((current) =>
      (current ?? []).map((item) =>
        item.id === document.id ? { ...item, [flag]: nextValue } : item,
      ),
    );
    try {
      const nextDocument = await api.updateDocumentFlags(document.id, {
        [flag]: nextValue,
      });
      patchDocument(nextDocument);
    } catch {
      mutate((current) =>
        (current ?? []).map((item) =>
          item.id === document.id ? { ...item, [flag]: document[flag] } : item,
        ),
      );
      props.notify("Document flag update failed", "error");
    }
  };

  return (
    <section class="page library-page">
      <header class="library-command-bar">
        <div class="library-search-field">
          <Icon name="search" />
          <input
            aria-label="Filter library"
            placeholder="Search title, authors, DOI, or keywords..."
            value={query()}
            onInput={(event) => setQuery(event.currentTarget.value)}
          />
          <kbd>/</kbd>
        </div>
        <button class="paper-control" type="button">
          <Icon name="filter" />
          <span>Filters</span>
          <Icon name="chevron-down" />
        </button>
        <button class="paper-control" type="button">
          <Icon name="sort" />
          <span>Sort: Date added</span>
          <Icon name="chevron-down" />
        </button>
        <button class="paper-control collection-control" type="button">
          <span>
            <small>Collections</small>
            All Papers
          </span>
          <Icon name="chevron-down" />
        </button>
        <button
          class="archive-button"
          type="button"
          aria-label="Import folder"
          onClick={() => setImportsOpen(true)}
        >
          <Icon name="folder-plus" />
          <span>Archive Papers</span>
          <Icon name="chevron-down" />
        </button>
      </header>
      <div class="library-title-row">
        <div class="library-heading">
          <h1>All Papers</h1>
          <span>{totalPapers().toLocaleString()} papers</span>
        </div>
        <div class="library-view-row">
          <ViewToggle mode={mode()} onMode={setMode} />
          <span class="selection-state">0 selected</span>
          <span class="page-size-state">View: 8 per page</span>
        </div>
      </div>
      <Modal
        title="Folder imports"
        open={importsOpen()}
        onClose={() => setImportsOpen(false)}
      >
        <form class="folder-import-form" onSubmit={importFolder}>
          <label class="folder-path-field">
            <span>Directory path</span>
            <input
              aria-label="Folder path"
              placeholder="/Users/example/Papers"
              value={folderPath()}
              onInput={(event) => setFolderPath(event.currentTarget.value)}
            />
          </label>
          <label class="checkbox-field">
            <input
              type="checkbox"
              checked={recursive()}
              onChange={(event) => setRecursive(event.currentTarget.checked)}
            />
            <span>Include subfolders</span>
          </label>
          <button type="submit" disabled={isImporting() || !folderPath().trim()}>
            {isImporting() ? "Queueing..." : "Import folder"}
          </button>
        </form>
        <div class="folder-list">
          <Show
            when={(folders() ?? []).length > 0}
            fallback={<p class="folder-empty">No imported folders yet.</p>}
          >
            <For each={folders() ?? []}>
              {(folder) => (
                <div class="folder-row">
                  <div class="folder-main">
                    <strong>{folder.path}</strong>
                    <span>
                      {folder.document_count} documents -{" "}
                      {folder.recursive ? "recursive" : "top level only"}
                      {folder.last_scan_at
                        ? ` - scanned ${formatTime(folder.last_scan_at)}`
                        : " - scan pending"}
                    </span>
                  </div>
                  <button
                    type="button"
                    disabled={!folder.enabled}
                    onClick={() => disableFolder(folder.id)}
                  >
                    {folder.enabled ? "Disable" : "Disabled"}
                  </button>
                </div>
              )}
            </For>
          </Show>
        </div>
      </Modal>
      <VirtualDocumentList
        mode={mode()}
        documents={items()}
        onAddTopic={addTopic}
        onToggleFlag={toggleDocumentFlag}
        emptyLabel={
          documents.loading
            ? "Loading indexed papers..."
            : "No indexed papers yet"
        }
      />
    </section>
  );
}
