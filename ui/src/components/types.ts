export type Route = 'search' | 'library' | 'chat' | 'graph' | 'notifications' | 'jobs';

export type ToastKind = 'info' | 'success' | 'error';

export type Toast = {
  id: number;
  message: string;
  kind: ToastKind;
};

export type Notify = (message: string, kind?: ToastKind) => void;
