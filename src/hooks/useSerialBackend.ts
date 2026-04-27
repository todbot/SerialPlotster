import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { SampleEvent, RawEvent, StatusEvent, ConnectionState } from '../types/serial';
import type { RingStore } from '../store/RingStore';
import type { ConsoleStore } from '../store/ConsoleStore';

export interface ConnectParams {
  path: string;
  baud: number;
  dataBits?: number;
  parity?: 'none' | 'odd' | 'even';
  stopBits?: number;
  flowControl?: 'none' | 'software' | 'hardware';
}

export function useSerialBackend(
  ringStore: RingStore,
  consoleStore: ConsoleStore,
  onNewSeries: () => void,
) {
  const [status, setStatus] = useState<ConnectionState>('disconnected');
  const [ports, setPorts] = useState<string[]>([]);
  const onNewSeriesRef = useRef(onNewSeries);
  onNewSeriesRef.current = onNewSeries;

  useEffect(() => {
    const subs = [
      listen<SampleEvent>('serial://sample', (e) => {
        const { t_ms, values, labels } = e.payload;
        const isNew = ringStore.addSample(t_ms, values, labels);
        if (isNew) onNewSeriesRef.current();
      }),
      listen<RawEvent>('serial://raw', (e) => {
        consoleStore.add(e.payload);
      }),
      listen<StatusEvent>('serial://status', (e) => {
        const s = e.payload.state;
        setStatus(s);
        if (s === 'disconnected' || s === 'error') ringStore.addGap();
      }),
    ];

    return () => {
      subs.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, [ringStore, consoleStore]);

  const listPorts = useCallback(async () => {
    const p = await invoke<string[]>('list_ports');
    setPorts(p);
    return p;
  }, []);

  const connect = useCallback(async (params: ConnectParams) => {
    await invoke('connect', {
      path: params.path,
      baud: params.baud,
      dataBits: params.dataBits ?? 8,
      parity: params.parity ?? 'none',
      stopBits: params.stopBits ?? 1,
      flowControl: params.flowControl ?? 'none',
    });
  }, []);

  const disconnect = useCallback(async () => {
    await invoke('disconnect');
  }, []);

  const sendLine = useCallback(async (text: string, lineEnding = '\n') => {
    await invoke('send_line', { text, lineEnding });
  }, []);

  const startMockStream = useCallback(async (shape = 'all', rateHz = 50) => {
    await invoke('start_mock_stream', { shape, rateHz });
  }, []);

  return { status, ports, listPorts, connect, disconnect, sendLine, startMockStream };
}
