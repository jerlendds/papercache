import { For, Show, createMemo, createSignal } from "solid-js";

import type { DocumentCard, ViewMode } from "../api";
import { EmptyState } from "./EmptyState";
import { DocumentCardView } from "./DocumentCardView";

const VIEWPORT_HEIGHT = 620;

export function VirtualDocumentList(props: {
  mode: ViewMode;
  documents: DocumentCard[];
  emptyLabel: string;
  onAddTopic: (document: DocumentCard, topic: string) => void;
}) {
  const [scrollTop, setScrollTop] = createSignal(0);
  const rowHeight = () => (props.mode === "grid" ? 248 : 132);
  const columns = () => (props.mode === "grid" ? 3 : 1);
  const rowCount = createMemo(() =>
    Math.ceil(props.documents.length / columns()),
  );
  const startRow = createMemo(() =>
    Math.max(0, Math.floor(scrollTop() / rowHeight()) - 2),
  );
  const visibleRows = createMemo(
    () => Math.ceil(VIEWPORT_HEIGHT / rowHeight()) + 5,
  );
  const visible = createMemo(() => {
    const start = startRow() * columns();
    const end = Math.min(
      props.documents.length,
      start + visibleRows() * columns(),
    );
    return props.documents.slice(start, end);
  });

  return (
    <Show
      when={props.documents.length > 0}
      fallback={<EmptyState label={props.emptyLabel} />}
    >
      <div
        class="virtual-viewport"
        style={{ height: `${VIEWPORT_HEIGHT}px` }}
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
