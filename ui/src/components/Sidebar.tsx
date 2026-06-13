import { For } from "solid-js";

import logoUrl from "../assets/logo.svg";
import type { Route } from "./types";
import { Icon } from "./Icon";

const paperRoutes: Array<{
  id: Route;
  label: string;
  tooltip: string;
  icon: string;
}> = [
  {
    id: "library",
    label: "Library",
    tooltip: "Browse indexed papers",
    icon: "library",
  },
  {
    id: "search",
    label: "Search",
    tooltip: "Search indexed PDF text",
    icon: "search",
  },
];

const workspaceRoutes: Array<{
  id: Route;
  label: string;
  tooltip: string;
  icon: string;
}> = [
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
  onOpenSettings: () => void;
}) {
  return (
    <aside class="sidebar" aria-label="Primary navigation">
      <img class="brand-mark" src={logoUrl} alt="papercache" />
      <nav class="nav-icons" aria-label="Paper navigation">
        <For each={paperRoutes}>
          {(item) => <SidebarButton item={item} active={props.route === item.id} onNavigate={props.onNavigate} />}
        </For>
      </nav>
      <nav class="nav-icons" aria-label="Workspace navigation">
        <span class="nav-section-label">Tools</span>
        <For each={workspaceRoutes}>
          {(item) => <SidebarButton item={item} active={props.route === item.id} onNavigate={props.onNavigate} />}
        </For>
      </nav>
      <div class="sidebar-footer">
        <span class="nav-section-label">System</span>
        <button
          class="nav-button"
          aria-label="Settings"
          onClick={props.onOpenSettings}
        >
          <Icon name="settings" />
          <span class="tooltip" role="tooltip">
            Settings
          </span>
        </button>
      </div>
    </aside>
  );
}

function SidebarButton(props: {
  item: {
    id: Route;
    label: string;
    tooltip: string;
    icon: string;
  };
  active: boolean;
  onNavigate: (route: Route) => void;
}) {
  return (
    <button
      class="nav-button"
      classList={{ active: props.active }}
      aria-label={props.item.label}
      onClick={() => props.onNavigate(props.item.id)}
    >
      <Icon name={props.item.icon} />
      <span class="tooltip" role="tooltip">
        {props.item.tooltip}
      </span>
    </button>
  );
}
