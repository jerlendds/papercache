import { For } from 'solid-js';

import type { Toast } from '../types';

export function ToastStack(props: { toasts: Toast[] }) {
  return (
    <div class="toast-stack" aria-live="polite" aria-atomic="true">
      <For each={props.toasts}>
        {(toast) => <div class={`toast ${toast.kind}`}>{toast.message}</div>}
      </For>
    </div>
  );
}
