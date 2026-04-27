import { useEffect, useRef, useState } from 'react';
import type { ConsoleStore } from '../store/ConsoleStore';
import type { RawEvent } from '../types/serial';

interface ConsoleLogProps {
  store: ConsoleStore;
}

export function ConsoleLog({ store }: ConsoleLogProps) {
  const [lines, setLines] = useState<readonly RawEvent[]>(store.lines);
  const bottomRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const atBottomRef = useRef(true);

  useEffect(() => store.subscribe(() => setLines([...store.lines])), [store]);

  useEffect(() => {
    if (atBottomRef.current) {
      bottomRef.current?.scrollIntoView();
    }
  }, [lines]);

  function onScroll() {
    const el = containerRef.current;
    if (!el) return;
    atBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 16;
  }

  return (
    <div
      ref={containerRef}
      onScroll={onScroll}
      className="flex-1 overflow-y-auto font-mono text-xs px-2 py-1 space-y-0.5"
    >
      {lines.map((line, i) => (
        <div key={i} className="flex gap-2 leading-5">
          <span className="text-gray-600 flex-none w-24 text-right">
            {new Date(line.t_ms).toISOString().slice(11, 23)}
          </span>
          <span className={line.direction === 'tx' ? 'text-yellow-400' : 'text-gray-300'}>
            {line.direction === 'tx' ? '→' : '←'}
          </span>
          <span className={`break-all ${line.direction === 'tx' ? 'text-yellow-300' : 'text-gray-200'}`}>
            {line.text}
          </span>
        </div>
      ))}
      <div ref={bottomRef} />
    </div>
  );
}
