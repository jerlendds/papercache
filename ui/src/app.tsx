import { Match, Switch, createEffect, createSignal, onCleanup } from "solid-js";
import { createStore } from "solid-js/store";

import { Sidebar } from "./components/Sidebar";
import { SettingsModal } from "./components/SettingsModal";
import { ToastStack } from "./components/ToastStack";
import { JobsPage } from "./pages/JobsPage";
import { LibraryPage } from "./pages/LibraryPage";
import { PlaceholderPage } from "./pages/PlaceholderPage";
import { SearchPage } from "./pages/SearchPage";
import type { Route, Toast, ToastKind } from "./components/types";
import { eventText } from "./components/utils";

type AppEvent = {
  type?: string;
  kind?: string;
  error?: string;
  folder_id?: string;
  document_id?: string;
  discovered?: number;
};

export function App() {
  const [route, setRoute] = createSignal<Route>("library");
  const [settingsOpen, setSettingsOpen] = createSignal(false);
  const [toasts, setToasts] = createStore<Toast[]>([]);
  const importProgress = new Map<string, { imported: number; total?: number }>();

  const notify = (message: string, kind: ToastKind = "info") => {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    setToasts(toasts.length, { id, message, kind });
    window.setTimeout(() => {
      setToasts((items) => items.filter((item) => item.id !== id));
    }, 3600);
  };

  const upsertToast = (
    key: string,
    message: string,
    kind: ToastKind = "info",
    autoDismiss = false,
  ) => {
    const index = toasts.findIndex((toast) => toast.key === key);
    if (index >= 0) {
      setToasts(index, "message", message);
      setToasts(index, "kind", kind);
      if (autoDismiss) {
        const id = toasts[index].id;
        window.setTimeout(() => {
          setToasts((items) => items.filter((item) => item.id !== id));
        }, 3600);
      }
      return;
    }

    const id = Date.now() + Math.floor(Math.random() * 1000);
    setToasts(toasts.length, { id, key, message, kind });
    if (autoDismiss) {
      window.setTimeout(() => {
        setToasts((items) => items.filter((item) => item.id !== id));
      }, 3600);
    }
  };

  const updateImportToast = (folderId: string) => {
    const progress = importProgress.get(folderId);
    if (!progress) return;
    const total = progress.total ?? 0;
    if (progress.total === 0) {
      upsertToast(`import:${folderId}`, "0/0 PDF files imported!", "success", true);
      return;
    }
    if (total > 0) {
      const imported = Math.min(progress.imported, total);
      const done = imported >= total;
      upsertToast(
        `import:${folderId}`,
        done
          ? `${imported}/${total} PDF files imported!`
          : `${imported}/${total} PDF files imported...`,
        done ? "success" : "info",
        done,
      );
      return;
    }
    upsertToast(`import:${folderId}`, "Scanning folder for PDFs...", "info");
  };

  const handleEvent = (event: AppEvent) => {
    window.dispatchEvent(
      new CustomEvent("papercache:event", { detail: event }),
    );

    if (event.type === "folder_scan_completed" && event.folder_id) {
      const progress = importProgress.get(event.folder_id) ?? { imported: 0 };
      progress.total = event.discovered ?? 0;
      importProgress.set(event.folder_id, progress);
      updateImportToast(event.folder_id);
      return;
    }

    if (event.type === "document_ready" && event.folder_id) {
      const progress = importProgress.get(event.folder_id) ?? { imported: 0 };
      progress.imported += 1;
      importProgress.set(event.folder_id, progress);
      updateImportToast(event.folder_id);
      return;
    }

    const text = eventText(event);
    if (text) notify(text.message, text.kind);
  };

  createEffect(() => {
    const source = new EventSource("/api/events");
    source.onmessage = (message) => {
      try {
        handleEvent(JSON.parse(message.data) as AppEvent);
      } catch {
        notify("Notification received", "info");
      }
    };
    source.onerror = () => source.close();
    onCleanup(() => source.close());
  });

  return (
    <div class="app-shell">
      <Sidebar
        route={route()}
        onNavigate={setRoute}
        onOpenSettings={() => setSettingsOpen(true)}
      />
      <main class="main-view">
        <Switch>
          <Match when={route() === "search"}>
            <SearchPage notify={notify} />
          </Match>
          <Match when={route() === "library"}>
            <LibraryPage notify={notify} />
          </Match>
          <Match when={route() === "chat"}>
            <PlaceholderPage
              title="Chat"
              body="Chat support will be added after the local retrieval pipeline is ready."
            />
          </Match>
          <Match when={route() === "graph"}>
            <PlaceholderPage
              title="Network Graph"
              body="I plan to eventually add support for extracting and visualizing citation network neighbourhoods for a paper with sigma.js"
            />
          </Match>
          <Match when={route() === "notifications"}>
            <PlaceholderPage
              title="Notifications"
              body="A persistent notification inbox will live here."
            />
          </Match>
          <Match when={route() === "jobs"}>
            <JobsPage />
          </Match>
        </Switch>
      </main>
      <SettingsModal
        open={settingsOpen()}
        onClose={() => setSettingsOpen(false)}
      />
      <ToastStack toasts={toasts} />
    </div>
  );
}
