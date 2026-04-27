const WINDOW_OPTIONS = [
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
  onTogglePause: () => void;
  onWindowChange: (ms: number) => void;
  onResetView: () => void;
}

export function PlotToolsOverlay({
  paused, windowMs, scrubbing, onTogglePause, onWindowChange, onResetView,
}: PlotToolsOverlayProps) {
  return (
    <div className="flex items-center gap-3 px-3 py-1.5 bg-gray-800 border-t border-gray-700 flex-none text-sm select-none">
      <button
        onClick={onTogglePause}
        title={paused ? 'Resume' : 'Pause'}
        className="text-gray-300 hover:text-white px-2 py-0.5 rounded bg-gray-700 hover:bg-gray-600 text-xs"
      >
        {paused ? '▶ Resume' : '⏸ Pause'}
      </button>

      <select
        value={windowMs}
        onChange={(e) => onWindowChange(Number(e.target.value))}
        className="bg-gray-700 text-white rounded px-2 py-0.5 border border-gray-600 text-xs"
      >
        {WINDOW_OPTIONS.map(({ label, ms }) => (
          <option key={ms} value={ms}>{label}</option>
        ))}
      </select>

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
    </div>
  );
}
