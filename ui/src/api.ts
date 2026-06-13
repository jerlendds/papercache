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
  folder_id?: string | null;
  path: string;
  canonical_path?: string | null;
  title?: string | null;
  file_name: string;
  file_size?: number | null;
  modified_at?: string | null;
  sha256?: string | null;
  authors?: string[];
  year?: number | null;
  doi?: string | null;
  arxiv_id?: string | null;
  status: string;
  error?: string | null;
  page_count?: number | null;
  classification?: Classification | null;
  cover_url: string;
  file_url?: string;
  created_at?: string;
  updated_at?: string;
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

export type ImportedFolder = {
  id: string;
  path: string;
  recursive: boolean;
  enabled: boolean;
  last_scan_at?: string | null;
  document_count: number;
};

export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

const AUTH_TOKEN_KEY = 'papercache.authToken';

export function getAuthToken() {
  return window.localStorage.getItem(AUTH_TOKEN_KEY) ?? '';
}

export function setAuthToken(token: string) {
  const value = token.trim();
  if (value) {
    window.localStorage.setItem(AUTH_TOKEN_KEY, value);
  } else {
    window.localStorage.removeItem(AUTH_TOKEN_KEY);
  }
}

function withAuth(init: RequestInit = {}): RequestInit {
  const token = getAuthToken();
  if (!token) return init;
  return {
    ...init,
    headers: {
      ...(init.headers ?? {}),
      Authorization: `Bearer ${token}`,
    },
  };
}

async function requestJson<T>(url: string, init?: RequestInit): Promise<T> {
  const headers = {
    'Content-Type': 'application/json',
    ...(init?.headers ?? {}),
  };
  const response = await fetch(url, {
    ...init,
    headers,
  });
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const body = (await response.json()) as { error?: unknown };
      if (typeof body.error === 'string' && body.error.trim()) {
        message = body.error;
      }
    } catch {
      try {
        const text = await response.text();
        if (text.trim()) message = text.trim();
      } catch {
        // Keep the status text fallback.
      }
    }
    throw new ApiError(response.status, response.statusText, message);
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
    return requestJson<{ status: string }>(`/api/documents/${documentId}/classification`, withAuth({
      method: 'PUT',
      body: JSON.stringify(classification),
    }));
  },

  async folders() {
    return requestJson<ImportedFolder[]>('/api/folders');
  },

  async importFolder(path: string, recursive: boolean) {
    return requestJson<{ folder_id: string; status: string }>('/api/folders', withAuth({
      method: 'POST',
      body: JSON.stringify({ path, recursive }),
    }));
  },

  async disableFolder(folderId: string) {
    return requestJson<{ status: string }>(`/api/folders/${folderId}`, withAuth({
      method: 'DELETE',
    }));
  },

  async jobs() {
    return requestJson<Job[]>('/api/jobs');
  },
};
