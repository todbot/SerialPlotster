import { useRef, useState } from 'react';

const LINE_ENDINGS = [
  { label: '\\n',   value: '\n' },
  { label: '\\r\\n', value: '\r\n' },
  { label: '\\r',   value: '\r' },
  { label: 'none',  value: '' },
];

interface ConsoleInputProps {
  disabled?: boolean;
  onSend: (text: string, lineEnding: string) => void;
}

export function ConsoleInput({ disabled, onSend }: ConsoleInputProps) {
  const [text, setText] = useState('');
  const [ending, setEnding] = useState('\n');
  const inputRef = useRef<HTMLInputElement>(null);

  function send() {
    const t = text.trim();
    if (!t) return;
    onSend(t, ending);
    setText('');
    inputRef.current?.focus();
  }

  function sendCtrl(char: string) {
    onSend(char, '');
    inputRef.current?.focus();
  }

  return (
    <div className="flex gap-2 px-2 py-2 bg-gray-800 border-t border-gray-700 flex-none">
      <button
        onClick={() => sendCtrl('\x03')}
        disabled={disabled}
        title="Send Ctrl-C (ETX, 0x03)"
        className="bg-gray-700 hover:bg-gray-600 disabled:opacity-40 text-gray-300 hover:text-white text-xs rounded px-2 py-1 border border-gray-600 font-mono flex-none"
      >
        ^C
      </button>
      <button
        onClick={() => sendCtrl('\x04')}
        disabled={disabled}
        title="Send Ctrl-D (EOT, 0x04)"
        className="bg-gray-700 hover:bg-gray-600 disabled:opacity-40 text-gray-300 hover:text-white text-xs rounded px-2 py-1 border border-gray-600 font-mono flex-none"
      >
        ^D
      </button>
      <input
        ref={inputRef}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => e.key === 'Enter' && send()}
        disabled={disabled}
        placeholder={disabled ? 'not connected' : 'send command…'}
        className="flex-1 bg-gray-700 text-sm text-white rounded px-2 py-1 border border-gray-600 placeholder-gray-500 disabled:opacity-40 font-mono"
      />
      <select
        value={ending}
        onChange={(e) => setEnding(e.target.value)}
        disabled={disabled}
        className="bg-gray-700 text-xs text-white rounded px-1 py-1 border border-gray-600 disabled:opacity-40 [&>option]:bg-gray-700 [&>option]:text-white [color-scheme:dark]"
      >
        {LINE_ENDINGS.map(({ label, value }) => (
          <option key={label} value={value}>{label}</option>
        ))}
      </select>
      <button
        onClick={send}
        disabled={disabled || !text.trim()}
        className="bg-blue-600 hover:bg-blue-500 disabled:opacity-40 text-white text-sm rounded px-3 py-1"
      >
        Send
      </button>
    </div>
  );
}
