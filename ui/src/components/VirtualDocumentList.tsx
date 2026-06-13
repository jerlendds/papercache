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

const GRID_CARD_HEIGHT = 386;
const LIST_CARD_HEIGHT = 188;
const ROW_GAP = 18;

export function VirtualDocumentList(props: {
  mode: ViewMode;
  documents: DocumentCard[];
  emptyLabel: string;
  onAddTopic: (document: DocumentCard, topic: string) => void;
  onToggleFlag?: (
    document: DocumentCard,
    flag: "is_favorite" | "is_bookmarked" | "is_pinned",
  ) => void;
}) {
  const [scrollTop, setScrollTop] = createSignal(0);
  const [viewportHeight, setViewportHeight] = createSignal(720);
  const [viewportWidth, setViewportWidth] = createSignal(1200);
  let viewport!: HTMLDivElement;
  const rowHeight = () =>
    (props.mode === "grid" ? GRID_CARD_HEIGHT : LIST_CARD_HEIGHT) + ROW_GAP;
  const columns = createMemo(() => {
    if (props.mode === "list") return 1;
    const contentWidth = Math.max(1, viewportWidth() - 56);
    const minCardWidth = 260;
    return Math.max(
      1,
      Math.min(4, Math.floor((contentWidth + ROW_GAP) / (minCardWidth + ROW_GAP))),
    );
  });
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
    const measure = () => {
      setViewportHeight(Math.max(1, viewport.clientHeight || 720));
      setViewportWidth(Math.max(1, viewport.clientWidth || 1200));
    };
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
            style={{
              "grid-template-columns":
                props.mode === "grid"
                  ? `repeat(${columns()}, minmax(260px, 1fr))`
                  : "1fr",
              transform: `translateY(${startRow() * rowHeight()}px)`,
            }}
          >
            <For each={visible()}>
              {(document) => (
                <DocumentCardView
                  document={document}
                  mode={props.mode}
                  onAddTopic={props.onAddTopic}
                  onToggleFlag={props.onToggleFlag}
                />
              )}
            </For>
          </div>
        </div>
      </div>
    </Show>
  );
}
