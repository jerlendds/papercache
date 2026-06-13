import {
  For,
  Show,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";

import type { DocumentCard, ViewMode } from "../api";
import { EmptyState } from "./EmptyState";
import { DocumentCardView } from "./DocumentCardView";

export function VirtualDocumentList(props: {
  mode: ViewMode;
  documents: DocumentCard[];
  emptyLabel: string;
  onAddTopic: (document: DocumentCard, topic: string) => void;
}) {
  const [scrollTop, setScrollTop] = createSignal(0);
  const [viewportHeight, setViewportHeight] = createSignal(720);
  let viewport!: HTMLDivElement;
  const rowHeight = () => (props.mode === "grid" ? 248 : 132);
  const columns = () => (props.mode === "grid" ? 3 : 1);
  const rowCount = createMemo(() =>
    Math.ceil(props.documents.length / columns()),
  );
  const startRow = createMemo(() =>
    Math.max(0, Math.floor(scrollTop() / rowHeight()) - 2),
  );
  const visibleRows = createMemo(
    () => Math.ceil(viewportHeight() / rowHeight()) + 5,
  );
  const visible = createMemo(() => {
    const start = startRow() * columns();
    const end = Math.min(
      props.documents.length,
      start + visibleRows() * columns(),
    );
    return props.documents.slice(start, end);
  });

  onMount(() => {
    if (!viewport) return;
    const measure = () => setViewportHeight(Math.max(1, viewport.clientHeight || 720));
    measure();
    if (typeof ResizeObserver === "undefined") {
      window.addEventListener("resize", measure);
      onCleanup(() => window.removeEventListener("resize", measure));
      return;
    }
    const observer = new ResizeObserver(measure);
    observer.observe(viewport);
    onCleanup(() => observer.disconnect());
  });

  return (
    <Show
      when={props.documents.length > 0}
      fallback={<EmptyState label={props.emptyLabel} />}
    >
      <div
        ref={viewport}
        class="virtual-viewport"
        onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
      >
        <div
          style={{
            height: `${rowCount() * rowHeight()}px`,
            position: "relative",
          }}
        >
          <div
            class={`document-collection ${props.mode}`}
            style={{ transform: `translateY(${startRow() * rowHeight()}px)` }}
          >
            <For each={visible()}>
              {(document) => (
                <DocumentCardView
                  document={document}
                  mode={props.mode}
                  onAddTopic={props.onAddTopic}
                />
              )}
            </For>
          </div>
        </div>
      </div>
    </Show>
  );
}
