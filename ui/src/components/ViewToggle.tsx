import type { ViewMode } from '../../api';

export function ViewToggle(props: { mode: ViewMode; onMode: (mode: ViewMode) => void }) {
  return (
    <div class="segmented" role="radiogroup" aria-label="View mode">
      <button
        role="radio"
        aria-checked={props.mode === 'grid'}
        classList={{ selected: props.mode === 'grid' }}
        onClick={() => props.onMode('grid')}
      >
        Grid
      </button>
      <button
        role="radio"
        aria-checked={props.mode === 'list'}
        classList={{ selected: props.mode === 'list' }}
        onClick={() => props.onMode('list')}
      >
        List
      </button>
    </div>
  );
}
