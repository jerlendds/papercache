export type ViewMode = 'grid' | 'list';

export type Classification = {
  document_type?: string;
  topics?: string[];
  status?: string;
  confidence?: number;
  source?: string;
  [key: string]: unknown;
};

export type DocumentCard = {
  id: string;
  path: string;
  title?: string | null;
  file_name: string;
  status: string;
  page_count?: number | null;
  classification?: Classification | null;
  cover_url: string;
};

export type SearchResult = {
  document_id: string;
  chunk_id: string;
  title: string;
  path: string;
  page_start?: number;
  page_end?: number;
  snippet: string;
  score: number;
};

export type Job = {
  id: string;
  kind: string;
  status: string;
  priority: number;
  attempts: number;
  max_attempts: number;
  error?: string | null;
  run_after?: string | null;
  locked_at?: string | null;
  created_at: string;
  updated_at: string;
};

async function requestJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    headers: { 'Content-Type': 'application/json', ...(init?.headers ?? {}) },
    ...init,
  });
  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText}`);
  }
  return response.json() as Promise<T>;
}

export const api = {
  async search(query: string, limit = 24, offset = 0) {
    return requestJson<{ results: SearchResult[] }>('/api/search', {
      method: 'POST',
      body: JSON.stringify({ query, limit, offset }),
    });
  },

  async documents(limit = 100, offset = 0, q?: string) {
    const params = new URLSearchParams({
      limit: String(limit),
      offset: String(offset),
    });
    if (q?.trim()) params.set('q', q.trim());
    return requestJson<DocumentCard[]>(`/api/documents?${params}`);
  },

  async updateClassification(documentId: string, classification: Classification) {
    return requestJson<{ status: string }>(`/api/documents/${documentId}/classification`, {
      method: 'PUT',
      body: JSON.stringify(classification),
    });
  },

  async jobs() {
    return requestJson<Job[]>('/api/jobs');
  },
};
