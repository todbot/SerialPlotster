import type { ConsoleStore } from '../store/ConsoleStore';
import { ConsoleLog } from './ConsoleLog';
import { ConsoleInput } from './ConsoleInput';

interface ConsolePaneProps {
  store: ConsoleStore;
  connected: boolean;
  onSend: (text: string, lineEnding: string) => void;
}

export function ConsolePane({ store, connected, onSend }: ConsolePaneProps) {
  return (
    <div className="flex flex-col flex-1 overflow-hidden">
      <ConsoleLog store={store} />
      <ConsoleInput disabled={!connected} onSend={onSend} />
    </div>
  );
}
