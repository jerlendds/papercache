import type { JSX } from 'solid-js';

export function PageHeader(props: { title: string; subtitle: string; right?: JSX.Element }) {
  return (
    <header class="page-header">
      <div>
        <h1>{props.title}</h1>
        <p>{props.subtitle}</p>
      </div>
      {props.right}
    </header>
  );
}
