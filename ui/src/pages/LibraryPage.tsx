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
import { PageHeader } from "../components/PageHeader";
import { ViewToggle } from "../components/ViewToggle";
import { VirtualDocumentList } from "../components/VirtualDocumentList";
import type { Notify } from "../components/types";
import { formatTime } from "../components/utils";

type LibraryRefreshEvent = {
  type?: string;
};

export function LibraryPage(props: { notify: Notify }) {
  const [mode, setMode] = createSignal<ViewMode>("grid");
  const [query, setQuery] = createSignal("");
  const [importsOpen, setImportsOpen] = createSignal(false);
  const [folderPath, setFolderPath] = createSignal("");
  const [recursive, setRecursive] = createSignal(true);
  const [isImporting, setIsImporting] = createSignal(false);
  const [documents, { refetch, mutate }] = createResource(
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
  const items = createMemo(() => documents() ?? []);
  let refreshTimer: number | undefined;

  const scheduleRefresh = () => {
    window.clearTimeout(refreshTimer);
    refreshTimer = window.setTimeout(() => {
      refetch();
      refetchFolders();
    }, 250);
  };

  onMount(() => {
    const onAppEvent = (event: Event) => {
      const detail = (event as CustomEvent<LibraryRefreshEvent>).detail;
      if (
        detail?.type === "document_discovered" ||
        detail?.type === "document_ready" ||
        detail?.type === "folder_scan_completed"
      ) {
        scheduleRefresh();
      }
    };

    window.addEventListener("papercache:event", onAppEvent);
    onCleanup(() => {
      window.removeEventListener("papercache:event", onAppEvent);
      window.clearTimeout(refreshTimer);
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
      refetch();
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
      refetch();
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
        right={
          <div class="header-actions">
            <button
              class="icon-text-button"
              type="button"
              onClick={() => setImportsOpen(true)}
            >
              <Icon name="folder-plus" />
              <span>Import folder</span>
            </button>
            <ViewToggle mode={mode()} onMode={setMode} />
          </div>
        }
      />
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
