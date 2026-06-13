import { fireEvent, render, waitFor } from '@solidjs/testing-library';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import { App } from './app';

class MockEventSource {
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: (() => void) | null = null;
  close = vi.fn();
  constructor(public url: string) {}
}

describe('<App />', () => {
  beforeEach(() => {
    vi.stubGlobal('EventSource', MockEventSource);
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: string, init?: RequestInit) => {
        if (url === '/api/search') {
          return jsonResponse({
            results: [
              {
                document_id: 'doc-1',
                chunk_id: 'chunk-1',
                title: 'Retrieval Paper',
                path: '/Users/example/Papers/retrieval.pdf',
                page_start: 1,
                page_end: 2,
                snippet: 'retrieval text',
                score: 12,
              },
            ],
          });
        }
        if (String(url).startsWith('/api/documents') && init?.method !== 'PUT') {
          return jsonResponse([
            {
              id: 'doc-1',
              path: '/Users/example/Papers/retrieval.pdf',
              title: 'Retrieval Paper',
              file_name: 'retrieval.pdf',
              status: 'ready',
              page_count: 12,
              classification: { topics: ['retrieval'] },
              cover_url: '',
            },
          ]);
        }
        if (String(url).includes('/classification')) {
          return jsonResponse({ status: 'updated' });
        }
        if (url === '/api/jobs') {
          return jsonResponse([
            {
              id: 'job-1',
              kind: 'ingest_pdf',
              status: 'running',
              priority: 0,
              attempts: 1,
              max_attempts: 3,
              created_at: '2026-06-13T12:00:00Z',
              updated_at: '2026-06-13T12:01:00Z',
            },
          ]);
        }
        return jsonResponse({});
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test('renders sidebar navigation and changes pages', async () => {
    const screen = render(() => <App />);
    expect(screen.getByLabelText('Search')).toBeInTheDocument();
    expect(screen.getByLabelText('Library')).toBeInTheDocument();
    expect(screen.getByLabelText('Jobs')).toBeInTheDocument();

    fireEvent.click(screen.getByLabelText('Network'));
    expect(screen.getByText(/sigma.js/)).toBeInTheDocument();

    fireEvent.click(screen.getByLabelText('Jobs'));
    await waitFor(() => expect(screen.getByText('Ingest pdf')).toBeInTheDocument());
  });

  test('search submits on enter and shows result cards', async () => {
    const screen = render(() => <App />);
    const input = screen.getByLabelText('Search papers') as HTMLInputElement;
    fireEvent.input(input, { target: { value: 'retrieval' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    await waitFor(() => expect(screen.getByText('Retrieval Paper')).toBeInTheDocument());
    expect(screen.getByText('/Users/example/Papers/retrieval.pdf')).toBeInTheDocument();
  });

  test('library allows adding classifications', async () => {
    const screen = render(() => <App />);
    fireEvent.click(screen.getByLabelText('Library'));

    await waitFor(() => expect(screen.getByText('Retrieval Paper')).toBeInTheDocument());
    const input = screen.getByLabelText('Add classification for retrieval.pdf') as HTMLInputElement;
    fireEvent.input(input, { target: { value: 'rag' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => expect(screen.getByText('Classification updated')).toBeInTheDocument());
  });
});

function jsonResponse(body: unknown) {
  return Promise.resolve({
    ok: true,
    status: 200,
    statusText: 'OK',
    json: () => Promise.resolve(body),
  } as Response);
}
