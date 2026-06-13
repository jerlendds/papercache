import { Show, type JSX } from "solid-js";

export function Modal(props: {
  title: string;
  open: boolean;
  onClose: () => void;
  children: JSX.Element;
}) {
  return (
    <Show when={props.open}>
      <div class="modal-backdrop" onClick={props.onClose}>
        <section
          class="modal"
          role="dialog"
          aria-modal="true"
          aria-label={props.title}
          onClick={(event) => event.stopPropagation()}
        >
          <header class="modal-header">
            <h2>{props.title}</h2>
            <button type="button" aria-label="Close" onClick={props.onClose}>
              X
            </button>
          </header>
          <div class="modal-body">{props.children}</div>
        </section>
      </div>
    </Show>
  );
}
