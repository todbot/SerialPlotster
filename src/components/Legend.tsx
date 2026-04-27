export const SERIES_COLORS = [
  '#00d9ff', '#ff6b35', '#b388ff', '#69f0ae',
  '#ff5252', '#ffeb3b', '#40c4ff', '#f48fb1',
];

export function seriesColor(index: number): string {
  return SERIES_COLORS[index % SERIES_COLORS.length];
}

interface LegendProps {
  seriesKeys: string[];
  visible: Set<string>;
  onToggle: (key: string) => void;
}

export function Legend({ seriesKeys, visible, onToggle }: LegendProps) {
  if (seriesKeys.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-x-4 gap-y-1 px-3 py-2 bg-gray-800 border-t border-gray-700 flex-none select-none">
      {seriesKeys.map((key, i) => {
        const color = seriesColor(i);
        const on = visible.has(key);
        return (
          <button
            key={key}
            onClick={() => onToggle(key)}
            className={`flex items-center gap-1.5 text-xs transition-opacity ${on ? 'opacity-100' : 'opacity-35'}`}
          >
            <span
              style={{ backgroundColor: color, width: 14, height: 2, display: 'inline-block', borderRadius: 1 }}
            />
            <span style={{ color }}>{key || `series ${i}`}</span>
          </button>
        );
      })}
    </div>
  );
}
