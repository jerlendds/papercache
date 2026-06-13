import { For, Show } from "solid-js";

import type { DocumentCard, ViewMode } from "../api";
import { humanize, initials } from "./utils";

export function DocumentCardView(props: {
  document: DocumentCard;
  mode: ViewMode;
  onAddTopic?: (document: DocumentCard, topic: string) => void;
}) {
  let topicInput!: HTMLInputElement;
  const topics = () => props.document.classification?.topics ?? [];

  return (
    <article class={`paper-card ${props.mode}`}>
      <div class="cover" aria-label="PDF cover">
        <Show
          when={props.document.cover_url}
          fallback={
            <span>
              {initials(props.document.title ?? props.document.file_name)}
            </span>
          }
        >
          <img src={props.document.cover_url} alt="" loading="lazy" />
        </Show>
      </div>
      <div class="paper-body">
        <h2>{props.document.title ?? props.document.file_name}</h2>
        <p class="filepath">{props.document.path}</p>
        <div class="meta-line">
          <span>{humanize(props.document.status)}</span>
          <span>{props.document.page_count ?? "Unknown"} pages</span>
          <span>
            {humanize(
              props.document.classification?.document_type ?? "unclassified",
            )}
          </span>
        </div>
        <div class="topic-row">
          <For each={topics()}>{(topic) => <span>{topic}</span>}</For>
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
            <button type="submit">Add</button>
          </form>
        </Show>
      </div>
    </article>
  );
}
