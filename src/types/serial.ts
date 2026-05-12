export interface SampleEvent {
  t_ms: number;
  values: number[];
  labels?: string[];
}

export interface RawEvent {
  t_ms: number;
  direction: 'rx' | 'tx';
  text: string;
}

export interface SampleBatch {
  samples: SampleEvent[];
}

export interface StatusEvent {
  state: 'connected' | 'disconnected' | 'error';
  reason?: string;
}

export type ConnectionState = StatusEvent['state'];
