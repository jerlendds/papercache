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
      </Switch>
    </svg>
  );
}
