import { useEffect, useRef, useState } from 'react';
import type { ConsoleStore } from '../store/ConsoleStore';
import type { RawEvent } from '../types/serial';

interface ConsoleLogProps {
  store: ConsoleStore;
}

export function ConsoleLog({ store }: ConsoleLogProps) {
  const [lines, setLines] = useState<readonly RawEvent[]>(store.lines);
  const containerRef = useRef<HTMLDivElement>(null);
  const atBottomRef = useRef(true);
  const lastScrollTopRef = useRef(0);

  function scrollToBottom() {
    const el = containerRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }

  useEffect(() => store.subscribe(() => setLines([...store.lines])), [store]);

  useEffect(() => { scrollToBottom(); }, []); // scroll to bottom on mount

  useEffect(() => {
    if (atBottomRef.current) scrollToBottom();
  }, [lines]);

  function onScroll() {
    const el = containerRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 16;
    if (atBottom) {
      atBottomRef.current = true;
    } else if (el.scrollTop < lastScrollTopRef.current) {
      // Only disable auto-scroll when the user scrolls up, not on spurious events.
      atBottomRef.current = false;
    }
    lastScrollTopRef.current = el.scrollTop;
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
    </div>
  );
}
