import { fireEvent, render, waitFor } from '@solidjs/testing-library';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import { App } from './app';

class MockEventSource {
  static instances: MockEventSource[] = [];
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: (() => void) | null = null;
  close = vi.fn();
  constructor(public url: string) {
    MockEventSource.instances.push(this);
  }
}

describe('<App />', () => {
  beforeEach(() => {
    MockEventSource.instances = [];
    window.localStorage.clear();
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
        if (url === '/api/folders' && init?.method === 'POST') {
          return jsonResponse({ folder_id: 'folder-1', status: 'queued' });
        }
        if (url === '/api/folders') {
          return jsonResponse([
            {
              id: 'folder-1',
              path: '/Users/example/Papers',
              recursive: true,
              enabled: true,
              last_scan_at: null,
              document_count: 1,
            },
          ]);
        }
        if (url === '/api/folders/folder-1' && init?.method === 'DELETE') {
          return jsonResponse({ status: 'disabled' });
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
    await waitFor(() => expect(screen.getByText('Retrieval Paper')).toBeInTheDocument());

    fireEvent.click(screen.getByLabelText('Network'));
    expect(screen.getByText(/sigma.js/)).toBeInTheDocument();

    fireEvent.click(screen.getByLabelText('Jobs'));
    await waitFor(() => expect(screen.getByText('Ingest pdf')).toBeInTheDocument());
  });

  test('search submits on enter and shows result cards', async () => {
    const screen = render(() => <App />);
    fireEvent.click(screen.getByLabelText('Search'));
    const input = screen.getByLabelText('Search papers') as HTMLInputElement;
    fireEvent.input(input, { target: { value: 'retrieval' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    await waitFor(() => expect(screen.getAllByText('Retrieval Paper').length).toBeGreaterThan(0));
    expect(screen.getAllByText('/Users/example/Papers/retrieval.pdf').length).toBeGreaterThan(0);
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

  test('library queues folder imports', async () => {
    window.localStorage.setItem('papercache.authToken', 'test-token');
    const screen = render(() => <App />);
    fireEvent.click(screen.getByRole('button', { name: 'Import folder' }));
    expect(screen.getByRole('dialog', { name: 'Folder imports' })).toBeInTheDocument();

    const input = screen.getByLabelText('Folder path') as HTMLInputElement;
    fireEvent.input(input, { target: { value: '/Users/example/Papers' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => expect(screen.getByText('Folder scan queued')).toBeInTheDocument());
    expect(fetch).toHaveBeenCalledWith(
      '/api/folders',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          Authorization: 'Bearer test-token',
          'Content-Type': 'application/json',
        }),
        body: JSON.stringify({ path: '/Users/example/Papers', recursive: true }),
      }),
    );
  });

  test('settings autosaves auth token', () => {
    const screen = render(() => <App />);
    fireEvent.click(screen.getByLabelText('Settings'));

    const input = screen.getByLabelText('Auth token') as HTMLInputElement;
    fireEvent.input(input, { target: { value: 'saved-token' } });

    expect(window.localStorage.getItem('papercache.authToken')).toBe('saved-token');
  });

  test('folder import events update one progress toast', async () => {
    const screen = render(() => <App />);
    const source = MockEventSource.instances[0];

    source.onmessage?.(
      new MessageEvent('message', {
        data: JSON.stringify({
          type: 'folder_scan_completed',
          folder_id: 'folder-1',
          discovered: 2,
        }),
      }),
    );
    source.onmessage?.(
      new MessageEvent('message', {
        data: JSON.stringify({
          type: 'document_ready',
          folder_id: 'folder-1',
          document_id: 'doc-1',
        }),
      }),
    );
    source.onmessage?.(
      new MessageEvent('message', {
        data: JSON.stringify({
          type: 'document_ready',
          folder_id: 'folder-1',
          document_id: 'doc-2',
        }),
      }),
    );

    await waitFor(() => expect(screen.getByText('2/2 PDF files imported!')).toBeInTheDocument());
    expect(screen.queryByText('1/2 PDF files imported...')).not.toBeInTheDocument();
  });

  test('library refetches when import events arrive', async () => {
    render(() => <App />);
    const source = MockEventSource.instances[0];

    await waitFor(() =>
      expect(
        vi.mocked(fetch).mock.calls.filter(([url]) => String(url).startsWith('/api/documents')).length,
      ).toBeGreaterThan(0),
    );
    const before = vi.mocked(fetch).mock.calls.filter(([url]) =>
      String(url).startsWith('/api/documents'),
    ).length;

    source.onmessage?.(
      new MessageEvent('message', {
        data: JSON.stringify({
          type: 'document_ready',
          folder_id: 'folder-1',
          document_id: 'doc-2',
        }),
      }),
    );

    await waitFor(() =>
      expect(
        vi.mocked(fetch).mock.calls.filter(([url]) => String(url).startsWith('/api/documents')).length,
      ).toBeGreaterThan(before),
    );
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
