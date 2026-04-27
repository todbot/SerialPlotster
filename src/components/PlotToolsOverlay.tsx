import { useEffect, useState } from 'react';

const WINDOW_OPTIONS = [
  { label: '1s',  ms:   1_000 },
  { label: '5s',  ms:   5_000 },
  { label: '10s', ms:  10_000 },
  { label: '30s', ms:  30_000 },
  { label: '1m',  ms:  60_000 },
  { label: '5m',  ms: 300_000 },
];

interface PlotToolsOverlayProps {
  paused: boolean;
  windowMs: number;
  scrubbing: boolean;
  yFixed: boolean;
  yMin: number;
  yMax: number;
  onTogglePause: () => void;
  onWindowChange: (ms: number) => void;
  onResetView: () => void;
  onToggleYFixed: () => void;
  onYRangeChange: (min: number, max: number) => void;
}

export function PlotToolsOverlay({
  paused, windowMs, scrubbing, yFixed, yMin, yMax,
  onTogglePause, onWindowChange, onResetView, onToggleYFixed, onYRangeChange,
}: PlotToolsOverlayProps) {
  const [minStr, setMinStr] = useState('');
  const [maxStr, setMaxStr] = useState('');

  // Seed inputs only when transitioning into fixed mode; leave them alone while the user types.
  useEffect(() => {
    if (yFixed) {
      setMinStr(String(yMin));
      setMaxStr(String(yMax));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [yFixed]);

  function commitMin(s: string) {
    const v = parseFloat(s);
    if (isFinite(v)) onYRangeChange(v, yMax);
    else setMinStr(String(yMin));
  }

  function commitMax(s: string) {
    const v = parseFloat(s);
    if (isFinite(v)) onYRangeChange(yMin, v);
    else setMaxStr(String(yMax));
  }

  return (
    <div className="flex items-center gap-3 px-3 py-1.5 bg-gray-800 border-t border-gray-700 flex-none text-sm select-none">

      {/* Left: Y-axis controls */}
      <button
        onClick={onToggleYFixed}
        title={yFixed ? 'Switch to auto Y-axis' : 'Switch to fixed Y-axis'}
        className={`text-xs px-2 py-0.5 rounded border ${
          yFixed
            ? 'bg-blue-700 border-blue-500 text-white'
            : 'bg-gray-700 border-gray-600 text-gray-300 hover:text-white'
        }`}
      >
        Y: {yFixed ? 'Fixed' : 'Auto'}
      </button>

      {yFixed && (
        <div className="flex items-center gap-1 text-xs">
          <input
            type="number"
            value={minStr}
            onChange={(e) => setMinStr(e.target.value)}
            onBlur={(e) => commitMin(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && commitMin(minStr)}
            className="w-16 bg-gray-700 text-white rounded px-1.5 py-0.5 border border-gray-600 text-right [appearance:textfield]"
            title="Y min"
          />
          <span className="text-gray-500">–</span>
          <input
            type="number"
            value={maxStr}
            onChange={(e) => setMaxStr(e.target.value)}
            onBlur={(e) => commitMax(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && commitMax(maxStr)}
            className="w-16 bg-gray-700 text-white rounded px-1.5 py-0.5 border border-gray-600 text-right [appearance:textfield]"
            title="Y max"
          />
        </div>
      )}

      <div className="flex-1" />

      {/* Right: scrub indicator, time window, pause */}
      {scrubbing && (
        <>
          <span className="text-yellow-400 text-xs">● scrubbing</span>
          <button
            onClick={onResetView}
            className="text-blue-400 hover:text-blue-300 text-xs underline"
          >
            back to live
          </button>
        </>
      )}

      <select
        value={windowMs}
        onChange={(e) => onWindowChange(Number(e.target.value))}
        className="bg-gray-700 text-white rounded px-2 py-0.5 border border-gray-600 text-xs [&>option]:bg-gray-700 [&>option]:text-white [color-scheme:dark]"
      >
        {WINDOW_OPTIONS.map(({ label, ms }) => (
          <option key={ms} value={ms}>{label}</option>
        ))}
      </select>

      <button
        onClick={onTogglePause}
        title={paused ? 'Resume' : 'Pause'}
        className="text-gray-300 hover:text-white px-2 py-0.5 rounded bg-gray-700 hover:bg-gray-600 text-xs"
      >
        {paused ? '▶ Resume' : '⏸ Pause'}
      </button>

    </div>
  );
}
