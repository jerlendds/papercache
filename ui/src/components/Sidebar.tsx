import { For } from "solid-js";

import type { Route } from "./types";
import { Icon } from "./Icon";

const routes: Array<{
  id: Route;
  label: string;
  tooltip: string;
  icon: string;
}> = [
  {
    id: "search",
    label: "Search",
    tooltip: "Search indexed PDF text",
    icon: "search",
  },
  {
    id: "library",
    label: "Library",
    tooltip: "Browse indexed papers",
    icon: "library",
  },
  {
    id: "chat",
    label: "Chat",
    tooltip: "Chat with your library",
    icon: "chat",
  },
  {
    id: "graph",
    label: "Network",
    tooltip: "Explore citation neighborhoods",
    icon: "graph",
  },
  {
    id: "notifications",
    label: "Notifications",
    tooltip: "Review app notifications",
    icon: "bell",
  },
  {
    id: "jobs",
    label: "Jobs",
    tooltip: "Inspect ingestion and indexing jobs",
    icon: "queue",
  },
];

export function Sidebar(props: {
  route: Route;
  onNavigate: (route: Route) => void;
}) {
  return (
    <aside class="sidebar" aria-label="Primary navigation">
      <div class="brand-mark" aria-label="papercache">
        P
      </div>
      <nav class="nav-icons">
        <For each={routes}>
          {(item) => (
            <button
              class="nav-button"
              classList={{ active: props.route === item.id }}
              aria-label={item.label}
              title={item.tooltip}
              onClick={() => props.onNavigate(item.id)}
            >
              <Icon name={item.icon} />
              <span class="tooltip" role="tooltip">
                {item.tooltip}
              </span>
            </button>
          )}
        </For>
      </nav>
    </aside>
  );
}
