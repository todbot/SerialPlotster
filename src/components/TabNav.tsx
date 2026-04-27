interface TabNavProps {
  active: 'chart' | 'console';
  onChange: (tab: 'chart' | 'console') => void;
}

export function TabNav({ active, onChange }: TabNavProps) {
  return (
    <div className="flex bg-gray-800 border-b border-gray-700 flex-none select-none">
      {(['chart', 'console'] as const).map((tab) => (
        <button
          key={tab}
          onClick={() => onChange(tab)}
          className={`px-4 py-2 text-sm capitalize border-b-2 transition-colors ${
            active === tab
              ? 'border-blue-500 text-white'
              : 'border-transparent text-gray-400 hover:text-white'
          }`}
        >
          {tab}
        </button>
      ))}
    </div>
  );
}
