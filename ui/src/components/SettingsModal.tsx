import { createSignal } from "solid-js";

import { getAuthToken, setAuthToken } from "../api";
import { Modal } from "./Modal";

export function SettingsModal(props: { open: boolean; onClose: () => void }) {
  const [token, setToken] = createSignal(getAuthToken());

  const updateToken = (value: string) => {
    setToken(value);
    setAuthToken(value);
  };

  return (
    <Modal title="Settings" open={props.open} onClose={props.onClose}>
      <label class="settings-field">
        <span>Auth token</span>
        <input
          aria-label="Auth token"
          autocomplete="off"
          spellcheck={false}
          value={token()}
          onInput={(event) => updateToken(event.currentTarget.value)}
        />
      </label>
    </Modal>
  );
}
