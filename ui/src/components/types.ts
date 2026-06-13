export type Route = 'search' | 'library' | 'chat' | 'graph' | 'notifications' | 'jobs';

export type ToastKind = 'info' | 'success' | 'error';

export type Toast = {
  id: number;
  message: string;
  kind: ToastKind;
  key?: string;
};

export type Notify = (message: string, kind?: ToastKind) => void;

export type IngestProgress = {
  percent: number | null;
  imported: number;
  total: number | null;
  active: boolean;
};
