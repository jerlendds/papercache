import { Match, Switch, createEffect, createSignal, onCleanup } from "solid-js";
import { createStore } from "solid-js/store";

import { Sidebar } from "./components/Sidebar";
import { ToastStack } from "./components/ToastStack";
import { JobsPage } from "./pages/JobsPage";
import { LibraryPage } from "./pages/LibraryPage";
import { PlaceholderPage } from "./pages/PlaceholderPage";
import { SearchPage } from "./pages/SearchPage";
import type { Route, Toast, ToastKind } from "./components/types";
import { eventText } from "./components/utils";

export function App() {
  const [route, setRoute] = createSignal<Route>("search");
  const [toasts, setToasts] = createStore<Toast[]>([]);

  const notify = (message: string, kind: ToastKind = "info") => {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    setToasts(toasts.length, { id, message, kind });
    window.setTimeout(() => {
      setToasts((items) => items.filter((item) => item.id !== id));
    }, 3600);
  };

  createEffect(() => {
    const source = new EventSource("/api/events");
    source.onmessage = (message) => {
      try {
        const event = JSON.parse(message.data) as {
          type?: string;
          error?: string;
        };
        const text = eventText(event);
        if (text) notify(text.message, text.kind);
      } catch {
        notify("Notification received", "info");
      }
    };
    source.onerror = () => source.close();
    onCleanup(() => source.close());
  });

  return (
    <div class="app-shell">
      <Sidebar route={route()} onNavigate={setRoute} />
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
      <ToastStack toasts={toasts} />
    </div>
  );
}
