import { For, Show } from "solid-js";

import type { DocumentCard, ViewMode } from "../api";
import { Icon } from "./Icon";
import { humanize, initials } from "./utils";

export function DocumentCardView(props: {
  document: DocumentCard;
  mode: ViewMode;
  onAddTopic?: (document: DocumentCard, topic: string) => void;
  onToggleFlag?: (
    document: DocumentCard,
    flag: "is_favorite" | "is_bookmarked" | "is_pinned",
  ) => void;
}) {
  let topicInput!: HTMLInputElement;
  const topics = () => props.document.classification?.topics ?? [];
  const title = () => props.document.title ?? props.document.file_name;
  const fileType = () => props.document.file_name.split(".").at(-1)?.toUpperCase() ?? "PDF";
  const folderName = () => props.document.path.split("/").slice(-2, -1)[0] ?? "Library";
  const fileDetail = () => {
    if (typeof props.document.file_size === "number") return formatBytes(props.document.file_size);
    if (typeof props.document.page_count === "number") return `${props.document.page_count} pages`;
    return "Unknown size";
  };

  return (
    <article class={`paper-card ${props.mode}`}>
      <div class="paper-card-actions">
        <button
          type="button"
          classList={{ active: Boolean(props.document.is_bookmarked) }}
          aria-label={`${props.document.is_bookmarked ? "Remove bookmark from" : "Bookmark"} ${props.document.file_name}`}
          aria-pressed={Boolean(props.document.is_bookmarked)}
          onClick={() => props.onToggleFlag?.(props.document, "is_bookmarked")}
        >
          <Icon name="bookmark" />
        </button>
        <button
          type="button"
          classList={{ active: Boolean(props.document.is_pinned) }}
          aria-label={`${props.document.is_pinned ? "Unpin" : "Pin"} ${props.document.file_name}`}
          aria-pressed={Boolean(props.document.is_pinned)}
          onClick={() => props.onToggleFlag?.(props.document, "is_pinned")}
        >
          <Icon name="pin" />
        </button>
        <button
          type="button"
          classList={{ active: Boolean(props.document.is_favorite) }}
          aria-label={`${props.document.is_favorite ? "Remove favorite from" : "Favorite"} ${props.document.file_name}`}
          aria-pressed={Boolean(props.document.is_favorite)}
          onClick={() => props.onToggleFlag?.(props.document, "is_favorite")}
        >
          <Icon name="star" />
        </button>
      </div>
      <div class="paper-main">
        <div class="cover" aria-label="PDF cover">
          <Show
            when={props.document.cover_url}
            fallback={<span>{initials(title())}</span>}
          >
            <img src={props.document.cover_url} alt="" loading="lazy" />
          </Show>
        </div>
        <div class="paper-body">
          <div class="paper-file-row">
            <span class="file-badge">{fileType()}</span>
            <span>{fileDetail()}</span>
          </div>
          <h2>{title()}</h2>
          <p class="filepath">{props.document.path}</p>
          <div class="paper-date-row">
            <span>{humanize(props.document.status)}</span>
            <span>{props.document.year ?? "No year"}</span>
          </div>
        </div>
      </div>
      <div class="topic-row">
        <For each={topics().slice(0, 3)}>{(topic) => <span>{topic}</span>}</For>
        <Show when={topics().length > 3}>
          <span>+{topics().length - 3}</span>
        </Show>
      </div>
      <Show when={props.onAddTopic}>
        <form
          class="classification-form"
          onSubmit={(event) => {
            event.preventDefault();
            props.onAddTopic?.(props.document, topicInput.value);
            topicInput.value = "";
          }}
        >
          <input
            ref={topicInput}
            aria-label={`Add classification for ${props.document.file_name}`}
            placeholder="Add topic"
          />
          <button type="submit">+</button>
        </form>
      </Show>
      <footer class="paper-footer">
        <div>
          <span>{folderName()}</span>
          <span>{humanize(props.document.classification?.document_type ?? "unclassified")}</span>
        </div>
        <div>
          <Icon name="quote" />
          <span>{props.document.page_count ?? 0}</span>
          <Icon name="more" />
        </div>
      </footer>
    </article>
  );
}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes < 0) return "Unknown size";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unitIndex]}`;
}
