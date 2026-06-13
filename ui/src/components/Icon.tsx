import { Match, Switch } from 'solid-js';

export function Icon(props: { name: string }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <Switch>
        <Match when={props.name === 'search'}>
          <circle cx="11" cy="11" r="6" />
          <path d="m16 16 4 4" />
        </Match>
        <Match when={props.name === 'library'}>
          <path d="M5 4h12a2 2 0 0 1 2 2v14H7a2 2 0 0 1-2-2z" />
          <path d="M7 4v14" />
          <path d="M9 8h6" />
        </Match>
        <Match when={props.name === 'chat'}>
          <path d="M5 6h14v9H8l-3 3z" />
        </Match>
        <Match when={props.name === 'graph'}>
          <circle cx="6" cy="12" r="2" />
          <circle cx="17" cy="7" r="2" />
          <circle cx="18" cy="17" r="2" />
          <path d="m8 11 7-3" />
          <path d="m8 13 8 3" />
        </Match>
        <Match when={props.name === 'bell'}>
          <path d="M18 10a6 6 0 0 0-12 0c0 5-2 5-2 7h16c0-2-2-2-2-7" />
          <path d="M10 20h4" />
        </Match>
        <Match when={props.name === 'queue'}>
          <path d="M5 7h14" />
          <path d="M5 12h14" />
          <path d="M5 17h10" />
        </Match>
        <Match when={props.name === 'folder-plus'}>
          <path d="M4 6h6l2 2h8v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z" />
          <path d="M12 12v5" />
          <path d="M9.5 14.5h5" />
        </Match>
        <Match when={props.name === 'settings'}>
          <circle cx="12" cy="12" r="3" />
          <path d="M12 3v3" />
          <path d="M12 18v3" />
          <path d="M4.2 7.5 6.8 9" />
          <path d="M17.2 15 19.8 16.5" />
          <path d="M4.2 16.5 6.8 15" />
          <path d="M17.2 9 19.8 7.5" />
        </Match>
        <Match when={props.name === 'filter'}>
          <path d="M4 5h16l-6 7v5l-4 2v-7z" />
        </Match>
        <Match when={props.name === 'sort'}>
          <path d="M8 5v14" />
          <path d="m5 8 3-3 3 3" />
          <path d="M16 19V5" />
          <path d="m13 16 3 3 3-3" />
        </Match>
        <Match when={props.name === 'chevron-down'}>
          <path d="m7 10 5 5 5-5" />
        </Match>
        <Match when={props.name === 'star'}>
          <path d="m12 3 2.7 5.5 6.1.9-4.4 4.3 1 6-5.4-2.9-5.4 2.9 1-6-4.4-4.3 6.1-.9z" />
        </Match>
        <Match when={props.name === 'pin'}>
          <path d="M8 4h8" />
          <path d="M10 4v6l-2 3h8l-2-3V4" />
          <path d="M12 13v7" />
        </Match>
        <Match when={props.name === 'more'}>
          <circle cx="12" cy="5" r="1" />
          <circle cx="12" cy="12" r="1" />
          <circle cx="12" cy="19" r="1" />
        </Match>
        <Match when={props.name === 'quote'}>
          <path d="M8 11H5c0-3 1-5 4-6" />
          <path d="M18 11h-3c0-3 1-5 4-6" />
          <path d="M5 11v5h5v-5" />
          <path d="M15 11v5h5v-5" />
        </Match>
      </Switch>
    </svg>
  );
}
