import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ConsoleStore } from './store/ConsoleStore';
import { useRingBuffer } from './hooks/useRingBuffer';
import { useSerialBackend } from './hooks/useSerialBackend';
import { Header } from './components/Header';
import { TabNav } from './components/TabNav';
import { PlotCanvas, type PlotCanvasHandle } from './components/PlotCanvas';
import { PlotToolsOverlay } from './components/PlotToolsOverlay';
import { Legend } from './components/Legend';
import { ConsolePane } from './components/ConsolePane';

// Expose IPC helpers on window for devtools testing in dev builds.
if (import.meta.env.DEV) {
  (window as any).__ipc = { invoke, listen };
}

const consoleStore = new ConsoleStore();

export default function App() {
  const { store: ringStore, seriesKeys, onNewSeries } = useRingBuffer();
  const { status, ports, listPorts, connect, disconnect, sendLine } =
    useSerialBackend(ringStore, consoleStore, onNewSeries);

  const [tab, setTab] = useState<'chart' | 'console'>('chart');
  const [paused, setPaused] = useState(false);
  const [windowMs, setWindowMs] = useState(30_000);
  const [scrubbing, setScrubbing] = useState(false);
  const [visible, setVisible] = useState<Set<string>>(new Set());

  const plotRef = useRef<PlotCanvasHandle>(null);

  // Auto-show new series as they appear
  useEffect(() => {
    setVisible(new Set(seriesKeys));
  }, [seriesKeys]);

  // Load ports on mount only. Use the ⟳ Ports button to refresh manually.
  useEffect(() => { listPorts(); }, [listPorts]);

  function toggleSeries(key: string) {
    setVisible((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  }

  function resetView() {
    plotRef.current?.resetToLive();
    setScrubbing(false);
  }

  const connected = status === 'connected';

  return (
    <div className="flex flex-col h-full bg-gray-900 text-gray-100 overflow-hidden">
      <Header
        status={status}
        ports={ports}
        onRefreshPorts={listPorts}
        onConnect={(path, baud) => connect({ path, baud })}
        onDisconnect={disconnect}
      />

      <TabNav active={tab} onChange={setTab} />

      {tab === 'chart' ? (
        <div className="flex flex-col flex-1 overflow-hidden">
          <PlotCanvas
            ref={plotRef}
            store={ringStore}
            seriesKeys={seriesKeys}
            visible={visible}
            windowMs={windowMs}
            paused={paused}
            onScrubChange={setScrubbing}
          />
          <PlotToolsOverlay
            paused={paused}
            windowMs={windowMs}
            scrubbing={scrubbing}
            onTogglePause={() => setPaused((p) => !p)}
            onWindowChange={setWindowMs}
            onResetView={resetView}
          />
          <Legend seriesKeys={seriesKeys} visible={visible} onToggle={toggleSeries} />
        </div>
      ) : (
        <ConsolePane
          store={consoleStore}
          connected={connected}
          onSend={sendLine}
        />
      )}
    </div>
  );
}
