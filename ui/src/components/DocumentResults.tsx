import { For, Show } from "solid-js";

import type { DocumentCard, ViewMode } from "../api";
import { EmptyState } from "../components/EmptyState";
import { DocumentCardView } from "./DocumentCardView";

export function DocumentResults(props: {
  mode: ViewMode;
  documents: DocumentCard[];
  emptyLabel: string;
}) {
  return (
    <Show
      when={props.documents.length > 0}
      fallback={<EmptyState label={props.emptyLabel} />}
    >
      <div class={`document-collection results ${props.mode}`}>
        <For each={props.documents}>
          {(document) => (
            <DocumentCardView document={document} mode={props.mode} />
          )}
        </For>
      </div>
    </Show>
  );
}
