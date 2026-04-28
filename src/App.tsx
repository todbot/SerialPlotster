import { useEffect, useMemo, useRef, useState } from 'react';
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
  const { store: ringStore, seriesKeys, onNewSeries, clearStore } = useRingBuffer();
  const { status, ports, listPorts, connect, disconnect, sendLine, startMockStream } =
    useSerialBackend(ringStore, consoleStore, onNewSeries, () => {
      clearStore();
      setHiddenSeries(new Set());
    });

  const [tab, setTab] = useState<'chart' | 'console'>('chart');
  const [paused, setPaused] = useState(false);
  const [windowMs, setWindowMs] = useState(30_000);
  const [scrubbing, setScrubbing] = useState(false);
  const [hiddenSeries, setHiddenSeries] = useState<Set<string>>(new Set());
  const visible = useMemo(
    () => new Set(seriesKeys.filter((k) => !hiddenSeries.has(k))),
    [seriesKeys, hiddenSeries],
  );
  const [yFixed, setYFixed] = useState(false);
  const [yMin, setYMin] = useState(-1);
  const [yMax, setYMax] = useState(1);

  const plotRef = useRef<PlotCanvasHandle>(null);

  // Load ports on mount only. Use the ⟳ Ports button to refresh manually.
  useEffect(() => { listPorts(); }, [listPorts]);

  // Exit scrub mode whenever a new connection is established.
  useEffect(() => {
    if (status === 'connected') resetView();
  }, [status]); // eslint-disable-line react-hooks/exhaustive-deps

  function toggleSeries(key: string) {
    setHiddenSeries((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  }

  function resetView() {
    plotRef.current?.resetToLive();
    setScrubbing(false);
  }

  function toggleYFixed() {
    if (!yFixed) {
      // Seed the fixed inputs from whatever auto-scale is currently showing,
      // rounded to 4 significant figures so the inputs aren't overwhelming.
      const r = plotRef.current?.getAutoYRange();
      if (r) {
        setYMin(parseFloat(r.min.toPrecision(4)));
        setYMax(parseFloat(r.max.toPrecision(4)));
      }
    }
    setYFixed((f) => !f);
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
        onMock={(shape) => startMockStream(shape)}
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
            yRange={yFixed ? { min: yMin, max: yMax } : null}
            onScrubChange={setScrubbing}
          />
          <Legend seriesKeys={seriesKeys} visible={visible} onToggle={toggleSeries} />
          <PlotToolsOverlay
            paused={paused}
            windowMs={windowMs}
            scrubbing={scrubbing}
            yFixed={yFixed}
            yMin={yMin}
            yMax={yMax}
            onTogglePause={() => setPaused((p) => !p)}
            onWindowChange={setWindowMs}
            onResetView={resetView}
            onToggleYFixed={toggleYFixed}
            onYRangeChange={(min, max) => { setYMin(min); setYMax(max); }}
          />
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
