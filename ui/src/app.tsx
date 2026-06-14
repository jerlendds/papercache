import { Match, Switch, createEffect, createSignal, onCleanup } from "solid-js";
import { createStore } from "solid-js/store";

import { Sidebar } from "./components/Sidebar";
import { ChatPaperIntro } from "./components/ChatPaperIntro";
import { SettingsModal } from "./components/SettingsModal";
import { ToastStack } from "./components/ToastStack";
import { JobsPage } from "./pages/JobsPage";
import { LibraryPage } from "./pages/LibraryPage";
import { PlaceholderPage } from "./pages/PlaceholderPage";
import { SearchPage } from "./pages/SearchPage";
import type { IngestProgress, Route, Toast, ToastKind } from "./components/types";
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
  const [ingestProgress, setIngestProgress] = createSignal<IngestProgress>({
    percent: null,
    imported: 0,
    total: null,
    active: false,
  });
  const [toasts, setToasts] = createStore<Toast[]>([]);
  const importProgress = new Map<string, { imported: number; total?: number }>();

  const notify = (message: string, kind: ToastKind = "info") => {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    setToasts(toasts.length, { id, message, kind });
    window.setTimeout(() => {
      setToasts((items) => items.filter((item) => item.id !== id));
    }, 3600);
  };

  const updateIngestProgress = () => {
    const progressItems = [...importProgress.values()];
    if (progressItems.length === 0) {
      setIngestProgress({
        percent: null,
        imported: 0,
        total: null,
        active: false,
      });
      return;
    }

    const imported = progressItems.reduce(
      (sum, progress) => sum + progress.imported,
      0,
    );
    const totals = progressItems
      .map((progress) => progress.total)
      .filter((total): total is number => typeof total === "number");
    const total =
      totals.length === progressItems.length
        ? totals.reduce((sum, value) => sum + value, 0)
        : null;
    const percent =
      total === null
        ? null
        : total === 0
          ? 100
          : Math.min(100, Math.round((imported / total) * 100));

    setIngestProgress({
      percent,
      imported,
      total,
      active: true,
    });
  };

  const handleEvent = (event: AppEvent) => {
    window.dispatchEvent(
      new CustomEvent("papercache:event", { detail: event }),
    );

    if (event.type === "folder_scan_completed" && event.folder_id) {
      const progress = importProgress.get(event.folder_id) ?? { imported: 0 };
      progress.total = event.discovered ?? 0;
      importProgress.set(event.folder_id, progress);
      updateIngestProgress();
      return;
    }

    if (event.type === "document_ready" && event.folder_id) {
      const progress = importProgress.get(event.folder_id) ?? { imported: 0 };
      progress.imported += 1;
      importProgress.set(event.folder_id, progress);
      updateIngestProgress();
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
        progress={ingestProgress()}
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
            <ChatPaperIntro />
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
