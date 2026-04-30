import { useEffect, useState } from 'react';
import type { ConnectionState } from '../types/serial';

const BAUD_RATES = [1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];

const MOCK_SHAPES = ['all', 'sincos', 'sin', 'cos', 'noise'];

interface HeaderProps {
  status: ConnectionState;
  ports: string[];
  onRefreshPorts: () => void;
  onConnect: (port: string, baud: number) => void;
  onDisconnect: () => void;
  onMock?: (shape: string) => void;
}

export function Header({ status, ports, onRefreshPorts, onConnect, onDisconnect, onMock }: HeaderProps) {
  const [port, setPort] = useState('');
  const [baud, setBaud] = useState(115200);
  const [mockShape, setMockShape] = useState('all');
  const connected = status === 'connected';

  useEffect(() => {
    if (ports.length > 0 && !port) setPort(ports[0]);
  }, [ports, port]);

  const statusColor =
    status === 'connected' ? 'bg-green-500' :
    status === 'error'     ? 'bg-red-500' :
                             'bg-gray-500';

  return (
    <header className="flex items-center gap-2 px-3 h-11 bg-gray-800 border-b border-gray-700 flex-none select-none">
      <button
        onClick={onRefreshPorts}
        title="Refresh port list"
        className="flex items-center gap-1 text-gray-300 hover:text-white text-xs bg-gray-700 hover:bg-gray-600 rounded px-2 py-1 border border-gray-600 whitespace-nowrap"
      >
        ⟳ Ports
      </button>

      <select
        value={port}
        onChange={(e) => setPort(e.target.value)}
        disabled={connected}
        className="bg-gray-700 text-sm text-white rounded px-2 py-1 border border-gray-600 disabled:opacity-50 min-w-32 [&>option]:bg-gray-700 [&>option]:text-white [color-scheme:dark]"
      >
        {ports.length === 0 && <option value="">No ports</option>}
        {ports.map((p) => <option key={p} value={p}>{p}</option>)}
      </select>

      <select
        value={baud}
        onChange={(e) => setBaud(Number(e.target.value))}
        disabled={connected}
        className="bg-gray-700 text-sm text-white rounded px-2 py-1 border border-gray-600 disabled:opacity-50 [&>option]:bg-gray-700 [&>option]:text-white [color-scheme:dark]"
      >
        {BAUD_RATES.map((b) => <option key={b} value={b}>{b}</option>)}
      </select>

      {connected ? (
        <button
          onClick={onDisconnect}
          className="bg-red-700 hover:bg-red-600 text-white text-sm rounded px-3 py-1"
        >
          Disconnect
        </button>
      ) : (
        <button
          onClick={() => port && onConnect(port, baud)}
          disabled={!port}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-40 text-white text-sm rounded px-3 py-1"
        >
          Connect
        </button>
      )}

      {import.meta.env.DEV && onMock && !connected && (
        <>
          <div className="w-px h-5 bg-gray-600 mx-1" />
          <select
            value={mockShape}
            onChange={(e) => setMockShape(e.target.value)}
            className="bg-gray-700 text-xs text-gray-300 rounded px-1.5 py-1 border border-gray-600 [&>option]:bg-gray-700 [&>option]:text-white [color-scheme:dark]"
          >
            {MOCK_SHAPES.map((s) => <option key={s} value={s}>{s}</option>)}
          </select>
          <button
            onClick={() => onMock(mockShape)}
            className="text-xs bg-purple-700 hover:bg-purple-600 text-white rounded px-2 py-1"
          >
            ▶ Mock
          </button>
        </>
      )}

      <div className="flex-1" />
      <div className={`w-2.5 h-2.5 rounded-full ${statusColor} flex-none`} title={status} />
      <span className="text-xs text-gray-400">{status}</span>
    </header>
  );
}
